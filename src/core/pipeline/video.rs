use std::sync::{Arc, Condvar, Mutex};

use eframe::egui;

use super::upscale_video_if_needed;

/// Простий лічильний семафор для обмеження паралельних потоків генерації медіа.
struct Semaphore {
    count: Mutex<usize>,
    condvar: Condvar,
}

impl Semaphore {
    fn new(n: usize) -> Self {
        Self {
            count: Mutex::new(n),
            condvar: Condvar::new(),
        }
    }

    fn acquire(&self) {
        let mut guard = self.count.lock().unwrap();
        while *guard == 0 {
            guard = self.condvar.wait(guard).unwrap();
        }
        *guard -= 1;
    }

    fn release(&self) {
        *self.count.lock().unwrap() += 1;
        self.condvar.notify_one();
    }
}

/// Перевіряє, чи є в папці media файл із заданим індексом (будь-яке розширення).
fn media_file_exists_by_index(media_dir: &std::path::Path, index: usize) -> bool {
    let stem = format!("{:04}", index);
    for ext in &["jpg", "jpeg", "png", "webp", "mp4", "webm", "mov"] {
        if media_dir.join(format!("{}.{}", stem, ext)).exists() {
            return true;
        }
    }
    false
}

/// Безпечно повертає byte-індекс для позиції в символах.
fn byte_index_at_char(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .map(|(idx, _)| idx)
        .nth(char_idx)
        .unwrap_or(text.len())
}

/// Рахує кількість Unicode-символів у byte-діапазоні.
fn char_count_between(text: &str, start: usize, end: usize) -> usize {
    text[start..end].chars().count()
}

/// Визначає кінець речення для розумних меж контексту.
fn is_sentence_end(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '…')
}

/// Зсуває початок контексту назад до попереднього кінця речення.
fn snap_context_start_to_sentence(text: &str, rough_start: usize) -> usize {
    if rough_start == 0 {
        return 0;
    }

    text[..rough_start]
        .char_indices()
        .rev()
        .find(|(_, ch)| is_sentence_end(*ch))
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0)
}

/// Зсуває кінець контексту вперед до наступного кінця речення.
fn snap_context_end_to_sentence(text: &str, rough_end: usize) -> usize {
    if rough_end >= text.len() {
        return text.len();
    }

    text[rough_end..]
        .char_indices()
        .find(|(_, ch)| is_sentence_end(*ch))
        .map(|(offset, ch)| rough_end + offset + ch.len_utf8())
        .unwrap_or(text.len())
}

/// Шукає сегмент у повному тексті, рухаючись уперед, щоб однакові фрази не плутали порядок.
fn find_segment_range(source_text: &str, segment: &str, cursor: &mut usize) -> Option<(usize, usize)> {
    if segment.is_empty() {
        return None;
    }

    let range = source_text[*cursor..]
        .find(segment)
        .map(|offset| *cursor + offset)
        .or_else(|| source_text.find(segment));

    range.map(|start| {
        let end = start + segment.len();
        *cursor = end;
        (start, end)
    })
}

/// Форматує контекст пояснювальними блоками, щоб модель чітко розуміла роль кожної частини.
fn format_around_context(before: &str, segment: &str, after: &str) -> String {
    format!(
        "Context before the segment you need to visualize:\n{}\n\nCurrent segment you need to visualize:\n{}\n\nContext after the segment you need to visualize:\n{}",
        before.trim(),
        segment.trim(),
        after.trim()
    )
}

/// Будує текст контексту для кожного сегмента відеоряду.
fn build_video_contexts(
    source_text: &str,
    segments: &[String],
    mode: &str,
    around_chars: usize,
) -> Vec<String> {
    if mode == "full" {
        return vec![source_text.to_string(); segments.len()];
    }

    let before_chars = around_chars / 2;
    let after_chars = around_chars - before_chars;
    let mut cursor = 0usize;

    segments
        .iter()
        .map(|segment| {
            let Some((seg_start, seg_end)) = find_segment_range(source_text, segment, &mut cursor) else {
                return format_around_context("", segment, "");
            };

            let chars_before_segment = char_count_between(source_text, 0, seg_start);
            let chars_until_segment_end = char_count_between(source_text, 0, seg_end);
            let total_chars = source_text.chars().count();

            let start_char = chars_before_segment.saturating_sub(before_chars);
            let end_char = (chars_until_segment_end + after_chars).min(total_chars);
            let rough_start_byte = byte_index_at_char(source_text, start_char);
            let rough_end_byte = byte_index_at_char(source_text, end_char);

            // Значення в UI — орієнтир. Реальні межі розширюємо до речень,
            // щоб контекст не починався і не закінчувався посеред фрази.
            let start_byte = snap_context_start_to_sentence(source_text, rough_start_byte);
            let end_byte = snap_context_end_to_sentence(source_text, rough_end_byte);

            let before = &source_text[start_byte..seg_start];
            let current = &source_text[seg_start..seg_end];
            let after = &source_text[seg_end..end_byte];
            format_around_context(before, current, after)
        })
        .collect()
}

/// Записує тимчасовий debug-файл із промтами після підстановки плейсхолдерів.
fn write_prompt_substitution_debug(
    save_dir: &std::path::Path,
    file_name: &str,
    title: &str,
    prompt_template: &str,
    segments: &[String],
    prompts: &[String],
) {
    use std::fmt::Write;

    let mut content = String::new();
    let _ = writeln!(content, "=== {} ===", title);
    let _ = writeln!(content, "Total: {}", prompts.len());
    let _ = writeln!(content, "\n--- ORIGINAL PROMPT TEMPLATE ---\n{}\n", prompt_template);

    for (i, prompt) in prompts.iter().enumerate() {
        let segment = segments.get(i).map(String::as_str).unwrap_or("");

        let _ = writeln!(content, "\n==================== [{} / {}] ====================", i + 1, prompts.len());
        let _ = writeln!(content, "\n{{{{text}}}}:\n{}", segment);
        let _ = writeln!(content, "\nFINAL REQUEST:\n{}", prompt);
    }

    let _ = std::fs::write(save_dir.join(file_name), content);
}

/// Підставляє текст сегмента і контекст у промт відеоряду.
fn fill_video_prompt(prompt: &str, segment: &str, context: Option<&str>) -> String {
    let mut filled = if prompt.contains("{{text}}") {
        prompt.replace("{{text}}", segment)
    } else if prompt.is_empty() {
        segment.to_string()
    } else {
        format!("{}\n\n{}", prompt, segment)
    };

    if let Some(context) = context.filter(|c| !c.trim().is_empty()) {
        if filled.contains("{{context}}") {
            filled = filled.replace("{{context}}", context);
        } else {
            filled.push_str("\n\nContext:\n");
            filled.push_str(context);
        }
    } else {
        filled = filled.replace("{{context}}", "");
    }

    filled
}

/// Валідує завантажені або декодовані медіа-байти на порожнечу та текстові помилки.
fn validate_media_bytes(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("Отримано порожній файл (0 байт)".to_string());
    }
    if bytes.len() < 500 {
        if let Ok(text) = std::str::from_utf8(bytes) {
            let trimmed = text.trim();
            if trimmed.starts_with('{')
                || trimmed.starts_with('<')
                || trimmed.starts_with("Error")
                || trimmed.starts_with("Unauthorized")
            {
                return Err(format!(
                    "Замість медіа-даних отримано текст помилки: {}",
                    trimmed
                ));
            }
        }
    }
    Ok(())
}

/// Декодує результат генерації (data URI або HTTP URL) і зберігає файл.
/// Повертає шлях до збереженого файлу.
fn decode_result(
    result: &str,
    index: usize,
    _total: usize,
    media_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    if result.starts_with("data:") {
        // data:[<mediatype>][;base64],<data>
        let rest = &result[5..];
        let comma = rest.find(',').ok_or("Invalid data URI: no comma")?;
        let header = &rest[..comma];
        let b64 = &rest[comma + 1..];

        let ext = if header.contains("mp4") {
            "mp4"
        } else if header.contains("webm") {
            "webm"
        } else if header.contains("mov") {
            "mov"
        } else if header.contains("png") {
            "png"
        } else if header.contains("webp") {
            "webp"
        } else if header.contains("gif") {
            "gif"
        } else {
            "jpg"
        };

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        validate_media_bytes(&bytes)?;

        let path = media_dir.join(format!("{:04}.{}", index, ext));
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    } else {
        // Звичайний HTTP URL — розширення беремо з URL
        let ext = result
            .split('?')
            .next()
            .unwrap_or(result)
            .rsplit('.')
            .next()
            .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("jpg");

        use std::io::Read;
        let resp = ureq::get(result)
            .call()
            .map_err(|e| format!("Download error: {}", e))?;
        let mut bytes = Vec::new();
        resp.into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Read error: {}", e))?;

        validate_media_bytes(&bytes)?;

        let path = media_dir.join(format!("{:04}.{}", index, ext));
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    }
}

/// Гілка Відеоряд (виконується паралельно з озвучкою та субтитрами).
/// Повертає Ok(()) або Err з описом першої помилки.
pub(super) fn run_video_branch(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    translated_text: Arc<Mutex<Option<String>>>,
    video_stage: Arc<Mutex<crate::queue::StageStatus>>,
    prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    media_progress: Arc<Mutex<Option<(usize, usize)>>>,
    total_cost: Arc<Mutex<Option<f64>>>,
    ctx: egui::Context,
) -> Result<(), String> {
    if !settings.video_enabled {
        return Ok(());
    }

    // Pexels/Pixabay режим — окрема гілка (lazy picker)
    if settings.video_service == "Pexels" || settings.video_service == "Pixabay" {
        return run_pexels_branch(
            job_id,
            job_name,
            &settings,
            translated_text,
            video_stage,
            prompts_progress,
            ctx,
        );
    }

    let media_label = if settings.video_media_type == "video" {
        "video"
    } else {
        "image"
    };
    crate::logger::log_job(
        job_id,
        &job_name,
        &format!("Starting video stage ({} generation)...", media_label),
    );
    *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    // В агентному режимі сегменти беруться з segments.json, LLM для промтів не викликається
    let is_agent_mode = settings.video_llm_service == "Claude Code"
        || settings.video_llm_service == "Gemini CLI"
        || settings.video_llm_service == "Codex CLI"
        || settings.video_llm_service == "AGY CLI"
        || settings.video_llm_service == "Pi CLI";

    // Визначаємо текст: перекладений якщо є, інакше оригінал
    let source_text = if settings.translation_enabled {
        translated_text
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| settings.text.clone())
    } else {
        settings.text.clone()
    };

    // Нарізаємо текст на сегменти: при агентному режимі — з segments.json
    let save_dir = std::path::Path::new(&settings.save_path);
    let segments = if is_agent_mode {
        match read_segments_from_timeline(save_dir) {
            Ok(segs) if !segs.is_empty() => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("Agent mode: {} segments from segments.json", segs.len()),
                );
                segs
            }
            _ => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Agent mode: segments.json not ready, using text split.",
                );
                crate::core::pipeline::timeline::text_splitter::split_text(
                    &source_text,
                    &settings.text_split_mode,
                    settings.text_split_char_limit,
                )
            }
        }
    } else {
        crate::core::pipeline::timeline::text_splitter::split_text(
            &source_text,
            &settings.text_split_mode,
            settings.text_split_char_limit,
        )
    };

    let total = segments.len();
    crate::logger::log_job(
        job_id,
        &job_name,
        &format!(
            "Text split into {} segments (mode: {})",
            total, settings.text_split_mode
        ),
    );

    let contexts = if settings.video_context_enabled && !is_agent_mode {
        build_video_contexts(
            &source_text,
            &segments,
            &settings.video_context_mode,
            settings.video_context_chars,
        )
    } else {
        vec![String::new(); total]
    };

    // Зберігаємо debug-файл сегментів
    let segments_path = save_dir.join("segments.txt");
    {
        let mut content = format!("=== Сегменти тексту: {} ===\n", total);
        for (i, seg) in segments.iter().enumerate() {
            content.push_str(&format!("\n[{}/{}]\n{}\n", i + 1, total, seg));
        }
        let _ = std::fs::write(&segments_path, &content);
        crate::logger::log_job(job_id, &job_name, "Segments saved: segments.txt");
    }

    // Фаза 1: будуємо промти для всіх сегментів
    // В агентному режимі LLM не викликається — video_prompt підставляється напряму
    let use_llm = !is_agent_mode
        && settings.video_llm_service != "None"
        && !settings.video_llm_service.is_empty();
    if use_llm {
        crate::logger::log_job(
            job_id,
            &job_name,
            &format!(
                "Generating prompts via {} for {} segments (parallel)...",
                settings.video_llm_service, total
            ),
        );
    } else {
        crate::logger::log_job(
            job_id,
            &job_name,
            &format!("Building prompts for {} segments...", total),
        );
    }
    *prompts_progress.lock().unwrap() = Some((0, total));
    ctx.request_repaint();

    let substituted_video_prompts: Vec<String> = if !is_agent_mode {
        segments
            .iter()
            .enumerate()
            .map(|(i, segment)| {
                fill_video_prompt(
                    &settings.video_prompt,
                    segment,
                    contexts.get(i).map(String::as_str),
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    if !is_agent_mode {
        write_prompt_substitution_debug(
            save_dir,
            "video_prompt_substitution_debug.txt",
            if use_llm {
                "Video LLM requests after placeholder substitution"
            } else {
                "Direct media prompts after placeholder substitution"
            },
            &settings.video_prompt,
            &segments,
            &substituted_video_prompts,
        );
        crate::logger::log_job(
            job_id,
            &job_name,
            "Debug prompts saved: video_prompt_substitution_debug.txt",
        );
    }

    // Резервуємо масив промтів з правильним порядком за індексом
    let mut prompts: Vec<String> = vec![String::new(); total];

    if use_llm {
        // Паралельна генерація: кожен сегмент в окремому потоці
        // Обмеження паралельності виконується всередині call_llm через глобальний лімітер
        let mut handles: Vec<std::thread::JoinHandle<(usize, String, Option<f64>)>> =
            Vec::with_capacity(total);

        for (i, segment) in segments.iter().enumerate() {
            let segment = segment.clone();
            let llm_service = settings.video_llm_service.clone();
            let openrouter_key = settings.openrouter_key.clone();
            let llm_model = settings.video_llm_model.clone();
            let user_prompt = substituted_video_prompts
                .get(i)
                .cloned()
                .unwrap_or_else(|| fill_video_prompt(&settings.video_prompt, &segment, None));
            let llm_temperature = settings.video_llm_temperature;
            let save_path = settings.save_path.clone();
            let prompts_progress_c = Arc::clone(&prompts_progress);
            let ctx_c = ctx.clone();
            let job_id_c = job_id;
            let job_name_c = job_name.clone();

            handles.push(std::thread::spawn(move || {
                let (prompt, cost) = match crate::core::llm::call_llm(
                    &llm_service,
                    &openrouter_key,
                    &llm_model,
                    "",
                    &user_prompt,
                    llm_temperature,
                    Some((job_id_c, job_name_c.clone())),
                    Some(save_path.as_str()),
                    false,
                ) {
                    Ok((generated, cost)) => (generated, cost),
                    Err(e) => {
                        crate::logger::log_job(
                            job_id_c,
                            &job_name_c,
                            &format!(
                                "LLM prompt {}/{} error: {}. Using fallback.",
                                i + 1,
                                total,
                                e
                            ),
                        );
                        // Fallback: проста підстановка
                        (user_prompt, None)
                    }
                };

                if let Ok(mut pp) = prompts_progress_c.lock() {
                    if let Some((ref mut done, _)) = *pp {
                        *done += 1;
                    }
                }
                ctx_c.request_repaint();

                (i, prompt, cost)
            }));
        }

        // Збираємо результати, зберігаючи порядок за індексом; накопичуємо вартість LLM-запитів
        for handle in handles {
            if let Ok((i, prompt, cost)) = handle.join() {
                prompts[i] = prompt;
                if let Some(c) = cost {
                    let mut tc = total_cost.lock().unwrap();
                    *tc = Some(tc.unwrap_or(0.0) + c);
                }
            } else {
                crate::logger::log_job(job_id, &job_name, "LLM prompt thread panicked.");
            }
        }
    } else {
        // Без ЛЛМ — в агентному режимі text йде напряму, інакше підстановка в video_prompt
        for (i, segment) in segments.iter().enumerate() {
            prompts[i] = if is_agent_mode
                && settings.video_style_enabled
                && !settings.video_style_prompt.is_empty()
            {
                settings.video_style_prompt.replace("{{text}}", segment)
            } else if is_agent_mode {
                segment.clone()
            } else {
                substituted_video_prompts
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| fill_video_prompt(&settings.video_prompt, segment, None))
            };
            if let Ok(mut pp) = prompts_progress.lock() {
                if let Some((ref mut done, _)) = *pp {
                    *done += 1;
                }
            }
            ctx.request_repaint();
        }
    }
    crate::logger::log_job(
        job_id,
        &job_name,
        "Prompts ready. Starting media generation...",
    );

    // Фаза 2: паралельна генерація медіа із семафором
    let use_video = settings.video_media_type == "video";

    let media_dir = std::path::Path::new(&settings.save_path).join("media");
    if let Err(e) = std::fs::create_dir_all(&media_dir) {
        crate::logger::log_job(
            job_id,
            &job_name,
            &format!("Cannot create media/ dir: {}", e),
        );
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(e.to_string());
    }

    // Зберігаємо промти та сирі тексти сегментів для порівняння при перебудові
    let prompts_path = media_dir.join("prompts.json");
    if let Ok(json) = serde_json::to_string_pretty(&prompts) {
        let _ = std::fs::write(&prompts_path, json);
    }
    if let Ok(json) = serde_json::to_string_pretty(&segments) {
        let _ = std::fs::write(media_dir.join("segment_texts.json"), json);
    }

    *media_progress.lock().unwrap() = Some((0, total));
    ctx.request_repaint();

    let sem = Arc::new(Semaphore::new(settings.googler_image_max_threads.max(1)));
    let mut handles = Vec::new();

    for (i, prompt) in prompts.into_iter().enumerate() {
        let sem = Arc::clone(&sem);
        let media_progress_c = Arc::clone(&media_progress);
        let ctx_c = ctx.clone();
        let key = settings.googler_key.clone();
        let priority = if use_video {
            settings.googler_video_priority.clone()
        } else {
            settings.googler_image_priority.clone()
        };
        let media_dir = media_dir.clone();
        let job_id_c = job_id;
        let job_name_c = job_name.clone();

        let upscale_enabled = settings.googler_video_upscale_enabled;
        let upscale_resolution = settings.googler_video_upscale_resolution.clone();
        let upscale_quality = settings.googler_video_upscale_quality.clone();
        let skip_existing = settings.skip_existing_media;

        let handle = std::thread::spawn(move || -> (usize, Result<(), String>) {
            sem.acquire();

            // Режим догенерації: якщо файл вже є — пропускаємо
            if skip_existing && media_file_exists_by_index(&media_dir, i + 1) {
                crate::logger::log_job(
                    job_id_c,
                    &job_name_c,
                    &format!(
                        "Segment {}/{}: file already exists, skipping.",
                        i + 1,
                        total
                    ),
                );
                if let Ok(mut pp) = media_progress_c.lock() {
                    if let Some((ref mut done, _)) = *pp {
                        *done += 1;
                    }
                }
                ctx_c.request_repaint();
                sem.release();
                return (i, Ok(()));
            }

            if !use_video {
                crate::logger::log_job(
                    job_id_c,
                    &job_name_c,
                    &format!("Generating image {}/{} ...", i + 1, total),
                );
            }

            if prompt.trim().is_empty() {
                crate::logger::log_job(
                    job_id_c,
                    &job_name_c,
                    &format!("Segment {}/{}: empty prompt, skipping.", i + 1, total),
                );
                return (i, Err("Порожній промпт — пропущено".to_string()));
            }

            let result = if use_video {
                crate::api::googler::generate_video_with_priority_logged(
                    &key,
                    &prompt,
                    "16:9",
                    &priority,
                    |provider| {
                        crate::logger::log_job(
                            job_id_c,
                            &job_name_c,
                            &format!(
                                "Generating video {}/{} ... (модель: {})",
                                i + 1,
                                total,
                                crate::api::googler::video_provider_model_name(provider)
                            ),
                        );
                    },
                )
                .map(|(p, d)| (p, d))
            } else {
                crate::api::googler::generate_image_with_priority(&key, &prompt, "16:9", &priority)
                    .map(|(p, d)| (p, d))
            };

            sem.release();

            match result {
                Err(e) => (i, Err(e)),
                Ok((provider_used, data_uri)) => {
                    match decode_result(&data_uri, i + 1, total, &media_dir) {
                        Err(e) => (i, Err(e)),
                        Ok(path) => {
                            crate::logger::log_job(
                                job_id_c,
                                &job_name_c,
                                &format!(
                                    "{} {}/{} saved: {} (провайдер: {})",
                                    if use_video { "Video" } else { "Image" },
                                    i + 1,
                                    total,
                                    path.display(),
                                    provider_used
                                ),
                            );
                            if use_video {
                                let is_omni = provider_used == "flow_omni_flash";
                                if let Err(err) = upscale_video_if_needed(
                                    &path,
                                    upscale_enabled,
                                    &upscale_resolution,
                                    &upscale_quality,
                                    is_omni,
                                    job_id_c,
                                    &job_name_c,
                                ) {
                                    crate::logger::log_job(
                                        job_id_c,
                                        &job_name_c,
                                        &format!(
                                            "Помилка апскейлу для сегмента {}: {}",
                                            i + 1,
                                            err
                                        ),
                                    );
                                }
                            }

                            if let Ok(mut mp) = media_progress_c.lock() {
                                if let Some((ref mut done, _)) = *mp {
                                    *done += 1;
                                }
                            }
                            ctx_c.request_repaint();
                            (i, Ok(()))
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Збираємо результати
    let mut errors: Vec<String> = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok((i, Err(e))) => {
                let msg = format!("Segment {}: {}", i + 1, e);
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("{} error — {}", media_label, msg),
                );
                errors.push(msg);
            }
            Err(_) => errors.push(format!("Thread panic during {} generation", media_label)),
            _ => {}
        }
    }

    if errors.is_empty() {
        crate::logger::log_job(
            job_id,
            &job_name,
            &format!("All {}s generated successfully.", media_label),
        );
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Done;
        ctx.request_repaint();
        Ok(())
    } else {
        let msg = format!(
            "{} generation failed ({} errors). First: {}",
            media_label,
            errors.len(),
            errors[0]
        );
        crate::logger::log_job(job_id, &job_name, &msg);
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        Err(msg)
    }
}

/// Гілка Pexels Stock: генерує ключові слова → шукає медіа → зберігає stock_cache.json.
/// Замість генерації медіафайлів одразу, результати чекають вибору користувача.
fn run_pexels_branch(
    job_id: u64,
    job_name: String,
    settings: &crate::queue::JobSettings,
    translated_text: Arc<Mutex<Option<String>>>,
    video_stage: Arc<Mutex<crate::queue::StageStatus>>,
    prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    ctx: egui::Context,
) -> Result<(), String> {
    crate::logger::log_job(
        job_id,
        &job_name,
        "Pexels: starting keyword generation + stock search...",
    );
    *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    let save_dir = std::path::Path::new(&settings.save_path);

    // 1. Отримуємо сегменти + ключові слова
    // Агентний режим (є segments.json): ключові слова = перші 5 слів кожного сегмента (без LLM)
    // Звичайний режим: ключові слова через LLM або перші 5 слів
    let (segments, keywords): (Vec<String>, Vec<String>) = {
        match read_segments_from_timeline(save_dir) {
            Ok(segs) if !segs.is_empty() => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("Pexels: using {} segments from segments.json", segs.len()),
                );
                // Агент написав описи сцен — використовуємо напряму як запит до Pexels
                let kws = segs.clone();
                (segs, kws)
            }
            _ => {
                let source_text = if settings.translation_enabled {
                    translated_text
                        .lock()
                        .unwrap()
                        .clone()
                        .unwrap_or_else(|| settings.text.clone())
                } else {
                    settings.text.clone()
                };
                let segs = crate::core::pipeline::timeline::text_splitter::split_text(
                    &source_text,
                    &settings.text_split_mode,
                    settings.text_split_char_limit,
                );

                let pexels_agent_mode = settings.video_llm_service == "Claude Code"
                    || settings.video_llm_service == "Gemini CLI"
                    || settings.video_llm_service == "Codex CLI"
                    || settings.video_llm_service == "AGY CLI"
                    || settings.video_llm_service == "Pi CLI";
                let contexts = if settings.video_context_enabled && !pexels_agent_mode {
                    build_video_contexts(
                        &source_text,
                        &segs,
                        &settings.video_context_mode,
                        settings.video_context_chars,
                    )
                } else {
                    vec![String::new(); segs.len()]
                };

                let use_llm =
                    settings.video_llm_service != "None" && !settings.video_llm_service.is_empty();
                let kws = if use_llm {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Pexels: generating keywords via {}...",
                            settings.video_llm_service
                        ),
                    );
                    let kw_instruction = if settings.video_prompt.is_empty() {
                        "Generate 3-5 short English search keywords for a stock footage website based on this text. Return only keywords separated by space, no explanation.".to_string()
                    } else {
                        settings.video_prompt.clone()
                    };
                    let total_segs = segs.len();
                    let substituted_keyword_prompts: Vec<String> = segs
                        .iter()
                        .enumerate()
                        .map(|(i, seg)| {
                            fill_video_prompt(
                                &kw_instruction,
                                seg,
                                contexts.get(i).map(String::as_str),
                            )
                        })
                        .collect();
                    write_prompt_substitution_debug(
                        save_dir,
                        "stock_keyword_prompt_substitution_debug.txt",
                        "Stock keyword LLM requests after placeholder substitution",
                        &kw_instruction,
                        &segs,
                        &substituted_keyword_prompts,
                    );
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Debug prompts saved: stock_keyword_prompt_substitution_debug.txt",
                    );

                    let mut handles = Vec::with_capacity(total_segs);
                    for (i, seg) in segs.iter().enumerate() {
                        let service = settings.video_llm_service.clone();
                        let model = settings.video_llm_model.clone();
                        let key = settings.openrouter_key.clone();
                        let temp = settings.video_llm_temperature;
                        let seg_clone = seg.clone();
                        let user_prompt = substituted_keyword_prompts
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| fill_video_prompt(&kw_instruction, &seg_clone, None));
                        let video_stage_c = Arc::clone(&video_stage);
                        let prompts_progress_c = Arc::clone(&prompts_progress);
                        let ctx_c = ctx.clone();
                        let jn = job_name.clone();
                        handles.push(std::thread::spawn(move || {
                            let kw = crate::core::llm::call_llm(
                                &service,
                                &key,
                                &model,
                                "",
                                &user_prompt,
                                temp,
                                Some((job_id, jn.clone())),
                                None,
                                false,
                            )
                            .map(|(text, _)| text)
                            .unwrap_or_else(|_| {
                                seg_clone
                                    .split_whitespace()
                                    .take(5)
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            });
                            let kw = kw.trim().to_string();
                            if let Ok(mut p) = prompts_progress_c.try_lock() {
                                if let Some((done, t)) = p.as_mut() {
                                    *done += 1;
                                    let _ = crate::logger::log_job(
                                        job_id,
                                        &jn,
                                        &format!("Pexels keyword {}/{}: \"{}\"", done, t, kw),
                                    );
                                }
                            }
                            *video_stage_c.lock().unwrap() = crate::queue::StageStatus::Running;
                            ctx_c.request_repaint();
                            (i, kw)
                        }));
                    }
                    let mut out = vec![String::new(); total_segs];
                    for h in handles {
                        let (i, kw) = h.join().unwrap_or((0, String::new()));
                        out[i] = kw;
                    }
                    out
                } else {
                    segs.iter()
                        .map(|s| s.split_whitespace().take(5).collect::<Vec<_>>().join(" "))
                        .collect()
                };
                (segs, kws)
            }
        }
    };

    let total = segments.len();

    // Тривалості сегментів з segments.json (якщо є)
    let seg_durations: Vec<f32> =
        read_segment_durations_from_timeline(save_dir).unwrap_or_else(|| vec![0.0; total]);

    // Зберігаємо skeleton cache — ключові слова без результатів пошуку.
    // Pexels пошук запускається лінивo в GUI при кліку на сегмент.
    let cache: Vec<crate::api::stock::SegmentCache> = segments
        .iter()
        .zip(keywords.iter())
        .zip(seg_durations.iter())
        .enumerate()
        .map(|(i, ((seg, kw), dur))| crate::api::stock::SegmentCache {
            index: i,
            keyword: kw.clone(),
            segment_text: seg.clone(),
            segment_duration: *dur,
            photos: vec![],
            videos: vec![],
            selected: None,
        })
        .collect();

    crate::api::stock::save_cache(save_dir, &cache)
        .map_err(|e| format!("Pexels cache save error: {}", e))?;

    crate::logger::log_job(
        job_id,
        &job_name,
        &format!(
            "Pexels: skeleton cache saved ({} segments). Waiting for user selection...",
            total
        ),
    );
    *video_stage.lock().unwrap() = crate::queue::StageStatus::Done;
    ctx.request_repaint();
    Ok(())
}

/// Читає сегменти тексту з segments.json (для агентного режиму).
fn read_segments_from_timeline(save_dir: &std::path::Path) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(save_dir.join("segments.json"))
        .map_err(|e| format!("Cannot read segments.json: {}", e))?;
    let timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|e| format!("Invalid segments.json: {}", e))?;
    Ok(timeline.segments.into_iter().map(|s| s.text).collect())
}

/// Читає тривалості сегментів з segments.json
fn read_segment_durations_from_timeline(save_dir: &std::path::Path) -> Option<Vec<f32>> {
    let content = std::fs::read_to_string(save_dir.join("segments.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let segs = v["segments"].as_array()?;
    Some(
        segs.iter()
            .map(|s| {
                let start = s["start_secs"].as_f64().unwrap_or(0.0);
                let end = s["end_secs"].as_f64().unwrap_or(0.0);
                (end - start).max(0.0) as f32
            })
            .collect(),
    )
}
