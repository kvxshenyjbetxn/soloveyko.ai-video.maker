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
/// Читає timeline.json для таймінгів, збирає медіафайли з media/,
/// записує montage_script.txt і запускає ffmpeg у папці save_dir
/// (всі шляхи — відносні, щоб не перевищувати ліміт командного рядка).
pub fn run_montage(
    save_dir: &Path,
    task_name: &str,
    audio_duration_hint: Option<f64>,
    fps: u32,
    preset: &str,
    bitrate_mbps: u32,
    log_fn: impl Fn(&str),
) -> Result<(), String> {
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
        duration: f64,
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

    // ─── Будуємо фільтр-граф ─────────────────────────────────────────────────
    let n = clips.len();
    let mut filter_parts: Vec<String> = Vec::new();

    for (i, clip) in clips.iter().enumerate() {
        let dur = clip.duration.max(0.05);
        let frames = (dur * fps as f64).round().max(1.0) as u64;

        if clip.is_video {
            filter_parts.push(format!(
                "[{i}:v]trim=duration={dur:.6},setpts=PTS-STARTPTS,\
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
                trim=duration={dur:.6},setpts=PTS-STARTPTS[v{i}_final]"
            ));
        }
    }

    // Concat усіх кліпів
    let concat_inputs: String = (0..n).map(|i| format!("[v{i}_final]")).collect();
    filter_parts.push(format!("{concat_inputs}concat=n={n}:v=1:a=0[v_montage_raw]"));
    filter_parts.push("[v_montage_raw]tpad=stop_mode=clone:stop=-1[v_padded]".to_string());
    filter_parts.push(format!("[v_padded]trim=duration={total_dur:.6},setpts=PTS-STARTPTS[v_montage]"));

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
        "-y".into(), "-hide_banner".into(), "-loglevel".into(), "info".into(), "-stats".into(),
    ];

    for clip in &clips {
        args.extend(["-i".into(), clip.path.clone()]);
    }
    args.extend(["-i".into(), audio_rel]);

    args.extend([
        "-filter_complex_script".into(), "montage_script.txt".into(),
        "-map".into(), "[v_montage]".into(),
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

    let output = std::process::Command::new(&ffmpeg)
        .args(&args)
        .current_dir(save_dir)
        .output()
        .map_err(|e| format!("FFmpeg launch error: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail = &stderr[stderr.len().saturating_sub(3000)..];
        return Err(format!("FFmpeg failed:\n{tail}"));
    }

    log_fn("Montage complete.");
    Ok(())
}
