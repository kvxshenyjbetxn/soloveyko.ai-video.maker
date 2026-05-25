use std::path::{Path, PathBuf};

/// Знаходить аудіо-файл озвучки: спочатку voice.wav, потім voice.mp3.
pub fn find_voice_file(save_dir: &Path) -> Option<PathBuf> {
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Будує та запускає FFmpeg-монтаж.
///
/// `transition` — "none", "random" або конкретна назва xfade-ефекту.
/// `transition_duration_secs` — тривалість переходу в секундах (ігнорується якщо "none").
///
/// При переходах кожен кліп (крім останнього) розтягується на `transition_duration_secs`,
/// щоб cumulative offset відповідав абсолютним start_secs із timeline і синхронізація
/// з аудіо не розходилась.
pub fn run_montage(
    save_dir: &Path,
    task_name: &str,
    audio_duration_hint: Option<f64>,
    fps: u32,
    preset: &str,
    bitrate_mbps: u32,
    transition: &str,
    transition_duration_secs: f32,
    burn_subtitles: bool,
    log_fn: impl Fn(&str),
    on_progress: impl Fn(f32),
) -> Result<u64, String> {
    // Займаємо слот лімітера — чекаємо якщо всі потоки FFmpeg зайняті
    let _ffmpeg_permit = crate::api::ffmpeg::FfmpegLimiter::get().acquire();

    // ─── Структури для timeline.json ─────────────────────────────────────────
    #[derive(serde::Deserialize)]
    struct SegTiming {
        start_secs: f64,
        end_secs: f64,
        media: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Timeline {
        total_duration_secs: f64,
        segments: Vec<SegTiming>,
    }

    struct Clip {
        path: String,  // відносний шлях від save_dir
        duration: f64, // оригінальна тривалість (без урахування overlap переходу)
        is_video: bool,
    }

    fn is_video_ext(path: &str) -> bool {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm")
    }

    // ─── Зчитуємо timeline.json ───────────────────────────────────────────────
    let mut clips: Vec<Clip> = Vec::new();
    let mut total_dur = 0.0f64;
    let timeline_path = save_dir.join("timeline.json");

    if timeline_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&timeline_path) {
            if let Ok(tl) = serde_json::from_str::<Timeline>(&content) {
                total_dur = tl.total_duration_secs;
                for seg in &tl.segments {
                    if let Some(ref media) = seg.media {
                        let dur = (seg.end_secs - seg.start_secs).max(0.05);
                        clips.push(Clip {
                            path: media.clone(),
                            duration: dur,
                            is_video: is_video_ext(media),
                        });
                    }
                }
            }
        }
    }

    // ─── Fallback: рівномірний розподіл якщо timeline порожній ───────────────
    if clips.is_empty() {
        log_fn("timeline.json not found or has no media — using equal distribution");

        let mut files: Vec<String> = std::fs::read_dir(save_dir.join("media"))
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let s = name.to_string_lossy();
                let ext = s.rsplit('.').next().unwrap_or("").to_lowercase();
                matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" |
                         "mp4" | "mov" | "avi" | "mkv" | "webm")
            })
            .map(|e| format!("media/{}", e.file_name().to_string_lossy()))
            .collect();
        files.sort();

        if files.is_empty() {
            return Err("No media files found in media/ folder".to_string());
        }

        total_dur = audio_duration_hint.unwrap_or(0.0);
        if total_dur <= 0.0 {
            return Err("Cannot assemble: audio duration unknown and timeline.json missing".to_string());
        }

        let clip_dur = total_dur / files.len() as f64;
        for f in files {
            let is_vid = is_video_ext(&f);
            clips.push(Clip { path: f, duration: clip_dur, is_video: is_vid });
        }
    }

    if clips.is_empty() {
        return Err("No clips to assemble".to_string());
    }

    if total_dur <= 0.0 {
        total_dur = audio_duration_hint.unwrap_or(0.0);
    }
    if total_dur <= 0.0 {
        return Err("Cannot assemble: total duration is zero".to_string());
    }

    // ─── Знаходимо аудіо-файл ────────────────────────────────────────────────
    let audio_path = find_voice_file(save_dir)
        .ok_or_else(|| "Audio file not found (voice.wav / voice.mp3)".to_string())?;
    let audio_rel = audio_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "voice.mp3".to_string());

    // ─── Параметри переходу ───────────────────────────────────────────────────
    let n = clips.len();
    // Обмежуємо тривалість переходу до половини найкоротшого кліпу
    let min_clip_dur = clips.iter().map(|c| c.duration).fold(f64::INFINITY, f64::min);
    let t = if transition == "none" || n < 2 {
        0.0f64
    } else {
        (transition_duration_secs as f64).clamp(0.05, min_clip_dur * 0.5)
    };
    let use_xfade = t > 0.0;

    // ─── Будуємо фільтр-граф ─────────────────────────────────────────────────
    let mut filter_parts: Vec<String> = Vec::new();

    for (i, clip) in clips.iter().enumerate() {
        // При xfade кожен кліп (крім останнього) потрібно подовжити на t,
        // щоб зберегти синхронізацію: cumulative_dur[k] = start_secs[k+1].
        let adj_dur = if use_xfade && i < n - 1 {
            clip.duration + t
        } else {
            clip.duration
        };
        let adj_dur = adj_dur.max(0.05);
        let frames = (adj_dur * fps as f64).round().max(1.0) as u64;

        if clip.is_video {
            filter_parts.push(format!(
                "[{i}:v]trim=duration={adj_dur:.6},setpts=PTS-STARTPTS,\
                scale=1920:1080:force_original_aspect_ratio=increase,\
                crop=1920:1080,format=yuv420p,setsar=1,fps={fps},settb=AVTB[v{i}_final]"
            ));
        } else {
            filter_parts.push(format!(
                "[{i}:v]scale=1920:1080:force_original_aspect_ratio=increase,\
                crop=1920:1080,format=yuv420p,setsar=1[v{i}_up]"
            ));
            filter_parts.push(format!(
                "[v{i}_up]zoompan=z='1':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':\
                d={frames}:s=1920x1080:fps={fps},format=yuv420p,setsar=1,settb=AVTB,\
                trim=duration={adj_dur:.6},setpts=PTS-STARTPTS[v{i}_final]"
            ));
        }
    }

    if use_xfade {
        // ─── Ланцюг xfade-переходів ───────────────────────────────────────────
        // offset для k-го xfade = sum(orig_dur[0..=k]) = start_secs[k+1]
        // Завдяки цьому cumulative відео-позиція кожного кліпу збігається
        // з його start_secs із timeline.
        let mut cumulative_offset = 0.0f64;
        let mut prev_label = "v0_final".to_string();

        for k in 0..n - 1 {
            cumulative_offset += clips[k].duration; // = start_secs[k+1]
            let trans_name = pick_transition(transition);
            let out_label = if k == n - 2 {
                "v_montage_raw".to_string()
            } else {
                format!("vchain{}", k + 1)
            };
            filter_parts.push(format!(
                "[{prev}][v{next}_final]xfade=transition={trans}:\
                duration={t:.6}:offset={offset:.6}[{out}]",
                prev = prev_label,
                next = k + 1,
                trans = trans_name,
                t = t,
                offset = cumulative_offset,
                out = out_label,
            ));
            prev_label = out_label;
        }
    } else {
        // ─── Concat усіх кліпів (без переходів) ──────────────────────────────
        let concat_inputs: String = (0..n).map(|i| format!("[v{i}_final]")).collect();
        filter_parts.push(format!("{concat_inputs}concat=n={n}:v=1:a=0[v_montage_raw]"));
    }

    filter_parts.push("[v_montage_raw]tpad=stop_mode=clone:stop=-1[v_padded]".to_string());
    filter_parts.push(format!(
        "[v_padded]trim=duration={total_dur:.6},setpts=PTS-STARTPTS[v_montage]"
    ));

    // Якщо burn-in субтитрів увімкнено і файл subtitle.srt існує — вбудовуємо в відео
    let srt_path = save_dir.join("subtitle.srt");
    let video_map = if burn_subtitles && srt_path.exists() {
        // subtitles фільтр потребує libass у збірці FFmpeg
        // Використовуємо відносний шлях (current_dir = save_dir)
        filter_parts.push("[v_montage]subtitles=subtitle.srt[v_with_subs]".to_string());
        log_fn("Subtitles burn-in enabled: subtitle.srt will be embedded into video.");
        "[v_with_subs]"
    } else {
        if burn_subtitles && !srt_path.exists() {
            log_fn("Warning: subtitles_enabled=true but subtitle.srt not found — skipping burn-in.");
        }
        "[v_montage]"
    };

    let script = filter_parts.join(";");
    std::fs::write(save_dir.join("montage_script.txt"), &script)
        .map_err(|e| format!("Failed to write montage_script.txt: {e}"))?;
    log_fn(&format!("Filter graph: {} parts, {} chars", filter_parts.len(), script.len()));

    // ─── Ім'я вихідного файлу ─────────────────────────────────────────────────
    let safe_name: String = task_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let output_file = format!("{}.mp4", safe_name.trim());

    // ─── FFmpeg аргументи ─────────────────────────────────────────────────────
    let ffmpeg = crate::bundle::ffmpeg_path();
    let audio_idx = n;
    let bitr = format!("{bitrate_mbps}M");
    let bufsize = format!("{}M", bitrate_mbps * 2);

    let mut args: Vec<String> = vec![
        "-y".into(), "-hide_banner".into(),
        "-loglevel".into(), "error".into(),
        "-progress".into(), "pipe:1".into(),
    ];

    for clip in &clips {
        args.extend(["-i".into(), clip.path.clone()]);
    }
    args.extend(["-i".into(), audio_rel]);

    args.extend([
        "-filter_complex_script".into(), "montage_script.txt".into(),
        "-map".into(), video_map.to_string(),
        "-map".into(), format!("{audio_idx}:a"),
        "-c:v".into(), "libx264".into(),
        "-preset".into(), preset.to_string(),
        "-b:v".into(), bitr.clone(),
        "-maxrate".into(), bitr,
        "-bufsize".into(), bufsize,
        "-pix_fmt".into(), "yuv420p".into(),
        "-r".into(), fps.to_string(),
        "-t".into(), format!("{total_dur:.3}"),
        "-c:a".into(), "aac".into(),
        "-b:a".into(), "192k".into(),
        "-movflags".into(), "+faststart".into(),
        output_file,
    ]);

    log_fn(&format!("ffmpeg {}", args.join(" ")));

    use std::io::{BufRead, BufReader, Read};
    use std::process::Stdio;

    let mut child = std::process::Command::new(&ffmpeg)
        .args(&args)
        .current_dir(save_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("FFmpeg launch error: {e}"))?;

    // Читаємо stderr у окремому потоці, щоб не заблокувати буфер
    let stderr_handle = {
        let stderr = child.stderr.take().unwrap();
        std::thread::spawn(move || {
            let mut s = String::new();
            BufReader::new(stderr).read_to_string(&mut s).ok();
            s
        })
    };

    // Парсимо прогрес з stdout (-progress pipe:1 format).
    // out_time_us — мікросекунди (незважаючи на назву out_time_ms, FFmpeg пише мікросекунди).
    let stdout = child.stdout.take().unwrap();
    let mut out_time_us: i64 = 0;
    let mut enc_fps = String::new();
    let mut speed = String::new();
    let mut bitrate = String::new();

    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if let Some(v) = line.strip_prefix("out_time_us=") {
            out_time_us = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("fps=") {
            enc_fps = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("speed=") {
            speed = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("bitrate=") {
            bitrate = v.trim().to_string();
        } else if line == "progress=continue" {
            // progress=end пропускаємо: значення там скинуті (N/A), on_progress(1.0) після циклу
            let pct = (out_time_us as f64 / 1_000_000.0 / total_dur).clamp(0.0, 1.0) as f32;
            on_progress(pct);
            log_fn(&format!(
                "{:.0}%  fps={}  speed={}  bitrate={}",
                pct * 100.0, enc_fps, speed, bitrate
            ));
        }
    }

    let status = child.wait().map_err(|e| format!("FFmpeg wait error: {e}"))?;
    let stderr_output = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        let tail = &stderr_output[stderr_output.len().saturating_sub(3000)..];
        return Err(format!("FFmpeg failed:\n{tail}"));
    }

    on_progress(1.0);
    let out_path = format!("{}.mp4", safe_name.trim());
    let file_size = std::fs::metadata(save_dir.join(&out_path))
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(file_size)
}

/// Повертає назву переходу: конкретну або випадкову з доступних xfade.
fn pick_transition(transition: &str) -> &'static str {
    use crate::gui::pipeline::editing::XFADE_TRANSITIONS;
    if transition == "random" {
        // Простий детермінований "рандом" на основі поточного часу
        let idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as usize)
            .unwrap_or(0)
            % XFADE_TRANSITIONS.len();
        XFADE_TRANSITIONS[idx]
    } else {
        // Повертаємо статичний рядок; якщо невідомо — fallback "fade"
        XFADE_TRANSITIONS.iter()
            .copied()
            .find(|&t| t == transition)
            .unwrap_or("fade")
    }
}
