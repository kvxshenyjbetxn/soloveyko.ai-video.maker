/// Синхронно виконує озвучку тексту через Voice Bot API.
/// Опитує статус задачі кожні 5 секунд, логи прив'язані до job_id.
pub fn run_voiceover_sync(
    job_id: u64,
    job_name: &str,
    voicebot_key: &str,
    template_uuid: &str,
    text: &str,
    save_path: &str,
) -> Result<(), String> {
    let template_opt = if template_uuid.is_empty() { None } else { Some(template_uuid) };

    let task_id = crate::api::voicebot::create_tts_task(voicebot_key, text, template_opt)?;

    crate::logger::log_job(
        job_id,
        job_name,
        &format!("TTS задачу створено (ID: {}). Опитуємо статус кожні 5 сек...", task_id),
    );

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));

        let task_status = crate::api::voicebot::get_task_status(voicebot_key, task_id)?;

        crate::logger::log_job(
            job_id,
            job_name,
            &format!("Статус TTS (ID: {}): {}", task_id, task_status),
        );

        match task_status.as_str() {
            "ending" | "ending_processed" => {
                let filename =
                    crate::api::voicebot::download_task_result(voicebot_key, task_id, save_path)?;
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Файл озвучки збережено: {}", filename),
                );
                return Ok(());
            }
            "error" | "error_handled" => {
                return Err(format!(
                    "Сервер повернув помилку обробки TTS (статус: {})",
                    task_status
                ));
            }
            _ => {
                // waiting або processing — продовжуємо опитування
            }
        }
    }
}
