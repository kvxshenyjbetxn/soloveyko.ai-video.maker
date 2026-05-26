pub mod voiceover;
pub mod timeline;
pub mod montage;

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
    media_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    if result.starts_with("data:") {
        // data:[<mediatype>][;base64],<data>
        let rest = &result[5..];
        let comma = rest.find(',').ok_or("Invalid data URI: no comma")?;
        let header = &rest[..comma];
        let b64    = &rest[comma + 1..];

        let ext = if header.contains("mp4")  { "mp4" }
             else if header.contains("webm") { "webm" }
             else if header.contains("mov")  { "mov" }
             else if header.contains("png")  { "png" }
             else if header.contains("webp") { "webp" }
             else if header.contains("gif")  { "gif" }
             else                            { "jpg" };

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        let path = media_dir.join(format!("{:04}.{}", index, ext));
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    } else {
        // Звичайний HTTP URL — розширення беремо з URL
        let ext = result.split('?').next().unwrap_or(result)
            .rsplit('.').next()
            .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("jpg");
        let path = media_dir.join(format!("{:04}.{}", index, ext));

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

/// Запускає WhisperX для генерації субтитрів та зберігає результат як subtitle.srt.
fn run_whisperx(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    subtitles_stage: &std::sync::Arc<std::sync::Mutex<crate::queue::StageStatus>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let reason = if settings.subtitles_enabled {
        "Starting subtitle generation via WhisperX (burn-in enabled)..."
    } else {
        "Starting subtitle generation via WhisperX (for timeline sync, burn-in disabled)..."
    };
    crate::logger::log_job(job_id, job_name, reason);
    *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    let save_dir = std::path::Path::new(&settings.save_path);

    // Перевіряємо наявність директорії whisperx у bin_dir
    if !crate::bundle::whisperx_local_exists() {
        let msg = "WhisperX not found. Download it in the Welcome window.".to_string();
        crate::logger::log_job(job_id, job_name, &format!("Subtitles error: {}", msg));
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    }

    // Вибираємо аудіо: WAV у пріоритеті, потім MP3
    let audio_path = if save_dir.join("voice.wav").exists() {
        save_dir.join("voice.wav")
    } else if save_dir.join("voice.mp3").exists() {
        save_dir.join("voice.mp3")
    } else {
        let msg = "Subtitles: audio file not found (voice.wav / voice.mp3)".to_string();
        crate::logger::log_job(job_id, job_name, &format!("Subtitles error: {}", msg));
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    };

    // Знаходимо виконуваний файл whisperx всередині папки bin_dir/whisperx_mac/
    let whisperx_dir = crate::bundle::bin_dir().join("whisperx_mac");
    let whisperx_cmd = whisperx_dir.join("whisperx_cli");

    // whisperx_cli --audio <file> --model <model> --output <base_without_ext>
    //              [--language <lang>] [--ffmpeg-path <ffmpeg>]
    // CLI збереже JSON як <output_base>.json (тобто subtitle.json з word-рівневими мітками).
    // Після цього самостійно генеруємо subtitle.srt із урахуванням max_line_width.
    let output_base = save_dir.join("subtitle");
    let output_json = save_dir.join("subtitle.json");
    let output_srt  = save_dir.join("subtitle.srt");

    let mut args: Vec<String> = vec![
        "--audio".to_string(), audio_path.to_str().unwrap_or("voice.wav").to_string(),
        "--model".to_string(), settings.whisper_model.clone(),
        "--output".to_string(), output_base.to_str().unwrap_or("subtitle").to_string(),
        "--ffmpeg-path".to_string(), crate::bundle::ffmpeg_path(),
    ];
    if settings.whisper_language != "auto" {
        args.push("--language".to_string());
        args.push(settings.whisper_language.clone());
    }

    crate::logger::log_job(
        job_id, job_name,
        &format!("Running: {} {}", whisperx_cmd.display(), args.join(" ")),
    );

    match std::process::Command::new(&whisperx_cmd).args(&args).output() {
        Ok(out) if out.status.success() => {
            // Зчитуємо subtitle.json і генеруємо subtitle.srt з max_line_width
            match std::fs::read_to_string(&output_json) {
                Ok(json_str) => {
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(json) => {
                            let words = json.get("words")
                                .and_then(|w| w.as_array())
                                .map(|v| v.as_slice())
                                .unwrap_or(&[]);

                            let srt = crate::api::assemblyai::whisperx_words_to_srt(
                                words,
                                settings.whisper_max_line_width,
                            );
                            if let Err(e) = std::fs::write(&output_srt, &srt) {
                                crate::logger::log_job(job_id, job_name, &format!("Failed to save subtitle.srt: {}", e));
                            }

                            // Генеруємо subtitle.ass зі стилем запеченим всередині
                            let ass = srt_to_ass(&srt, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
                            let ass_path = save_dir.join("subtitle.ass");
                            if let Err(e) = std::fs::write(&ass_path, &ass) {
                                crate::logger::log_job(job_id, job_name, &format!("Failed to save subtitle.ass: {}", e));
                            }

                            // Зберігаємо тільки words + language (без segments)
                            let filtered = serde_json::json!({
                                "language": json.get("language").cloned().unwrap_or(serde_json::Value::Null),
                                "words": json.get("words").cloned().unwrap_or(serde_json::Value::Array(vec![])),
                            });
                            if let Ok(s) = serde_json::to_string_pretty(&filtered) {
                                let _ = std::fs::write(&output_json, s);
                            }
                        }
                        Err(e) => {
                            crate::logger::log_job(job_id, job_name, &format!("WhisperX: failed to parse subtitle.json: {}", e));
                        }
                    }
                }
                Err(e) => {
                    crate::logger::log_job(job_id, job_name, &format!("WhisperX: subtitle.json not found: {}", e));
                }
            }

            crate::logger::log_job(job_id, job_name, "Subtitles saved: subtitle.srt");
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
            ctx.request_repaint();
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let msg = if !stderr.is_empty() { stderr.to_string() } else { stdout.to_string() };
            let short = format!("WhisperX error: {}", msg.chars().take(120).collect::<String>());
            crate::logger::log_job(job_id, job_name, &format!("WhisperX error: {}", msg.trim()));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(short)
        }
        Err(e) => {
            let msg = format!("WhisperX launch error: {}", e);
            crate::logger::log_job(job_id, job_name, &format!("WhisperX not found or failed to start: {}", e));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(msg)
        }
    }
}

/// Транскрибує аудіо через AssemblyAI та зберігає subtitle.srt і subtitle.json.
fn run_assemblyai(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    subtitles_stage: &std::sync::Arc<std::sync::Mutex<crate::queue::StageStatus>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let reason = if settings.subtitles_enabled {
        "Starting subtitle generation via AssemblyAI (burn-in enabled)..."
    } else {
        "Starting subtitle generation via AssemblyAI (for timeline sync, burn-in disabled)..."
    };
    crate::logger::log_job(job_id, job_name, reason);
    *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    if settings.assemblyai_key.trim().is_empty() {
        let msg = "AssemblyAI key is not set.".to_string();
        crate::logger::log_job(job_id, job_name, &format!("Subtitles error: {}", msg));
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    }

    let save_dir = std::path::Path::new(&settings.save_path);

    let audio_path = if save_dir.join("voice.wav").exists() {
        save_dir.join("voice.wav")
    } else if save_dir.join("voice.mp3").exists() {
        save_dir.join("voice.mp3")
    } else {
        let msg = "Subtitles: audio file not found (voice.wav / voice.mp3)".to_string();
        crate::logger::log_job(job_id, job_name, &format!("Subtitles error: {}", msg));
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    };

    crate::logger::log_job(job_id, job_name, &format!("Uploading audio to AssemblyAI: {}", audio_path.display()));

    match crate::api::assemblyai::transcribe(
        &settings.assemblyai_key,
        &audio_path,
        &settings.whisper_language,
        settings.whisper_max_line_width,
    ) {
        Ok((srt, json_response)) => {
            let srt_path = save_dir.join("subtitle.srt");
            let json_path = save_dir.join("subtitle.json");

            std::fs::write(&srt_path, &srt)
                .map_err(|e| format!("Failed to save subtitle.srt: {}", e))?;

            // Генеруємо subtitle.ass зі стилем запеченим всередині
            let ass = srt_to_ass(&srt, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
            let ass_path = save_dir.join("subtitle.ass");
            if let Err(e) = std::fs::write(&ass_path, &ass) {
                crate::logger::log_job(job_id, job_name, &format!("Failed to save subtitle.ass: {}", e));
            }

            // Зберігаємо лише масив words — решта метадані API-запиту, не потрібні для таймлайну
            let words_only = json_response.get("words").cloned().unwrap_or(serde_json::Value::Array(vec![]));
            if let Ok(json_str) = serde_json::to_string_pretty(&words_only) {
                let _ = std::fs::write(&json_path, json_str);
            }

            crate::logger::log_job(job_id, job_name, "Subtitles saved: subtitle.srt (AssemblyAI)");
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
            ctx.request_repaint();
            Ok(())
        }
        Err(e) => {
            crate::logger::log_job(job_id, job_name, &format!("AssemblyAI error: {}", e));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(e)
        }
    }
}

/// Гілка Озвучка + Субтитри (виконується паралельно з відеорядом).
/// Повертає Ok(()) або Err з описом першої помилки.
fn run_av_branch(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    voice_text: String,
    voiceover_stage: Arc<Mutex<crate::queue::StageStatus>>,
    subtitles_stage: Arc<Mutex<crate::queue::StageStatus>>,
    audio_duration: Arc<Mutex<Option<f64>>>,
    ctx: egui::Context,
) -> Result<(), String> {
    // Озвучка
    if settings.voiceover_enabled {
        let src_label = if settings.translation_enabled { "translation" } else { "original" };
        crate::logger::log_job(
            job_id,
            &job_name,
            &format!("Starting voiceover (text source: {})...", src_label),
        );
        *voiceover_stage.lock().unwrap() = crate::queue::StageStatus::Running;
        ctx.request_repaint();

        match voiceover::run_voiceover_sync(job_id, &job_name, &settings, &voice_text) {
            Ok(_) => {
                crate::logger::log_job(job_id, &job_name, "Voiceover done.");

                let save_dir = std::path::Path::new(&settings.save_path);
                let mp3_path = save_dir.join("voice.mp3");

                // Конвертація в WAV через FFmpeg, якщо увімкнено
                let final_audio_path = if settings.voiceover_convert_to_wav && mp3_path.exists() {
                    let wav_path = save_dir.join("voice.wav");
                    crate::logger::log_job(job_id, &job_name, "Converting audio to WAV via FFmpeg...");

                    let ffmpeg_cmd = crate::bundle::ffmpeg_path();
                    let result = std::process::Command::new(&ffmpeg_cmd)
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
                return Err(e);
            }
        }
    }

    // Субтитри (Whisper/WhisperX/AssemblyAI) — завжди генеруються якщо увімкнено (потрібні для синхронізації відеоряду).
    // Накладання на відео контролюється окремо через параметр burn_subtitles у монтажі.
    if settings.subtitles_service == "WhisperX" {
        run_whisperx(&settings, job_id, &job_name, &subtitles_stage, &ctx)?;
    } else if settings.subtitles_service == "AssemblyAI" {
        run_assemblyai(&settings, job_id, &job_name, &subtitles_stage, &ctx)?;
    } else if settings.subtitles_service == "Whisper" {
        let reason = if settings.subtitles_enabled {
            "Starting subtitle generation via Whisper (burn-in enabled)..."
        } else {
            "Starting subtitle generation via Whisper (for timeline sync, burn-in disabled)..."
        };
        crate::logger::log_job(job_id, &job_name, reason);
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Running;
        ctx.request_repaint();

        let save_dir = std::path::Path::new(&settings.save_path);

        // Перевіряємо наявність ggml-моделі
        let model_path = crate::bundle::whisper_model_path(&settings.whisper_model);
        if !model_path.exists() {
            let msg = format!(
                "Subtitles error: model '{}' not found at '{}'. Download it in the subtitles settings.",
                settings.whisper_model,
                model_path.display()
            );
            crate::logger::log_job(job_id, &job_name, &msg);
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            return Err(msg);
        }

        // Вибираємо аудіо: WAV у пріоритеті, потім MP3
        let audio_path = if save_dir.join("voice.wav").exists() {
            save_dir.join("voice.wav")
        } else if save_dir.join("voice.mp3").exists() {
            save_dir.join("voice.mp3")
        } else {
            let msg = "Subtitles: audio file not found (voice.wav / voice.mp3)".to_string();
            crate::logger::log_job(job_id, &job_name, &format!("Subtitles error: {}", msg));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            return Err(msg);
        };

        // Шлях вихідного файлу без розширення — whisper.cpp додасть .srt сам
        let output_stem = save_dir.join("subtitle");
        let whisper_cmd = crate::bundle::whisper_path();

        // whisper.cpp аргументи: -m <model_file> --output-srt -of <stem> [-l <lang>]
        let mut args: Vec<String> = vec![
            audio_path.to_str().unwrap_or("voice.wav").to_string(),
            "-m".to_string(), model_path.to_str().unwrap().to_string(),
            "--output-srt".to_string(),
            "-of".to_string(), output_stem.to_str().unwrap().to_string(),
        ];
        if settings.whisper_language != "auto" {
            args.push("-l".to_string());
            args.push(settings.whisper_language.clone());
        }
        if settings.whisper_max_line_width > 0 {
            args.push("--max-len".to_string());
            args.push(settings.whisper_max_line_width.to_string());
            // Розбивати лише на межах слів, щоб не обрізати слова посередині
            args.push("--split-on-word".to_string());
        }

        crate::logger::log_job(
            job_id, &job_name,
            &format!("Running: {} {}", whisper_cmd, args.join(" ")),
        );

        match std::process::Command::new(&whisper_cmd).args(&args).output() {
            Ok(out) if out.status.success() => {
                crate::logger::log_job(job_id, &job_name, "Subtitles saved: subtitle.srt");

                // Генеруємо subtitle.ass зі стилем запеченим всередині
                let srt_path = save_dir.join("subtitle.srt");
                if let Ok(srt) = std::fs::read_to_string(&srt_path) {
                    let ass = srt_to_ass(&srt, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
                    let ass_path = save_dir.join("subtitle.ass");
                    if let Err(e) = std::fs::write(&ass_path, &ass) {
                        crate::logger::log_job(job_id, &job_name, &format!("Failed to save subtitle.ass: {}", e));
                    }
                }

                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                ctx.request_repaint();
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let msg = if !stderr.is_empty() { stderr.to_string() } else { stdout.to_string() };
                let short = format!("Whisper error: {}", msg.chars().take(120).collect::<String>());
                crate::logger::log_job(job_id, &job_name, &format!("Whisper error: {}", msg.trim()));
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(short);
            }
            Err(e) => {
                let msg = format!("Whisper launch error: {}", e);
                crate::logger::log_job(job_id, &job_name, &format!("Whisper not found or failed to start: {}", e));
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(msg);
            }
        }
    }

    // Генеруємо subtitle.ass з karaoke-ефектом якщо увімкнено і є subtitle.json з word-мітками
    if settings.subtitle_karaoke
        && (settings.subtitles_service == "WhisperX" || settings.subtitles_service == "AssemblyAI")
    {
        let save_dir = std::path::Path::new(&settings.save_path);
        let json_path = save_dir.join("subtitle.json");
        if json_path.exists() {
            match generate_karaoke_ass(
                &json_path,
                &settings.subtitles_service,
                settings.subtitle_font_size,
                settings.subtitle_color,
                settings.subtitle_margin_v,
            ) {
                Ok(ass_content) => {
                    let ass_path = save_dir.join("subtitle.ass");
                    match std::fs::write(&ass_path, &ass_content) {
                        Ok(_) => crate::logger::log_job(job_id, &job_name, "Karaoke ASS generated: subtitle.ass"),
                        Err(e) => crate::logger::log_job(job_id, &job_name, &format!("Failed to save subtitle.ass (karaoke): {}", e)),
                    }
                }
                Err(e) => crate::logger::log_job(job_id, &job_name, &format!("Karaoke generation error: {}", e)),
            }
        } else {
            crate::logger::log_job(job_id, &job_name, "Karaoke: subtitle.json not found, skipping.");
        }
    }

    Ok(())
}

/// Генерує ASS файл з karaoke-ефектом (\kf теги) з word-level timestamps.
/// Підтримує формати WhisperX (секунди) та AssemblyAI (мілісекунди).
fn generate_karaoke_ass(
    json_path: &std::path::Path,
    service: &str,
    font_size: u32,
    color: [u8; 3],
    margin_v: u32,
) -> Result<String, String> {
    let content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("Cannot read subtitle.json: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse subtitle.json: {}", e))?;

    // Формат кольору ASS: &HAABBGGRR (A=00 = непрозорий)
    let primary = format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]);
    // Жовтий для підсвічування вже вимовленого
    let secondary = "&H0000FFFF".to_string();
    let outline = "&H00000000".to_string();
    let back = "&H80000000".to_string();

    let header = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,Arial,{font_size},{primary},{secondary},{outline},{back},0,0,0,0,100,100,0,0,1,2,1,2,10,10,{margin_v},1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        font_size = font_size,
        primary = primary,
        secondary = secondary,
        outline = outline,
        back = back,
        margin_v = margin_v,
    );

    // Розбираємо слова залежно від сервісу
    struct Word {
        text: String,
        start_ms: u64,
        end_ms: u64,
    }

    let words: Vec<Word> = if service == "WhisperX" {
        // {"language": ..., "words": [{"word": "...", "start": 0.5, "end": 1.0, ...}]}
        let arr = json.get("words").and_then(|w| w.as_array())
            .ok_or("WhisperX subtitle.json: no 'words' array")?;
        arr.iter().filter_map(|w| {
            let text = w.get("word")?.as_str()?.to_string();
            let start = w.get("start")?.as_f64()?;
            let end = w.get("end")?.as_f64()?;
            Some(Word { text, start_ms: (start * 1000.0) as u64, end_ms: (end * 1000.0) as u64 })
        }).collect()
    } else {
        // AssemblyAI: [{text, start, end, confidence}] в мілісекундах
        let arr = if let Some(arr) = json.as_array() {
            arr.as_slice()
        } else {
            json.get("words").and_then(|w| w.as_array())
                .ok_or("AssemblyAI subtitle.json: expected array or 'words' array")?
                .as_slice()
        };
        arr.iter().filter_map(|w| {
            let text = w.get("text")?.as_str()?.to_string();
            let start = w.get("start")?.as_u64()?;
            let end = w.get("end")?.as_u64()?;
            Some(Word { text, start_ms: start, end_ms: end })
        }).collect()
    };

    if words.is_empty() {
        return Err("No words in subtitle.json".to_string());
    }

    // Групуємо слова у рядки по ~5 секунд або по ~50 символів
    let mut lines: Vec<(u64, u64, Vec<&Word>)> = Vec::new();
    let mut group_start = 0usize;

    while group_start < words.len() {
        let mut group_end = group_start;
        let line_start_ms = words[group_start].start_ms;
        let mut char_count = 0usize;

        while group_end < words.len() {
            char_count += words[group_end].text.len() + 1;
            let dur_ms = words[group_end].end_ms.saturating_sub(line_start_ms);
            group_end += 1;
            // Закінчуємо групу якщо перевищено ліміт символів (~50) або тривалість (~5s)
            if char_count >= 50 || dur_ms >= 5000 {
                break;
            }
        }

        let group = &words[group_start..group_end];
        let line_end_ms = group.last().map(|w| w.end_ms).unwrap_or(line_start_ms + 1000);
        lines.push((line_start_ms, line_end_ms, group.iter().collect()));
        group_start = group_end;
    }

    // Генеруємо ASS Dialogue рядки з \kf тегами
    let mut events = String::new();
    for (start_ms, end_ms, group) in &lines {
        let start_str = ms_to_ass_time(*start_ms);
        let end_str = ms_to_ass_time(*end_ms);

        let mut text = String::new();
        for word in group {
            let dur_centisecs = (word.end_ms.saturating_sub(word.start_ms) / 10).max(1);
            text.push_str(&format!("{{\\kf{}}}{} ", dur_centisecs, word.text));
        }
        events.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_str, end_str, text.trim_end()
        ));
    }

    Ok(format!("{}{}", header, events))
}

/// Конвертує мілісекунди у формат часу ASS (H:MM:SS.CC).
fn ms_to_ass_time(ms: u64) -> String {
    let total_cs = ms / 10;
    let cs = total_cs % 100;
    let total_secs = total_cs / 100;
    let secs = total_secs % 60;
    let total_mins = total_secs / 60;
    let mins = total_mins % 60;
    let hours = total_mins / 60;
    format!("{}:{:02}:{:02}.{:02}", hours, mins, secs, cs)
}

/// Конвертує SRT-рядок у ASS-формат із запеченим стилем.
fn srt_to_ass(srt: &str, font_size: u32, color: [u8; 3], margin_v: u32) -> String {
    // Формат кольору ASS: &HAABBGGRR (A=00 = непрозорий)
    let primary = format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]);

    let header = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,Arial,{font_size},{primary},&H00FFFFFF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,{margin_v},1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        font_size = font_size,
        primary = primary,
        margin_v = margin_v,
    );

    let mut events = String::new();
    for block in srt.split("\n\n") {
        let block = block.trim();
        if block.is_empty() { continue; }

        let mut lines = block.lines();
        let first = lines.next().unwrap_or("").trim();

        // Якщо перший рядок — номер субтитра, пропускаємо його
        let timing_line = if first.parse::<u32>().is_ok() {
            lines.next().unwrap_or("").trim()
        } else {
            first
        };

        if let Some((start_str, end_str)) = timing_line.split_once(" --> ") {
            let start = srt_time_to_ass(start_str.trim());
            let end   = srt_time_to_ass(end_str.trim());
            let text: String = lines.collect::<Vec<_>>().join("\\N");
            if !text.is_empty() {
                events.push_str(&format!(
                    "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                    start, end, text
                ));
            }
        }
    }

    format!("{}{}", header, events)
}

/// Конвертує SRT час (HH:MM:SS,mmm) у ASS час (H:MM:SS.cc).
fn srt_time_to_ass(srt: &str) -> String {
    let parts: Vec<&str> = srt.splitn(2, ',').collect();
    if parts.len() != 2 { return "0:00:00.00".to_string(); }
    let hms: Vec<&str> = parts[0].split(':').collect();
    if hms.len() != 3 { return "0:00:00.00".to_string(); }
    let hours: u32 = hms[0].parse().unwrap_or(0);
    let mins: u32  = hms[1].parse().unwrap_or(0);
    let secs: u32  = hms[2].parse().unwrap_or(0);
    let cs: u32    = parts[1].trim().parse::<u32>().unwrap_or(0) / 10;
    format!("{}:{:02}:{:02}.{:02}", hours, mins, secs, cs)
}

/// Гілка Відеоряд (виконується паралельно з озвучкою та субтитрами).
/// Повертає Ok(()) або Err з описом першої помилки.
fn run_video_branch(
    job_id: u64,
    job_name: String,
    settings: crate::queue::JobSettings,
    translated_text: Arc<Mutex<Option<String>>>,
    video_stage: Arc<Mutex<crate::queue::StageStatus>>,
    prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    media_progress: Arc<Mutex<Option<(usize, usize)>>>,
    ctx: egui::Context,
) -> Result<(), String> {
    if !settings.video_enabled {
        return Ok(());
    }

    let media_label = if settings.video_media_type == "video" { "video" } else { "image" };
    crate::logger::log_job(job_id, &job_name, &format!("Starting video stage ({} generation)...", media_label));
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

    let total = segments.len();
    crate::logger::log_job(
        job_id, &job_name,
        &format!("Text split into {} segments (mode: {})", total, settings.text_split_mode),
    );

    // Зберігаємо debug-файл сегментів
    let save_dir = std::path::Path::new(&settings.save_path);
    let segments_path = save_dir.join("segments.txt");
    {
        let mut content = format!("=== Сегменти тексту: {} ===\n", total);
        for (i, seg) in segments.iter().enumerate() {
            content.push_str(&format!("\n[{}/{}]\n{}\n", i + 1, total, seg));
        }
        let _ = std::fs::write(&segments_path, &content);
        crate::logger::log_job(job_id, &job_name, "Segments saved: segments.txt");
    }

    // Фаза 1: будуємо промти для всіх сегментів
    let use_llm = settings.video_llm_service != "None" && !settings.video_llm_service.is_empty();
    if use_llm {
        crate::logger::log_job(
            job_id, &job_name,
            &format!("Generating prompts via {} for {} segments (parallel)...", settings.video_llm_service, total),
        );
    } else {
        crate::logger::log_job(job_id, &job_name, &format!("Building prompts for {} segments...", total));
    }
    *prompts_progress.lock().unwrap() = Some((0, total));
    ctx.request_repaint();

    // Резервуємо масив промтів з правильним порядком за індексом
    let mut prompts: Vec<String> = vec![String::new(); total];

    if use_llm {
        // Паралельна генерація: кожен сегмент в окремому потоці
        // Обмеження паралельності виконується всередині call_llm через глобальний лімітер
        let mut handles: Vec<std::thread::JoinHandle<(usize, String)>> = Vec::with_capacity(total);

        for (i, segment) in segments.iter().enumerate() {
            let segment          = segment.clone();
            let llm_service      = settings.video_llm_service.clone();
            let openrouter_key   = settings.openrouter_key.clone();
            let llm_model        = settings.video_llm_model.clone();
            let video_prompt     = settings.video_prompt.clone();
            let llm_temperature  = settings.video_llm_temperature;
            let prompts_progress_c = Arc::clone(&prompts_progress);
            let ctx_c            = ctx.clone();
            let job_id_c         = job_id;
            let job_name_c       = job_name.clone();

            handles.push(std::thread::spawn(move || {
                let prompt = match crate::core::llm::call_llm(
                    &llm_service,
                    &openrouter_key,
                    &llm_model,
                    &video_prompt,
                    &segment,
                    llm_temperature,
                    Some((job_id_c, job_name_c.clone())),
                ) {
                    Ok((generated, _)) => generated,
                    Err(e) => {
                        crate::logger::log_job(
                            job_id_c, &job_name_c,
                            &format!("LLM prompt {}/{} error: {}. Using fallback.", i + 1, total, e),
                        );
                        // Fallback: проста підстановка
                        if video_prompt.contains("{{text}}") {
                            video_prompt.replace("{{text}}", &segment)
                        } else if video_prompt.is_empty() {
                            segment.clone()
                        } else {
                            format!("{}\n\n{}", video_prompt, segment)
                        }
                    }
                };

                if let Ok(mut pp) = prompts_progress_c.lock() {
                    if let Some((ref mut done, _)) = *pp {
                        *done += 1;
                    }
                }
                ctx_c.request_repaint();

                (i, prompt)
            }));
        }

        // Збираємо результати, зберігаючи порядок за індексом
        for handle in handles {
            if let Ok((i, prompt)) = handle.join() {
                prompts[i] = prompt;
            } else {
                crate::logger::log_job(job_id, &job_name, "LLM prompt thread panicked.");
            }
        }
    } else {
        // Без ЛЛМ — миттєва підстановка
        for (i, segment) in segments.iter().enumerate() {
            prompts[i] = if settings.video_prompt.contains("{{text}}") {
                settings.video_prompt.replace("{{text}}", segment)
            } else if settings.video_prompt.is_empty() {
                segment.clone()
            } else {
                format!("{}\n\n{}", settings.video_prompt, segment)
            };
            if let Ok(mut pp) = prompts_progress.lock() {
                if let Some((ref mut done, _)) = *pp {
                    *done += 1;
                }
            }
            ctx.request_repaint();
        }
    }
    crate::logger::log_job(job_id, &job_name, "Prompts ready. Starting media generation...");

    // Фаза 2: паралельна генерація медіа із семафором
    let use_video = settings.video_media_type == "video";

    let media_dir = save_dir.join("media");
    if let Err(e) = std::fs::create_dir_all(&media_dir) {
        crate::logger::log_job(job_id, &job_name, &format!("Cannot create media/ dir: {}", e));
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(e.to_string());
    }

    // Зберігаємо промти у JSON для можливої перегенерації окремих файлів
    let prompts_path = media_dir.join("prompts.json");
    if let Ok(json) = serde_json::to_string_pretty(&prompts) {
        let _ = std::fs::write(&prompts_path, json);
    }

    *media_progress.lock().unwrap() = Some((0, total));
    ctx.request_repaint();

    let sem = Arc::new(Semaphore::new(settings.googler_image_max_threads.max(1)));
    let mut handles = Vec::new();

    for (i, prompt) in prompts.into_iter().enumerate() {
        let sem              = Arc::clone(&sem);
        let media_progress_c = Arc::clone(&media_progress);
        let ctx_c            = ctx.clone();
        let key              = settings.googler_key.clone();
        let priority         = if use_video {
            settings.googler_video_priority.clone()
        } else {
            settings.googler_image_priority.clone()
        };
        let media_dir  = media_dir.clone();
        let job_id_c   = job_id;
        let job_name_c = job_name.clone();

        let handle = std::thread::spawn(move || -> (usize, Result<(), String>) {
            sem.acquire();

            crate::logger::log_job(
                job_id_c, &job_name_c,
                &format!("Generating {} {}/{} ...", if use_video { "video" } else { "image" }, i + 1, total),
            );

            if prompt.trim().is_empty() {
                crate::logger::log_job(
                    job_id_c, &job_name_c,
                    &format!("Segment {}/{}: empty prompt, skipping.", i + 1, total),
                );
                return (i, Err("Порожній промпт — пропущено".to_string()));
            }

            let result = if use_video {
                crate::api::googler::generate_video_with_priority(&key, &prompt, "16:9", &priority)
            } else {
                crate::api::googler::generate_image_with_priority(&key, &prompt, "16:9", &priority)
            };

            sem.release();

            match result {
                Err(e) => (i, Err(e)),
                Ok(data_uri) => {
                    match decode_result(&data_uri, i + 1, total, &media_dir) {
                        Err(e) => (i, Err(e)),
                        Ok(path) => {
                            crate::logger::log_job(
                                job_id_c, &job_name_c,
                                &format!("{} {}/{} saved: {}", if use_video { "Video" } else { "Image" }, i + 1, total, path.display()),
                            );
                            if let Ok(mut mp) = media_progress_c.lock() {
                                if let Some((ref mut done, _)) = *mp {
                                    *done += 1;
                                }
                            }
                            ctx_c.request_repaint();
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
                crate::logger::log_job(job_id, &job_name, &format!("{} error — {}", media_label, msg));
                errors.push(msg);
            }
            Err(_) => errors.push(format!("Thread panic during {} generation", media_label)),
            _ => {}
        }
    }

    if errors.is_empty() {
        crate::logger::log_job(job_id, &job_name, &format!("All {}s generated successfully.", media_label));
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Done;
        ctx.request_repaint();
        Ok(())
    } else {
        let msg = format!("{} generation failed ({} errors). First: {}", media_label, errors.len(), errors[0]);
        crate::logger::log_job(job_id, &job_name, &msg);
        *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        Err(msg)
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
    translation_cost: Arc<Mutex<Option<f64>>>,
    audio_duration: Arc<Mutex<Option<f64>>>,
    prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    media_progress: Arc<Mutex<Option<(usize, usize)>>>,
    montage_progress: Arc<Mutex<Option<f32>>>,
    montage_file_size: Arc<Mutex<Option<u64>>>,
    media_control_resume: Arc<(Mutex<bool>, Condvar)>,
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

        // Паралельні гілки: [Озвучка + Субтитри] || [Відеоряд]
        // Озвучка та субтитри залежать одна від одної (субтитри потребують аудіо),
        // тому вони послідовні всередині гілки AV, але гілка AV паралельна з відеорядом.

        // AV гілка виконується якщо є озвучка (Whisper залежить від аудіо, тому теж тут)
        let run_av = settings.voiceover_enabled;
        let run_video = settings.video_enabled;

        if run_av {
            crate::logger::log_job(job_id, &job_name, "Starting AV branch (voiceover + subtitles) in parallel with video...");
        }
        if run_video {
            crate::logger::log_job(job_id, &job_name, "Starting video branch in parallel with AV...");
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
                    ctx_video,
                )
            }))
        } else {
            None
        };

        // Спочатку чекаємо відеогілку, щоб мати можливість зробити паузу для контролю зображень
        // поки гілка AV (озвучка + субтитри) продовжує виконуватись паралельно.
        let video_result = video_handle.map(|h| h.join().unwrap_or_else(|_| Err("Video thread panicked".to_string())));

        // Пауза для контролю зображень — AV гілка продовжує виконуватись
        if settings.media_control_enabled && settings.video_enabled {
            if let Some(Ok(())) = &video_result {
                crate::logger::log_job(job_id, &job_name, "Video done. Awaiting media review by user...");
                *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMediaControl;
                ctx.request_repaint();

                let (lock, cvar) = &*media_control_resume;
                let mut resumed = lock.lock().unwrap();
                while !*resumed {
                    resumed = cvar.wait(resumed).unwrap();
                }
                *resumed = false;

                crate::logger::log_job(job_id, &job_name, "Media review confirmed. Resuming pipeline...");
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();
            }
        }

        // Тепер чекаємо AV гілку
        let av_result = av_handle.map(|h| h.join().unwrap_or_else(|_| Err("AV thread panicked".to_string())));

        // Перевіряємо помилки обох гілок
        if let Some(Err(e)) = av_result {
            *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
            ctx.request_repaint();
            return;
        }
        if let Some(Err(e)) = video_result {
            *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
            ctx.request_repaint();
            return;
        }

        // Генеруємо timeline.json якщо відеоряд увімкнено і є тривалість аудіо
        // (підготовка для етапу монтажу)
        if settings.video_enabled {
            let source_text = if settings.translation_enabled {
                translated_text.lock().unwrap().clone().unwrap_or_else(|| settings.text.clone())
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

            match crate::core::pipeline::timeline::sync::build_timeline(save_dir, &segments, audio_dur, &job_name) {
                Ok(_) => crate::logger::log_job(job_id, &job_name, "Timeline saved: timeline.json"),
                Err(e) => crate::logger::log_job(job_id, &job_name, &format!("Timeline warning: {}", e)),
            }
        }

        // Етап 5: Монтаж
        if settings.montage_enabled {
            crate::logger::log_job(job_id, &job_name, "Starting montage stage...");
            *montage_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            let audio_dur = *audio_duration.lock().unwrap();
            let save_dir = std::path::Path::new(&settings.save_path);
            let job_id_log = job_id;
            let job_name_log = job_name.clone();

            let montage_progress_arc = Arc::clone(&montage_progress);
            let ctx_montage = ctx.clone();

            match crate::core::pipeline::montage::run_montage(
                save_dir,
                &job_name,
                audio_dur,
                settings.montage_fps,
                &settings.montage_preset,
                settings.montage_bitrate,
                &settings.montage_transition,
                settings.montage_transition_duration,
                settings.subtitles_enabled,
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
                    crate::logger::log_job(job_id, &job_name, &format!("Montage failed: {}", e));
                    *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(format!("Montage: {}", e));
                    ctx.request_repaint();
                    return;
                }
            }
            ctx.request_repaint();
        }

        crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
        *status.lock().unwrap() = crate::queue::JobStatus::Done;
        ctx.request_repaint();
    });
}

/// Зчитує збережений промт для конкретного медіафайлу з prompts.json.
/// Індекс визначається з імені файлу (0001.jpg → індекс 0).
/// Анімує одне зображення у відео у фоновому потоці (image-to-video).
/// Зчитує файл як base64 data URI, відправляє на Googler, зберігає .mp4,
/// видаляє оригінальне зображення. Прибирає шлях з loading_set після завершення.
pub fn animate_single_image(
    file_path: std::path::PathBuf,
    priority: Vec<String>,
    googler_key: String,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    loading_set: Arc<Mutex<std::collections::HashSet<std::path::PathBuf>>>,
) {
    std::thread::spawn(move || {
        loading_set.lock().unwrap().insert(file_path.clone());
        ctx.request_repaint();

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let result = (|| -> Result<std::path::PathBuf, String> {
            let bytes = std::fs::read(&file_path)
                .map_err(|e| format!("Помилка читання файлу: {}", e))?;

            let ext = file_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_lowercase();
            let mime = match ext.as_str() {
                "png"  => "image/png",
                "webp" => "image/webp",
                _      => "image/jpeg",
            };
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let data_uri = format!("data:{};base64,{}", mime, b64);

            let prompt = "Animate this image with smooth, natural motion.";
            crate::logger::log_job(job_id, &job_name, &format!("Animate {}: запуск image-to-video", file_name));

            let api_result = crate::api::googler::animate_image_with_priority(
                &googler_key, &data_uri, prompt, &priority,
            )?;

            // Зберігаємо відео поряд з оригінальним зображенням (.mp4)
            let video_path = file_path.with_extension("mp4");
            save_media_bytes(&api_result, &video_path)?;

            // Видаляємо оригінальне зображення
            if video_path != file_path {
                let _ = std::fs::remove_file(&file_path);
            }

            Ok(video_path)
        })();

        match &result {
            Ok(out) => crate::logger::log_job(
                job_id, &job_name,
                &format!("Animate {} → {} готово", file_name, out.file_name().unwrap_or_default().to_string_lossy()),
            ),
            Err(e) => crate::logger::log_job(
                job_id, &job_name,
                &format!("Animate {} помилка: {}", file_name, e),
            ),
        }

        loading_set.lock().unwrap().remove(&file_path);
        ctx.request_repaint();
    });
}

pub(crate) fn read_prompt_for_file(file_path: &std::path::Path) -> String {
    let media_dir = match file_path.parent() {
        Some(d) => d,
        None => return String::new(),
    };
    let index = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<usize>().ok())
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0);

    std::fs::read_to_string(media_dir.join("prompts.json"))
        .ok()
        .and_then(|c| serde_json::from_str::<Vec<String>>(&c).ok())
        .and_then(|v| v.into_iter().nth(index))
        .unwrap_or_default()
}

/// Зберігає байти медіа (data URI або HTTP URL) у вказаний файл, перезаписуючи його.
fn save_media_bytes(data_uri: &str, file_path: &std::path::Path) -> Result<(), String> {
    let bytes = if data_uri.starts_with("data:") {
        let rest = &data_uri[5..];
        let comma = rest.find(',').ok_or("Invalid data URI: no comma")?;
        let b64 = &rest[comma + 1..];
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?
    } else {
        use std::io::Read;
        let resp = ureq::get(data_uri)
            .call()
            .map_err(|e| format!("Download error: {}", e))?;
        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        buf
    };
    std::fs::write(file_path, &bytes).map_err(|e| format!("Save error: {}", e))
}

/// Перегенерує один медіафайл у фоновому потоці.
/// Якщо custom_prompt = None або порожній — читає збережений промт з prompts.json.
pub fn regenerate_single_media(
    file_path: std::path::PathBuf,
    media_type: String,
    priority: Vec<String>,
    googler_key: String,
    custom_prompt: Option<String>,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    result_slot: Arc<Mutex<Option<Result<(), String>>>>,
    loading: Arc<Mutex<bool>>,
) {
    std::thread::spawn(move || {
        *loading.lock().unwrap() = true;

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let prompt = custom_prompt
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| read_prompt_for_file(&file_path));

        crate::logger::log_job(
            job_id, &job_name,
            &format!("Regen {}: {} (prompt: {}...)", media_type, file_name, prompt.chars().take(60).collect::<String>()),
        );

        let api_result = if media_type == "video" {
            crate::api::googler::generate_video_with_priority(&googler_key, &prompt, "16:9", &priority)
        } else {
            crate::api::googler::generate_image_with_priority(&googler_key, &prompt, "16:9", &priority)
        };

        let outcome = match api_result {
            Err(e) => {
                crate::logger::log_job(job_id, &job_name, &format!("Regen {} failed: {}", file_name, e));
                Err(e)
            }
            Ok(data_uri) => match save_media_bytes(&data_uri, &file_path) {
                Ok(()) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Regen {} done.", file_name));
                    Ok(())
                }
                Err(e) => {
                    crate::logger::log_job(job_id, &job_name, &format!("Regen {} save error: {}", file_name, e));
                    Err(e)
                }
            },
        };

        *result_slot.lock().unwrap() = Some(outcome);
        *loading.lock().unwrap() = false;
        ctx.request_repaint();
    });
}
