pub mod voiceover;
pub mod timeline;
pub mod montage;
pub mod capcut;

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

/// Валідує завантажені або декодовані медіа-байти на порожнечу та текстові помилки.
fn validate_media_bytes(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("Отримано порожній файл (0 байт)".to_string());
    }
    if bytes.len() < 500 {
        if let Ok(text) = std::str::from_utf8(bytes) {
            let trimmed = text.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('<') || trimmed.starts_with("Error") || trimmed.starts_with("Unauthorized") {
                return Err(format!("Замість медіа-даних отримано текст помилки: {}", trimmed));
            }
        }
    }
    Ok(())
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

        validate_media_bytes(&bytes)?;

        let path = media_dir.join(format!("{:04}.{}", index, ext));
        std::fs::write(&path, &bytes).map_err(|e| format!("Save error: {}", e))?;
        Ok(path)
    } else {
        // Звичайний HTTP URL — розширення беремо з URL
        let ext = result.split('?').next().unwrap_or(result)
            .rsplit('.').next()
            .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("jpg");

        use std::io::Read;
        let resp = ureq::get(result).call()
            .map_err(|e| format!("Download error: {}", e))?;
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)
            .map_err(|e| format!("Read error: {}", e))?;

        validate_media_bytes(&bytes)?;

        let path = media_dir.join(format!("{:04}.{}", index, ext));
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

    // Знаходимо виконуваний файл whisperx (платформозалежно: whisperx_win/whisperx_cli.exe або whisperx_mac/whisperx_cli)
    let whisperx_cmd = crate::bundle::whisperx_cmd_path();

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
                            let ass = srt_to_ass(&srt, &settings.subtitle_font, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
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
            let ass = srt_to_ass(&srt, &settings.subtitle_font, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
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

/// Транскрибує аудіо через Whisper AMD (AMD GPU-оптимізований whisper.cpp).
/// Використовує ті самі ggml-моделі що й звичайний Whisper.
fn run_whisper_amd(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    subtitles_stage: &std::sync::Arc<std::sync::Mutex<crate::queue::StageStatus>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let reason = if settings.subtitles_enabled {
        "Starting subtitle generation via Whisper AMD (burn-in enabled)..."
    } else {
        "Starting subtitle generation via Whisper AMD (for timeline sync, burn-in disabled)..."
    };
    crate::logger::log_job(job_id, job_name, reason);
    *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    if !crate::bundle::whisper_amd_local_exists() {
        let msg = "Whisper AMD not found. Install it via the Welcome window.".to_string();
        crate::logger::log_job(job_id, job_name, &format!("Subtitles error: {}", msg));
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    }

    let save_dir = std::path::Path::new(&settings.save_path);
    let model_path = crate::bundle::whisper_model_path(&settings.whisper_model);
    if !model_path.exists() {
        let msg = format!(
            "Subtitles error: model '{}' not found. Download it in the subtitles settings.",
            settings.whisper_model
        );
        crate::logger::log_job(job_id, job_name, &msg);
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
        return Err(msg);
    }

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

    // Whisper AMD зберігає SRT поруч з аудіо: voice.srt, потім перейменовуємо у subtitle.srt
    let audio_stem = audio_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("voice");
    let generated_srt = save_dir.join(format!("{}.srt", audio_stem));

    let whisper_amd_cmd = crate::bundle::whisper_amd_cmd_path();
    let mut args: Vec<String> = vec![
        "-f".to_string(), audio_path.to_str().unwrap_or("voice.wav").to_string(),
        "-m".to_string(), model_path.to_str().unwrap().to_string(),
        "-osrt".to_string(),
    ];
    if settings.whisper_language != "auto" {
        args.push("-l".to_string());
        args.push(settings.whisper_language.clone());
    }
    if settings.whisper_max_line_width > 0 {
        args.push("-ml".to_string());
        args.push(settings.whisper_max_line_width.to_string());
    }

    crate::logger::log_job(job_id, job_name, &format!("Running: {} {}", whisper_amd_cmd.display(), args.join(" ")));

    match std::process::Command::new(&whisper_amd_cmd).args(&args).output() {
        Ok(out) if out.status.success() => {
            let srt_path = save_dir.join("subtitle.srt");

            // Перейменовуємо voice.srt → subtitle.srt
            if generated_srt.exists() && generated_srt != srt_path {
                if let Err(e) = std::fs::rename(&generated_srt, &srt_path) {
                    crate::logger::log_job(job_id, job_name, &format!("Whisper AMD: failed to rename SRT: {}", e));
                }
            }

            crate::logger::log_job(job_id, job_name, "Subtitles saved: subtitle.srt (Whisper AMD)");

            if let Ok(srt) = std::fs::read_to_string(&srt_path) {
                let ass = srt_to_ass(&srt, &settings.subtitle_font, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
                let _ = std::fs::write(save_dir.join("subtitle.ass"), &ass);
            }
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
            ctx.request_repaint();
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let msg = if !stderr.is_empty() { stderr.to_string() } else { stdout.to_string() };
            let short = format!("Whisper AMD error: {}", msg.chars().take(120).collect::<String>());
            crate::logger::log_job(job_id, job_name, &format!("Whisper AMD error: {}", msg.trim()));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(short)
        }
        Err(e) => {
            let msg = format!("Whisper AMD launch error: {}", e);
            crate::logger::log_job(job_id, job_name, &format!("Whisper AMD not found or failed to start: {}", e));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(msg)
        }
    }
}

/// Виконує лише генерацію субтитрів (без озвучки).
/// Використовується як для основного пайплайну, так і для повтору субтитрів.
fn run_subtitles_only(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    subtitles_stage: &Arc<Mutex<crate::queue::StageStatus>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    if settings.subtitles_service == "WhisperX" {
        run_whisperx(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if settings.subtitles_service == "AssemblyAI" {
        run_assemblyai(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if settings.subtitles_service == "WhisperAMD" {
        run_whisper_amd(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if settings.subtitles_service == "Whisper" {
        let reason = if settings.subtitles_enabled {
            "Starting subtitle generation via Whisper (burn-in enabled)..."
        } else {
            "Starting subtitle generation via Whisper (for timeline sync, burn-in disabled)..."
        };
        crate::logger::log_job(job_id, job_name, reason);
        *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Running;
        ctx.request_repaint();

        let save_dir = std::path::Path::new(&settings.save_path);
        let model_path = crate::bundle::whisper_model_path(&settings.whisper_model);
        if !model_path.exists() {
            let msg = format!(
                "Subtitles error: model '{}' not found at '{}'. Download it in the subtitles settings.",
                settings.whisper_model, model_path.display()
            );
            crate::logger::log_job(job_id, job_name, &msg);
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            return Err(msg);
        }

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

        let output_stem = save_dir.join("subtitle");
        let whisper_cmd = crate::bundle::whisper_path();
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
            args.push("--split-on-word".to_string());
        }

        crate::logger::log_job(job_id, job_name, &format!("Running: {} {}", whisper_cmd, args.join(" ")));

        match std::process::Command::new(&whisper_cmd).args(&args).output() {
            Ok(out) if out.status.success() => {
                crate::logger::log_job(job_id, job_name, "Subtitles saved: subtitle.srt");
                let srt_path = save_dir.join("subtitle.srt");
                if let Ok(srt) = std::fs::read_to_string(&srt_path) {
                    let ass = srt_to_ass(&srt, &settings.subtitle_font, settings.subtitle_font_size, settings.subtitle_color, settings.subtitle_margin_v);
                    let _ = std::fs::write(save_dir.join("subtitle.ass"), &ass);
                }
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                ctx.request_repaint();
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let msg = if !stderr.is_empty() { stderr.to_string() } else { stdout.to_string() };
                let short = format!("Whisper error: {}", msg.chars().take(120).collect::<String>());
                crate::logger::log_job(job_id, job_name, &format!("Whisper error: {}", msg.trim()));
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(short);
            }
            Err(e) => {
                let msg = format!("Whisper launch error: {}", e);
                crate::logger::log_job(job_id, job_name, &format!("Whisper not found or failed to start: {}", e));
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(msg);
            }
        }
    }

    // Karaoke-ефект якщо увімкнено
    if settings.subtitle_karaoke
        && (settings.subtitles_service == "WhisperX" || settings.subtitles_service == "AssemblyAI")
    {
        let save_dir = std::path::Path::new(&settings.save_path);
        let json_path = save_dir.join("subtitle.json");
        if json_path.exists() {
            match generate_karaoke_ass(
                &json_path,
                &settings.subtitles_service,
                &settings.subtitle_font,
                settings.subtitle_font_size,
                settings.subtitle_color,
                settings.subtitle_margin_v,
                settings.subtitle_karaoke_mode,
                settings.subtitle_karaoke_highlight_color,
                settings.subtitle_karaoke_outline_color,
                settings.subtitle_karaoke_bold,
                settings.subtitle_karaoke_scale,
            ) {
                Ok(ass_content) => {
                    match std::fs::write(save_dir.join("subtitle.ass"), &ass_content) {
                        Ok(_) => crate::logger::log_job(job_id, job_name, "Karaoke ASS generated: subtitle.ass"),
                        Err(e) => crate::logger::log_job(job_id, job_name, &format!("Failed to save subtitle.ass (karaoke): {}", e)),
                    }
                }
                Err(e) => crate::logger::log_job(job_id, job_name, &format!("Karaoke generation error: {}", e)),
            }
        } else {
            crate::logger::log_job(job_id, job_name, "Karaoke: subtitle.json not found, skipping.");
        }
    }

    Ok(())
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

    // Субтитри — виконуємо через спільний хелпер
    run_subtitles_only(&settings, job_id, &job_name, &subtitles_stage, &ctx)?;

    Ok(())
}

/// Генерує ASS файл з karaoke-ефектом з word-level timestamps.
/// Підтримує формати WhisperX (секунди) та AssemblyAI (мілісекунди).
/// karaoke_mode: 0 = fill (\kf), 1 = switch (\k), 2 = follow (підсвітка + повернення).
fn generate_karaoke_ass(
    json_path: &std::path::Path,
    service: &str,
    font_name: &str,
    font_size: u32,
    color: [u8; 3],
    margin_v: u32,
    karaoke_mode: u8,
    highlight_color: [u8; 3],
    outline_color: [u8; 3],
    bold: bool,
    scale: u32,
) -> Result<String, String> {
    let content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("Cannot read subtitle.json: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse subtitle.json: {}", e))?;

    // Формат кольору ASS у заголовку: &H00BBGGRR
    // Для режимів fill/switch:
    //   PrimaryColour   = highlight_color (колір ПІСЛЯ активації — слово вже проговорене)
    //   SecondaryColour = color           (колір ДО активації — слово ще не проговорене)
    // Для режиму follow — обидва = color (inline \1c теги перекривають)
    let (header_primary, header_secondary) = if karaoke_mode == 2 {
        let c = format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]);
        (c.clone(), c)
    } else {
        (
            format!("&H00{:02X}{:02X}{:02X}", highlight_color[2], highlight_color[1], highlight_color[0]),
            format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]),
        )
    };
    let outline_hex = format!("&H00{:02X}{:02X}{:02X}", outline_color[2], outline_color[1], outline_color[0]);
    let bold_flag = if bold { 1 } else { 0 };

    let header = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,{font_name},{font_size},{primary},{secondary},{outline},&H80000000,{bold},0,0,0,100,100,0,0,1,2,1,2,10,10,{margin_v},1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        font_name = font_name,
        font_size = font_size,
        primary = header_primary,
        secondary = header_secondary,
        outline = outline_hex,
        bold = bold_flag,
        margin_v = margin_v,
    );

    struct Word {
        text: String,
        start_ms: u64,
        end_ms: u64,
    }

    let words: Vec<Word> = if service == "WhisperX" {
        let arr = json.get("words").and_then(|w| w.as_array())
            .ok_or("WhisperX subtitle.json: no 'words' array")?;
        arr.iter().filter_map(|w| {
            let text = w.get("word")?.as_str()?.to_string();
            let start = w.get("start")?.as_f64()?;
            let end = w.get("end")?.as_f64()?;
            Some(Word { text, start_ms: (start * 1000.0) as u64, end_ms: (end * 1000.0) as u64 })
        }).collect()
    } else {
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
    let mut lines: Vec<(u64, u64, Vec<usize>)> = Vec::new();
    let mut group_start = 0usize;
    while group_start < words.len() {
        let mut group_end = group_start;
        let line_start_ms = words[group_start].start_ms;
        let mut char_count = 0usize;
        while group_end < words.len() {
            char_count += words[group_end].text.len() + 1;
            let dur_ms = words[group_end].end_ms.saturating_sub(line_start_ms);
            group_end += 1;
            if char_count >= 50 || dur_ms >= 5000 {
                break;
            }
        }
        let line_end_ms = words[group_end - 1].end_ms;
        lines.push((line_start_ms, line_end_ms, (group_start..group_end).collect()));
        group_start = group_end;
    }

    let mut events = String::new();

    // Closure для побудови рядка follow-режиму: current = usize::MAX → всі слова нормальним кольором.
    let build_follow_text = |indices: &[usize], current: usize, hi_hex: &str, normal_hex: &str,
                              scale_tag: &str, reset_scale: &str| -> String {
        let mut s = String::new();
        for (j, &idx) in indices.iter().enumerate() {
            if j == current {
                s.push_str(&format!("{{\\1c&H{}&{}}}{} ", hi_hex, scale_tag, words[idx].text));
            } else {
                s.push_str(&format!("{{\\1c&H{}&{}}}{} ", normal_hex, reset_scale, words[idx].text));
            }
        }
        s.trim_end().to_string()
    };

    if karaoke_mode == 2 {
        // Follow-режим: окрема Dialogue-подія для кожного слова.
        // Кожна подія показує весь рядок, але лише поточне слово підсвічене.
        // Після проговорення слово повертається до нормального кольору.
        let normal_hex = format!("{:02X}{:02X}{:02X}", color[2], color[1], color[0]);
        let hi_hex = format!("{:02X}{:02X}{:02X}", highlight_color[2], highlight_color[1], highlight_color[0]);
        let scale_tag = if scale != 100 { format!("\\fscx{}\\fscy{}", scale, scale) } else { String::new() };
        let reset_scale = if scale != 100 { "\\fscx100\\fscy100" } else { "" };

        for (group_start_ms, group_end_ms, indices) in &lines {
            let n = indices.len();

            // Пауза до першого слова — всі нормальним кольором
            let first_word_start = words[indices[0]].start_ms;
            if first_word_start > *group_start_ms {
                let text = build_follow_text(indices, usize::MAX, &hi_hex, &normal_hex, &scale_tag, reset_scale);
                events.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                    ms_to_ass_time(*group_start_ms), ms_to_ass_time(first_word_start), text));
            }

            for (i, &word_idx) in indices.iter().enumerate() {
                let event_start = words[word_idx].start_ms;
                let event_end = if i + 1 < n { words[indices[i + 1]].start_ms } else { *group_end_ms };
                let text = build_follow_text(indices, i, &hi_hex, &normal_hex, &scale_tag, reset_scale);
                events.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                    ms_to_ass_time(event_start), ms_to_ass_time(event_end), text));
            }
        }
    } else {
        // Режими fill (\kf) та switch (\k): одна Dialogue-подія на групу.
        // cursor_ms компенсує паузи між словами порожнім {\k{gap}}.
        let ktag = if karaoke_mode == 0 { "kf" } else { "k" };

        for (start_ms, end_ms, indices) in &lines {
            let start_str = ms_to_ass_time(*start_ms);
            let end_str = ms_to_ass_time(*end_ms);
            let mut text = String::new();
            let mut cursor_ms = *start_ms;

            for &word_idx in indices {
                let word = &words[word_idx];
                if word.start_ms > cursor_ms {
                    let gap_cs = (word.start_ms - cursor_ms) / 10;
                    text.push_str(&format!("{{\\k{}}}", gap_cs));
                }
                let dur_cs = (word.end_ms.saturating_sub(word.start_ms) / 10).max(1);
                text.push_str(&format!("{{\\{}{}}}{} ", ktag, dur_cs, word.text));
                cursor_ms = word.end_ms;
            }

            events.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                start_str, end_str, text.trim_end()));
        }
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
fn srt_to_ass(srt: &str, font_name: &str, font_size: u32, color: [u8; 3], margin_v: u32) -> String {
    // Формат кольору ASS: &HAABBGGRR (A=00 = непрозорий)
    let primary = format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]);

    let header = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Default,{font_name},{font_size},{primary},&H00FFFFFF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,{margin_v},1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        font_name = font_name,
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
    total_cost: Arc<Mutex<Option<f64>>>,
    ctx: egui::Context,
) -> Result<(), String> {
    if !settings.video_enabled {
        return Ok(());
    }

    let media_label = if settings.video_media_type == "video" { "video" } else { "image" };
    crate::logger::log_job(job_id, &job_name, &format!("Starting video stage ({} generation)...", media_label));
    *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
    ctx.request_repaint();

    // В агентному режимі сегменти беруться з timeline.json, LLM для промтів не викликається
    let is_agent_mode = settings.video_llm_service == "Claude Code"
        || settings.video_llm_service == "Gemini CLI";

    // Визначаємо текст: перекладений якщо є, інакше оригінал
    let source_text = if settings.translation_enabled {
        translated_text.lock().unwrap().clone().unwrap_or_else(|| settings.text.clone())
    } else {
        settings.text.clone()
    };

    // Нарізаємо текст на сегменти: при агентному режимі — з timeline.json
    let save_dir = std::path::Path::new(&settings.save_path);
    let segments = if is_agent_mode {
        match read_segments_from_timeline(save_dir) {
            Ok(segs) if !segs.is_empty() => {
                crate::logger::log_job(job_id, &job_name,
                    &format!("Agent mode: {} segments from timeline.json", segs.len()));
                segs
            }
            _ => {
                crate::logger::log_job(job_id, &job_name,
                    "Agent mode: timeline.json not ready, using text split.");
                crate::core::pipeline::timeline::text_splitter::split_text(
                    &source_text, &settings.text_split_mode, settings.text_split_char_limit,
                )
            }
        }
    } else {
        crate::core::pipeline::timeline::text_splitter::split_text(
            &source_text,
            &settings.text_split_mode,
            settings.text_split_char_limit,
        )
    };

    let total = segments.len();
    crate::logger::log_job(
        job_id, &job_name,
        &format!("Text split into {} segments (mode: {})", total, settings.text_split_mode),
    );

    // Зберігаємо debug-файл сегментів
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
    // В агентному режимі LLM не викликається — video_prompt підставляється напряму
    let use_llm = !is_agent_mode && settings.video_llm_service != "None" && !settings.video_llm_service.is_empty();
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
        let mut handles: Vec<std::thread::JoinHandle<(usize, String, Option<f64>)>> = Vec::with_capacity(total);

        for (i, segment) in segments.iter().enumerate() {
            let segment          = segment.clone();
            let llm_service      = settings.video_llm_service.clone();
            let openrouter_key   = settings.openrouter_key.clone();
            let llm_model        = settings.video_llm_model.clone();
            let video_prompt     = settings.video_prompt.clone();
            let llm_temperature  = settings.video_llm_temperature;
            let save_path        = settings.save_path.clone();
            let prompts_progress_c = Arc::clone(&prompts_progress);
            let ctx_c            = ctx.clone();
            let job_id_c         = job_id;
            let job_name_c       = job_name.clone();

            handles.push(std::thread::spawn(move || {
                let (prompt, cost) = match crate::core::llm::call_llm(
                    &llm_service,
                    &openrouter_key,
                    &llm_model,
                    &video_prompt,
                    &segment,
                    llm_temperature,
                    Some((job_id_c, job_name_c.clone())),
                    Some(save_path.as_str()),
                    false,
                ) {
                    Ok((generated, cost)) => (generated, cost),
                    Err(e) => {
                        crate::logger::log_job(
                            job_id_c, &job_name_c,
                            &format!("LLM prompt {}/{} error: {}. Using fallback.", i + 1, total, e),
                        );
                        // Fallback: проста підстановка
                        let fallback = if video_prompt.contains("{{text}}") {
                            video_prompt.replace("{{text}}", &segment)
                        } else if video_prompt.is_empty() {
                            segment.clone()
                        } else {
                            format!("{}\n\n{}", video_prompt, segment)
                        };
                        (fallback, None)
                    }
                };

                if let Ok(mut pp) = prompts_progress_c.lock() {
                    if let Some((ref mut done, _)) = *pp {
                        *done += 1;
                    }
                }
                ctx_c.request_repaint();

                (i, prompt, cost)
            }));
        }

        // Збираємо результати, зберігаючи порядок за індексом; накопичуємо вартість LLM-запитів
        for handle in handles {
            if let Ok((i, prompt, cost)) = handle.join() {
                prompts[i] = prompt;
                if let Some(c) = cost {
                    let mut tc = total_cost.lock().unwrap();
                    *tc = Some(tc.unwrap_or(0.0) + c);
                }
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

    let media_dir = std::path::Path::new(&settings.save_path).join("media");
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

        let upscale_enabled = settings.googler_video_upscale_enabled;
        let upscale_resolution = settings.googler_video_upscale_resolution.clone();
        let upscale_quality = settings.googler_video_upscale_quality.clone();

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
                            if use_video {
                                if let Err(err) = upscale_video_if_needed(
                                    &path,
                                    upscale_enabled,
                                    &upscale_resolution,
                                    &upscale_quality,
                                    job_id_c,
                                    &job_name_c,
                                ) {
                                    crate::logger::log_job(
                                        job_id_c, &job_name_c,
                                        &format!("Помилка апскейлу для сегмента {}: {}", i + 1, err),
                                    );
                                }
                            }

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

/// Читає сегменти тексту з timeline.json (для агентного режиму).
fn read_segments_from_timeline(save_dir: &std::path::Path) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(save_dir.join("timeline.json"))
        .map_err(|e| format!("Cannot read timeline.json: {}", e))?;
    let timeline = serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
        .map_err(|e| format!("Invalid timeline.json: {}", e))?;
    Ok(timeline.segments.into_iter().map(|s| s.text).collect())
}

/// Запускає агента (Claude Code або Gemini CLI) для створення timeline.json на основі subtitle.srt.
/// Якщо `agent_control_enabled`, зберігає session_id та чекає підтвердження користувача.
fn run_agent_timeline(
    job_id: u64,
    job_name: &str,
    settings: &crate::queue::JobSettings,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let save_dir = std::path::Path::new(&settings.save_path);
    let srt_path = save_dir.join("subtitle.srt");

    let srt_content = std::fs::read_to_string(&srt_path)
        .map_err(|e| format!("subtitle.srt not found (run subtitles stage first): {}", e))?;

    let timeline_path = save_dir.join("timeline.json");
    let agent_prompt = settings.video_agent_prompt
        .replace("{{srt}}", &srt_content)
        .replace("{{path}}", &timeline_path.to_string_lossy());

    crate::logger::log_job(job_id, job_name,
        &format!("Agent ({}): generating timeline.json...", settings.video_llm_service));

    let session_id = uuid_v4();
    crate::logger::log_job(job_id, job_name, &format!("Agent session: {}", session_id));

    // Початкове повідомлення з аргументами запуску — chunks додаватимуться після нього
    let initial_text = format!(
        "Running: {} --model {} --session-id {}\n\n",
        settings.video_llm_service,
        settings.video_llm_model,
        session_id,
    );
    agent_chat.lock().unwrap().push(crate::queue::AgentChatMessage {
        role: "agent".to_string(),
        content: initial_text,
    });
    ctx.request_repaint();

    let agent_chat_for_chunk = Arc::clone(&agent_chat);
    let ctx_for_chunk = ctx.clone();

    let response = call_agent_new_session_streaming(
        &settings.video_llm_service,
        &settings.video_llm_model,
        &agent_prompt,
        &session_id,
        Some((job_id, job_name.to_string())),
        Some(&settings.save_path),
        move |chunk| {
            let mut chat = agent_chat_for_chunk.lock().unwrap();
            if let Some(last) = chat.last_mut() {
                last.content.push_str(chunk);
            }
            ctx_for_chunk.request_repaint();
        },
    ).map_err(|e| format!("Agent error: {}", e))?;

    save_agent_chat_to_file(save_dir, &agent_chat.lock().unwrap());

    // Сесію зберігаємо тільки якщо увімкнено контроль — для можливості продовження чату
    if settings.agent_control_enabled {
        *agent_session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
            session_id: session_id.clone(),
            service: settings.video_llm_service.clone(),
            model: settings.video_llm_model.clone(),
        });
    }

    let _ = response;

    if !timeline_path.exists() {
        return Err("Agent did not create timeline.json".to_string());
    }
    let content = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Cannot read agent timeline.json: {}", e))?;
    serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
        .map_err(|e| format!("Agent timeline.json is invalid: {}", e))?;

    crate::logger::log_job(job_id, job_name, "Agent timeline.json created and validated.");

    // Пауза для контролю агента — чекаємо підтвердження користувача
    if settings.agent_control_enabled {
        crate::logger::log_job(job_id, job_name, "Awaiting agent control confirmation from user...");
        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingAgentControl;
        ctx.request_repaint();

        let (lock, cvar) = &*agent_control_resume;
        let mut resumed = lock.lock().unwrap();
        while !*resumed {
            resumed = cvar.wait(resumed).unwrap();
        }
        *resumed = false;

        crate::logger::log_job(job_id, job_name, "Agent control confirmed. Resuming pipeline...");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();
    }

    ctx.request_repaint();
    Ok(())
}

/// Генерує простий UUID v4.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Простий унікальний рядок на основі часу та PID
    format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        nanos,
        std::process::id() & 0xFFFF,
        (nanos >> 16) & 0x0FFF,
        ((nanos >> 8) & 0x3FFF) | 0x8000,
        (nanos as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) & 0xFFFFFFFFFFFF,
    )
}

/// Streaming-версія: on_chunk викликається по мірі отримання chunks від агента.
pub fn call_agent_new_session_streaming(
    service: &str,
    model: &str,
    prompt: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str) + Send,
) -> Result<String, String> {
    if service == "Claude Code" {
        crate::api::claude::call_claude_code_new_session_streaming(model, prompt, session_id, job_info, working_dir, on_chunk)
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_new_session_streaming(model, prompt, session_id, job_info, working_dir, on_chunk)
    } else {
        Err(format!("Agent sessions not supported for service: {}", service))
    }
}

/// Продовжує сесію агента (--resume) залежно від сервісу.
pub fn call_agent_resume(
    service: &str,
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    if service == "Claude Code" {
        crate::api::claude::call_claude_code_resume(model, message, session_id, job_info, working_dir)
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_resume(model, message, session_id, job_info, working_dir)
    } else {
        Err(format!("Agent sessions not supported for service: {}", service))
    }
}

/// Зберігає историю чату агента у файл agent_chat.json у папці задачі.
fn save_agent_chat_to_file(save_dir: &std::path::Path, chat: &[crate::queue::AgentChatMessage]) {
    let messages: Vec<serde_json::Value> = chat.iter().map(|m| {
        serde_json::json!({ "role": m.role, "content": m.content })
    }).collect();
    let json = serde_json::to_string_pretty(&messages).unwrap_or_default();
    let _ = std::fs::write(save_dir.join("agent_chat.json"), json);
}

/// Після генерації медіафайлів заповнює поле `media` в timeline.json фактичними шляхами.
fn assign_media_to_timeline(save_dir: &std::path::Path) -> Result<(), String> {
    let timeline_path = save_dir.join("timeline.json");
    let content = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Cannot read timeline.json: {}", e))?;
    let mut timeline = serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
        .map_err(|e| format!("Invalid timeline.json: {}", e))?;

    let media_dir = save_dir.join("media");
    let mut files: Vec<String> = std::fs::read_dir(&media_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            let ext = s.rsplit('.').next().unwrap_or("").to_lowercase();
            matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "mp4" | "mov" | "avi" | "mkv" | "webm")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    files.sort();

    if files.is_empty() {
        return Ok(());
    }

    let n_segs = timeline.segments.len();
    let n_files = files.len();

    for (i, seg) in timeline.segments.iter_mut().enumerate() {
        let file_idx = if n_files <= n_segs {
            (i as f64 * n_files as f64 / n_segs as f64).floor() as usize
        } else {
            i.min(n_files - 1)
        };
        seg.media = files.get(file_idx).map(|f| format!("media/{}", f));
    }

    let json = serde_json::to_string_pretty(&timeline)
        .map_err(|e| format!("JSON error: {}", e))?;
    std::fs::write(&timeline_path, json)
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
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
    agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    montage_control_resume: Arc<(Mutex<bool>, Condvar)>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        crate::logger::log_job(job_id, &job_name, "Job started.");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();

        // Гарантуємо існування кінцевої папки з самого початку обробки
        if let Err(e) = std::fs::create_dir_all(&settings.save_path) {
            crate::logger::log_job(job_id, &job_name, &format!("Failed to create output dir: {}", e));
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

        // При агентному режимі (Claude Code / Gemini CLI) гілки виконуються послідовно:
        // AV → Агент → Медіа. В звичайному режимі — паралельно.
        let run_av = settings.voiceover_enabled;
        let run_video = settings.video_enabled;
        let is_agent_mode = run_video &&
            (settings.video_llm_service == "Claude Code" || settings.video_llm_service == "Gemini CLI");

        if is_agent_mode {
            // === Агентний режим: послідовно ===
            if run_av {
                crate::logger::log_job(job_id, &job_name, "Agent mode: starting AV branch (voiceover + subtitles)...");
                if let Err(e) = run_av_branch(
                    job_id, job_name.clone(), settings.clone(), voice_text.clone(),
                    Arc::clone(&voiceover_stage), Arc::clone(&subtitles_stage),
                    Arc::clone(&audio_duration), ctx.clone(),
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }
            }

            // Агент створює timeline.json на основі subtitle.srt
            crate::logger::log_job(job_id, &job_name, "Agent mode: creating timeline.json...");
            *video_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            if let Err(e) = run_agent_timeline(
                job_id, &job_name, &settings,
                Arc::clone(&status),
                Arc::clone(&agent_chat),
                Arc::clone(&agent_session),
                Arc::clone(&agent_control_resume),
                &ctx,
            ) {
                crate::logger::log_job(job_id, &job_name, &format!("Agent timeline error: {}", e));
                *video_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                ctx.request_repaint();
                return;
            }

            // Генерація медіа (сегменти читаються з timeline.json)
            if let Err(e) = run_video_branch(
                job_id, job_name.clone(), settings.clone(), Arc::clone(&translated_text),
                Arc::clone(&video_stage), Arc::clone(&prompts_progress),
                Arc::clone(&media_progress), Arc::clone(&total_cost), ctx.clone(),
            ) {
                *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                ctx.request_repaint();
                return;
            }

            // Патчимо timeline.json фактичними шляхами медіафайлів
            let save_dir_agent = std::path::Path::new(&settings.save_path);
            if let Err(e) = assign_media_to_timeline(save_dir_agent) {
                crate::logger::log_job(job_id, &job_name, &format!("assign_media warning: {}", e));
            } else {
                crate::logger::log_job(job_id, &job_name, "Timeline patched with media paths.");
            }

        } else {
            // === Звичайний режим: паралельно ===
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
        }

        // Пауза контролю монтажу перед рендером
        if settings.montage_control_enabled {
            crate::logger::log_job(job_id, &job_name, "Awaiting montage control confirmation from user...");
            *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMontageControl;
            ctx.request_repaint();

            let (lock, cvar) = &*montage_control_resume;
            let mut resumed = lock.lock().unwrap();
            while !*resumed {
                resumed = cvar.wait(resumed).unwrap();
            }
            *resumed = false;

            crate::logger::log_job(job_id, &job_name, "Montage control confirmed. Resuming pipeline...");
            *status.lock().unwrap() = crate::queue::JobStatus::Running;
            ctx.request_repaint();
        }

        // Етап 5: Монтаж (FFmpeg або CapCut)
        if settings.montage_enabled {
            crate::logger::log_job(job_id, &job_name, "Starting montage stage...");
            *montage_stage.lock().unwrap() = crate::queue::StageStatus::Running;
            ctx.request_repaint();

            let audio_dur = *audio_duration.lock().unwrap();
            let save_dir = std::path::Path::new(&settings.save_path);

            if settings.capcut_enabled {
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
                        *status.lock().unwrap() = crate::queue::JobStatus::Failed(format!("CapCut: {}", e));
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
                        crate::logger::log_job(job_id, &job_name, &format!("Montage failed: {}", e));
                        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                        *status.lock().unwrap() = crate::queue::JobStatus::Failed(format!("Montage: {}", e));
                        ctx.request_repaint();
                        return;
                    }
                }
            }
            ctx.request_repaint();
        }

        crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
        *status.lock().unwrap() = crate::queue::JobStatus::Done;
        ctx.request_repaint();
    });
}

/// Виконує timeline + montage (спільна фінальна частина для retry-функцій).
fn run_final_stages(
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
    ctx: &egui::Context,
) -> Result<(), String> {
    // Timeline
    if settings.video_enabled {
        let source_text = if settings.translation_enabled {
            translated_text.lock().unwrap().clone().unwrap_or_else(|| settings.text.clone())
        } else {
            settings.text.clone()
        };
        let segments = crate::core::pipeline::timeline::text_splitter::split_text(
            &source_text, &settings.text_split_mode, settings.text_split_char_limit,
        );
        let audio_dur = *audio_duration.lock().unwrap();
        let save_dir = std::path::Path::new(&settings.save_path);
        match crate::core::pipeline::timeline::sync::build_timeline(save_dir, &segments, audio_dur, job_name) {
            Ok(_) => crate::logger::log_job(job_id, job_name, "Timeline saved: timeline.json"),
            Err(e) => crate::logger::log_job(job_id, job_name, &format!("Timeline warning: {}", e)),
        }
    }

    // Пауза контролю монтажу перед рендером (якщо увімкнено)
    if settings.montage_enabled && settings.montage_control_enabled {
        crate::logger::log_job(job_id, job_name, "Awaiting montage control confirmation from user...");
        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMontageControl;
        ctx.request_repaint();

        let (lock, cvar) = &**montage_control_resume;
        let mut resumed = lock.lock().unwrap();
        while !*resumed {
            resumed = cvar.wait(resumed).unwrap();
        }
        *resumed = false;

        crate::logger::log_job(job_id, job_name, "Montage control confirmed. Resuming pipeline...");
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();
    }

    // Монтаж (FFmpeg або CapCut)
    if settings.montage_enabled {
        crate::logger::log_job(job_id, job_name, "Starting montage stage...");
        *montage_stage.lock().unwrap() = crate::queue::StageStatus::Running;
        ctx.request_repaint();

        let audio_dur = *audio_duration.lock().unwrap();
        let save_dir = std::path::Path::new(&settings.save_path);
        let job_id_log = job_id;
        let job_name_log = job_name.to_string();

        if settings.capcut_enabled {
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
            save_dir, job_name, audio_dur,
            settings.montage_fps, &settings.montage_preset, settings.montage_bitrate,
            &settings.montage_transition, settings.montage_transition_duration,
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
    agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    montage_control_resume: Arc<(Mutex<bool>, Condvar)>,
    agent_chat: Arc<Mutex<Vec<crate::queue::AgentChatMessage>>>,
    agent_session: Arc<Mutex<Option<crate::queue::AgentSessionInfo>>>,
    ctx: egui::Context,
) {
    use crate::queue::RetryStage::*;
    use crate::queue::StageStatus::Pending as SPending;

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
            *agent_control_resume.0.lock().unwrap() = false;
            *montage_control_resume.0.lock().unwrap() = false;
            run_pipeline(
                job_id, job_name, settings, status,
                translation_stage, voiceover_stage, video_stage, subtitles_stage, montage_stage,
                translated_text, total_cost, audio_duration,
                prompts_progress, media_progress, montage_progress, montage_file_size,
                media_control_resume, agent_control_resume, montage_control_resume,
                agent_chat, agent_session, ctx,
            );
        }

        // Повтор озвучки → субтитри → монтаж (відеоряд не чіпаємо)
        Voiceover => {
            *voiceover_stage.lock().unwrap() = SPending;
            *subtitles_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *audio_duration.lock().unwrap() = None;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;

            let voice_text = translated_text.lock().unwrap().clone()
                .unwrap_or_else(|| settings.text.clone());

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();

                crate::logger::log_job(job_id, &job_name, "Retry: AV branch (voiceover + subtitles)...");
                if let Err(e) = run_av_branch(
                    job_id, job_name.clone(), settings.clone(), voice_text,
                    Arc::clone(&voiceover_stage), Arc::clone(&subtitles_stage),
                    Arc::clone(&audio_duration), ctx.clone(),
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = run_final_stages(
                    job_id, &job_name, &settings, &translated_text, &audio_duration,
                    &status, &montage_stage, &montage_progress, &montage_file_size, &montage_control_resume, &ctx,
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
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

                crate::logger::log_job(job_id, &job_name, "Retry: video branch...");
                let video_result = run_video_branch(
                    job_id, job_name.clone(), settings.clone(), Arc::clone(&translated_text),
                    Arc::clone(&video_stage), Arc::clone(&prompts_progress),
                    Arc::clone(&media_progress), Arc::clone(&total_cost), ctx.clone(),
                );

                // Пауза для контролю зображень
                if settings.media_control_enabled && settings.video_enabled {
                    if let Ok(()) = &video_result {
                        crate::logger::log_job(job_id, &job_name, "Video done. Awaiting media review...");
                        *status.lock().unwrap() = crate::queue::JobStatus::AwaitingMediaControl;
                        ctx.request_repaint();
                        let (lock, cvar) = &*media_control_resume;
                        let mut resumed = lock.lock().unwrap();
                        while !*resumed { resumed = cvar.wait(resumed).unwrap(); }
                        *resumed = false;
                        crate::logger::log_job(job_id, &job_name, "Media review confirmed. Resuming...");
                        *status.lock().unwrap() = crate::queue::JobStatus::Running;
                        ctx.request_repaint();
                    }
                }

                if let Err(e) = video_result {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = run_final_stages(
                    job_id, &job_name, &settings, &translated_text, &audio_duration,
                    &status, &montage_stage, &montage_progress, &montage_file_size, &montage_control_resume, &ctx,
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }

                crate::logger::log_job(job_id, &job_name, "Job completed successfully.");
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
                ctx.request_repaint();
            });
        }

        // Повтор лише субтитрів → монтаж
        Subtitles => {
            *subtitles_stage.lock().unwrap() = SPending;
            *montage_stage.lock().unwrap() = SPending;
            *montage_progress.lock().unwrap() = None;
            *montage_file_size.lock().unwrap() = None;

            std::thread::spawn(move || {
                *status.lock().unwrap() = crate::queue::JobStatus::Running;
                ctx.request_repaint();

                crate::logger::log_job(job_id, &job_name, "Retry: subtitles...");
                if let Err(e) = run_subtitles_only(&settings, job_id, &job_name, &subtitles_stage, &ctx) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
                    ctx.request_repaint();
                    return;
                }

                if let Err(e) = run_final_stages(
                    job_id, &job_name, &settings, &translated_text, &audio_duration,
                    &status, &montage_stage, &montage_progress, &montage_file_size, &montage_control_resume, &ctx,
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
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

                crate::logger::log_job(job_id, &job_name, "Retry: montage...");
                if let Err(e) = run_final_stages(
                    job_id, &job_name, &settings, &translated_text, &audio_duration,
                    &status, &montage_stage, &montage_progress, &montage_file_size, &montage_control_resume, &ctx,
                ) {
                    *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
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
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: String,
    googler_video_upscale_quality: String,
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

            if let Err(e) = upscale_video_if_needed(
                &video_path,
                googler_video_upscale_enabled,
                &googler_video_upscale_resolution,
                &googler_video_upscale_quality,
                job_id,
                &job_name,
            ) {
                crate::logger::log_job(job_id, &job_name, &format!("Помилка апскейлу/кропу анімованого відео {}: {}", file_name, e));
            }

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
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: String,
    googler_video_upscale_quality: String,
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
                    if media_type == "video" {
                        if let Err(e) = upscale_video_if_needed(
                            &file_path,
                            googler_video_upscale_enabled,
                            &googler_video_upscale_resolution,
                            &googler_video_upscale_quality,
                            job_id,
                            &job_name,
                        ) {
                            crate::logger::log_job(job_id, &job_name, &format!("Помилка апскейлу/кропу перегенерованого відео {}: {}", file_name, e));
                        }
                    }
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

/// Виконує апскейл та кроп відеофайлу за допомогою FFmpeg.
/// Робить це in-place: перейменовує файл у тимчасовий, запускає FFmpeg,
/// записує результат у оригінальний шлях, видаляє тимчасовий файл.
pub fn upscale_video_if_needed(
    video_path: &std::path::Path,
    enabled: bool,
    resolution: &str,
    quality: &str,
    job_id: u64,
    job_name: &str,
) -> Result<(), String> {
    if !video_path.exists() {
        return Err(format!("Файл не існує: {}", video_path.display()));
    }

    crate::logger::log_job(
        job_id,
        job_name,
        &format!("Обробка відео (апскейл: {}, роздільна здатність: {}, кроп: 107% (дефолт), якість: {})...", enabled, resolution, quality),
    );

    // Створюємо шлях для тимчасового файлу
    let temp_path = video_path.with_extension("upscale_temp.mp4");
    if let Err(e) = std::fs::rename(video_path, &temp_path) {
        return Err(format!("Не вдалося перейменувати файл для апскейлу: {}", e));
    }

    // Виконуємо ffprobe для зчитування FPS та розмірів відео
    let ffprobe_cmd = crate::bundle::ffprobe_path();
    let ffprobe_out = std::process::Command::new(&ffprobe_cmd)
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate,avg_frame_rate,nb_frames,duration",
            "-of", "csv=p=0",
        ])
        .arg(&temp_path)
        .output();

    let mut width = 1280;
    let mut height = 720;
    let mut fps = 30.0;
    if let Ok(out) = ffprobe_out {
        let s = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = s.trim().split(',').collect();
        if parts.len() >= 4 {
            width = parts[0].trim().parse().unwrap_or(1280);
            height = parts[1].trim().parse().unwrap_or(720);
            let r_fps = parts[2];
            let avg_fps = parts[3];
            
            let mut calculated_fps = None;
            if parts.len() >= 6 {
                let nb_frames: f64 = parts[4].trim().parse().unwrap_or(0.0);
                let duration: f64 = parts[5].trim().parse().unwrap_or(0.0);
                if duration > 0.0 && nb_frames > 0.0 {
                    calculated_fps = Some(nb_frames / duration);
                }
            }

            let parsed_fps = calculated_fps.unwrap_or_else(|| {
                let rate = if avg_fps != "0/0" && !avg_fps.is_empty() { avg_fps } else { r_fps };
                if rate.contains('/') {
                    let subparts: Vec<&str> = rate.split('/').collect();
                    if subparts.len() == 2 {
                        let num: f64 = subparts[0].trim().parse().unwrap_or(30.0);
                        let den: f64 = subparts[1].trim().parse().unwrap_or(1.0);
                        if den > 0.0 { num / den } else { 30.0 }
                    } else {
                        30.0
                    }
                } else {
                    rate.trim().parse().unwrap_or(30.0)
                }
            });

            if parsed_fps > 1.0 && parsed_fps < 120.0 {
                fps = (parsed_fps * 1000.0).round() / 1000.0;
            }
        }
    }

    let (target_w, target_h) = if enabled {
        match resolution {
            "2K" => (2560, 1440),
            "4K" => (3840, 2160),
            _ => (1920, 1080), // 1080p
        }
    } else {
        (width, height)
    };

    let sharpen = if enabled {
        match quality {
            "fast" => "unsharp=5:5:0.55:3:3:0.25".to_string(),
            "max" => "hqdn3d=1.5:1.5:5:5,unsharp=7:7:0.85:5:5:0.4".to_string(),
            _ => "hqdn3d=1.2:1.2:4:4,unsharp=5:5:0.75:5:5:0.35".to_string(), // balanced
        }
    } else {
        "".to_string()
    };
    
    let ffmpeg_preset = if enabled {
        match quality {
            "fast" => "veryfast",
            "max" => "slow",
            _ => "medium", // balanced
        }
    } else {
        "ultrafast"
    };
    
    let crf = if enabled {
        match quality {
            "fast" => "20",
            "max" => "16",
            _ => "18", // balanced
        }
    } else {
        "18"
    };
    
    let scale_w = ((target_w as f64 * 1.07).round() as i32) & !1;
    let scale_h = ((target_h as f64 * 1.07).round() as i32) & !1;
    let fit = format!("scale={}:{}:flags=lanczos:force_original_aspect_ratio=increase,crop={}:{}:iw-{}:0", scale_w, scale_h, target_w, target_h, target_w);

    let vf = if sharpen.is_empty() {
        format!("setpts=N/({}*TB),{}", fps, fit)
    } else {
        format!("setpts=N/({}*TB),{},{}", fps, fit, sharpen)
    };
    let fps_str = format!("{}", fps);

    let ffmpeg_cmd = crate::bundle::ffmpeg_path();
    let mut args = vec![
        "-y",
        "-hide_banner",
        "-fflags", "+genpts",
        "-i", temp_path.to_str().unwrap(),
        "-vf", &vf,
        "-r", &fps_str,
        "-fps_mode", "cfr",
        "-c:v", "libx264",
        "-preset", ffmpeg_preset,
        "-crf", crf,
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
    ];

    if !enabled {
        args.extend_from_slice(&["-threads", "2"]);
    }

    args.extend_from_slice(&["-map", "0:v:0", "-map", "0:a?", "-c:a", "aac", "-b:a", "192k"]);
    args.push(video_path.to_str().unwrap());

    let child = std::process::Command::new(&ffmpeg_cmd)
        .args(&args)
        .output();

    let restore_original = || {
        if temp_path.exists() {
            if video_path.exists() {
                let _ = std::fs::remove_file(video_path);
            }
            let _ = std::fs::rename(&temp_path, video_path);
        }
    };

    let clean_up = || {
        let _ = std::fs::remove_file(&temp_path);
    };

    match child {
        Ok(output) => {
            if output.status.success() {
                clean_up();
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Апскейл/кроп завершено успішно: {}", video_path.file_name().unwrap_or_default().to_string_lossy()),
                );
                Ok(())
            } else {
                restore_original();
                let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("FFmpeg error: {}", err_msg.trim()))
            }
        }
        Err(e) => {
            restore_original();
            Err(format!("Не вдалося запустити FFmpeg: {}", e))
        }
    }
}

