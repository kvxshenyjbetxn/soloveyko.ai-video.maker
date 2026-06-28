use std::sync::{Arc, Condvar, Mutex};

use eframe::egui;

/// Виконує timeline + montage (спільна фінальна частина для retry-функцій).
pub(super) fn run_final_stages(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    translated_text: &Arc<Mutex<Option<String>>>,
    audio_duration: &Arc<Mutex<Option<f64>>>,
    status: &Arc<Mutex<crate::queue::JobStatus>>,
    montage_stage: &Arc<Mutex<crate::queue::StageStatus>>,
    montage_progress: &Arc<Mutex<Option<f32>>>,
    montage_file_size: &Arc<Mutex<Option<u64>>>,
    montage_control_resume: &Arc<(Mutex<bool>, Condvar)>,
    capcut_mode_override: &Arc<Mutex<Option<bool>>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    // Timeline — в агентному режимі або при відновленні segments.json вже є, не перезаписуємо
    let is_agent_mode = settings.video_llm_service == "Claude Code"
        || settings.video_llm_service == "Gemini CLI"
        || settings.video_llm_service == "Codex CLI"
        || settings.video_llm_service == "AGY CLI"
        || settings.video_llm_service == "Pi CLI";
    if settings.video_enabled && !is_agent_mode && !settings.skip_agent_on_resume {
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
        let save_dir = std::path::Path::new(&settings.save_path);
        match crate::core::pipeline::timeline::sync::build_timeline(
            save_dir, &segments, audio_dur, job_name,
        ) {
            Ok(_) => crate::logger::log_job(job_id, job_name, "Segments saved: segments.json"),
            Err(e) => crate::logger::log_job(job_id, job_name, &format!("Timeline warning: {}", e)),
        }
    }

    // Пауза контролю монтажу перед рендером (якщо увімкнено)
    if settings.montage_enabled && settings.montage_control_enabled {
        crate::logger::log_job(
            job_id,
            job_name,
            "Awaiting montage control confirmation from user...",
        );
        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMontageControl;
        ctx.request_repaint();

        let (lock, cvar) = &**montage_control_resume;
        let mut resumed = lock.lock().unwrap();
        while !*resumed {
            resumed = cvar.wait(resumed).unwrap();
        }
        *resumed = false;

        super::ensure_job_not_cancelled(job_id)?;

        crate::logger::log_job(
            job_id,
            job_name,
            "Montage control confirmed. Resuming pipeline...",
        );
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();
    }

    // Монтаж (FFmpeg або CapCut)
    if settings.montage_enabled {
        super::ensure_job_not_cancelled(job_id)?;
        crate::logger::log_job(job_id, job_name, "Starting montage stage...");
        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Running;
        ctx.request_repaint();

        let audio_dur = *audio_duration.lock().unwrap();
        let save_dir = std::path::Path::new(&settings.save_path);
        let job_id_log = job_id;
        let job_name_log = job_name.to_string();

        let use_capcut = capcut_mode_override
            .lock()
            .unwrap()
            .unwrap_or(settings.capcut_enabled);
        if use_capcut {
            if settings.capcut_draft_path.is_empty() {
                let msg = "CapCut: не вказано папку чернеток CapCut";
                crate::logger::log_job(job_id, job_name, msg);
                *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(msg.to_string());
            }
            let draft_root = std::path::Path::new(&settings.capcut_draft_path);
            match crate::core::pipeline::capcut::generate_capcut_project(
                save_dir,
                draft_root,
                job_name,
                audio_dur,
                |msg| crate::logger::log_job(job_id_log, &job_name_log, msg),
            ) {
                Ok(_) => {
                    super::ensure_job_not_cancelled(job_id)?;
                    crate::logger::log_job(job_id, job_name, "CapCut project generated.");
                    *montage_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                }
                Err(e) => {
                    let msg = format!("CapCut: {}", e);
                    crate::logger::log_job(job_id, job_name, &msg);
                    *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    return Err(msg);
                }
            }
            return Ok(());
        }

        let montage_progress_arc = Arc::clone(montage_progress);
        let ctx_montage = ctx.clone();

        match crate::core::pipeline::montage::run_montage(
            job_id,
            save_dir,
            job_name,
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
                super::ensure_job_not_cancelled(job_id)?;
                *montage_file_size.lock().unwrap() = Some(size);
                crate::logger::log_job(job_id, job_name, "Montage complete.");
                *montage_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                ctx.request_repaint();
            }
            Err(e) => {
                crate::logger::log_job(job_id, job_name, &format!("Montage failed: {}", e));
                *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(format!("Montage: {}", e));
            }
        }
    }

    Ok(())
}
