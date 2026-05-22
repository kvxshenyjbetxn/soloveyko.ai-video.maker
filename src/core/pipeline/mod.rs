pub mod translate;
pub mod voiceover;

use std::sync::{Arc, Mutex};
use eframe::egui;

/// Виконує весь пайплайн у фоновому потоці.
/// Послідовно запускає увімкнені етапи: переклад → озвучка.
pub fn run_pipeline(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    translation_stage: Arc<Mutex<crate::queue::StageStatus>>,
    voiceover_stage: Arc<Mutex<crate::queue::StageStatus>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        crate::logger::log_job(job_id, &job_name, "Початок виконання задачі.");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();

        // Текст, який буде передано в озвучку (оригінал або результат перекладу)
        let mut voice_text = settings.text.clone();

        // Етап 1: Переклад
        if settings.translation_enabled {
            crate::logger::log_job(job_id, &job_name, "Запуск етапу перекладу...");
            *translation_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            match translate::translate_text(
                &settings.translation_service,
                &settings.openrouter_key,
                &settings.translation_model,
                &settings.translation_prompt,
                &settings.text,
                settings.translation_temperature,
                Some((job_id, job_name.clone())),
            ) {
                Ok(translated) => {
                    let dir = std::path::Path::new(&settings.save_path);
                    if std::fs::create_dir_all(dir).is_ok() {
                        let _ = std::fs::write(dir.join("text.txt"), &translated);
                    }
                    crate::logger::log_job(job_id, &job_name, "Переклад збережено: text.txt");
                    voice_text = translated;
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                    ctx.request_repaint();
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Помилка перекладу: {}", e));
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }
            }
        }

        // Етап 2: Озвучка
        if settings.voiceover_enabled {
            let src_label = if settings.translation_enabled { "переклад" } else { "оригінал" };
            crate::logger::log_job(
                job_id,
                &job_name,
                &format!("Запуск озвучки (джерело тексту: {})...", src_label),
            );
            *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            match voiceover::run_voiceover_sync(
                job_id,
                &job_name,
                &settings,
                &voice_text,
            ) {
                Ok(_) => {
                    crate::logger::log_job(job_id, &job_name, "Озвучку завершено.");
                    *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                    ctx.request_repaint();
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Помилка озвучки: {}", e));
                    *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }
            }
        }

        crate::logger::log_job(job_id, &job_name, "Задачу успішно завершено.");
        *status.lock().unwrap() = crate::queue::JobStatus::Done;
        ctx.request_repaint();
    });
}
