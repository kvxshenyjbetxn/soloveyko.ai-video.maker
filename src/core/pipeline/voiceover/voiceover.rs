use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::io::Read;

/// Синхронно виконує озвучку тексту через обраного провайдера (Voice Bot API або Edge TTS).
pub fn run_voiceover_sync(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    text: &str,
) -> Result<(), String> {
    if settings.voiceover_provider == "Edge TTS" {
        run_edge_tts_voiceover(job_id, job_name, settings, text)
    } else {
        run_voicebot_voiceover(job_id, job_name, settings, text)
    }
}

/// Виконує озвучку через Voice Bot API.
fn run_voicebot_voiceover(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    text: &str,
) -> Result<(), String> {
    let template_uuid = &settings.voiceover_template_uuid;
    let voicebot_key = &settings.voicebot_key;
    let save_path = &settings.save_path;

    let template_opt = if template_uuid.is_empty() { None } else { Some(template_uuid.as_str()) };

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

/// Виконує озвучку через безкоштовний Edge TTS з розбиттям на чанки, лімітером потоків та повторними спробами.
fn run_edge_tts_voiceover(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    text: &str,
) -> Result<(), String> {
    let save_path = Path::new(&settings.save_path);
    let final_output_path = save_path.join("voice.mp3");

    // За замовчуванням голос Polina Neural для української, якщо не вибрано
    let voice_id = if settings.edge_tts_voice.is_empty() {
        "uk-UA-PolinaNeural"
    } else {
        &settings.edge_tts_voice
    };

    crate::logger::log_job(
        job_id,
        job_name,
        &format!("[EdgeTTS] Початок озвучки. Голос: {}, Темп: {}, Тональність: {}, Гучність: {}", 
            voice_id, settings.edge_tts_rate, settings.edge_tts_pitch, settings.edge_tts_volume
        ),
    );

    // Ліміт символів кирилиці / тексту для Edge TTS
    let char_limit = 6000;
    let chunks = split_text_by_chunks(text, char_limit);
    let total_chunks_count = chunks.len();

    if total_chunks_count > 1 {
        crate::logger::log_job(
            job_id,
            job_name,
            &format!("[EdgeTTS] Текст задовгий (символів: {}). Розбиваємо на {} чанків (ліміт: {})...", 
                text.chars().count(), total_chunks_count, char_limit
            ),
        );
    }

    let temp_dir = save_path.join("temp_edgetts");
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        return Err(format!("Не вдалося створити тимчасову папку для чанків озвучки: {}", e));
    }

    let mut thread_handles = Vec::new();
    let first_error = Arc::new(Mutex::new(None));
    let mut chunk_paths = Vec::new();

    for idx in 0..total_chunks_count {
        chunk_paths.push(temp_dir.join(format!("chunk_{:03}.mp3", idx)));
    }

    for (idx, chunk) in chunks.into_iter().enumerate() {
        let chunk_path = chunk_paths[idx].clone();
        let voice_id_clone = voice_id.to_string();
        let rate_clone = settings.edge_tts_rate.clone();
        let pitch_clone = settings.edge_tts_pitch.clone();
        let volume_clone = settings.edge_tts_volume.clone();

        let first_error_clone = Arc::clone(&first_error);
        let job_id = job_id;
        let job_name_clone = job_name.to_string();

        let handle = std::thread::spawn(move || {
            // Отримуємо дозвіл від лімітера потоків
            let _permit = crate::api::edgetts::EdgeTTSLimiter::get().acquire();

            if first_error_clone.lock().unwrap().is_some() {
                return;
            }

            crate::logger::log_job(
                job_id,
                &job_name_clone,
                &format!("[EdgeTTS] Запуск синтезу чанку {}/{} (символів: {})...", idx + 1, total_chunks_count, chunk.chars().count()),
            );

            let backoffs = [
                std::time::Duration::from_secs(5),
                std::time::Duration::from_secs(10),
                std::time::Duration::from_secs(15),
            ];

            let mut attempt_err = None;
            for attempt in 0..=3 {
                if attempt > 0 {
                    crate::logger::log_job(
                        job_id,
                        &job_name_clone,
                        &format!("[EdgeTTS] Чанк {}/{} спроба повтору {}/3 після {:?}", idx + 1, total_chunks_count, attempt, backoffs[attempt - 1]),
                    );
                    std::thread::sleep(backoffs[attempt - 1]);
                }

                if first_error_clone.lock().unwrap().is_some() {
                    return;
                }

                // Екрануємо/налаштовуємо параметри перед синтезом
                let rate_param = if rate_clone.is_empty() { "0" } else { &rate_clone };
                let pitch_param = if pitch_clone.is_empty() { "0" } else { &pitch_clone };
                let volume_param = if volume_clone.is_empty() { "0" } else { &volume_clone };

                match crate::api::edgetts::synthesize(
                    &chunk,
                    &voice_id_clone,
                    rate_param,
                    pitch_param,
                    volume_param,
                    chunk_path.to_str().unwrap(),
                ) {
                    Ok(_) => {
                        crate::logger::log_job(
                            job_id,
                            &job_name_clone,
                            &format!("[EdgeTTS] Чанк {}/{} успішно синтезовано.", idx + 1, total_chunks_count),
                        );
                        return;
                    }
                    Err(e) => {
                        crate::logger::log_job(
                            job_id,
                            &job_name_clone,
                            &format!("[EdgeTTS] Помилка синтезу чанку {}/{} (спроба {}): {}", idx + 1, total_chunks_count, attempt + 1, e),
                        );
                        attempt_err = Some(e);
                    }
                }
            }

            if let Some(err) = attempt_err {
                let mut lock = first_error_clone.lock().unwrap();
                if lock.is_none() {
                    *lock = Some(format!("Помилка синтезу чанку {}/{}: {}", idx + 1, total_chunks_count, err));
                }
            }
        });

        thread_handles.push(handle);
    }

    for handle in thread_handles {
        let _ = handle.join();
    }

    // Перевіряємо, чи були якісь помилки в фонових процесах
    if let Some(err_msg) = first_error.lock().unwrap().clone() {
        return Err(err_msg);
    }

    // Склеюємо чанки
    if total_chunks_count > 1 {
        crate::logger::log_job(job_id, job_name, "[EdgeTTS] Об'єднуємо чанки озвучки через FFmpeg...");
        match merge_audio_ffmpeg(&chunk_paths, &final_output_path) {
            Ok(_) => {
                crate::logger::log_job(job_id, job_name, "[EdgeTTS] Чанки успішно об'єднано через FFmpeg.");
            }
            Err(e) => {
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("[EdgeTTS] Попередження: Не вдалося об'єднати через FFmpeg ({}). Використовуємо надійне бінарне злиття байтів...", e),
                );
                if let Err(binary_err) = merge_audio_binary(&chunk_paths, &final_output_path) {
                    return Err(format!("Не вдалося об'єднати аудіофайли бінарним методом: {}", binary_err));
                }
                crate::logger::log_job(job_id, job_name, "[EdgeTTS] Чанки успішно об'єднано через пряме бінарне злиття.");
            }
        }
    } else if total_chunks_count == 1 {
        // Якщо лише один чанк, просто перейменовуємо/копіюємо файл
        if let Err(e) = std::fs::rename(&chunk_paths[0], &final_output_path) {
            return Err(format!("Не вдалося перемістити тимчасовий файл озвучки: {}", e));
        }
    }

    // Видаляємо тимчасову папку
    let _ = std::fs::remove_dir_all(&temp_dir);

    crate::logger::log_job(job_id, job_name, "[EdgeTTS] Озвучку успішно завершено.");
    Ok(())
}

/// Акуратне розбиття тексту на чанки до `max_chars` символів, зберігаючи цілісність слів та абзаців.
fn split_text_by_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for paragraph in text.split('\n') {
        let paragraph = paragraph.trim();
        if paragraph.is_empty() {
            continue;
        }

        let p_len = paragraph.chars().count();

        if current_chunk.chars().count() + p_len + 1 > max_chars {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }

            if p_len > max_chars {
                let words: Vec<&str> = paragraph.split_whitespace().collect();
                for word in words {
                    let w_len = word.chars().count();
                    if current_chunk.chars().count() + w_len + 1 > max_chars {
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.clone());
                            current_chunk.clear();
                        }
                    }
                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }
                    current_chunk.push_str(word);
                }
            } else {
                current_chunk.push_str(paragraph);
            }
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(paragraph);
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Об'єднання аудіофайлів через FFmpeg за допомогою concat demuxer.
fn merge_audio_ffmpeg(chunk_paths: &[PathBuf], output_path: &Path) -> Result<(), String> {
    let parent_dir = output_path.parent().ok_or("Cannot get parent dir of output path")?;
    let concat_list_path = parent_dir.join("concat_list.txt");

    let mut file_content = String::new();
    for path in chunk_paths {
        if let Some(file_name) = path.file_name() {
            let name_str = file_name.to_string_lossy().replace('\\', "/");
            let escaped = name_str.replace('\'', "'\\''");
            file_content.push_str(&format!("file '{}'\n", escaped));
        } else {
            return Err("Не вдалося отримати назву файлу чанку".to_string());
        }
    }

    std::fs::write(&concat_list_path, file_content)
        .map_err(|e| format!("Не вдалося створити concat_list.txt: {}", e))?;

    #[cfg(target_os = "windows")]
    let ffmpeg_cmd = "ffmpeg.exe";
    #[cfg(not(target_os = "windows"))]
    let ffmpeg_cmd = "ffmpeg";

    let output = std::process::Command::new(ffmpeg_cmd)
        .current_dir(parent_dir)
        .args(&[
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            "concat_list.txt",
            "-c",
            "copy",
            output_path.file_name().unwrap().to_str().unwrap(),
        ])
        .output();

    let _ = std::fs::remove_file(concat_list_path);

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                Err(format!("FFmpeg повернув помилку: {}", err_msg))
            }
        }
        Err(e) => Err(format!("Не вдалося запустити FFmpeg: {}", e)),
    }
}

/// Пряме бінарне склеювання MP3-файлів чанків в один файл (fallback).
fn merge_audio_binary(chunk_paths: &[PathBuf], output_path: &Path) -> Result<(), String> {
    let mut output_file = std::fs::File::create(output_path)
        .map_err(|e| format!("Не вдалося створити фінальний файл озвучки: {}", e))?;

    for path in chunk_paths {
        let mut chunk_file = std::fs::File::open(path)
            .map_err(|e| format!("Не вдалося відкрити файл чанку {:?}: {}", path, e))?;

        let mut buffer = Vec::new();
        chunk_file.read_to_end(&mut buffer)
            .map_err(|e| format!("Не вдалося прочитати файл чанку {:?}: {}", path, e))?;

        use std::io::Write;
        output_file.write_all(&buffer)
            .map_err(|e| format!("Не вдалося записати чанк у фінальний файл: {}", e))?;
    }

    Ok(())
}
