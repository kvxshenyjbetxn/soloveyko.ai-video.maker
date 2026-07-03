use std::sync::{Arc, Mutex};

use eframe::egui;

/// Зчитує збережений промт для конкретного медіафайлу з prompts.json.
/// Індекс визначається з імені файлу (0001.jpg → індекс 0).
/// Анімує одне зображення у відео у фоновому потоці (image-to-video).
/// Зчитує файл як base64 data URI, відправляє на Googler, зберігає .mp4,
/// видаляє оригінальне зображення. Прибирає шлях з loading_set після завершення.
pub fn animate_single_image(
    file_path: std::path::PathBuf,
    priority: Vec<String>,
    googler_key: String,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    loading_set: Arc<Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: String,
    googler_video_upscale_quality: String,
) {
    std::thread::spawn(move || {
        loading_set.lock().unwrap().insert(file_path.clone());
        ctx.request_repaint();

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let result = (|| -> Result<std::path::PathBuf, String> {
            let bytes =
                std::fs::read(&file_path).map_err(|e| format!("Помилка читання файлу: {}", e))?;

            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_lowercase();
            let mime = match ext.as_str() {
                "png" => "image/png",
                "webp" => "image/webp",
                _ => "image/jpeg",
            };
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let data_uri = format!("data:{};base64,{}", mime, b64);

            let saved_prompt = read_prompt_for_file(&file_path);
            let prompt = if saved_prompt.is_empty() {
                "Animate this image with smooth, natural motion.".to_string()
            } else {
                saved_prompt
            };
            let (anim_provider, api_result) = crate::api::googler::animate_image_with_priority(
                &googler_key,
                &data_uri,
                &prompt,
                &priority,
                |provider| {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Animate {}: старт — {}",
                            file_name,
                            crate::api::googler::video_provider_model_name(provider)
                        ),
                    );
                },
                Some(job_id),
            )?;

            // Зберігаємо відео поряд з оригінальним зображенням (.mp4)
            let video_path = file_path.with_extension("mp4");
            save_media_bytes(&api_result, &video_path)?;

            let is_omni = anim_provider == "flow_omni_flash";
            if let Err(e) = upscale_video_if_needed(
                &video_path,
                googler_video_upscale_enabled,
                &googler_video_upscale_resolution,
                &googler_video_upscale_quality,
                is_omni,
                job_id,
                &job_name,
            ) {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!(
                        "Помилка апскейлу/кропу анімованого відео {}: {}",
                        file_name, e
                    ),
                );
            }

            // Видаляємо оригінальне зображення

            if video_path != file_path {
                let _ = std::fs::remove_file(&file_path);
            }

            Ok(video_path)
        })();

        match &result {
            Ok(out) => crate::logger::log_job(
                job_id,
                &job_name,
                &format!(
                    "Animate {} → {} готово",
                    file_name,
                    out.file_name().unwrap_or_default().to_string_lossy()
                ),
            ),
            Err(e) => crate::logger::log_job(
                job_id,
                &job_name,
                &format!("Animate {} помилка: {}", file_name, e),
            ),
        }

        loading_set.lock().unwrap().remove(&file_path);
        ctx.request_repaint();
    });
}

pub(crate) fn read_prompt_for_file(file_path: &std::path::Path) -> String {
    let media_dir = match file_path.parent() {
        Some(d) => d,
        None => return String::new(),
    };
    let index = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0);

    std::fs::read_to_string(media_dir.join("prompts.json"))
        .ok()
        .and_then(|c| serde_json::from_str::<Vec<String>>(&c).ok())
        .and_then(|v| v.into_iter().nth(index))
        .unwrap_or_default()
}

/// Зберігає байти медіа (data URI або HTTP URL) у вказаний файл, перезаписуючи його.
fn save_media_bytes(data_uri: &str, file_path: &std::path::Path) -> Result<(), String> {
    let bytes = if data_uri.starts_with("data:") {
        let rest = &data_uri[5..];
        let comma = rest.find(',').ok_or("Invalid data URI: no comma")?;
        let b64 = &rest[comma + 1..];
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?
    } else {
        use std::io::Read;
        let resp = ureq::get(data_uri)
            .call()
            .map_err(|e| format!("Download error: {}", e))?;
        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        buf
    };
    std::fs::write(file_path, &bytes).map_err(|e| format!("Save error: {}", e))
}

/// Перегенерує один медіафайл у фоновому потоці.
/// Якщо custom_prompt = None або порожній — читає збережений промт з prompts.json.
pub fn regenerate_single_media(
    file_path: std::path::PathBuf,
    media_type: String,
    priority: Vec<String>,
    googler_key: String,
    custom_prompt: Option<String>,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    result_slot: Arc<Mutex<Option<Result<(), String>>>>,
    loading: Arc<Mutex<bool>>,
    // Якщо передано — файл додається в набір "завантажується" (для підтримки паралельних регенерацій)
    path_loading_set: Option<Arc<Mutex<std::collections::HashSet<std::path::PathBuf>>>>,
    // Якщо передано — результат також потрапляє у чергу (для обробки паралельних результатів)
    results_queue: Option<Arc<Mutex<Vec<(std::path::PathBuf, Result<(), String>)>>>>,
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: String,
    googler_video_upscale_quality: String,
) {
    std::thread::spawn(move || {
        *loading.lock().unwrap() = true;
        if let Some(ref set) = path_loading_set {
            set.lock().unwrap().insert(file_path.clone());
        }

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let prompt = custom_prompt
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| read_prompt_for_file(&file_path));

        crate::logger::log_job(
            job_id,
            &job_name,
            &format!(
                "Regen {}: {} (prompt: {}...)",
                media_type,
                file_name,
                prompt.chars().take(60).collect::<String>()
            ),
        );

        let api_result = if media_type == "video" {
            crate::api::googler::generate_video_with_priority_logged(
                &googler_key,
                &prompt,
                "16:9",
                &priority,
                |provider| {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Regen {}: модель — {}",
                            file_name,
                            crate::api::googler::video_provider_model_name(provider)
                        ),
                    );
                },
                Some(job_id),
            )
        } else {
            crate::api::googler::generate_image_with_priority(
                &googler_key,
                &prompt,
                "16:9",
                &priority,
                Some(job_id),
            )
        };

        // Для відео генеруємо з правильним розширенням (.mp4), аналогічно animate_single_image
        let mut file_path = file_path;
        let save_path = if media_type == "video" {
            file_path.with_extension("mp4")
        } else {
            file_path.clone()
        };

        let outcome = match api_result {
            Err(e) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("Regen {} failed: {}", file_name, e),
                );
                Err(e)
            }
            Ok((provider_used, data_uri)) => match save_media_bytes(&data_uri, &save_path) {
                Ok(()) => {
                    // Якщо розширення змінилося (.jpg → .mp4) — видаляємо оригінальний файл
                    if save_path != file_path {
                        let _ = std::fs::remove_file(&file_path);
                        file_path = save_path.clone();
                    }
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!("Regen {} done (провайдер: {}).", file_name, provider_used),
                    );
                    if media_type == "video" {
                        let is_omni = provider_used == "flow_omni_flash";
                        if let Err(e) = upscale_video_if_needed(
                            &file_path,
                            googler_video_upscale_enabled,
                            &googler_video_upscale_resolution,
                            &googler_video_upscale_quality,
                            is_omni,
                            job_id,
                            &job_name,
                        ) {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!(
                                    "Помилка апскейлу/кропу перегенерованого відео {}: {}",
                                    file_name, e
                                ),
                            );
                        }
                    }
                    Ok(())
                }

                Err(e) => {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!("Regen {} save error: {}", file_name, e),
                    );
                    Err(e)
                }
            },
        };

        if let Some(ref q) = results_queue {
            q.lock().unwrap().push((file_path.clone(), outcome.clone()));
        }
        *result_slot.lock().unwrap() = Some(outcome);
        *loading.lock().unwrap() = false;
        if let Some(ref set) = path_loading_set {
            set.lock().unwrap().remove(&file_path);
        }
        ctx.request_repaint();
    });
}

/// Порівнює тексти сегментів у segments.json з segment_texts.json (сирі тексти до стилю).
/// Повертає список (шлях до медіафайлу, готовий промт) для сегментів де текст змінився.
/// Одразу оновлює segment_texts.json — щоб наступний rebuild порівнював з новим станом.
/// Повний промт будується як: video_style_prompt.replace("{{text}}", new_text).
pub fn find_changed_prompts_for_rebuild(
    save_dir: &std::path::Path,
    video_style_enabled: bool,
    video_style_prompt: &str,
) -> Vec<(std::path::PathBuf, String)> {
    let timeline_path = save_dir.join("segments.json");
    let content = match std::fs::read_to_string(&timeline_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let segs = match v["segments"].as_array() {
        Some(s) => s,
        None => return vec![],
    };

    // Витягуємо тексти з segments.json (лише pipeline-формат має поле "text")
    let new_texts: Vec<String> = segs
        .iter()
        .map(|seg| seg["text"].as_str().unwrap_or("").to_string())
        .collect();

    if new_texts.iter().all(|t| t.is_empty()) {
        return vec![]; // editor-формат без полів text — нічого порівнювати
    }

    let media_dir = save_dir.join("media");

    // Старі тексти сегментів на момент останньої генерації/перебудови
    let old_texts: Vec<String> = std::fs::read_to_string(media_dir.join("segment_texts.json"))
        .ok()
        .and_then(|c| serde_json::from_str::<Vec<String>>(&c).ok())
        .unwrap_or_default();

    let mut changed = vec![];

    for (i, new_text) in new_texts.iter().enumerate() {
        if new_text.trim().is_empty() {
            continue;
        }

        let old_text = old_texts.get(i).map(|s| s.as_str()).unwrap_or("");

        if new_text.trim() != old_text.trim() {
            // Використовуємо реальний шлях медіа з сегменту, а не сортований індекс файлу.
            // Якщо media == null (агент не зберіг шляхи) — не регенеруємо нічого.
            if let Some(media_str) = segs[i]["media"].as_str() {
                let file_path = save_dir.join(media_str);
                if file_path.exists() {
                    // Будуємо повний промт так само як при початковій генерації
                    let full_prompt = if video_style_enabled && !video_style_prompt.is_empty() {
                        if video_style_prompt.contains("{{text}}") {
                            video_style_prompt.replace("{{text}}", new_text)
                        } else {
                            format!("{}\n\n{}", video_style_prompt, new_text)
                        }
                    } else {
                        new_text.clone()
                    };
                    changed.push((file_path, full_prompt));
                }
            }
        }
    }

    // Одразу оновлюємо segment_texts.json — щоб наступний rebuild порівнював з поточним станом
    if !changed.is_empty() {
        if let Ok(json) = serde_json::to_string_pretty(&new_texts) {
            let _ = std::fs::write(media_dir.join("segment_texts.json"), json);
        }
    }

    changed
}

/// Виконує апскейл та кроп відеофайлу за допомогою FFmpeg.
/// Робить це in-place: перейменовує файл у тимчасовий, запускає FFmpeg,
/// записує результат у оригінальний шлях, видаляє тимчасовий файл.
pub fn upscale_video_if_needed(
    video_path: &std::path::Path,
    enabled: bool,
    resolution: &str,
    quality: &str,
    is_omni: bool,
    job_id: u64,
    job_name: &str,
) -> Result<(), String> {
    if !video_path.exists() {
        return Err(format!("Файл не існує: {}", video_path.display()));
    }

    crate::logger::log_job(
        job_id,
        job_name,
        &format!(
            "Обробка відео (апскейл: {}, роздільна здатність: {}, кроп: 107% (дефолт){}, якість: {})...",
            enabled,
            resolution,
            if is_omni {
                " + omni watermark crop 10%"
            } else {
                ""
            },
            quality
        ),
    );

    // Створюємо шлях для тимчасового файлу
    let temp_path = video_path.with_extension("upscale_temp.mp4");
    if let Err(e) = std::fs::rename(video_path, &temp_path) {
        return Err(format!("Не вдалося перейменувати файл для апскейлу: {}", e));
    }

    // Виконуємо ffprobe для зчитування FPS та розмірів відео
    let ffprobe_cmd = crate::bundle::ffprobe_path();
    let mut ffprobe_proc = std::process::Command::new(&ffprobe_cmd);
    ffprobe_proc
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate,avg_frame_rate,nb_frames,duration",
            "-of",
            "csv=p=0",
        ])
        .arg(&temp_path);
    crate::bundle::set_no_window(&mut ffprobe_proc);
    let ffprobe_out = crate::api::process::output_tracked(&mut ffprobe_proc, Some(job_id));

    let mut width = 1280;
    let mut height = 720;
    let mut fps = 30.0;
    if let Ok(out) = ffprobe_out {
        let s = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = s.trim().split(',').collect();
        if parts.len() >= 4 {
            width = parts[0].trim().parse().unwrap_or(1280);
            height = parts[1].trim().parse().unwrap_or(720);
            let r_fps = parts[2];
            let avg_fps = parts[3];

            let mut calculated_fps = None;
            if parts.len() >= 6 {
                let nb_frames: f64 = parts[4].trim().parse().unwrap_or(0.0);
                let duration: f64 = parts[5].trim().parse().unwrap_or(0.0);
                if duration > 0.0 && nb_frames > 0.0 {
                    calculated_fps = Some(nb_frames / duration);
                }
            }

            let parsed_fps = calculated_fps.unwrap_or_else(|| {
                let rate = if avg_fps != "0/0" && !avg_fps.is_empty() {
                    avg_fps
                } else {
                    r_fps
                };
                if rate.contains('/') {
                    let subparts: Vec<&str> = rate.split('/').collect();
                    if subparts.len() == 2 {
                        let num: f64 = subparts[0].trim().parse().unwrap_or(30.0);
                        let den: f64 = subparts[1].trim().parse().unwrap_or(1.0);
                        if den > 0.0 { num / den } else { 30.0 }
                    } else {
                        30.0
                    }
                } else {
                    rate.trim().parse().unwrap_or(30.0)
                }
            });

            if parsed_fps > 1.0 && parsed_fps < 120.0 {
                fps = (parsed_fps * 1000.0).round() / 1000.0;
            }
        }
    }

    let (target_w, target_h) = if enabled {
        match resolution {
            "2K" => (2560, 1440),
            "4K" => (3840, 2160),
            _ => (1920, 1080), // 1080p
        }
    } else {
        (width, height)
    };

    let sharpen = if enabled {
        match quality {
            "fast" => "unsharp=5:5:0.55:3:3:0.25".to_string(),
            "max" => "hqdn3d=1.5:1.5:5:5,unsharp=7:7:0.85:5:5:0.4".to_string(),
            _ => "hqdn3d=1.2:1.2:4:4,unsharp=5:5:0.75:5:5:0.35".to_string(), // balanced
        }
    } else {
        "".to_string()
    };

    let ffmpeg_preset = if enabled {
        match quality {
            "fast" => "veryfast",
            "max" => "slow",
            _ => "medium", // balanced
        }
    } else {
        "ultrafast"
    };

    let crf = if enabled {
        match quality {
            "fast" => "20",
            "max" => "16",
            _ => "18", // balanced
        }
    } else {
        "18"
    };

    let scale_w = ((target_w as f64 * 1.07).round() as i32) & !1;
    let scale_h = ((target_h as f64 * 1.07).round() as i32) & !1;
    let fit = format!(
        "scale={}:{}:flags=lanczos:force_original_aspect_ratio=increase,crop={}:{}:iw-{}:0",
        scale_w, scale_h, target_w, target_h, target_w
    );

    // Для omni-провайдера спочатку прибираємо вотермарку (10% знизу і справа)
    let omni_crop = if is_omni {
        "crop=iw*0.89:ih*0.89:0:0,"
    } else {
        ""
    };
    let vf = if sharpen.is_empty() {
        format!("{}setpts=N/({}*TB),{}", omni_crop, fps, fit)
    } else {
        format!("{}setpts=N/({}*TB),{},{}", omni_crop, fps, fit, sharpen)
    };
    let fps_str = format!("{}", fps);

    let ffmpeg_cmd = crate::bundle::ffmpeg_path();
    let mut args = vec![
        "-y",
        "-hide_banner",
        "-fflags",
        "+genpts",
        "-i",
        temp_path.to_str().unwrap(),
        "-vf",
        &vf,
        "-r",
        &fps_str,
        "-fps_mode",
        "cfr",
        "-c:v",
        "libx264",
        "-preset",
        ffmpeg_preset,
        "-crf",
        crf,
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
    ];

    if !enabled {
        args.extend_from_slice(&["-threads", "2"]);
    }

    args.extend_from_slice(&[
        "-map", "0:v:0", "-map", "0:a?", "-c:a", "aac", "-b:a", "192k",
    ]);
    // Завжди виводимо у тимчасовий .mp4, щоб FFmpeg не плутався з розширенням вихідного файлу
    let out_temp = video_path.with_extension("upscale_out.mp4");
    args.push(out_temp.to_str().unwrap());

    let mut ffmpeg_upscale_proc = std::process::Command::new(&ffmpeg_cmd);
    ffmpeg_upscale_proc.args(&args);
    crate::bundle::set_no_window(&mut ffmpeg_upscale_proc);
    let child = crate::api::process::output_tracked(&mut ffmpeg_upscale_proc, Some(job_id));

    let restore_original = || {
        if temp_path.exists() {
            if video_path.exists() {
                let _ = std::fs::remove_file(video_path);
            }
            let _ = std::fs::rename(&temp_path, video_path);
        }
    };

    let clean_up = || {
        let _ = std::fs::remove_file(&temp_path);
    };

    match child {
        Ok(output) => {
            if output.status.success() {
                // Перейменовуємо тимчасовий результат в оригінальний шлях
                if video_path.exists() {
                    let _ = std::fs::remove_file(video_path);
                }
                if let Err(_) = std::fs::rename(&out_temp, video_path) {
                    // fallback: копіювання + видалення (напр. різні диски)
                    let _ = std::fs::copy(&out_temp, video_path);
                    let _ = std::fs::remove_file(&out_temp);
                }
                clean_up();
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!(
                        "Апскейл/кроп завершено успішно: {}",
                        video_path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                );
                Ok(())
            } else {
                restore_original();
                let _ = std::fs::remove_file(&out_temp);
                let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("FFmpeg error: {}", err_msg.trim()))
            }
        }
        Err(e) => {
            restore_original();
            let _ = std::fs::remove_file(&out_temp);
            Err(format!("Не вдалося запустити FFmpeg: {}", e))
        }
    }
}
