pub mod agent_prompts;
mod agent_timeline;
pub mod capcut;
mod final_stages;
mod media_regen;
pub mod montage;
mod retry;
mod subtitles;
pub mod timeline;
mod video;
pub mod voiceover;

use std::sync::{Arc, Condvar, Mutex};

use eframe::egui;

use self::agent_timeline::{assign_media_to_timeline, run_agent_timeline};
pub use self::media_regen::{
    animate_single_image, find_changed_prompts_for_rebuild, regenerate_single_media,
    upscale_video_if_needed,
};
pub use self::retry::retry_from_stage;

pub(crate) use self::media_regen::read_prompt_for_file;

use self::subtitles::run_av_branch;
use self::video::run_video_branch;

/// Продовжує сесію агента (--resume) залежно від сервісу.
pub fn call_agent_resume(
    service: &str,
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    agent_timeline::call_agent_resume(service, model, message, session_id, job_info, working_dir)
}

/// Повертає текст, який слід використати для побудови segments.json.
pub(super) fn source_text_for_segments(
    settings: &crate::queue::JobSettings,
    translated_text: &Arc<Mutex<Option<String>>>,
) -> String {
    if settings.translation_enabled {
        translated_text
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| settings.text.clone())
    } else {
        settings.text.clone()
    }
}

/// Чи працює задача у стоковому режимі, де потрібен ручний вибір медіа в редакторі монтажу.
pub(super) fn uses_stock_montage_control(settings: &crate::queue::JobSettings) -> bool {
    settings.video_enabled && matches!(settings.video_service.as_str(), "Pexels" | "Pixabay")
}

/// У режимі Prompt Only програма спочатку сама будує базовий segments.json.
pub(super) fn prepare_prompt_only_segments(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    source_text: &str,
    audio_duration_secs: Option<f64>,
) -> Result<(), String> {
    let save_dir = std::path::Path::new(&settings.save_path);
    let segments = crate::core::pipeline::timeline::text_splitter::split_text(
        source_text,
        &settings.text_split_mode,
        settings.text_split_char_limit,
    );

    crate::logger::log_job(
        job_id,
        job_name,
        &format!(
            "Prompt Only: building base segments.json (mode={}, segments={})...",
            settings.text_split_mode,
            segments.len()
        ),
    );

    crate::core::pipeline::timeline::sync::build_timeline(
        save_dir,
        &segments,
        audio_duration_secs,
        job_name,
    )
}

/// Перевіряє, чи користувач вже натиснув «скасувати задачу».
pub(super) fn ensure_job_not_cancelled(job_id: u64) -> Result<(), String> {
    if crate::queue::is_job_cancelled(job_id) {
        Err(crate::queue::cancelled_error())
    } else {
        Ok(())
    }
}

/// Виставляє кінцевий статус задачі: Cancelled або Failed.
pub(super) fn set_job_error_status(status: &Arc<Mutex<crate::queue::JobStatus>>, error: String) {
    if crate::queue::is_cancelled_error(&error) {
        *status.lock().unwrap() = crate::queue::JobStatus::Cancelled;
    } else {
        *status.lock().unwrap() = crate::queue::JobStatus::Failed(error);
    }
}

/// Виконує весь пайплайн у фоновому потоці.
/// Послідовно: Переклад → [Озвучка+Субтитри || Відеоряд] → Timeline → Монтаж.
/// Озвучка+Субтитри та Відеоряд виконуються паралельно між собою.
pub fn run_pipeline(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    translation_stage: Arc<Mutex<crate::queue::StageStatus>>,
    voiceover_stage: Arc<Mutex<crate::queue::StageStatus>>,
    video_stage: Arc<Mutex<crate::queue::StageStatus>>,
    subtitles_stage: Arc<Mutex<crate::queue::StageStatus>>,
    montage_stage: Arc<Mutex<crate::queue::StageStatus>>,
    translated_text: Arc<Mutex<Option<String>>>,
    total_cost: Arc<Mutex<Option<f64>>>,
    audio_duration: Arc<Mutex<Option<f64>>>,
    prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    media_progress: Arc<Mutex<Option<(usize, usize)>>>,
    montage_progress: Arc<Mutex<Option<f32>>>,
    montage_file_size: Arc<Mutex<Option<u64>>>,
    media_control_resume: Arc<(Mutex<bool>, Condvar)>,
    montage_control_resume: Arc<(Mutex<bool>, Condvar)>,
    agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    capcut_mode_override: Arc<Mutex<Option<bool>>>,
    ctx: egui::Context,
) {
    crate::queue::reset_job_runtime(job_id);

    std::thread::spawn(move || {
        crate::logger::log_job(job_id, &job_name, "Job started.");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();
        if let Err(e) = ensure_job_not_cancelled(job_id) {
            set_job_error_status(&status, e);
            ctx.request_repaint();
            return;
        }

        // Гарантуємо існування кінцевої папки з самого початку обробки
        if let Err(e) = std::fs::create_dir_all(&settings.save_path) {
            crate::logger::log_job(
                job_id,
                &job_name,
                &format!("Failed to create output dir: {}", e),
            );
        }

        // Текст, який буде передано в озвучку (оригінал або результат перекладу)
        let mut voice_text = settings.text.clone();

        // Перевіряємо, чи переклад уже був виконаний раніше (наприклад, при повторному запуску після контролю)
        let has_translation = if settings.translation_enabled {
            let tr_stage = translation_stage.lock().unwrap().clone();
            tr_stage == crate::queue::StageStatus::Done
        } else {
            false
        };

        // Етап 1: Переклад (послідовно)
        if settings.translation_enabled && !has_translation {
            crate::logger::log_job(job_id, &job_name, "Starting translation stage...");
            *translation_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            match crate::core::llm::call_llm(
                &settings.translation_service,
                &settings.openrouter_key,
                &settings.translation_model,
                &settings.translation_prompt,
                &settings.text,
                settings.translation_temperature,
                Some((job_id, job_name.clone())),
                Some(settings.save_path.as_str()),
                false,
            ) {
                Ok((translated, cost)) => {
                    let dir = std::path::Path::new(&settings.save_path);
                    if std::fs::create_dir_all(dir).is_ok() {
                        let _ = std::fs::write(dir.join("text.txt"), &translated);
                    }
                    crate::logger::log_job(job_id, &job_name, "Translation saved: text.txt");
                    voice_text = translated.clone();
                    *translated_text.lock().unwrap() = Some(translated);
                    // Накопичуємо вартість (враховуючи перегенерацію)
                    if let Some(c) = cost {
                        let mut tc = total_cost.lock().unwrap();
                        *tc = Some(tc.unwrap_or(0.0) + c);
                    }
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Done;

                    if settings.translation_control_enabled {
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            "Translation done. Job is awaiting translation review.",
                        );
                        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingControl;
                        ctx.request_repaint();
                        return; // Зупиняємо пайплайн для контролю
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Translation error: {}", e));
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }
            }

            if let Err(e) = ensure_job_not_cancelled(job_id) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }
        } else if settings.translation_enabled && has_translation {
            // Якщо переклад уже виконано і ми продовжуємо після контролю
            if let Some(text) = translated_text.lock().unwrap().as_ref() {
                voice_text = text.clone();
            }
        }

        // При агентному режимі (Claude Code / Gemini CLI / Codex CLI) гілки виконуються послідовно:
        // AV → Агент → Медіа. В звичайному режимі — паралельно.
        let run_av = settings.voiceover_enabled;
        let run_video = settings.video_enabled;
        let uses_stock_control = uses_stock_montage_control(&settings);
        let is_agent_mode = run_video && settings.is_agent_service();

        if is_agent_mode {
            // === Агентний режим: послідовно ===
            if run_av {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Agent mode: starting AV branch (voiceover + subtitles)...",
                );
                if let Err(e) = run_av_branch(
                    job_id,
                    job_name.clone(),
                    settings.clone(),
                    voice_text.clone(),
                    Arc::clone(&voiceover_stage),
                    Arc::clone(&subtitles_stage),
                    Arc::clone(&audio_duration),
                    ctx.clone(),
                ) {
                    set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }
                if let Err(e) = ensure_job_not_cancelled(job_id) {
                    set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }
            }

            // У Prompt Only спочатку сама програма будує базовий segments.json.
            if settings.is_prompt_only_agent_mode() {
                let source_text = source_text_for_segments(&settings, &translated_text);
                let audio_dur = *audio_duration.lock().unwrap();
                if let Err(e) = prepare_prompt_only_segments(
                    job_id,
                    &job_name,
                    &settings,
                    &source_text,
                    audio_dur,
                ) {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!("Prompt Only base timeline error: {}", e),
                    );
                    *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }
            }

            crate::logger::log_job(job_id, &job_name, "Agent mode: processing segments.json...");
            *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            if let Err(e) = run_agent_timeline(
                job_id,
                &job_name,
                &settings,
                Arc::clone(&status),
                Arc::clone(&agent_control_resume),
                Arc::clone(&agent_chat),
                Arc::clone(&agent_session),
                &ctx,
            ) {
                crate::logger::log_job(job_id, &job_name, &format!("Agent timeline error: {}", e));
                *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }

            if let Err(e) = ensure_job_not_cancelled(job_id) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }

            // Генерація медіа (сегменти читаються з segments.json)
            if let Err(e) = run_video_branch(
                job_id,
                job_name.clone(),
                settings.clone(),
                Arc::clone(&translated_text),
                Arc::clone(&video_stage),
                Arc::clone(&prompts_progress),
                Arc::clone(&media_progress),
                Arc::clone(&total_cost),
                ctx.clone(),
            ) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }
            if let Err(e) = ensure_job_not_cancelled(job_id) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }
        } else {
            // === Звичайний режим: паралельно ===
            if run_av {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Starting AV branch (voiceover + subtitles) in parallel with video...",
                );
            }
            if run_video {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Starting video branch in parallel with AV...",
                );
            }

            // Гілка AV: Озвучка → Субтитри
            let av_handle: Option<std::thread::JoinHandle<Result<(), String>>> = if run_av {
                let settings_av = settings.clone();
                let voice_text_av = voice_text.clone();
                let voiceover_stage_av = Arc::clone(&voiceover_stage);
                let subtitles_stage_av = Arc::clone(&subtitles_stage);
                let audio_duration_av = Arc::clone(&audio_duration);
                let ctx_av = ctx.clone();
                let job_id_av = job_id;
                let job_name_av = job_name.clone();

                Some(std::thread::spawn(move || {
                    run_av_branch(
                        job_id_av,
                        job_name_av,
                        settings_av,
                        voice_text_av,
                        voiceover_stage_av,
                        subtitles_stage_av,
                        audio_duration_av,
                        ctx_av,
                    )
                }))
            } else {
                None
            };

            // Гілка Video: Відеоряд
            let video_handle: Option<std::thread::JoinHandle<Result<(), String>>> = if run_video {
                let settings_video = settings.clone();
                let translated_text_video = Arc::clone(&translated_text);
                let video_stage_video = Arc::clone(&video_stage);
                let prompts_progress_video = Arc::clone(&prompts_progress);
                let media_progress_video = Arc::clone(&media_progress);
                let total_cost_video = Arc::clone(&total_cost);
                let ctx_video = ctx.clone();
                let job_id_video = job_id;
                let job_name_video = job_name.clone();

                Some(std::thread::spawn(move || {
                    run_video_branch(
                        job_id_video,
                        job_name_video,
                        settings_video,
                        translated_text_video,
                        video_stage_video,
                        prompts_progress_video,
                        media_progress_video,
                        total_cost_video,
                        ctx_video,
                    )
                }))
            } else {
                None
            };

            // Спочатку чекаємо відеогілку, щоб мати можливість зробити паузу для контролю зображень
            // поки гілка AV (озвучка + субтитри) продовжує виконуватись паралельно.
            let video_result = video_handle.map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("Video thread panicked".to_string()))
            });

            // Пауза для контролю зображень — AV гілка продовжує виконуватись паралельно.
            // У стоковому режимі замість цього використовується контроль монтажу.
            if settings.media_control_enabled && settings.video_enabled && !uses_stock_control {
                if let Some(Ok(())) = &video_result {
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Video done. Awaiting media review by user...",
                    );
                    *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMediaControl;
                    ctx.request_repaint();

                    let (lock, cvar) = &*media_control_resume;
                    let mut resumed = lock.lock().unwrap();
                    while !*resumed {
                        resumed = cvar.wait(resumed).unwrap();
                    }
                    *resumed = false;

                    if let Err(e) = ensure_job_not_cancelled(job_id) {
                        set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }

                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Media review confirmed. Resuming pipeline...",
                    );
                    *status.lock().unwrap() = crate::queue::JobStatus::Running;
                    ctx.request_repaint();
                }
            }

            // Тепер чекаємо AV гілку
            let av_result = av_handle.map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("AV thread panicked".to_string()))
            });

            // Перевіряємо помилки обох гілок
            if let Some(Err(e)) = av_result {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }
            if let Some(Err(e)) = video_result {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }
            if let Err(e) = ensure_job_not_cancelled(job_id) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }

            // Генеруємо segments.json якщо відеоряд увімкнено і є тривалість аудіо
            if settings.video_enabled {
                let save_dir = std::path::Path::new(&settings.save_path);

                // У стоковому режимі: якщо segments.json ще немає (non-agent) — будуємо з SRT, потім патчимо.
                if uses_stock_control {
                    if !save_dir.join("segments.json").exists() {
                        let source_text = if settings.translation_enabled {
                            translated_text
                                .lock()
                                .unwrap()
                                .clone()
                                .unwrap_or_else(|| settings.text.clone())
                        } else {
                            settings.text.clone()
                        };
                        let segments = crate::core::pipeline::timeline::text_splitter::split_text(
                            &source_text,
                            &settings.text_split_mode,
                            settings.text_split_char_limit,
                        );
                        let audio_dur = *audio_duration.lock().unwrap();
                        if let Err(e) = crate::core::pipeline::timeline::sync::build_timeline(
                            save_dir, &segments, audio_dur, &job_name,
                        ) {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!("Timeline warning: {}", e),
                            );
                        }
                    }
                } else {
                    let source_text = if settings.translation_enabled {
                        translated_text
                            .lock()
                            .unwrap()
                            .clone()
                            .unwrap_or_else(|| settings.text.clone())
                    } else {
                        settings.text.clone()
                    };

                    let segments = crate::core::pipeline::timeline::text_splitter::split_text(
                        &source_text,
                        &settings.text_split_mode,
                        settings.text_split_char_limit,
                    );

                    let audio_dur = *audio_duration.lock().unwrap();

                    match crate::core::pipeline::timeline::sync::build_timeline(
                        save_dir, &segments, audio_dur, &job_name,
                    ) {
                        Ok(_) => crate::logger::log_job(
                            job_id,
                            &job_name,
                            "Segments saved: segments.json",
                        ),
                        Err(e) => crate::logger::log_job(
                            job_id,
                            &job_name,
                            &format!("Timeline warning: {}", e),
                        ),
                    }
                }
            }
        }

        // Пауза контролю монтажу перед рендером.
        // Для стокового режиму вона вмикається автоматично, бо користувач має вручну обрати медіа.
        if settings.montage_control_enabled || uses_stock_control {
            crate::logger::log_job(
                job_id,
                &job_name,
                "Awaiting montage control confirmation from user...",
            );
            *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMontageControl;
            ctx.request_repaint();

            let (lock, cvar) = &*montage_control_resume;
            let mut resumed = lock.lock().unwrap();
            while !*resumed {
                resumed = cvar.wait(resumed).unwrap();
            }
            *resumed = false;

            if let Err(e) = ensure_job_not_cancelled(job_id) {
                set_job_error_status(&status, e);
                ctx.request_repaint();
                return;
            }

            crate::logger::log_job(
                job_id,
                &job_name,
                "Montage control confirmed. Resuming pipeline...",
            );
            *status.lock().unwrap() = crate::queue::JobStatus::Running;
            ctx.request_repaint();
        }

        // Після підтвердження монтажу призначаємо стокові медіа в timeline.
        if uses_stock_control {
            let save_dir = std::path::Path::new(&settings.save_path);
            if let Err(e) = assign_media_to_timeline(save_dir) {
                crate::logger::log_job(job_id, &job_name, &format!("assign_media warning: {}", e));
            } else {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Timeline patched with stock media paths.",
                );
            }
        }

        // Етап 5: Монтаж (FFmpeg або CapCut)
        if settings.montage_enabled {
            crate::logger::log_job(job_id, &job_name, "Starting montage stage...");
            *montage_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            let audio_dur = *audio_duration.lock().unwrap();
            let save_dir = std::path::Path::new(&settings.save_path);
            let use_capcut = capcut_mode_override
                .lock()
                .unwrap()
                .unwrap_or(settings.capcut_enabled);

            if use_capcut {
                let job_id_log = job_id;
                let job_name_log = job_name.clone();
                let draft_root = std::path::Path::new(&settings.capcut_draft_path);
                match crate::core::pipeline::capcut::generate_capcut_project(
                    save_dir,
                    draft_root,
                    &job_name,
                    audio_dur,
                    |msg| crate::logger::log_job(job_id_log, &job_name_log, msg),
                ) {
                    Ok(_) => {
                        crate::logger::log_job(job_id, &job_name, "CapCut project generated.");
                        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                    }
                    Err(e) => {
                        crate::logger::log_job(job_id, &job_name, &format!("CapCut failed: {}", e));
                        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        set_job_error_status(&status, format!("CapCut: {}", e));
                        ctx.request_repaint();
                        return;
                    }
                }
            } else {
                let job_id_log = job_id;
                let job_name_log = job_name.clone();
                let montage_progress_arc = Arc::clone(&montage_progress);
                let ctx_montage = ctx.clone();

                match crate::core::pipeline::montage::run_montage(
                    job_id,
                    save_dir,
                    &job_name,
                    audio_dur,
                    settings.montage_fps,
                    &settings.montage_preset,
                    settings.montage_bitrate,
                    &settings.montage_transition,
                    settings.montage_transition_duration,
                    settings.subtitles_enabled,
                    settings.overlay_triggers_enabled,
                    &settings.overlay_triggers,
                    settings.montage_image_zoom_enabled,
                    settings.montage_image_zoom_intensity,
                    &settings.montage_image_zoom_mode,
                    settings.montage_image_zoom_scale,
                    settings.montage_image_shake_enabled,
                    settings.montage_image_shake_intensity,
                    |msg| crate::logger::log_job(job_id_log, &job_name_log, msg),
                    move |pct| {
                        *montage_progress_arc.lock().unwrap() = Some(pct);
                        ctx_montage.request_repaint();
                    },
                ) {
                    Ok(size) => {
                        *montage_file_size.lock().unwrap() = Some(size);
                        crate::logger::log_job(job_id, &job_name, "Montage complete.");
                        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                    }
                    Err(e) => {
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            &format!("Montage failed: {}", e),
                        );
                        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        set_job_error_status(&status, format!("Montage: {}", e));
                        ctx.request_repaint();
                        return;
                    }
                }
            }
            ctx.request_repaint();
        }

        if let Err(e) = ensure_job_not_cancelled(job_id) {
            set_job_error_status(&status, e);
            ctx.request_repaint();
            return;
        }

        crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
        *status.lock().unwrap() = crate::queue::JobStatus::Done;
        ctx.request_repaint();
    });
}
