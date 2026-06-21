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
    image_zoom_enabled: bool,
    _image_zoom_intensity: f32,
    image_zoom_mode: &str,
    image_zoom_scale: f32,
    image_shake_enabled: bool,
    image_shake_intensity: f32,
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
        #[serde(default)]
        trim_start: f64,
        #[serde(default = "default_fade")]
        overlap_transition: String,
    }
    fn default_fade() -> String { "fade".to_string() }

    #[derive(serde::Deserialize)]
    struct OverlaySeg {
        start_secs: f64,
        end_secs: f64,
        media: Option<String>,
        #[serde(default)]
        trim_start: f64,
        #[serde(default = "default_scale")]
        scale: f64,
        #[serde(default)]
        pos_x: f64,
        #[serde(default)]
        pos_y: f64,
        #[serde(default = "default_opacity")]
        #[allow(dead_code)]
        opacity: f64,
        /// true = вбудоване аудіо відеофайлу; треба використовувати як аудіо-вхід
        #[serde(default)]
        is_embedded_audio: bool,
    }
    fn default_scale() -> f64 { 1.0 }
    fn default_opacity() -> f64 { 1.0 }

    #[derive(serde::Deserialize)]
    struct OverlayTrack {
        #[allow(dead_code)]
        track_idx: usize,
        segments: Vec<OverlaySeg>,
    }

    #[derive(serde::Deserialize)]
    struct Timeline {
        total_duration_secs: f64,
        #[serde(default)]
        audio_start_secs: f64,
        #[serde(default = "default_volume")]
        voiceover_volume: f64,
        #[serde(default)]
        track_volumes: Vec<f64>,
        segments: Vec<SegTiming>,
        #[serde(default)]
        overlay_tracks: Vec<OverlayTrack>,
    }
    fn default_volume() -> f64 { 1.0 }

    struct Clip {
        path: Option<String>, // None = чорна заставка (gap)
        duration: f64,
        /// Абсолютний час початку кліпу на таймлінії (секунди)
        start_secs: f64,
        is_video: bool,
        /// Початок обрізки у вихідному файлі (секунди); 0.0 = з початку
        trim_start: f64,
        /// Тип xfade для переходу після цього кліпу ("fade", "wipeleft", …)
        overlap_transition: String,
    }

    fn is_video_ext(path: &str) -> bool {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm")
    }

    fn is_audio_ext(path: &str) -> bool {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac")
    }

    struct AudioClip {
        path: String,
        start_secs: f64,
        duration: f64,
        trim_start: f64,
        /// Індекс доріжки (для пошуку гучності в track_volumes)
        track_idx: usize,
    }

    // ─── Зчитуємо timeline.json ───────────────────────────────────────────────
    let mut clips: Vec<Clip> = Vec::new();
    let mut total_dur = 0.0f64;
    let mut audio_start_secs = 0.0f64;
    let mut voiceover_volume = 1.0f64;
    let mut tl_track_volumes: Vec<f64> = Vec::new();
    let mut overlay_tracks: Vec<OverlayTrack> = Vec::new();
    let mut extra_audios: Vec<AudioClip> = Vec::new();
    let timeline_path = save_dir.join("timeline.json");

    if timeline_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&timeline_path) {
            if let Ok(tl) = serde_json::from_str::<Timeline>(&content) {
                total_dur = tl.total_duration_secs;
                audio_start_secs = tl.audio_start_secs;
                voiceover_volume = tl.voiceover_volume;
                tl_track_volumes = tl.track_volumes;

                let mut video_overlay_tracks = Vec::new();
                for track in tl.overlay_tracks {
                    let mut video_segs = Vec::new();
                    let track_ti = track.track_idx;
                    for seg in track.segments {
                        let dur = (seg.end_secs - seg.start_secs).max(0.05);
                        if let Some(ref media) = seg.media {
                            if seg.is_embedded_audio || is_audio_ext(media) {
                                // Вбудоване аудіо відео або звичайний аудіо-файл
                                extra_audios.push(AudioClip {
                                    path: media.clone(),
                                    start_secs: seg.start_secs,
                                    duration: dur,
                                    trim_start: seg.trim_start,
                                    track_idx: track_ti,
                                });
                            } else {
                                video_segs.push(seg);
                            }
                        }
                    }
                    if !video_segs.is_empty() {
                        video_overlay_tracks.push(OverlayTrack {
                            track_idx: track_ti,
                            segments: video_segs,
                        });
                    }
                }
                overlay_tracks = video_overlay_tracks;

                let seg_count = tl.segments.len();
                for seg in &tl.segments {
                    let dur = (seg.end_secs - seg.start_secs).max(0.05);
                    if let Some(ref media) = seg.media {
                        if is_audio_ext(media) {
                            extra_audios.push(AudioClip {
                                path: media.clone(),
                                start_secs: seg.start_secs,
                                duration: dur,
                                trim_start: seg.trim_start,
                                track_idx: 0,
                            });
                        } else {
                            clips.push(Clip { path: Some(media.clone()), duration: dur, start_secs: seg.start_secs, is_video: is_video_ext(media), trim_start: seg.trim_start, overlap_transition: seg.overlap_transition.clone() });
                        }
                    } else {
                        if matches!(clips.last(), Some(Clip { path: None, .. })) {
                            clips.last_mut().unwrap().duration += dur;
                        } else {
                            clips.push(Clip { path: None, duration: dur, start_secs: seg.start_secs, is_video: false, trim_start: 0.0, overlap_transition: "fade".to_string() });
                        }
                    }
                }
                log_fn(&format!(
                    "timeline.json: {} segments → {} clips, {} overlay tracks, {} extra audios, total={:.2}s",
                    seg_count, clips.len(), overlay_tracks.len(), extra_audios.len(), total_dur,
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
        for (idx, f) in files.iter().enumerate() {
            let is_vid = is_video_ext(f);
            clips.push(Clip { path: Some(f.clone()), duration: clip_dur, start_secs: idx as f64 * clip_dur, is_video: is_vid, trim_start: 0.0, overlap_transition: "fade".to_string() });
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

    // Будуємо per-pair інформацію про переходи:
    // Для кожної пари (i → i+1): чи є overlap, яка тривалість, який тип переходу
    struct PairTransition {
        /// Тривалість xfade в секундах (0 = без переходу)
        duration: f64,
        /// Назва переходу для xfade=transition=...
        name: String,
        /// true = кліпи накладаються на таймлінії; false = глобальний перехід між послідовними кліпами
        is_overlap: bool,
    }
    let mut pairs: Vec<PairTransition> = Vec::new();
    for k in 0..n.saturating_sub(1) {
        let a = &clips[k];
        let b = &clips[k + 1];
        let a_end = a.start_secs + a.duration;
        let b_start = b.start_secs;
        let b_end = b.start_secs + b.duration;

        // Чи є накладання?
        let overlap_dur = if a_end > b_start + 0.001 {
            (a_end.min(b_end) - b_start).max(0.0)
        } else {
            0.0
        };

        if overlap_dur > 0.0 {
            // Overlap → використовуємо per-clip налаштування з редактора
            let max_ov = (a.duration.min(b.duration) * 0.5).max(0.05);
            let ov = overlap_dur.clamp(0.05, max_ov);
            pairs.push(PairTransition {
                duration: ov,
                name: b.overlap_transition.clone(),
                is_overlap: true,
            });
        } else if transition != "none" {
            // Нема накладання, але глобальний перехід увімкнено
            let min_dur = a.duration.min(b.duration);
            let max_t = (min_dur * 0.5).max(0.05);
            let t = (transition_duration_secs as f64).clamp(0.05, max_t);
            pairs.push(PairTransition {
                duration: t,
                name: transition.to_string(),
                is_overlap: false,
            });
        } else {
            // Без переходу
            pairs.push(PairTransition { duration: 0.0, name: String::new(), is_overlap: false });
        }
    }

    // ─── Будуємо фільтр-граф ─────────────────────────────────────────────────
    let mut filter_parts: Vec<String> = Vec::new();
    // adj_dur[i] — скоригована тривалість кліпу для фільтру:
    // для глобальних переходів (не overlap) кліп розтягується на тривалість переходу,
    // щоб мати кадри для xfade. Для overlap-переходів розтягнення непотрібне —
    // кліпи вже перекриваються на таймлінії.
    let mut adj_durs: Vec<f64> = Vec::with_capacity(n);

    let mut file_idx = 0usize; // input-файл index (тільки для media-кліпів, не для black)
    let mut img_idx = 0usize;  // лічильник зображень для режиму "alternate"
    for (i, clip) in clips.iter().enumerate() {
        let ext = if i < pairs.len() && pairs[i].duration > 0.001 && !pairs[i].is_overlap {
            // Глобальний перехід: подовжуємо кліп щоб xfade мав кадри для плавного переходу
            pairs[i].duration
        } else {
            0.0
        };
        let adj_dur = (clip.duration + ext).max(0.05);
        adj_durs.push(adj_dur);
        let frames = (adj_dur * fps as f64).round().max(1.0) as u64;

        if clip.path.is_none() {
            // Чорна заставка — генерується FFmpeg inline без input-файлу
            filter_parts.push(format!(
                "color=black:s=1920x1080:r={fps}:d={adj_dur:.6},\
                format=yuv420p,setsar=1,settb=AVTB[v{i}_final]"
            ));
        } else if clip.is_video {
            let ts = clip.trim_start;
            filter_parts.push(format!(
                "[{file_idx}:v]trim=start={ts:.6}:duration={adj_dur:.6},setpts=PTS-STARTPTS,\
                scale=1920:1080:force_original_aspect_ratio=increase,\
                crop=1920:1080,format=yuv420p,setsar=1,fps={fps},settb=AVTB[v{i}_final]"
            ));
            file_idx += 1;
        } else {
            // Зображення — застосовуємо ефекти зуму та покачування
            let img_parts = build_image_filter_parts(
                i, file_idx, frames, adj_dur, fps,
                image_zoom_enabled, image_zoom_scale, image_zoom_mode, img_idx,
                image_shake_enabled, image_shake_intensity,
            );
            filter_parts.extend(img_parts);
            file_idx += 1;
            img_idx += 1;
        }
    }

    // ─── Послідовний ланцюг xfade-переходів + concat ───────────────────────────
    // Будуємо ланцюг зліва направо: кожен новий кліп або приєднується через xfade
    // до поточного ланцюга, або починає новий сегмент (якщо переходу немає).
    // chain_dur відстежує поточну тривалість виходу ланцюга для правильного offset.
    // offset = chain_dur - pair.duration (єдина формула для обох типів переходів):
    //   - overlap: adj_dur = original (без розтягнення), offset = original - overlap
    //   - global: adj_dur = original + trans (розтягнення), offset = (original+trans) - trans = original
    let mut result_labels: Vec<String> = Vec::new();
    let mut chain_label: Option<String> = None;
    let mut chain_dur = 0.0f64;

    for i in 0..n {
        let clip_label = format!("v{i}_final");
        match chain_label.take() {
            None => {
                chain_label = Some(clip_label);
                chain_dur = adj_durs[i];
            }
            Some(prev_label) => {
                let pair = &pairs[i - 1];
                if pair.duration > 0.001 {
                    let new_label = format!("v_merge_{i}");
                    let offset = (chain_dur - pair.duration).max(0.0);
                    let trans_name = if pair.name == "random" {
                        pick_transition("random")
                    } else {
                        pick_transition(&pair.name)
                    };
                    filter_parts.push(format!(
                        "[{prev_label}][{clip_label}]xfade=transition={trans}:\
                        duration={dur:.6}:offset={offset:.6}[{new_label}]",
                        trans = trans_name,
                        dur = pair.duration,
                    ));
                    chain_dur = offset + adj_durs[i];
                    chain_label = Some(new_label);
                } else {
                    result_labels.push(prev_label);
                    chain_label = Some(clip_label);
                    chain_dur = adj_durs[i];
                }
            }
        }
    }
    if let Some(label) = chain_label {
        result_labels.push(label);
    }

    if result_labels.len() == 1 {
        filter_parts.push(format!("[{}]null[v_montage_raw]", result_labels[0]));
    } else {
        let inputs: String = result_labels.iter()
            .map(|l| format!("[{l}]"))
            .collect();
        let count = result_labels.len();
        filter_parts.push(format!("{inputs}concat=n={count}:v=1:a=0[v_montage_raw]"));
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
                let mut ffprobe_proc = std::process::Command::new(&ffprobe);
                ffprobe_proc.args([
                    "-v", "error", "-show_entries", "format=duration",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    &tr.path,
                ]);
                crate::bundle::set_no_window(&mut ffprobe_proc);
                let out = ffprobe_proc.output()
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

    // ─── Збираємо overlay-кліпи з доріжок 1+ ────────────────────────────────
    struct OverlayItem {
        input_idx: usize,
        start: f64,
        end: f64,
        trim_start: f64,
        w: i32,
        h: i32,
        x: i32,
        y: i32,
        is_video: bool,
    }

    let mut overlay_items: Vec<OverlayItem> = Vec::new();
    let mut overlay_input_paths: Vec<(String, bool)> = Vec::new();

    // Вищий трек (менший track_idx) = візуально вище = рендериться ОСТАННІМ (поверх усіх).
    // Тому сортуємо: спочатку більший track_idx (нижній), потім менший (верхній).
    overlay_tracks.sort_by(|a, b| b.track_idx.cmp(&a.track_idx));

    for track in &overlay_tracks {
        for seg in &track.segments {
            let media_path_str = match &seg.media {
                Some(m) if !m.is_empty() => m.clone(),
                _ => continue,
            };
            let media_abs = save_dir.join(&media_path_str);
            if !media_abs.exists() {
                log_fn(&format!("Overlay media not found, skipping: {media_path_str}"));
                continue;
            }
            let is_vid = is_video_ext(&media_path_str);

            // Розмір у пікселях з нормалізованого масштабу (округлюємо до парного)
            let raw_w = (1920.0 * seg.scale).round() as u32;
            let raw_h = (1080.0 * seg.scale).round() as u32;
            let w = (raw_w.max(2) + 1) / 2 * 2;
            let h = (raw_h.max(2) + 1) / 2 * 2;

            // Позиція центру в пікселях; pos_x/pos_y ∈ [-1..1] = від лівого/верхнього краю до правого/нижнього
            let cx = 960.0 * (1.0 + seg.pos_x);
            let cy = 540.0 * (1.0 + seg.pos_y);
            let x = (cx - w as f64 / 2.0).round() as i32;
            let y = (cy - h as f64 / 2.0).round() as i32;

            let input_idx = media_file_count + 1 + trigger_input_paths.len() + overlay_input_paths.len();
            overlay_input_paths.push((media_path_str.clone(), is_vid));

            log_fn(&format!("Overlay: {media_path_str} [{w}x{h} @ ({x},{y})] t={:.2}s-{:.2}s",
                seg.start_secs, seg.end_secs));

            overlay_items.push(OverlayItem { input_idx, start: seg.start_secs, end: seg.end_secs, trim_start: seg.trim_start, w: w as i32, h: h as i32, x, y, is_video: is_vid });
        }
    }

    // Будуємо фільтри для тригерів та overlay-кліпів поверх поточного відео-потоку
    let video_map_label = {
        let mut current = after_subs_label.clone();

        // ── Тригери (як раніше) ───────────────────────────────────────────
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
                    tr.input_idx, tr.start,
                ));
                filter_parts.push(format!(
                    "[{current}][{ready_label}]overlay=x={x}:y={y}:eof_action=pass:enable='{enable_expr}'[{out_label}]",
                    x = tr.x, y = tr.y,
                ));
            } else {
                filter_parts.push(format!(
                    "[{}:v]format=yuva420p,scale={w}:{h}:force_original_aspect_ratio=increase,\
                    crop={w}:{h},setpts=PTS-STARTPTS+{:.3}/TB[{ready_label}]",
                    tr.input_idx, tr.start,
                ));
                filter_parts.push(format!(
                    "[{current}][{ready_label}]overlay=x={x}:y={y}:enable='{enable_expr}'[{out_label}]",
                    x = tr.x, y = tr.y,
                ));
            }
            current = out_label;
        }

        // ── Overlay-доріжки (track 1+) ────────────────────────────────────
        // Overlay-треки накладаються ПОВЕРХ V1 (current).
        // Порядок: вищий track_idx — перший (нижній шар), менший — останній (верхній шар).
        // overlay_items вже відсортовані: спочатку більший track_idx.
        //
        // Для відео: setpts=PTS-STARTPTS+{start}/TB зміщує PTS щоб overlay з'явився
        // у правильний момент на таймлінії. FFmpeg overlay синхронізує по PTS,
        // тому до ov.start V1 відображається без overlay, потім overlay з'являється.
        // Для зображень: enable='between(t,start,end)' з -loop 1 робить те саме.
        if !overlay_items.is_empty() {
            for (i, ov) in overlay_items.iter().enumerate() {
                let (w, h) = (ov.w, ov.h);
                let out_label = format!("v_ol_{i}");
                let prep = format!("v_ol_{i}_prep");

                if ov.is_video {
                    let ov_dur = ov.end - ov.start;
                    let enable_expr = format!("between(t,{:.6},{:.6})", ov.start, ov.end);
                    // settb до setpts — нормалізуємо timebase перед обчисленням /TB
                    filter_parts.push(format!(
                        "[{}:v]trim=start={trim:.6}:duration={ov_dur:.6},\
                        setpts=PTS-STARTPTS,settb=AVTB,setpts=PTS+{start:.6}/TB,\
                        scale={w}:{h}:force_original_aspect_ratio=decrease,\
                        pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,format=yuv420p,fps={fps},setsar=1[{prep}]",
                        ov.input_idx, trim = ov.trim_start, start = ov.start,
                    ));
                    filter_parts.push(format!(
                        "[{current}][{prep}]overlay=x={x}:y={y}:enable='{enable_expr}':eof_action=pass[{out_label}]",
                        x = ov.x, y = ov.y,
                    ));
                } else {
                    // Зображення (-loop 1): enable= обмежує видимість часовим діапазоном
                    let enable_expr = format!("between(t,{:.3},{:.3})", ov.start, ov.end);
                    filter_parts.push(format!(
                        "[{}:v]scale={w}:{h}:force_original_aspect_ratio=decrease,\
                        pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,format=yuv420p,setsar=1[{prep}]",
                        ov.input_idx,
                    ));
                    filter_parts.push(format!(
                        "[{current}][{prep}]overlay=x={x}:y={y}:enable='{enable_expr}':eof_action=pass[{out_label}]",
                        x = ov.x, y = ov.y,
                    ));
                }

                current = out_label;
            }
        }

        format!("[{current}]")
    };

    let audio_idx = media_file_count;
    let extra_audio_start_idx = media_file_count + 1 + trigger_input_paths.len() + overlay_input_paths.len();

    // Допоміжна функція: рядок фільтру гучності якщо відрізняється від 1.0
    let vol_filter = |vol: f64| -> String {
        if (vol - 1.0).abs() > 0.001 {
            format!(",volume={:.4}", vol.max(0.0))
        } else {
            String::new()
        }
    };

    let audio_map_label = if extra_audios.is_empty() {
        let vf = vol_filter(voiceover_volume);
        if audio_start_secs > 0.001 {
            let ms = (audio_start_secs * 1000.0).round() as i64;
            filter_parts.push(format!(
                "[{audio_idx}:a]{vf_stripped}adelay={ms}|{ms}[a_delayed]",
                vf_stripped = if vf.is_empty() { String::new() } else { format!("{}," , &vf[1..]) },
            ));
            "[a_delayed]".to_string()
        } else if vf.is_empty() {
            format!("{audio_idx}:a")
        } else {
            filter_parts.push(format!("[{audio_idx}:a]{}[a_vo_vol]", &vf[1..]));
            "[a_vo_vol]".to_string()
        }
    } else {
        let vf = vol_filter(voiceover_volume);
        if audio_start_secs > 0.001 {
            let ms = (audio_start_secs * 1000.0).round() as i64;
            filter_parts.push(format!(
                "[{audio_idx}:a]{vf_stripped}adelay={ms}|{ms}[a_orig]",
                vf_stripped = if vf.is_empty() { String::new() } else { format!("{},", &vf[1..]) },
            ));
        } else if vf.is_empty() {
            filter_parts.push(format!("[{audio_idx}:a]anull[a_orig]"));
        } else {
            filter_parts.push(format!("[{audio_idx}:a]{}[a_orig]", &vf[1..]));
        }

        for (i, ea) in extra_audios.iter().enumerate() {
            let input_idx = extra_audio_start_idx + i;
            let delay_ms = (ea.start_secs * 1000.0).round() as i64;
            let track_vol = tl_track_volumes.get(ea.track_idx).copied().unwrap_or(1.0);
            let vf = vol_filter(track_vol);
            filter_parts.push(format!(
                "[{input_idx}:a]atrim=start={trim:.6}:end={end:.6},asetpts=PTS-STARTPTS{vf},adelay={delay_ms}|{delay_ms}[a_extra_{i}]",
                trim = ea.trim_start,
                end = ea.trim_start + ea.duration,
            ));
        }

        let mut mix_inputs = "[a_orig]".to_string();
        for i in 0..extra_audios.len() {
            mix_inputs.push_str(&format!("[a_extra_{i}]"));
        }
        let mix_count = 1 + extra_audios.len();
        filter_parts.push(format!(
            "{mix_inputs}amix=inputs={mix_count}:duration=longest:dropout_transition=0[a_mixed]"
        ));
        "[a_mixed]".to_string()
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

    // Тригери: -loop 1 для зображень
    for (tp, is_vid) in &trigger_input_paths {
        if !is_vid {
            args.push("-loop".into());
            args.push("1".into());
        }
        args.extend(["-i".into(), tp.clone()]);
    }

    // Overlay-кліпи (доріжки 1+): -loop 1 для зображень
    for (op, is_vid) in &overlay_input_paths {
        if !is_vid {
            args.push("-loop".into());
            args.push("1".into());
        }
        args.extend(["-i".into(), op.clone()]);
    }

    // Додаткові аудіокліпи
    for ea in &extra_audios {
        args.extend(["-i".into(), ea.path.clone()]);
    }

    args.extend([
        "-filter_complex_script".into(), "montage_script.txt".into(),
        "-map".into(), video_map_label.clone(),
        "-map".into(), audio_map_label,
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

    let mut ffmpeg_proc = std::process::Command::new(&ffmpeg);
    ffmpeg_proc.args(&args)
        .current_dir(save_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::bundle::set_no_window(&mut ffmpeg_proc);
    let mut child = ffmpeg_proc.spawn()
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


/// Будує ланцюжок FFmpeg-фільтрів для зображення з ефектами зуму та/або покачування.
///
/// Zoom:
///   - "alternate": парні зображення збільшуються (1.0→scale), непарні — зменшуються (scale→1.0)
///   - "oscillate": одне зображення зумується туди-сюди (split → zoom-in half + zoom-out half → concat)
///   Крок розраховується автоматично з кількості кадрів, щоб заповнити весь кліп.
/// Shake реалізований через crop з sin(t) — crop підтримує змінну `t` (час у секундах),
/// на відміну від zoompan де `t` та `n` недоступні.
/// При shake зображення попередньо масштабується більшим на величину амплітуди.
fn build_image_filter_parts(
    i: usize,
    file_idx: usize,
    frames: u64,
    adj_dur: f64,
    fps: u32,
    zoom_enabled: bool,
    zoom_scale: f32,
    zoom_mode: &str,
    img_idx: usize,
    shake_enabled: bool,
    shake_intensity: f32,
) -> Vec<String> {
    let mut parts = Vec::new();

    // ─── Розмір canvas (більший при shake, стандартний інакше) ───
    let (canvas_w, canvas_h, amp_f) = if shake_enabled {
        let a_px = (40.0 * shake_intensity) as u32;
        let a_f = 40.0 * shake_intensity;
        (1920 + 2 * a_px, 1080 + 2 * a_px, a_f)
    } else {
        (1920u32, 1080u32, 0.0f32)
    };

    // ─── Scale → canvas ───────────────────────────────────────────
    parts.push(format!(
        "[{file_idx}:v]scale={canvas_w}:{canvas_h}:force_original_aspect_ratio=increase,\
        crop={canvas_w}:{canvas_h},format=yuv420p,setsar=1[v{i}_up]"
    ));

    // ─── Zoom (zoompan) ───────────────────────────────────────────
    // Важливо: zoompan затискає zoom до [1..10], тому zoom=0 на першому кадрі
    // автоматично стає 1.0. Для zoom-out початкове значення задається через eq(zoom,0).
    // Змінна `zoom` (а не `pzoom`) дає актуальне значення поточного кадру.
    if zoom_enabled && zoom_mode == "oscillate" {
        // Режим "туди-сюди": косинусоїдна осциляція — плавно наближується до max_z
        // і плавно повертається до 1.0 за один цикл протягом усього кліпу.
        // Формула: z = 1.0 + zAmp*(1-cos(2π*t/duration))/2, де t = on/fps.
        // `on` — номер вихідного кадру (доступна змінна zoompan).
        let z_amp = zoom_scale - 1.0;
        let z_expr = format!(
            "1.0+{z_amp:.4}*(1-cos(6.2832*(on/{fps}/{adj_dur:.6})))/2"
        );
        parts.push(format!(
            "[v{i}_up]zoompan=z='{z_expr}':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':\
            d={frames}:s={canvas_w}x{canvas_h}:fps={fps},\
            format=yuv420p,setsar=1,settb=AVTB[v{i}_zoomed]"
        ));
    } else if zoom_enabled {
        // Режим "чергування" (alternate): парні збільшуються, непарні зменшуються.
        // Використовуємо `on` (номер вихідного кадру) — лінійний зум без хаків ініціалізації.
        let max_z = zoom_scale;
        let z_amp = max_z - 1.0;
        let z_expr = if img_idx % 2 == 0 {
            // Zoom in: від 1.0 до max_z лінійно
            format!("min(1.0+{z_amp:.4}*on/{fps}/{adj_dur:.6},{max_z:.4})")
        } else {
            // Zoom out: від max_z до 1.0 лінійно
            format!("max({max_z:.4}-{z_amp:.4}*on/{fps}/{adj_dur:.6},1.0)")
        };
        parts.push(format!(
            "[v{i}_up]zoompan=z='{z_expr}':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':\
            d={frames}:s={canvas_w}x{canvas_h}:fps={fps},\
            format=yuv420p,setsar=1,settb=AVTB[v{i}_zoomed]"
        ));
    } else {
        // Без зуму — статичний zoompan z=1 для контролю fps/тривалості
        parts.push(format!(
            "[v{i}_up]zoompan=z='1':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':\
            d={frames}:s={canvas_w}x{canvas_h}:fps={fps},\
            format=yuv420p,setsar=1,settb=AVTB[v{i}_zoomed]"
        ));
    }

    // ─── Shake (crop) або фінальне обрізання до 1920x1080 ────────
    if shake_enabled {
        // crop підтримує змінну t (час у секундах) — ідеально для sin-хвиль
        // Центр: x=amp, y=amp; зміщення ±amp через sin із різними частотами
        parts.push(format!(
            "[v{i}_zoomed]crop=1920:1080:\
            {amp_f:.1}+{amp_f:.1}*sin(PI*0.7*t):\
            {amp_f:.1}+{amp_f:.1}*sin(PI*0.53*t),\
            trim=duration={adj_dur:.6},setpts=PTS-STARTPTS[v{i}_final]"
        ));
    } else {
        parts.push(format!(
            "[v{i}_zoomed]trim=duration={adj_dur:.6},setpts=PTS-STARTPTS[v{i}_final]"
        ));
    }

    parts
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
