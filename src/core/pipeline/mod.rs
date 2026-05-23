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
    translated_text: Arc<Mutex<Option<String>>>,
    translation_cost: Arc<Mutex<Option<f64>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        crate::logger::log_job(job_id, &job_name, "Початок виконання задачі.");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();

        // Текст, який буде передано в озвучку (оригінал або результат перекладу)
        let mut voice_text = settings.text.clone();

        // Перевіряємо, чи переклад уже був виконаний раніше (наприклад, при повторному запуску після контролю)
        let has_translation = if settings.translation_enabled {
            let tr_stage = translation_stage.lock().unwrap().clone();
            tr_stage == crate::queue::StageStatus::Done
        } else {
            false
        };

        // Етап 1: Переклад
        if settings.translation_enabled && !has_translation {
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
                Ok((translated, cost)) => {
                    let dir = std::path::Path::new(&settings.save_path);
                    if std::fs::create_dir_all(dir).is_ok() {
                        let _ = std::fs::write(dir.join("text.txt"), &translated);
                    }
                    crate::logger::log_job(job_id, &job_name, "Переклад збережено: text.txt");
                    voice_text = translated.clone();
                    *translated_text.lock().unwrap() = Some(translated);
                    *translation_cost.lock().unwrap() = cost;
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Done;

                    if settings.translation_control_enabled {
                        crate::logger::log_job(job_id, &job_name, "Переклад виконано. Задача очікує на контроль перекладу.");
                        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingControl;
                        ctx.request_repaint();
                        return; // Зупиняємо пайплайн для контролю
                    }
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
        } else if settings.translation_enabled && has_translation {
            // Якщо переклад уже виконано і ми продовжуємо після контролю
            if let Some(text) = translated_text.lock().unwrap().as_ref() {
                voice_text = text.clone();
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
