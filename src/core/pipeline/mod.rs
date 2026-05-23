pub mod translate;
pub mod voiceover;
pub mod timeline;

use std::sync::{Arc, Condvar, Mutex};
use eframe::egui;

/// Простий лічильний семафор для обмеження паралельних потоків.
struct Semaphore {
    count:   Mutex<usize>,
    condvar: Condvar,
}

impl Semaphore {
    fn new(n: usize) -> Self {
        Self { count: Mutex::new(n), condvar: Condvar::new() }
    }
    fn acquire(&self) {
        let mut guard = self.count.lock().unwrap();
        while *guard == 0 {
            guard = self.condvar.wait(guard).unwrap();
        }
        *guard -= 1;
    }
    fn release(&self) {
        *self.count.lock().unwrap() += 1;
        self.condvar.notify_one();
    }
}

/// Визначає тривалість WAV-файлу в секундах, зчитуючи RIFF/fmt заголовок.
fn get_wav_duration_secs(path: &std::path::Path) -> Option<f64> {
    let data = std::fs::read(path).ok()?;

    // Перевіряємо RIFF підпис та WAVE формат
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }

    // Шукаємо fmt чанк
    let mut i = 12usize;
    let mut byte_rate = 0u32;
    let mut found_fmt = false;
    while i + 8 <= data.len() {
        let chunk_id = &data[i..i + 4];
        let chunk_size = u32::from_le_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]) as usize;

        if chunk_id == b"fmt " && chunk_size >= 16 && i + 8 + chunk_size <= data.len() {
            byte_rate = u32::from_le_bytes([data[i+12], data[i+13], data[i+14], data[i+15]]);
            found_fmt = true;
        }

        if chunk_id == b"data" && found_fmt && byte_rate > 0 {
            let data_size = chunk_size.min(data.len() - i - 8);
            return Some(data_size as f64 / byte_rate as f64);
        }

        i += 8 + chunk_size + (chunk_size % 2); // вирівнювання до парного байту
    }

    None
}

/// Визначає тривалість аудіофайлу в секундах.
/// Підтримує MP3 (MPEG1 Layer3) та WAV (RIFF).
fn get_audio_duration_secs(path: &std::path::Path) -> Option<f64> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "wav" {
        return get_wav_duration_secs(path);
    }
    get_mp3_duration_secs(path)
}

/// Визначає тривалість MP3-файлу в секундах, розбираючи заголовки MPEG-фреймів.
/// Підтримує Xing/Info VBR заголовок (точно) та CBR оцінку по розміру файлу.
fn get_mp3_duration_secs(path: &std::path::Path) -> Option<f64> {
    let data = std::fs::read(path).ok()?;

    // Пропускаємо ID3v2 тег
    let start = if data.len() >= 10 && &data[0..3] == b"ID3" {
        let tag_size = ((data[6] as usize) << 21)
            | ((data[7] as usize) << 14)
            | ((data[8] as usize) << 7)
            | (data[9] as usize);
        10 + tag_size
    } else {
        0
    };

    if start >= data.len() {
        return None;
    }

    let slice = &data[start..];

    // Шукаємо перший валідний MPEG1 Layer3 фрейм
    let mut i = 0;
    while i + 4 <= slice.len() {
        if slice[i] != 0xFF || (slice[i + 1] & 0xE0) != 0xE0 {
            i += 1;
            continue;
        }

        let b1 = slice[i + 1];
        let b2 = slice[i + 2];

        let mpeg_version = (b1 >> 3) & 0x03; // 0x03 = MPEG1
        let layer = (b1 >> 1) & 0x03;        // 0x01 = Layer3
        let bitrate_idx = (b2 >> 4) & 0x0F;
        let samplerate_idx = (b2 >> 2) & 0x03;
        let channel_mode = (slice[i + 3] >> 6) & 0x03; // 0x03 = Mono

        if bitrate_idx == 0x0F || samplerate_idx == 0x03 || layer == 0
            || mpeg_version == 0x01 || mpeg_version != 0x03 || layer != 0x01
        {
            i += 1;
            continue;
        }

        let bitrates: [u32; 16] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
        let samplerates: [u32; 3] = [44100, 48000, 32000];

        let bitrate_kbps = bitrates[bitrate_idx as usize];
        let samplerate = samplerates[samplerate_idx as usize];

        if bitrate_kbps == 0 || samplerate == 0 {
            i += 1;
            continue;
        }

        let samples_per_frame = 1152u32;

        // Перевіряємо наявність Xing/Info VBR заголовку
        let side_info_size = if channel_mode == 0x03 { 17usize } else { 32usize };
        let xing_offset = i + 4 + side_info_size;

        if xing_offset + 12 <= slice.len() {
            let tag = &slice[xing_offset..xing_offset + 4];
            if tag == b"Xing" || tag == b"Info" {
                let flags = u32::from_be_bytes([
                    slice[xing_offset + 4],
                    slice[xing_offset + 5],
                    slice[xing_offset + 6],
                    slice[xing_offset + 7],
                ]);
                // Прапор 0x01 = присутня загальна кількість фреймів
                if flags & 0x01 != 0 && xing_offset + 16 <= slice.len() {
                    let total_frames = u32::from_be_bytes([
                        slice[xing_offset + 8],
                        slice[xing_offset + 9],
                        slice[xing_offset + 10],
                        slice[xing_offset + 11],
                    ]);
                    if total_frames > 0 {
                        return Some(total_frames as f64 * samples_per_frame as f64 / samplerate as f64);
                    }
                }
            }
        }

        // CBR: оцінка тривалості по розміру аудіо-даних та бітрейту
        let audio_bytes = slice.len().saturating_sub(i) as f64;
        return Some(audio_bytes * 8.0 / (bitrate_kbps as f64 * 1000.0));
    }

    None
}

/// Декодує результат генерації (data URI або HTTP URL) і зберігає файл.
/// Повертає шлях до збереженого файлу.
fn decode_result(
    result: &str,
    index: usize,
    _total: usize,
    images_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    if result.starts_with("data:") {
        // data:[<mediatype>][;base64],<data>
        let rest = &result[5..];
        let comma = rest.find(',').ok_or("Invalid data URI: no comma")?;
        let header = &rest[..comma];
        let b64    = &rest[comma + 1..];

        let ext = if header.contains("png")  { "png" }
             else if header.contains("webp") { "webp" }
             else if header.contains("gif")  { "gif" }
             else                            { "jpg" };

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        let path = images_dir.join(format!("{:04}.{}", index, ext));
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    } else {
        // Звичайний HTTP URL
        let ext = result.split('?').next().unwrap_or(result)
            .rsplit('.').next()
            .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("jpg");
        let path = images_dir.join(format!("{:04}.{}", index, ext));

        use std::io::Read;
        let resp = ureq::get(result).call()
            .map_err(|e| format!("Download error: {}", e))?;
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)
            .map_err(|e| format!("Read error: {}", e))?;
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    }
}

/// Виконує весь пайплайн у фоновому потоці.
/// Послідовно запускає увімкнені етапи: переклад → озвучка → відеоряд.
pub fn run_pipeline(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    translation_stage: Arc<Mutex<crate::queue::StageStatus>>,
    voiceover_stage: Arc<Mutex<crate::queue::StageStatus>>,
    video_stage: Arc<Mutex<crate::queue::StageStatus>>,
    translated_text: Arc<Mutex<Option<String>>>,
    translation_cost: Arc<Mutex<Option<f64>>>,
    audio_duration: Arc<Mutex<Option<f64>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        crate::logger::log_job(job_id, &job_name, "Job started.");
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
            crate::logger::log_job(job_id, &job_name, "Starting translation stage...");
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
                    crate::logger::log_job(job_id, &job_name, "Translation saved: text.txt");
                    voice_text = translated.clone();
                    *translated_text.lock().unwrap() = Some(translated);
                    *translation_cost.lock().unwrap() = cost;
                    *translation_stage.lock().unwrap() = crate::queue::StageStatus::Done;

                    if settings.translation_control_enabled {
                        crate::logger::log_job(job_id, &job_name, "Translation done. Job is awaiting translation review.");
                        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingControl;
                        ctx.request_repaint();
                        return; // Зупиняємо пайплайн для контролю
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Translation error: {}", e));
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
            let src_label = if settings.translation_enabled { "translation" } else { "original" };
            crate::logger::log_job(
                job_id,
                &job_name,
                &format!("Starting voiceover (text source: {})...", src_label),
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
                    crate::logger::log_job(job_id, &job_name, "Voiceover done.");

                    let save_dir = std::path::Path::new(&settings.save_path);
                    let mp3_path = save_dir.join("voice.mp3");

                    // Конвертація в WAV через FFmpeg, якщо увімкнено
                    let final_audio_path = if settings.voiceover_convert_to_wav && mp3_path.exists() {
                        let wav_path = save_dir.join("voice.wav");
                        crate::logger::log_job(job_id, &job_name, "Converting audio to WAV via FFmpeg...");

                        #[cfg(target_os = "windows")]
                        let ffmpeg_cmd = "ffmpeg.exe";
                        #[cfg(not(target_os = "windows"))]
                        let ffmpeg_cmd = "ffmpeg";

                        let result = std::process::Command::new(ffmpeg_cmd)
                            .args(&[
                                "-y", "-hide_banner", "-loglevel", "error",
                                "-i", mp3_path.to_str().unwrap_or("voice.mp3"),
                                wav_path.to_str().unwrap_or("voice.wav"),
                            ])
                            .output();

                        match result {
                            Ok(out) if out.status.success() => {
                                crate::logger::log_job(job_id, &job_name, "WAV conversion successful.");
                                wav_path
                            }
                            Ok(out) => {
                                let err = String::from_utf8_lossy(&out.stderr);
                                crate::logger::log_job(
                                    job_id, &job_name,
                                    &format!("WAV conversion failed: {}. Using MP3.", err),
                                );
                                mp3_path
                            }
                            Err(e) => {
                                crate::logger::log_job(
                                    job_id, &job_name,
                                    &format!("FFmpeg not found for WAV conversion: {}. Using MP3.", e),
                                );
                                mp3_path
                            }
                        }
                    } else {
                        mp3_path
                    };

                    // Визначаємо тривалість фінального аудіофайлу
                    if let Some(dur) = get_audio_duration_secs(&final_audio_path) {
                        crate::logger::log_job(
                            job_id, &job_name,
                            &format!("Audio duration: {:.1}s", dur),
                        );
                        *audio_duration.lock().unwrap() = Some(dur);
                    }

                    *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                    ctx.request_repaint();
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Voiceover error: {}", e));
                    *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }
            }
        }

        // Етап 3: Відеоряд
        if settings.video_enabled {
            crate::logger::log_job(job_id, &job_name, "Starting video stage (image generation)...");
            *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            // Визначаємо текст: перекладений якщо є, інакше оригінал
            let source_text = if settings.translation_enabled {
                translated_text.lock().unwrap().clone().unwrap_or_else(|| settings.text.clone())
            } else {
                settings.text.clone()
            };

            // Нарізаємо текст на сегменти
            let segments = crate::core::pipeline::timeline::text_splitter::split_text(
                &source_text,
                &settings.text_split_mode,
                settings.text_split_char_limit,
            );

            crate::logger::log_job(
                job_id, &job_name,
                &format!("Text split into {} segments (mode: {})", segments.len(), settings.text_split_mode),
            );

            // Зберігаємо debug-файл сегментів
            let save_dir = std::path::Path::new(&settings.save_path);
            let segments_path = save_dir.join("segments.txt");
            {
                let mut content = format!("=== Сегменти тексту: {} ===\n", segments.len());
                for (i, seg) in segments.iter().enumerate() {
                    content.push_str(&format!("\n[{}/{}]\n{}\n", i + 1, segments.len(), seg));
                }
                let _ = std::fs::write(&segments_path, &content);
                crate::logger::log_job(job_id, &job_name, "Segments saved: segments.txt");
            }

            // Створюємо папку images/
            let images_dir = save_dir.join("images");
            if let Err(e) = std::fs::create_dir_all(&images_dir) {
                crate::logger::log_job(job_id, &job_name, &format!("Cannot create images dir: {}", e));
                *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                *status.lock().unwrap() = crate::queue::JobStatus::Failed(e.to_string());
                ctx.request_repaint();
                return;
            }

            // Паралельна генерація зображень із семафором
            let sem = Arc::new(Semaphore::new(settings.googler_image_max_threads.max(1)));
            let mut handles = Vec::new();

            for (i, segment) in segments.iter().enumerate() {
                let sem        = Arc::clone(&sem);
                let key        = settings.googler_key.clone();
                let priority   = settings.googler_image_priority.clone();
                let images_dir = images_dir.clone();
                let prompt     = settings.video_prompt.replace("{{text}}", segment);
                let job_id_c   = job_id;
                let job_name_c = job_name.clone();
                let total      = segments.len();

                let handle = std::thread::spawn(move || -> (usize, Result<(), String>) {
                    sem.acquire();

                    crate::logger::log_job(
                        job_id_c, &job_name_c,
                        &format!("Generating image {}/{} ...", i + 1, total),
                    );

                    let result = crate::api::googler::generate_image_with_priority(
                        &key,
                        &prompt,
                        "16:9",
                        &priority,
                    );

                    sem.release();

                    match result {
                        Err(e) => (i, Err(e)),
                        Ok(data_uri) => {
                            match decode_result(&data_uri, i + 1, total, &images_dir) {
                                Err(e) => (i, Err(e)),
                                Ok(path) => {
                                    crate::logger::log_job(
                                        job_id_c, &job_name_c,
                                        &format!("Image {}/{} saved: {}", i + 1, total, path.display()),
                                    );
                                    (i, Ok(()))
                                }
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            // Збираємо результати
            let mut errors: Vec<String> = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok((i, Err(e))) => {
                        let msg = format!("Segment {}: {}", i + 1, e);
                        crate::logger::log_job(job_id, &job_name, &format!("Image error — {}", msg));
                        errors.push(msg);
                    }
                    Err(_) => errors.push("Thread panic during image generation".to_string()),
                    _ => {}
                }
            }

            if errors.is_empty() {
                crate::logger::log_job(job_id, &job_name, "All images generated successfully.");
                *video_stage.lock().unwrap() = crate::queue::StageStatus::Done;
            } else {
                let msg = format!("Image generation failed ({} errors). First: {}", errors.len(), errors[0]);
                crate::logger::log_job(job_id, &job_name, &msg);
                *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                *status.lock().unwrap() = crate::queue::JobStatus::Failed(msg);
                ctx.request_repaint();
                return;
            }

            ctx.request_repaint();
        }

        crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
        *status.lock().unwrap() = crate::queue::JobStatus::Done;
        ctx.request_repaint();
    });
}
