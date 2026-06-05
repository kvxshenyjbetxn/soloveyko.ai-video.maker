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
    overlay_triggers_enabled: bool,
    overlay_triggers: &[super::trigger::OverlayTrigger],
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
        path: Option<String>, // None = чорна заставка (gap)
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
                let seg_count = tl.segments.len();
                // null-сегменти → чорна заставка між кліпами
                for seg in &tl.segments {
                    let dur = (seg.end_secs - seg.start_secs).max(0.05);
                    if let Some(ref media) = seg.media {
                        clips.push(Clip { path: Some(media.clone()), duration: dur, is_video: is_video_ext(media) });
                    } else {
                        // Об'єднуємо суміжні null-сегменти в один чорний кліп
                        if matches!(clips.last(), Some(Clip { path: None, .. })) {
                            clips.last_mut().unwrap().duration += dur;
                        } else {
                            clips.push(Clip { path: None, duration: dur, is_video: false });
                        }
                    }
                }
                log_fn(&format!(
                    "timeline.json: {} segments → {} clips, total={:.2}s, first={}",
                    seg_count, clips.len(), total_dur,
                    clips.first().and_then(|c| c.path.as_deref()).unwrap_or("(black gap)"),
                ));
            } else {
                log_fn("timeline.json: PARSE ERROR — invalid JSON format");
            }
        } else {
            log_fn("timeline.json: READ ERROR — cannot open file");
        }
    } else {
        log_fn("timeline.json: NOT FOUND");
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
            clips.push(Clip { path: Some(f), duration: clip_dur, is_video: is_vid });
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
    // Кількість реальних медіа-файлів (black-кліпи не мають input-файлу)
    let media_file_count = clips.iter().filter(|c| c.path.is_some()).count();
    // Обмежуємо тривалість переходу до половини найкоротшого медіа-кліпу
    let min_clip_dur = clips.iter()
        .filter(|c| c.path.is_some())
        .map(|c| c.duration)
        .fold(f64::INFINITY, f64::min);
    let t = if transition == "none" || n < 2 {
        0.0f64
    } else {
        (transition_duration_secs as f64).clamp(0.05, min_clip_dur * 0.5)
    };
    let use_xfade = t > 0.0;

    // ─── Будуємо фільтр-граф ─────────────────────────────────────────────────
    let mut filter_parts: Vec<String> = Vec::new();

    let mut file_idx = 0usize; // input-файл index (тільки для media-кліпів, не для black)
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

        if clip.path.is_none() {
            // Чорна заставка — генерується FFmpeg inline без input-файлу
            filter_parts.push(format!(
                "color=black:s=1920x1080:r={fps}:d={adj_dur:.6},\
                format=yuv420p,setsar=1,settb=AVTB[v{i}_final]"
            ));
        } else if clip.is_video {
            filter_parts.push(format!(
                "[{file_idx}:v]trim=duration={adj_dur:.6},setpts=PTS-STARTPTS,\
                scale=1920:1080:force_original_aspect_ratio=increase,\
                crop=1920:1080,format=yuv420p,setsar=1,fps={fps},settb=AVTB[v{i}_final]"
            ));
            file_idx += 1;
        } else {
            filter_parts.push(format!(
                "[{file_idx}:v]scale=1920:1080:force_original_aspect_ratio=increase,\
                crop=1920:1080,format=yuv420p,setsar=1[v{i}_up]"
            ));
            filter_parts.push(format!(
                "[v{i}_up]zoompan=z='1':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':\
                d={frames}:s=1920x1080:fps={fps},format=yuv420p,setsar=1,settb=AVTB,\
                trim=duration={adj_dur:.6},setpts=PTS-STARTPTS[v{i}_final]"
            ));
            file_idx += 1;
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

    // Якщо burn-in субтитрів увімкнено — шукаємо subtitle.ass, потім subtitle.srt як запасний.
    // FFmpeg запускається з current_dir=save_dir, тому використовуємо відносний шлях —
    // це повністю уникає проблем з пробілами та двокрапками у Windows-шляхах.
    let after_subs_label = if burn_subtitles {
        let ass_path = save_dir.join("subtitle.ass");
        let srt_path = save_dir.join("subtitle.srt");
        if ass_path.exists() {
            filter_parts.push("[v_montage]ass=filename=subtitle.ass[v_with_subs]".to_string());
            log_fn("Subtitles burn-in: subtitle.ass embedded.");
            "v_with_subs".to_string()
        } else if srt_path.exists() {
            filter_parts.push("[v_montage]ass=filename=subtitle.srt[v_with_subs]".to_string());
            log_fn("Subtitles burn-in: subtitle.srt (fallback).");
            "v_with_subs".to_string()
        } else {
            log_fn("Warning: subtitles_enabled=true but no subtitle file found — skipping burn-in.");
            "v_montage".to_string()
        }
    } else {
        "v_montage".to_string()
    };

    // ─── Тригери накладення медіа ─────────────────────────────────────────────
    // Кожен активний тригер стає окремим FFmpeg input і накладається через overlay.
    struct ActiveTrigger {
        input_idx: usize,
        start: f64,
        duration: f64,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        is_video: bool,
    }

    let mut active_triggers: Vec<ActiveTrigger> = Vec::new();

    // Шукаємо файл субтитрів для пошуку тайм-кодів фраз
    let sub_path = {
        let ass = save_dir.join("subtitle.ass");
        let srt = save_dir.join("subtitle.srt");
        if ass.exists() { Some(ass) } else if srt.exists() { Some(srt) } else { None }
    };

    // Список додаткових input-файлів тригерів: (шлях, is_video)
    // is_video=false → додаємо -loop 1 щоб зображення повторювалось протягом тривалості
    let mut trigger_input_paths: Vec<(String, bool)> = Vec::new();

    if overlay_triggers_enabled && !overlay_triggers.is_empty() {
        for tr in overlay_triggers {
            if tr.phrase.is_empty() || tr.path.is_empty() { continue; }
            let tr_path = std::path::Path::new(&tr.path);
            if !tr_path.exists() {
                log_fn(&format!("Trigger path not found, skipping: {}", tr.path));
                continue;
            }

            // Визначаємо час початку: явний або пошук по субтитрах
            let start = if let Some(t) = tr.start_time {
                t
            } else if let Some(ref sp) = sub_path {
                match super::trigger::find_text_timing(sp, &tr.phrase) {
                    Some(t) => {
                        log_fn(&format!("Trigger '{}' found at {:.3}s", tr.phrase, t));
                        t
                    }
                    None => {
                        log_fn(&format!("Trigger phrase not found: '{}'", tr.phrase));
                        continue;
                    }
                }
            } else {
                log_fn(&format!("No subtitle file for trigger '{}', skipping", tr.phrase));
                continue;
            };

            let is_video = is_video_ext(&tr.path);

            // Тривалість: явна, або з ffprobe для відео, або 3.0 за замовчуванням
            let duration = if let Some(d) = tr.duration {
                d
            } else if is_video {
                // Намагаємося отримати тривалість через ffprobe
                let ffprobe = crate::bundle::ffprobe_path();
                let out = std::process::Command::new(&ffprobe)
                    .args([
                        "-v", "error", "-show_entries", "format=duration",
                        "-of", "default=noprint_wrappers=1:nokey=1",
                        &tr.path,
                    ])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(3.0);
                out
            } else {
                3.0
            };

            // Індекс цього input у FFmpeg (clips + 1 аудіо + попередні тригери)
            let input_idx = media_file_count + 1 + trigger_input_paths.len();
            trigger_input_paths.push((tr.path.clone(), is_video));

            active_triggers.push(ActiveTrigger {
                input_idx,
                start,
                duration,
                x: tr.x,
                y: tr.y,
                w: if tr.w > 0 { tr.w } else { 1920 },
                h: if tr.h > 0 { tr.h } else { 1080 },
                is_video,
            });
        }
        log_fn(&format!("Active triggers: {}", active_triggers.len()));
    }

    // Будуємо фільтри для тригерів поверх поточного відео-потоку
    let video_map_label = if active_triggers.is_empty() {
        format!("[{after_subs_label}]")
    } else {
        let mut current = after_subs_label.clone();
        for (i, tr) in active_triggers.iter().enumerate() {
            let w = (tr.w / 2) * 2;
            let h = (tr.h / 2) * 2;
            let ready_label = format!("v_trig_ready_{i}");
            let out_label = format!("v_trig_out_{i}");
            let enable_expr = format!("between(t,{:.3},{:.3})", tr.start, tr.start + tr.duration);

            if tr.is_video {
                filter_parts.push(format!(
                    "[{}:v]format=yuva420p,scale={w}:{h}:force_original_aspect_ratio=increase,\
                    crop={w}:{h},setpts=PTS-STARTPTS+{:.3}/TB[{ready_label}]",
                    tr.input_idx, tr.start, ready_label = ready_label, w = w, h = h,
                ));
                filter_parts.push(format!(
                    "[{current}][{ready_label}]overlay=x={x}:y={y}:eof_action=pass:enable='{enable_expr}'[{out_label}]",
                    current = current, ready_label = ready_label, x = tr.x, y = tr.y,
                    enable_expr = enable_expr, out_label = out_label,
                ));
            } else {
                filter_parts.push(format!(
                    "[{}:v]format=yuva420p,scale={w}:{h}:force_original_aspect_ratio=increase,\
                    crop={w}:{h},setpts=PTS-STARTPTS+{:.3}/TB[{ready_label}]",
                    tr.input_idx, tr.start, ready_label = ready_label, w = w, h = h,
                ));
                filter_parts.push(format!(
                    "[{current}][{ready_label}]overlay=x={x}:y={y}:enable='{enable_expr}'[{out_label}]",
                    current = current, ready_label = ready_label, x = tr.x, y = tr.y,
                    enable_expr = enable_expr, out_label = out_label,
                ));
            }
            current = out_label;
        }
        format!("[{current}]")
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
    let audio_idx = media_file_count;
    let bitr = format!("{bitrate_mbps}M");
    let bufsize = format!("{}M", bitrate_mbps * 2);

    let mut args: Vec<String> = vec![
        "-y".into(), "-hide_banner".into(),
        "-loglevel".into(), "error".into(),
        "-progress".into(), "pipe:1".into(),
    ];

    for clip in &clips {
        if let Some(ref path) = clip.path {
            args.extend(["-i".into(), path.clone()]);
        }
    }
    args.extend(["-i".into(), audio_rel]);

    // Додаємо input-файли тригерів: -loop 1 для зображень (щоб frame повторювався)
    for (tp, is_vid) in &trigger_input_paths {
        if !is_vid {
            args.push("-loop".into());
            args.push("1".into());
        }
        args.extend(["-i".into(), tp.clone()]);
    }

    args.extend([
        "-filter_complex_script".into(), "montage_script.txt".into(),
        "-map".into(), video_map_label.clone(),
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
