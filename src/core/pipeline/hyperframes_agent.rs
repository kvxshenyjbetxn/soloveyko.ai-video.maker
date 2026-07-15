use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui;

static GENERATION_RUNNING: AtomicBool = AtomicBool::new(false);

/// Запускає окремі ізольовані сесії для всіх HyperFrames-сцен без готового HTML.
pub fn generate_pending_clips_async(
    settings: crate::queue::JobSettings,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    timeline_rebuild_requested: Arc<Mutex<bool>>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
) {
    if GENERATION_RUNNING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        crate::logger::log_job(job_id, &job_name, "HyperFrames: генерація вже виконується.");
        return;
    }

    {
        let mut chat = agent_chat.lock().unwrap();
        chat.clear();
        chat.push(crate::queue::AgentChatMessage {
            role: "agent".to_string(),
            content: "HyperFrames: готуємо ізольовані сесії для кліпів...\n\n".to_string(),
        });
    }
    *agent_session.lock().unwrap() = None;
    ctx.request_repaint();

    std::thread::spawn(move || {
        let result = generate_pending_clips(
            &settings,
            job_id,
            &job_name,
            &agent_chat,
            &agent_session,
            &timeline_rebuild_requested,
            &ctx,
        );
        GENERATION_RUNNING.store(false, Ordering::Release);

        match result {
            Ok(0) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "HyperFrames: немає незгенерованих кліпів.",
                );
                agent_chat
                    .lock()
                    .unwrap()
                    .push(crate::queue::AgentChatMessage {
                        role: "agent".to_string(),
                        content: "[->] HyperFrames: усі кліпи вже готові.".to_string(),
                    });
            }
            Ok(count) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("HyperFrames: створено кліпів: {}", count),
                );
                agent_chat.lock().unwrap().push(crate::queue::AgentChatMessage {
                    role: "agent".to_string(),
                    content: format!(
                        "[->] HyperFrames: генерацію завершено. Створено кліпів: {}. Тепер їх можна перевірити або зарендерити.",
                        count
                    ),
                });
                *timeline_rebuild_requested.lock().unwrap() = true;
            }
            Err(error) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("HyperFrames generation error: {}", error),
                );
                agent_chat
                    .lock()
                    .unwrap()
                    .push(crate::queue::AgentChatMessage {
                        role: "agent".to_string(),
                        content: format!("[!!] {}", error),
                    });
            }
        }
        ctx.request_repaint();
    });
}

fn generate_pending_clips(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    agent_chat: &Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: &Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    timeline_rebuild_requested: &Arc<Mutex<bool>>,
    ctx: &egui::Context,
) -> Result<usize, String> {
    if !settings.is_agent_service() {
        return Err("Для HyperFrames потрібен CLI-агент у налаштуваннях відеоряду.".to_string());
    }

    let save_dir = Path::new(&settings.save_path);
    let segments_path = save_dir.join("segments.json");
    let content = std::fs::read_to_string(&segments_path)
        .map_err(|error| format!("Не вдалося прочитати segments.json: {}", error))?;
    let mut timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|error| format!("Невалідний segments.json: {}", error))?;
    let mut generated = 0;

    for (position, segment) in timeline.segments.iter_mut().enumerate() {
        if segment.media_type
            != crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes
        {
            continue;
        }

        let output_rel = format!("clips/{:04}-scene/index.html", position + 1);
        let output_path = save_dir.join(&output_rel);
        if output_path.is_file() {
            segment.media = Some(output_rel);
            continue;
        }

        let brief = segment
            .hyperframes_brief
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "HyperFrames-сегмент {} не має hyperframes_brief. Перезапусти агента-аналітика.",
                    position + 1
                )
            })?;

        std::fs::create_dir_all(output_path.parent().unwrap_or(save_dir))
            .map_err(|error| format!("Не вдалося створити папку кліпу: {}", error))?;

        crate::logger::log_job(
            job_id,
            job_name,
            &format!(
                "HyperFrames: створюємо ізольований кліп {}...",
                position + 1
            ),
        );
        let prompt = build_clip_prompt(brief, segment.duration_secs, &output_path);
        let session_id = format!("hyperframes-{}-{}", position + 1, std::process::id());
        *agent_session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
            session_id: session_id.clone(),
            service: settings.video_llm_service.clone(),
            model: settings.video_llm_model.clone(),
        });
        agent_chat
            .lock()
            .unwrap()
            .push(crate::queue::AgentChatMessage {
                role: "agent".to_string(),
                content: format!(
                    "HyperFrames-кліп {}\nRunning: {} --model {}\n\n",
                    position + 1,
                    settings.video_llm_service,
                    settings.video_llm_model,
                ),
            });
        ctx.request_repaint();

        let chat_for_chunk = Arc::clone(agent_chat);
        let ctx_for_chunk = ctx.clone();
        let (_, actual_session_id) = crate::core::pipeline::call_agent_new_session_streaming(
            &settings.video_llm_service,
            &settings.video_llm_model,
            &prompt,
            &session_id,
            Some((job_id, job_name.to_string())),
            Some(&settings.save_path),
            move |chunk| {
                if let Some(last) = chat_for_chunk.lock().unwrap().last_mut() {
                    last.content.push_str(chunk);
                }
                ctx_for_chunk.request_repaint();
            },
        )
        .map_err(|error| format!("HyperFrames-сегмент {}: {}", position + 1, error))?;
        if let Some(session) = agent_session.lock().unwrap().as_mut() {
            session.session_id = actual_session_id;
        }

        if !output_path.is_file() {
            return Err(format!(
                "HyperFrames-сегмент {}: агент не створив {}",
                position + 1,
                output_path.display()
            ));
        }

        segment.media = Some(output_rel);
        generated += 1;
        // HTML вже є на диску: редактор одразу перераховує решту кліпів,
        // не чекаючи завершення всієї черги ізольованих агентів.
        *timeline_rebuild_requested.lock().unwrap() = true;
        ctx.request_repaint();
    }

    save_timeline(&segments_path, &timeline)?;
    crate::api::hyperframes::rebuild_preview_all(save_dir)?;
    Ok(generated)
}

fn build_clip_prompt(brief: &str, duration_secs: f64, output_path: &Path) -> String {
    crate::core::pipeline::agent_prompts::HYPERFRAMES_CLIP_AGENT_PROMPT
        .replace("{{brief}}", brief)
        .replace("{{duration}}", &format!("{:.3}", duration_secs))
        .replace("{{output}}", &output_path.to_string_lossy())
}

fn save_timeline(
    path: &Path,
    timeline: &crate::core::pipeline::timeline::sync::Timeline,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(timeline)
        .map_err(|error| format!("Не вдалося серіалізувати segments.json: {}", error))?;
    std::fs::write(path, json)
        .map_err(|error| format!("Не вдалося записати segments.json: {}", error))
}
