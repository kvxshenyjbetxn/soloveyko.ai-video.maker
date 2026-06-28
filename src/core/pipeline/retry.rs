use std::sync::{Arc, Condvar, Mutex};

use eframe::egui;

use super::agent_timeline::{assign_media_to_timeline, run_agent_timeline};
use super::final_stages::run_final_stages;
use super::subtitles::{run_av_branch, run_subtitles_only};
use super::video::run_video_branch;
use super::{prepare_prompt_only_segments, run_pipeline, source_text_for_segments};

/// Повторює пайплайн починаючи з вказаного етапу.
/// Скидає статуси цього та всіх наступних етапів, потім запускає їх у фоновому потоці.
pub fn retry_from_stage(
    stage: crate::queue::RetryStage,
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
    use crate::queue::RetryStage::*;
    use crate::queue::StageStatus::Pending as SPending;

    crate::queue::reset_job_runtime(job_id);

    match stage {
        // Повний перезапуск з перекладу
        Translation => {
            *translation_stage.lock().unwrap() = SPending;
            *voiceover_stage.lock().unwrap() = SPending;
            *video_stage.lock().unwrap() = SPending;
            *subtitles_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *translated_text.lock().unwrap() = None;
            *total_cost.lock().unwrap() = None;
            *audio_duration.lock().unwrap() = None;
            *prompts_progress.lock().unwrap() = None;
            *media_progress.lock().unwrap() = None;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;
            *media_control_resume.0.lock().unwrap() = false;
            *montage_control_resume.0.lock().unwrap() = false;
            run_pipeline(
                job_id,
                job_name,
                settings,
                status,
                translation_stage,
                voiceover_stage,
                video_stage,
                subtitles_stage,
                montage_stage,
                translated_text,
                total_cost,
                audio_duration,
                prompts_progress,
                media_progress,
                montage_progress,
                montage_file_size,
                media_control_resume,
                montage_control_resume,
                agent_control_resume,
                agent_chat,
                agent_session,
                capcut_mode_override,
                ctx,
            );
        }

        // Повтор озвучки → субтитри → (агент + відеоряд якщо агентний режим і медіа немає) → монтаж
        Voiceover => {
            *voiceover_stage.lock().unwrap() = SPending;
            *subtitles_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *audio_duration.lock().unwrap() = None;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;

            let voice_text = translated_text
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| settings.text.clone());

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();
                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(
                    job_id,
                    &job_name,
                    "Retry: AV branch (voiceover + subtitles)...",
                );
                if let Err(e) = run_av_branch(
                    job_id,
                    job_name.clone(),
                    settings.clone(),
                    voice_text,
                    Arc::clone(&voiceover_stage),
                    Arc::clone(&subtitles_stage),
                    Arc::clone(&audio_duration),
                    ctx.clone(),
                ) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                // В агентному режимі після AV гілки запускаємо агента та відеогілку
                // (якщо відеоряд ще не був виконаний раніше — тобто немає збережених медіафайлів)
                let is_agent_mode = settings.video_enabled && settings.is_agent_service();
                let video_already_done =
                    *video_stage.lock().unwrap() == crate::queue::StageStatus::Done;

                if is_agent_mode && !video_already_done {
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
                            super::set_job_error_status(&status, e);
                            ctx.request_repaint();
                            return;
                        }
                    }

                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Agent mode: processing segments.json...",
                    );
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
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            &format!("Agent timeline error: {}", e),
                        );
                        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }

                    let video_result = run_video_branch(
                        job_id,
                        job_name.clone(),
                        settings.clone(),
                        Arc::clone(&translated_text),
                        Arc::clone(&video_stage),
                        Arc::clone(&prompts_progress),
                        Arc::clone(&media_progress),
                        Arc::clone(&total_cost),
                        ctx.clone(),
                    );

                    if let Ok(()) = &video_result {
                        let save_dir = std::path::Path::new(&settings.save_path);
                        if let Err(e) = assign_media_to_timeline(save_dir) {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!("assign_media warning: {}", e),
                            );
                        } else {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                "Timeline patched with media paths.",
                            );
                        }
                    }

                    if let Err(e) = video_result {
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }

                    if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }
                }

                if let Err(e) = run_final_stages(
                    job_id,
                    &job_name,
                    &settings,
                    &translated_text,
                    &audio_duration,
                    &status,
                    &montage_stage,
                    &montage_progress,
                    &montage_file_size,
                    &montage_control_resume,
                    &capcut_mode_override,
                    &ctx,
                ) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
                ctx.request_repaint();
            });
        }

        // Повтор відеоряду → медіа-контроль → монтаж
        Video => {
            *video_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *prompts_progress.lock().unwrap() = None;
            *media_progress.lock().unwrap() = None;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;
            *media_control_resume.0.lock().unwrap() = false;

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();
                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                let is_agent_mode = !settings.skip_agent_on_resume
                    && settings.video_enabled
                    && settings.is_agent_service();

                // В агентному режимі спочатку запускаємо агента для створення або редагування segments.json
                if is_agent_mode {
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
                            super::set_job_error_status(&status, e);
                            ctx.request_repaint();
                            return;
                        }
                    }

                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Retry Video (agent mode): processing segments.json...",
                    );
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
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            &format!("Agent timeline error: {}", e),
                        );
                        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }
                }

                crate::logger::log_job(job_id, &job_name, "Retry: video branch...");
                let video_result = run_video_branch(
                    job_id,
                    job_name.clone(),
                    settings.clone(),
                    Arc::clone(&translated_text),
                    Arc::clone(&video_stage),
                    Arc::clone(&prompts_progress),
                    Arc::clone(&media_progress),
                    Arc::clone(&total_cost),
                    ctx.clone(),
                );

                // В агентному режимі патчимо segments.json фактичними шляхами медіафайлів
                if is_agent_mode {
                    if let Ok(()) = &video_result {
                        let save_dir = std::path::Path::new(&settings.save_path);
                        if let Err(e) = assign_media_to_timeline(save_dir) {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!("assign_media warning: {}", e),
                            );
                        } else {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                "Timeline patched with media paths.",
                            );
                        }
                    }
                }

                // Пауза для контролю зображень
                if settings.media_control_enabled && settings.video_enabled {
                    if let Ok(()) = &video_result {
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            "Video done. Awaiting media review...",
                        );
                        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMediaControl;
                        ctx.request_repaint();
                        let (lock, cvar) = &*media_control_resume;
                        let mut resumed = lock.lock().unwrap();
                        while !*resumed {
                            resumed = cvar.wait(resumed).unwrap();
                        }
                        *resumed = false;
                        if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                            super::set_job_error_status(&status, e);
                            ctx.request_repaint();
                            return;
                        }
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            "Media review confirmed. Resuming...",
                        );
                        *status.lock().unwrap() = crate::queue::JobStatus::Running;
                        ctx.request_repaint();
                    }
                }

                if let Err(e) = video_result {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = run_final_stages(
                    job_id,
                    &job_name,
                    &settings,
                    &translated_text,
                    &audio_duration,
                    &status,
                    &montage_stage,
                    &montage_progress,
                    &montage_file_size,
                    &montage_control_resume,
                    &capcut_mode_override,
                    &ctx,
                ) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
                ctx.request_repaint();
            });
        }

        // Повтор лише субтитрів → (агент + відеоряд якщо агентний режим) → монтаж
        Subtitles => {
            *subtitles_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();
                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Retry: subtitles...");
                if let Err(e) =
                    run_subtitles_only(&settings, job_id, &job_name, &subtitles_stage, &ctx)
                {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                // В агентному режимі після субтитрів запускаємо агента та відеогілку
                // (якщо відеоряд ще не був виконаний раніше)
                let is_agent_mode = settings.video_enabled && settings.is_agent_service();
                let video_already_done =
                    *video_stage.lock().unwrap() == crate::queue::StageStatus::Done;

                if is_agent_mode && !video_already_done {
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
                            super::set_job_error_status(&status, e);
                            ctx.request_repaint();
                            return;
                        }
                    }

                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Agent mode: processing segments.json...",
                    );
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
                        crate::logger::log_job(
                            job_id,
                            &job_name,
                            &format!("Agent timeline error: {}", e),
                        );
                        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }

                    let video_result = run_video_branch(
                        job_id,
                        job_name.clone(),
                        settings.clone(),
                        Arc::clone(&translated_text),
                        Arc::clone(&video_stage),
                        Arc::clone(&prompts_progress),
                        Arc::clone(&media_progress),
                        Arc::clone(&total_cost),
                        ctx.clone(),
                    );

                    if let Ok(()) = &video_result {
                        let save_dir = std::path::Path::new(&settings.save_path);
                        if let Err(e) = assign_media_to_timeline(save_dir) {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!("assign_media warning: {}", e),
                            );
                        } else {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                "Timeline patched with media paths.",
                            );
                        }
                    }

                    if let Err(e) = video_result {
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }

                    if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                        super::set_job_error_status(&status, e);
                        ctx.request_repaint();
                        return;
                    }
                }

                if let Err(e) = run_final_stages(
                    job_id,
                    &job_name,
                    &settings,
                    &translated_text,
                    &audio_duration,
                    &status,
                    &montage_stage,
                    &montage_progress,
                    &montage_file_size,
                    &montage_control_resume,
                    &capcut_mode_override,
                    &ctx,
                ) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
                ctx.request_repaint();
            });
        }

        // Повтор лише монтажу (timeline + montage)
        Montage => {
            *montage_stage.lock().unwrap() = SPending;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();
                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Retry: montage...");
                if let Err(e) = run_final_stages(
                    job_id,
                    &job_name,
                    &settings,
                    &translated_text,
                    &audio_duration,
                    &status,
                    &montage_stage,
                    &montage_progress,
                    &montage_file_size,
                    &montage_control_resume,
                    &capcut_mode_override,
                    &ctx,
                ) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = super::ensure_job_not_cancelled(job_id) {
                    super::set_job_error_status(&status, e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
                ctx.request_repaint();
            });
        }
    }
}
