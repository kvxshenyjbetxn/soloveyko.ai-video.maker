use std::sync::{Arc, Mutex};

use eframe::egui;

use super::voiceover;

/// Визначає тривалість WAV-файлу в секундах, зчитуючи RIFF/fmt заголовок.
/// Визначає точну тривалість аудіофайлу в секундах через ffprobe.
fn get_audio_duration_secs(path: &std::path::Path) -> Option<f64> {
    let ffprobe = crate::bundle::ffprobe_path();
    let mut cmd = std::process::Command::new(&ffprobe);
    cmd.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ])
    .arg(path);
    crate::bundle::set_no_window(&mut cmd);
    let out = cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<f64>().ok()
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
    let output_srt = save_dir.join("subtitle.srt");

    let mut args: Vec<String> = vec![
        "--audio".to_string(),
        audio_path.to_str().unwrap_or("voice.wav").to_string(),
        "--model".to_string(),
        settings.whisper_model.clone(),
        "--output".to_string(),
        output_base.to_str().unwrap_or("subtitle").to_string(),
        "--ffmpeg-path".to_string(),
        crate::bundle::ffmpeg_path(),
    ];
    if settings.whisper_language != "auto" {
        args.push("--language".to_string());
        args.push(settings.whisper_language.clone());
    }

    crate::logger::log_job(
        job_id,
        job_name,
        &format!("Running: {} {}", whisperx_cmd.display(), args.join(" ")),
    );

    let mut whisperx_proc = std::process::Command::new(&whisperx_cmd);
    whisperx_proc.args(&args);
    crate::bundle::set_no_window(&mut whisperx_proc);
    match crate::api::process::output_tracked(&mut whisperx_proc, Some(job_id)) {
        Ok(out) if out.status.success() => {
            // Зчитуємо subtitle.json і генеруємо subtitle.srt з max_line_width
            match std::fs::read_to_string(&output_json) {
                Ok(json_str) => {
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(json) => {
                            let raw_words: Vec<serde_json::Value> = json
                                .get("words")
                                .and_then(|w| w.as_array())
                                .cloned()
                                .unwrap_or_default();

                            // Відрізаємо слова що виходять за межі реального аудіо (WhisperX галюцинує в кінці)
                            let audio_max_secs = get_audio_duration_secs(&audio_path);
                            let words_trimmed: Vec<serde_json::Value> =
                                if let Some(max) = audio_max_secs {
                                    raw_words
                                        .into_iter()
                                        .filter(|w| {
                                            w.get("start")
                                                .and_then(|s| s.as_f64())
                                                .map_or(true, |t| t < max)
                                        })
                                        .collect()
                                } else {
                                    raw_words
                                };

                            let srt = crate::api::assemblyai::whisperx_words_to_srt(
                                &words_trimmed,
                                settings.whisper_max_line_width,
                            );
                            if let Err(e) = std::fs::write(&output_srt, &srt) {
                                crate::logger::log_job(
                                    job_id,
                                    job_name,
                                    &format!("Failed to save subtitle.srt: {}", e),
                                );
                            }

                            // Генеруємо subtitle.ass зі стилем запеченим всередині
                            let ass = srt_to_ass(
                                &srt,
                                &settings.subtitle_font,
                                settings.subtitle_font_size,
                                settings.subtitle_color,
                                settings.subtitle_margin_v,
                            );
                            let ass_path = save_dir.join("subtitle.ass");
                            if let Err(e) = std::fs::write(&ass_path, &ass) {
                                crate::logger::log_job(
                                    job_id,
                                    job_name,
                                    &format!("Failed to save subtitle.ass: {}", e),
                                );
                            }

                            // Зберігаємо тільки words + language (без segments)
                            let filtered = serde_json::json!({
                                "language": json.get("language").cloned().unwrap_or(serde_json::Value::Null),
                                "words": words_trimmed,
                            });
                            if let Ok(s) = serde_json::to_string_pretty(&filtered) {
                                let _ = std::fs::write(&output_json, s);
                            }
                        }
                        Err(e) => {
                            crate::logger::log_job(
                                job_id,
                                job_name,
                                &format!("WhisperX: failed to parse subtitle.json: {}", e),
                            );
                        }
                    }
                }
                Err(e) => {
                    crate::logger::log_job(
                        job_id,
                        job_name,
                        &format!("WhisperX: subtitle.json not found: {}", e),
                    );
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
            let msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            let short = format!(
                "WhisperX error: {}",
                msg.chars().take(120).collect::<String>()
            );
            crate::logger::log_job(job_id, job_name, &format!("WhisperX error: {}", msg.trim()));
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(short)
        }
        Err(e) => {
            let msg = format!("WhisperX launch error: {}", e);
            crate::logger::log_job(
                job_id,
                job_name,
                &format!("WhisperX not found or failed to start: {}", e),
            );
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

    crate::logger::log_job(
        job_id,
        job_name,
        &format!("Uploading audio to AssemblyAI: {}", audio_path.display()),
    );

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
            let ass = srt_to_ass(
                &srt,
                &settings.subtitle_font,
                settings.subtitle_font_size,
                settings.subtitle_color,
                settings.subtitle_margin_v,
            );
            let ass_path = save_dir.join("subtitle.ass");
            if let Err(e) = std::fs::write(&ass_path, &ass) {
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Failed to save subtitle.ass: {}", e),
                );
            }

            // Зберігаємо лише масив words — решта метадані API-запиту, не потрібні для таймлайну
            let words_only = json_response
                .get("words")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            if let Ok(json_str) = serde_json::to_string_pretty(&words_only) {
                let _ = std::fs::write(&json_path, json_str);
            }

            crate::logger::log_job(
                job_id,
                job_name,
                "Subtitles saved: subtitle.srt (AssemblyAI)",
            );
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
        crate::logger::log_job(
            job_id,
            job_name,
            &format!(
                "Model '{}' not found — downloading...",
                settings.whisper_model
            ),
        );
        if let Err(e) = crate::bundle::download_whisper_model(&settings.whisper_model, |label| {
            crate::logger::log_job(job_id, job_name, &label);
        }) {
            let msg = format!(
                "Subtitles error: failed to download model '{}': {}",
                settings.whisper_model, e
            );
            crate::logger::log_job(job_id, job_name, &msg);
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            return Err(msg);
        }
        crate::logger::log_job(
            job_id,
            job_name,
            &format!("Model '{}' downloaded.", settings.whisper_model),
        );
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
    let audio_stem = audio_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("voice");
    let generated_srt = save_dir.join(format!("{}.srt", audio_stem));

    let whisper_amd_cmd = crate::bundle::whisper_amd_cmd_path();
    let mut args: Vec<String> = vec![
        "-f".to_string(),
        audio_path.to_str().unwrap_or("voice.wav").to_string(),
        "-m".to_string(),
        model_path.to_str().unwrap().to_string(),
        "-osrt".to_string(),
    ];
    if settings.whisper_language != "auto" {
        args.push("-l".to_string());
        args.push(settings.whisper_language.clone());
    }
    crate::logger::log_job(
        job_id,
        job_name,
        &format!("Running: {} {}", whisper_amd_cmd.display(), args.join(" ")),
    );

    let mut whisper_amd_proc = std::process::Command::new(&whisper_amd_cmd);
    whisper_amd_proc.args(&args);
    crate::bundle::set_no_window(&mut whisper_amd_proc);
    match crate::api::process::output_tracked(&mut whisper_amd_proc, Some(job_id)) {
        Ok(out) if out.status.success() => {
            let srt_path = save_dir.join("subtitle.srt");

            if generated_srt.exists() && generated_srt != srt_path {
                if let Err(e) = std::fs::rename(&generated_srt, &srt_path) {
                    crate::logger::log_job(
                        job_id,
                        job_name,
                        &format!("Whisper AMD: failed to rename SRT: {}", e),
                    );
                }
            }

            crate::logger::log_job(
                job_id,
                job_name,
                "Subtitles saved: subtitle.srt (Whisper AMD)",
            );

            if let Ok(srt) = std::fs::read_to_string(&srt_path) {
                let ass = srt_to_ass(
                    &srt,
                    &settings.subtitle_font,
                    settings.subtitle_font_size,
                    settings.subtitle_color,
                    settings.subtitle_margin_v,
                );
                let _ = std::fs::write(save_dir.join("subtitle.ass"), &ass);
            }
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
            ctx.request_repaint();
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            let short = format!(
                "Whisper AMD error: {}",
                msg.chars().take(120).collect::<String>()
            );
            crate::logger::log_job(
                job_id,
                job_name,
                &format!("Whisper AMD error: {}", msg.trim()),
            );
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(short)
        }
        Err(e) => {
            let msg = format!("Whisper AMD launch error: {}", e);
            crate::logger::log_job(
                job_id,
                job_name,
                &format!("Whisper AMD not found or failed to start: {}", e),
            );
            *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
            Err(msg)
        }
    }
}

/// Виконує лише генерацію субтитрів (без озвучки).
/// Використовується як для основного пайплайну, так і для повтору субтитрів.
pub(super) fn run_subtitles_only(
    settings: &crate::queue::JobSettings,
    job_id: u64,
    job_name: &str,
    subtitles_stage: &Arc<Mutex<crate::queue::StageStatus>>,
    ctx: &egui::Context,
) -> Result<(), String> {
    // На non-Windows WhisperAMD недоступний — використовуємо Whisper як запасний варіант
    let effective_service: &str =
        if !cfg!(target_os = "windows") && settings.subtitles_service == "WhisperAMD" {
            "Whisper"
        } else {
            &settings.subtitles_service
        };

    super::ensure_job_not_cancelled(job_id)?;

    if effective_service == "WhisperX" {
        run_whisperx(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if effective_service == "AssemblyAI" {
        run_assemblyai(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if effective_service == "WhisperAMD" {
        run_whisper_amd(settings, job_id, job_name, subtitles_stage, ctx)?;
    } else if effective_service == "Whisper" {
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
            crate::logger::log_job(
                job_id,
                job_name,
                &format!(
                    "Model '{}' not found — downloading...",
                    settings.whisper_model
                ),
            );
            if let Err(e) =
                crate::bundle::download_whisper_model(&settings.whisper_model, |label| {
                    crate::logger::log_job(job_id, job_name, &label);
                })
            {
                let msg = format!(
                    "Subtitles error: failed to download model '{}': {}",
                    settings.whisper_model, e
                );
                crate::logger::log_job(job_id, job_name, &msg);
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(msg);
            }
            crate::logger::log_job(
                job_id,
                job_name,
                &format!("Model '{}' downloaded.", settings.whisper_model),
            );
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

        // whisper-cli.exe (C runtime, ANSI argv) не розуміє Unicode-шляхи на Windows —
        // копіюємо аудіо у тимчасову папку з гарантовано ASCII-шляхом.
        #[cfg(target_os = "windows")]
        let (whisper_audio, whisper_out_stem, whisper_temp) = {
            let tmp = std::env::temp_dir().join("soloveyko_whisper");
            let _ = std::fs::create_dir_all(&tmp);
            let ext = audio_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("wav");
            let tmp_audio = tmp.join(format!("voice.{}", ext));
            if let Err(e) = std::fs::copy(&audio_path, &tmp_audio) {
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Whisper: не вдалося скопіювати аудіо у temp: {}", e),
                );
            }
            let tmp_stem = tmp.join("subtitle");
            (tmp_audio, tmp_stem, Some(tmp))
        };
        #[cfg(not(target_os = "windows"))]
        let (whisper_audio, whisper_out_stem, _whisper_temp) = {
            let stem = save_dir.join("subtitle");
            (audio_path.clone(), stem, None::<std::path::PathBuf>)
        };

        let whisper_cmd = crate::bundle::whisper_path();
        let mut args: Vec<String> = vec![
            whisper_audio.to_str().unwrap_or("voice.wav").to_string(),
            "-m".to_string(),
            model_path.to_str().unwrap().to_string(),
            "--output-srt".to_string(),
            "-of".to_string(),
            whisper_out_stem.to_str().unwrap().to_string(),
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

        crate::logger::log_job(
            job_id,
            job_name,
            &format!("Running: {} {}", whisper_cmd, args.join(" ")),
        );

        let mut whisper_proc = std::process::Command::new(&whisper_cmd);
        whisper_proc.args(&args);
        crate::bundle::set_no_window(&mut whisper_proc);
        match crate::api::process::output_tracked(&mut whisper_proc, Some(job_id)) {
            Ok(out) if out.status.success() => {
                // На Windows переносимо результат з temp-папки назад у save_dir
                #[cfg(target_os = "windows")]
                if let Some(ref tmp) = whisper_temp {
                    let tmp_srt = tmp.join("subtitle.srt");
                    if tmp_srt.exists() {
                        let _ = std::fs::copy(&tmp_srt, save_dir.join("subtitle.srt"));
                    }
                    let _ = std::fs::remove_dir_all(tmp);
                }
                crate::logger::log_job(job_id, job_name, "Subtitles saved: subtitle.srt");
                let srt_path = save_dir.join("subtitle.srt");
                if let Ok(srt) = std::fs::read_to_string(&srt_path) {
                    let ass = srt_to_ass(
                        &srt,
                        &settings.subtitle_font,
                        settings.subtitle_font_size,
                        settings.subtitle_color,
                        settings.subtitle_margin_v,
                    );
                    let _ = std::fs::write(save_dir.join("subtitle.ass"), &ass);
                }
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Done;
                ctx.request_repaint();
            }
            Ok(out) => {
                #[cfg(target_os = "windows")]
                if let Some(ref tmp) = whisper_temp {
                    let _ = std::fs::remove_dir_all(tmp);
                }
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let msg = if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    stdout.to_string()
                };
                let short = format!(
                    "Whisper error: {}",
                    msg.chars().take(120).collect::<String>()
                );
                crate::logger::log_job(job_id, job_name, &format!("Whisper error: {}", msg.trim()));
                *subtitles_stage.lock().unwrap() = crate::queue::StageStatus::Failed;
                return Err(short);
            }
            Err(e) => {
                #[cfg(target_os = "windows")]
                if let Some(ref tmp) = whisper_temp {
                    let _ = std::fs::remove_dir_all(tmp);
                }
                let msg = format!("Whisper launch error: {}", e);
                crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Whisper not found or failed to start: {}", e),
                );
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
                        Ok(_) => crate::logger::log_job(
                            job_id,
                            job_name,
                            "Karaoke ASS generated: subtitle.ass",
                        ),
                        Err(e) => crate::logger::log_job(
                            job_id,
                            job_name,
                            &format!("Failed to save subtitle.ass (karaoke): {}", e),
                        ),
                    }
                }
                Err(e) => crate::logger::log_job(
                    job_id,
                    job_name,
                    &format!("Karaoke generation error: {}", e),
                ),
            }
        } else {
            crate::logger::log_job(
                job_id,
                job_name,
                "Karaoke: subtitle.json not found, skipping.",
            );
        }
    }

    super::ensure_job_not_cancelled(job_id)?;
    Ok(())
}

/// Гілка Озвучка + Субтитри (виконується паралельно з відеорядом).
/// Повертає Ok(()) або Err з описом першої помилки.
pub(super) fn run_av_branch(
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
        let src_label = if settings.translation_enabled {
            "translation"
        } else {
            "original"
        };
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
                    crate::logger::log_job(
                        job_id,
                        &job_name,
                        "Converting audio to WAV via FFmpeg...",
                    );

                    let ffmpeg_cmd = crate::bundle::ffmpeg_path();
                    let mut ffmpeg_proc = std::process::Command::new(&ffmpeg_cmd);
                    ffmpeg_proc.args(&[
                        "-y",
                        "-hide_banner",
                        "-loglevel",
                        "error",
                        "-i",
                        mp3_path.to_str().unwrap_or("voice.mp3"),
                        wav_path.to_str().unwrap_or("voice.wav"),
                    ]);
                    crate::bundle::set_no_window(&mut ffmpeg_proc);
                    let result = crate::api::process::output_tracked(&mut ffmpeg_proc, Some(job_id));

                    match result {
                        Ok(out) if out.status.success() => {
                            crate::logger::log_job(job_id, &job_name, "WAV conversion successful.");
                            wav_path
                        }
                        Ok(out) => {
                            let err = String::from_utf8_lossy(&out.stderr);
                            crate::logger::log_job(
                                job_id,
                                &job_name,
                                &format!("WAV conversion failed: {}. Using MP3.", err),
                            );
                            mp3_path
                        }
                        Err(e) => {
                            crate::logger::log_job(
                                job_id,
                                &job_name,
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
                        job_id,
                        &job_name,
                        &format!("Audio duration: {:.1}s", dur),
                    );
                    *audio_duration.lock().unwrap() = Some(dur);
                }

                super::ensure_job_not_cancelled(job_id)?;
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
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse subtitle.json: {}", e))?;

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
            format!(
                "&H00{:02X}{:02X}{:02X}",
                highlight_color[2], highlight_color[1], highlight_color[0]
            ),
            format!("&H00{:02X}{:02X}{:02X}", color[2], color[1], color[0]),
        )
    };
    let outline_hex = format!(
        "&H00{:02X}{:02X}{:02X}",
        outline_color[2], outline_color[1], outline_color[0]
    );
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
        let arr = json
            .get("words")
            .and_then(|w| w.as_array())
            .ok_or("WhisperX subtitle.json: no 'words' array")?;
        arr.iter()
            .filter_map(|w| {
                let text = w.get("word")?.as_str()?.to_string();
                let start = w.get("start")?.as_f64()?;
                let end = w.get("end")?.as_f64()?;
                Some(Word {
                    text,
                    start_ms: (start * 1000.0) as u64,
                    end_ms: (end * 1000.0) as u64,
                })
            })
            .collect()
    } else {
        let arr = if let Some(arr) = json.as_array() {
            arr.as_slice()
        } else {
            json.get("words")
                .and_then(|w| w.as_array())
                .ok_or("AssemblyAI subtitle.json: expected array or 'words' array")?
                .as_slice()
        };
        arr.iter()
            .filter_map(|w| {
                let text = w.get("text")?.as_str()?.to_string();
                let start = w.get("start")?.as_u64()?;
                let end = w.get("end")?.as_u64()?;
                Some(Word {
                    text,
                    start_ms: start,
                    end_ms: end,
                })
            })
            .collect()
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
        lines.push((
            line_start_ms,
            line_end_ms,
            (group_start..group_end).collect(),
        ));
        group_start = group_end;
    }

    let mut events = String::new();

    // Closure для побудови рядка follow-режиму: current = usize::MAX → всі слова нормальним кольором.
    let build_follow_text = |indices: &[usize],
                             current: usize,
                             hi_hex: &str,
                             normal_hex: &str,
                             scale_tag: &str,
                             reset_scale: &str|
     -> String {
        let mut s = String::new();
        for (j, &idx) in indices.iter().enumerate() {
            if j == current {
                s.push_str(&format!(
                    "{{\\1c&H{}&{}}}{} ",
                    hi_hex, scale_tag, words[idx].text
                ));
            } else {
                s.push_str(&format!(
                    "{{\\1c&H{}&{}}}{} ",
                    normal_hex, reset_scale, words[idx].text
                ));
            }
        }
        s.trim_end().to_string()
    };

    if karaoke_mode == 2 {
        // Follow-режим: окрема Dialogue-подія для кожного слова.
        // Кожна подія показує весь рядок, але лише поточне слово підсвічене.
        // Після проговорення слово повертається до нормального кольору.
        let normal_hex = format!("{:02X}{:02X}{:02X}", color[2], color[1], color[0]);
        let hi_hex = format!(
            "{:02X}{:02X}{:02X}",
            highlight_color[2], highlight_color[1], highlight_color[0]
        );
        let scale_tag = if scale != 100 {
            format!("\\fscx{}\\fscy{}", scale, scale)
        } else {
            String::new()
        };
        let reset_scale = if scale != 100 {
            "\\fscx100\\fscy100"
        } else {
            ""
        };

        for (group_start_ms, group_end_ms, indices) in &lines {
            let n = indices.len();

            // Пауза до першого слова — всі нормальним кольором
            let first_word_start = words[indices[0]].start_ms;
            if first_word_start > *group_start_ms {
                let text = build_follow_text(
                    indices,
                    usize::MAX,
                    &hi_hex,
                    &normal_hex,
                    &scale_tag,
                    reset_scale,
                );
                events.push_str(&format!(
                    "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                    ms_to_ass_time(*group_start_ms),
                    ms_to_ass_time(first_word_start),
                    text
                ));
            }

            for (i, &word_idx) in indices.iter().enumerate() {
                let event_start = words[word_idx].start_ms;
                let event_end = if i + 1 < n {
                    words[indices[i + 1]].start_ms
                } else {
                    *group_end_ms
                };
                let text =
                    build_follow_text(indices, i, &hi_hex, &normal_hex, &scale_tag, reset_scale);
                events.push_str(&format!(
                    "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                    ms_to_ass_time(event_start),
                    ms_to_ass_time(event_end),
                    text
                ));
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

            events.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                start_str,
                end_str,
                text.trim_end()
            ));
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
        if block.is_empty() {
            continue;
        }

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
            let end = srt_time_to_ass(end_str.trim());
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
    if parts.len() != 2 {
        return "0:00:00.00".to_string();
    }
    let hms: Vec<&str> = parts[0].split(':').collect();
    if hms.len() != 3 {
        return "0:00:00.00".to_string();
    }
    let hours: u32 = hms[0].parse().unwrap_or(0);
    let mins: u32 = hms[1].parse().unwrap_or(0);
    let secs: u32 = hms[2].parse().unwrap_or(0);
    let cs: u32 = parts[1].trim().parse::<u32>().unwrap_or(0) / 10;
    format!("{}:{:02}:{:02}.{:02}", hours, mins, secs, cs)
}
