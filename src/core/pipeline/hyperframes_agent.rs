use std::path::{Path, PathBuf};
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
            Ok(result) if result.generated == 0 && result.errors.is_empty() => {
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
            Ok(result) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("HyperFrames: створено кліпів: {}", result.generated),
                );
                agent_chat.lock().unwrap().push(crate::queue::AgentChatMessage {
                    role: "agent".to_string(),
                    content: format!(
                        "[->] HyperFrames: генерацію завершено. Створено кліпів: {}. Тепер їх можна перевірити або зарендерити.",
                        result.generated
                    ),
                });
                if result.generated > 0 {
                    *timeline_rebuild_requested.lock().unwrap() = true;
                }
                for error in result.errors {
                    crate::logger::log_job(job_id, &job_name, &error);
                    agent_chat
                        .lock()
                        .unwrap()
                        .push(crate::queue::AgentChatMessage {
                            role: "agent".to_string(),
                            content: format!("[!!] {}", error),
                        });
                }
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

/// Відкриває нову ізольовану сесію для редагування одного готового кліпу.
pub fn edit_clip_async(
    settings: crate::queue::JobSettings,
    job_id: u64,
    job_name: String,
    segment_index: usize,
    ctx: egui::Context,
    timeline_rebuild_requested: Arc<Mutex<bool>>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
) {
    std::thread::spawn(move || {
        let result = edit_clip(
            &settings,
            job_id,
            &job_name,
            segment_index,
            &ctx,
            &timeline_rebuild_requested,
            &agent_chat,
            &agent_session,
        );
        if let Err(error) = result {
            agent_chat
                .lock()
                .unwrap()
                .push(crate::queue::AgentChatMessage {
                    role: "agent".to_string(),
                    content: format!("[!!] {}", error),
                });
            crate::logger::log_job(job_id, &job_name, &error);
        }
        ctx.request_repaint();
    });
}

fn edit_clip(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    segment_index: usize,
    ctx: &egui::Context,
    timeline_rebuild_requested: &Arc<Mutex<bool>>,
    agent_chat: &Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: &Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
) -> Result<(), String> {
    if !settings.is_agent_service() {
        return Err("Для редагування HyperFrames потрібен CLI-агент.".to_string());
    }

    let save_dir = Path::new(&settings.save_path);
    let content = std::fs::read_to_string(save_dir.join("segments.json"))
        .map_err(|error| format!("Не вдалося прочитати segments.json: {}", error))?;
    let timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|error| format!("Невалідний segments.json: {}", error))?;
    let segment = timeline
        .segments
        .get(segment_index)
        .ok_or_else(|| format!("HyperFrames-сегмент {} не знайдено.", segment_index + 1))?;
    if segment.media_type != crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes {
        return Err(format!(
            "Сегмент {} не є HyperFrames-кліпом.",
            segment_index + 1
        ));
    }
    let output_path = save_dir.join(format!("clips/{:04}-scene/index.html", segment_index + 1));
    if !output_path.is_file() {
        return Err(format!(
            "Спочатку створи HyperFrames-кліп {}.",
            segment_index + 1
        ));
    }

    let session_id = format!(
        "hyperframes-edit-{}-{}",
        segment_index + 1,
        std::process::id()
    );
    *agent_session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
        session_id: session_id.clone(),
        service: settings.video_llm_service.clone(),
        model: settings.video_llm_model.clone(),
    });
    {
        let mut chat = agent_chat.lock().unwrap();
        chat.clear();
        chat.push(crate::queue::AgentChatMessage {
            role: "agent".to_string(),
            content: format!(
                "Редагування HyperFrames-кліпу #{}\nRunning: {} --model {}\n\n",
                segment_index + 1,
                settings.video_llm_service,
                settings.video_llm_model,
            ),
        });
    }
    ctx.request_repaint();

    let prompt = crate::core::pipeline::agent_prompts::HYPERFRAMES_EDIT_AGENT_PROMPT
        .replace("{{output}}", &output_path.to_string_lossy())
        .replace("{{duration}}", &format!("{:.3}", segment.duration_secs));
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
    )?;
    if let Some(session) = agent_session.lock().unwrap().as_mut() {
        session.session_id = actual_session_id;
    }
    crate::api::hyperframes::rebuild_preview_all(save_dir)?;
    *timeline_rebuild_requested.lock().unwrap() = true;
    Ok(())
}

struct ClipTask {
    position: usize,
    brief: String,
    duration_secs: f64,
    output_rel: String,
    output_path: PathBuf,
}

struct GenerationResult {
    generated: usize,
    errors: Vec<String>,
}

fn generate_pending_clips(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    agent_chat: &Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: &Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    timeline_rebuild_requested: &Arc<Mutex<bool>>,
    ctx: &egui::Context,
) -> Result<GenerationResult, String> {
    if !settings.is_agent_service() {
        return Err("Для HyperFrames потрібен CLI-агент у налаштуваннях відеоряду.".to_string());
    }

    let save_dir = Path::new(&settings.save_path).to_path_buf();
    let segments_path = save_dir.join("segments.json");
    let content = std::fs::read_to_string(&segments_path)
        .map_err(|error| format!("Не вдалося прочитати segments.json: {}", error))?;
    let mut timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|error| format!("Невалідний segments.json: {}", error))?;
    let mut tasks = Vec::new();
    let mut errors = Vec::new();
    let mut timeline_changed = false;

    for (position, segment) in timeline.segments.iter_mut().enumerate() {
        if segment.media_type
            != crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes
        {
            continue;
        }

        let output_rel = format!("clips/{:04}-scene/index.html", position + 1);
        let output_path = save_dir.join(&output_rel);
        if output_path.is_file() {
            if segment.media.as_deref() != Some(&output_rel) {
                segment.media = Some(output_rel);
                timeline_changed = true;
            }
            continue;
        }

        let Some(brief) = segment
            .hyperframes_brief
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        else {
            errors.push(format!(
                "HyperFrames-сегмент {} не має hyperframes_brief. Перезапусти агента-аналітика.",
                position + 1
            ));
            continue;
        };

        tasks.push(ClipTask {
            position,
            brief: brief.to_string(),
            duration_secs: segment.duration_secs,
            output_rel,
            output_path,
        });
    }

    if timeline_changed {
        save_timeline(&segments_path, &timeline)?;
    }

    let timeline = Arc::new(Mutex::new(timeline));
    let mut handles = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Err(error) = std::fs::create_dir_all(task.output_path.parent().unwrap_or(&save_dir))
        {
            errors.push(format!(
                "HyperFrames-сегмент {}: не вдалося створити папку кліпу: {}",
                task.position + 1,
                error
            ));
            continue;
        }

        crate::logger::log_job(
            job_id,
            job_name,
            &format!(
                "HyperFrames: створюємо ізольований кліп {}...",
                task.position + 1
            ),
        );
        let session_id = format!("hyperframes-{}-{}", task.position + 1, std::process::id());
        let chat_prefix = format!(
            "HyperFrames-кліп {}\nRunning: {} --model {}\n\n",
            task.position + 1,
            settings.video_llm_service,
            settings.video_llm_model,
        );
        agent_chat
            .lock()
            .unwrap()
            .push(crate::queue::AgentChatMessage {
                role: "agent".to_string(),
                content: chat_prefix.clone(),
            });
        ctx.request_repaint();

        let service = settings.video_llm_service.clone();
        let model = settings.video_llm_model.clone();
        let save_path = settings.save_path.clone();
        let task_job_name = job_name.to_string();
        let chat = Arc::clone(agent_chat);
        let session = Arc::clone(agent_session);
        let timeline = Arc::clone(&timeline);
        let timeline_rebuild_requested = Arc::clone(timeline_rebuild_requested);
        let ctx = ctx.clone();
        let segments_path = segments_path.clone();
        handles.push(std::thread::spawn(move || {
            *session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
                session_id: session_id.clone(),
                service: service.clone(),
                model: model.clone(),
            });
            let prompt = build_clip_prompt(&task.brief, task.duration_secs, &task.output_path);
            let chunk_prefix = chat_prefix;
            let chat_for_chunk = Arc::clone(&chat);
            let ctx_for_chunk = ctx.clone();
            let (_, actual_session_id) = crate::core::pipeline::call_agent_new_session_streaming(
                &service,
                &model,
                &prompt,
                &session_id,
                Some((job_id, task_job_name.clone())),
                Some(&save_path),
                move |chunk| {
                    if let Some(message) = chat_for_chunk
                        .lock()
                        .unwrap()
                        .iter_mut()
                        .find(|message| message.content.starts_with(&chunk_prefix))
                    {
                        message.content.push_str(chunk);
                    }
                    ctx_for_chunk.request_repaint();
                },
            )
            .map_err(|error| format!("HyperFrames-сегмент {}: {}", task.position + 1, error))?;
            if let Some(current_session) = session.lock().unwrap().as_mut() {
                current_session.session_id = actual_session_id;
            }

            if !task.output_path.is_file() {
                return Err(format!(
                    "HyperFrames-сегмент {}: агент не створив {}",
                    task.position + 1,
                    task.output_path.display()
                ));
            }

            // Один м'ютекс послідовно оновлює пам'ять і файл після кожного успіху.
            let mut timeline = timeline.lock().unwrap();
            let previous_media = timeline.segments[task.position].media.clone();
            timeline.segments[task.position].media = Some(task.output_rel);
            if let Err(error) = save_timeline(&segments_path, &timeline) {
                timeline.segments[task.position].media = previous_media;
                return Err(error);
            }
            *timeline_rebuild_requested.lock().unwrap() = true;
            ctx.request_repaint();
            Ok(())
        }));
    }

    let mut generated = 0;
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => generated += 1,
            Ok(Err(error)) => errors.push(error),
            Err(_) => errors.push("Потік генерації HyperFrames завершився аварійно.".to_string()),
        }
    }

    if generated > 0 {
        crate::api::hyperframes::rebuild_preview_all(&save_dir)?;
    }
    Ok(GenerationResult { generated, errors })
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
