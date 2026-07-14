use std::sync::{Arc, Condvar, Mutex};

use eframe::egui;

use super::agent_prompts;

/// Запускає агента для створення segments.json.
/// Після завершення роботи агента (успіх або помилка квоти) пайплайн переходить у стан
/// AwaitingAgentControl — користувач підтверджує продовження кнопкою «Підтвердити».
/// Це дозволяє дописати агенту «продовжи» через чат і лише тоді відновити пайплайн.
pub(super) fn run_agent_timeline(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let save_dir = std::path::Path::new(&settings.save_path);
    let srt_path = save_dir.join("subtitle.srt");

    if !srt_path.exists() {
        return Err("subtitle.srt not found (run subtitles stage first)".to_string());
    }

    let segments_path = save_dir.join("segments.json");
    let is_prompt_only = settings.is_prompt_only_agent_mode();
    let system_instruction = if is_prompt_only {
        agent_prompts::VIDEO_AGENT_SYSTEM_PROMPT_PROMPT_ONLY
    } else {
        agent_prompts::VIDEO_AGENT_SYSTEM_PROMPT_FULL
    }
    .replace("{{srt}}", &srt_path.to_string_lossy())
    .replace("{{path}}", &segments_path.to_string_lossy());
    let original_timeline = if is_prompt_only {
        Some(
            read_timeline_file(&segments_path)
                .map_err(|e| format!("Prompt Only: базовий segments.json не готовий: {}", e))?,
        )
    } else {
        None
    };
    let user_part = settings
        .video_agent_prompt
        .replace("{{srt}}", &srt_path.to_string_lossy())
        .replace("{{path}}", &segments_path.to_string_lossy());
    let agent_prompt = if user_part.trim().is_empty() {
        system_instruction
    } else {
        format!("{}\n\n{}", system_instruction, user_part)
    };

    crate::logger::log_job(
        job_id,
        job_name,
        &format!(
            "Agent ({}, mode={}): {} segments.json...",
            settings.video_llm_service,
            settings.video_agent_mode,
            if is_prompt_only {
                "updating"
            } else {
                "generating"
            }
        ),
    );

    let session_id = uuid_v4();
    crate::logger::log_job(job_id, job_name, &format!("Agent session: {}", session_id));

    // Зберігаємо сесію ДО запуску — чат стає доступним навіть якщо агент завершиться з помилкою
    *agent_session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
        session_id: session_id.clone(),
        service: settings.video_llm_service.clone(),
        model: settings.video_llm_model.clone(),
    });

    let initial_text = if settings.video_llm_service == "Codex CLI" {
        format!(
            "Running: codex exec --model {} --json\n\n",
            settings.video_llm_model
        )
    } else if settings.video_llm_service == "Claude Code" {
        format!(
            "Running: claude --model {} --session-id {}\n\n",
            settings.video_llm_model, session_id
        )
    } else {
        format!(
            "Running: {} --model {} --session-id {}\n\n",
            settings.video_llm_service, settings.video_llm_model, session_id
        )
    };
    agent_chat
        .lock()
        .unwrap()
        .push(crate::queue::AgentChatMessage {
            role: "agent".to_string(),
            content: initial_text,
        });
    ctx.request_repaint();

    let agent_chat_for_chunk = Arc::clone(&agent_chat);
    let ctx_for_chunk = ctx.clone();

    let agent_result = call_agent_new_session_streaming(
        &settings.video_llm_service,
        &settings.video_llm_model,
        &agent_prompt,
        &session_id,
        Some((job_id, job_name.to_string())),
        Some(&settings.save_path),
        move |chunk| {
            let mut chat = agent_chat_for_chunk.lock().unwrap();
            if let Some(last) = chat.last_mut() {
                last.content.push_str(chunk);
            }
            ctx_for_chunk.request_repaint();
        },
    );

    super::ensure_job_not_cancelled(job_id)?;
    save_agent_chat_to_file(save_dir, &agent_chat.lock().unwrap());

    match &agent_result {
        Ok((_, actual_session_id)) => {
            // Оновлюємо session_id на реальний (актуально для Codex CLI)
            if let Some(ref mut sess) = *agent_session.lock().unwrap() {
                sess.session_id = actual_session_id.clone();
            }
            // Оновлюємо заголовок для Codex CLI — підставляємо реальний thread ID
            if settings.video_llm_service == "Codex CLI" {
                let mut chat = agent_chat.lock().unwrap();
                if let Some(first) = chat.first_mut() {
                    let tail = first
                        .content
                        .find('\n')
                        .map(|p| first.content[p..].to_string())
                        .unwrap_or_default();
                    first.content = format!(
                        "Running: codex exec --model {} --json [Thread: {}]{}",
                        settings.video_llm_model, actual_session_id, tail
                    );
                }
            }
        }
        Err(e) => {
            crate::logger::log_job(
                job_id,
                job_name,
                &format!("Agent error: {} — чат доступний для продовження", e),
            );
        }
    }

    // Зберігаємо фінальний session_id на диск (для відновлення після перезапуску)
    if let Some(ref sess) = *agent_session.lock().unwrap() {
        save_agent_session_to_file(save_dir, sess);
    }

    // Якщо segments.json відсутній або невалідний — ставимо на паузу замість провалу.
    // Користувач може продовжити розмову в чаті та натиснути «Продовжити пайплайн».
    loop {
        let validation_err = if !segments_path.exists() {
            Some("segments.json не створено агентом".to_string())
        } else {
            match std::fs::read_to_string(&segments_path) {
                Err(e) => Some(format!("Не вдалося прочитати segments.json: {}", e)),
                Ok(content) => {
                    match serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(
                        &content,
                    ) {
                        Err(e) => Some(format!("segments.json невалідний: {}", e)),
                        Ok(current) => {
                            if let Some(original) = &original_timeline {
                                match merge_prompt_only_texts(original, &current) {
                                    Ok(merged) => {
                                        if let Err(e) = write_timeline_file(&segments_path, &merged)
                                        {
                                            Some(format!(
                                                "Не вдалося зберегти merged segments.json: {}",
                                                e
                                            ))
                                        } else {
                                            None
                                        }
                                    }
                                    Err(e) => Some(e),
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        };

        if let Some(err_msg) = validation_err {
            crate::logger::log_job(
                job_id,
                job_name,
                &format!(
                    "Agent: {} — очікуємо підтвердження від користувача...",
                    err_msg
                ),
            );
            *status.lock().unwrap() = crate::queue::JobStatus::AwaitingAgentControl;
            ctx.request_repaint();

            // Чекаємо поки користувач натисне «Продовжити пайплайн» у вікні чату
            let (lock, cvar) = &*agent_control_resume;
            let mut resumed = lock.lock().unwrap();
            while !*resumed {
                resumed = cvar.wait(resumed).unwrap();
            }
            *resumed = false;
            super::ensure_job_not_cancelled(job_id)?;

            crate::logger::log_job(job_id, job_name, "Продовжуємо перевірку segments.json...");
            *status.lock().unwrap() = crate::queue::JobStatus::Running;
            ctx.request_repaint();
            // Повторюємо перевірку
        } else {
            break;
        }
    }

    crate::logger::log_job(
        job_id,
        job_name,
        "Agent segments.json created and validated.",
    );
    ctx.request_repaint();
    Ok(())
}

/// Читає та валідує timeline-файл segments.json.
fn read_timeline_file(
    path: &std::path::Path,
) -> Result<crate::core::pipeline::timeline::sync::Timeline, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Не вдалося прочитати {}: {}", path.display(), e))?;
    serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
        .map_err(|e| format!("Невалідний JSON у {}: {}", path.display(), e))
}

/// Зберігає timeline назад у segments.json.
fn write_timeline_file(
    path: &std::path::Path,
    timeline: &crate::core::pipeline::timeline::sync::Timeline,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(timeline).map_err(|e| format!("JSON error: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))
}

/// У режимі Prompt Only зберігаємо оригінальну структуру та копіюємо назад лише text.
fn merge_prompt_only_texts(
    original: &crate::core::pipeline::timeline::sync::Timeline,
    edited: &crate::core::pipeline::timeline::sync::Timeline,
) -> Result<crate::core::pipeline::timeline::sync::Timeline, String> {
    if original.segments.len() != edited.segments.len() {
        return Err(format!(
            "Prompt Only: кількість сегментів змінилась (було {}, стало {})",
            original.segments.len(),
            edited.segments.len()
        ));
    }

    let mut merged = crate::core::pipeline::timeline::sync::Timeline {
        total_duration_secs: original.total_duration_secs,
        audio_start_secs: original.audio_start_secs,
        segments: original.segments.clone(),
    };

    for (dst, src) in merged.segments.iter_mut().zip(edited.segments.iter()) {
        dst.text = src.text.clone();
    }

    Ok(merged)
}

/// Генерує простий UUID v4.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Простий унікальний рядок на основі часу та PID
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        nanos,
        std::process::id() & 0xFFFF,
        (nanos >> 16) & 0x0FFF,
        ((nanos >> 8) & 0x3FFF) | 0x8000,
        (nanos as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            & 0xFFFFFFFFFFFF,
    )
}

/// Streaming-версія: on_chunk викликається по мірі отримання chunks від агента.
pub fn call_agent_new_session_streaming(
    service: &str,
    model: &str,
    prompt: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str) + Send,
) -> Result<(String, String), String> {
    if service == "Claude Code" {
        crate::api::claude::call_claude_code_new_session_streaming(
            model,
            prompt,
            session_id,
            job_info,
            working_dir,
            on_chunk,
        )
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_new_session_streaming(
            model,
            prompt,
            session_id,
            job_info,
            working_dir,
            on_chunk,
        )
    } else if service == "Codex CLI" {
        crate::api::codex::call_codex_new_session_streaming(
            model,
            prompt,
            session_id,
            job_info,
            working_dir,
            on_chunk,
        )
    } else if service == "AGY CLI" {
        crate::api::agy::call_agy_new_session_streaming(
            model,
            prompt,
            session_id,
            job_info,
            working_dir,
            on_chunk,
        )
    } else if service == "Pi CLI" {
        crate::api::pi::call_pi_new_session_streaming(
            model,
            prompt,
            session_id,
            job_info,
            working_dir,
            on_chunk,
        )
    } else {
        Err(format!(
            "Agent sessions not supported for service: {}",
            service
        ))
    }
}

/// Продовжує сесію агента (--resume) залежно від сервісу.
pub fn call_agent_resume(
    service: &str,
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    if service == "Claude Code" {
        crate::api::claude::call_claude_code_resume(
            model,
            message,
            session_id,
            job_info,
            working_dir,
        )
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_resume(model, message, session_id, job_info, working_dir)
    } else if service == "Codex CLI" {
        crate::api::codex::call_codex_resume(model, message, session_id, job_info, working_dir)
    } else if service == "AGY CLI" {
        crate::api::agy::call_agy_resume(model, message, session_id, job_info, working_dir)
    } else if service == "Pi CLI" {
        crate::api::pi::call_pi_resume(model, message, session_id, job_info, working_dir)
    } else {
        Err(format!(
            "Agent sessions not supported for service: {}",
            service
        ))
    }
}

/// Зберігає інформацію про сесію агента у файл agent_session.json у папці задачі.
fn save_agent_session_to_file(
    save_dir: &std::path::Path,
    session: &crate::queue::AgentSessionInfo,
) {
    let json = serde_json::to_string_pretty(&serde_json::json!({
        "session_id": session.session_id,
        "service": session.service,
        "model": session.model,
    }))
    .unwrap_or_default();
    let _ = std::fs::write(save_dir.join("agent_session.json"), json);
}

/// Зберігає историю чату агента у файл agent_chat.json у папці задачі.
fn save_agent_chat_to_file(save_dir: &std::path::Path, chat: &[crate::queue::AgentChatMessage]) {
    let messages: Vec<serde_json::Value> = chat
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();
    let json = serde_json::to_string_pretty(&messages).unwrap_or_default();
    let _ = std::fs::write(save_dir.join("agent_chat.json"), json);
}

/// Після генерації медіафайлів заповнює поле `media` в segments.json фактичними шляхами.
pub(super) fn assign_media_to_timeline(save_dir: &std::path::Path) -> Result<(), String> {
    fn existing_segment_media_rel_path(
        save_dir: &std::path::Path,
        seg_idx: usize,
    ) -> Option<String> {
        let media_dir = save_dir.join("media");
        let stem = format!("{:04}", seg_idx + 1);
        for ext in [
            "jpg", "jpeg", "png", "gif", "webp", "mp4", "mov", "avi", "mkv", "webm",
        ] {
            let file_name = format!("{}.{}", stem, ext);
            if media_dir.join(&file_name).exists() {
                return Some(format!("media/{}", file_name));
            }
        }
        None
    }

    let timeline_path = save_dir.join("segments.json");
    let content = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Cannot read segments.json: {}", e))?;
    let mut timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|e| format!("Invalid segments.json: {}", e))?;
    let stock_cache = crate::api::stock::load_cache(save_dir);

    for (i, seg) in timeline.segments.iter_mut().enumerate() {
        if seg.media_type == crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes {
            continue;
        }

        let explicit_media = seg.media.as_ref().and_then(|media| {
            let media_path = save_dir.join(media);
            if media_path.exists() {
                Some(media.clone())
            } else {
                None
            }
        });
        let stock_media = stock_cache
            .as_ref()
            .and_then(|cache| cache.get(i))
            .and_then(|entry| entry.selected.as_ref())
            .and_then(|sel| {
                let rel = format!("media/{}", sel.filename);
                if save_dir.join(&rel).exists() {
                    Some((rel, sel.trim_start as f64))
                } else {
                    None
                }
            });

        if let Some(media) = explicit_media {
            seg.media = Some(media);
        } else if let Some((media, trim_start)) = stock_media {
            seg.media = Some(media);
            seg.trim_start = trim_start;
        } else {
            seg.media = existing_segment_media_rel_path(save_dir, i);
        }
    }

    let json = serde_json::to_string_pretty(&timeline).map_err(|e| format!("JSON error: {}", e))?;
    std::fs::write(&timeline_path, json).map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
