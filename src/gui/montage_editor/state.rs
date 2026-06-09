use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;
use super::types::{
    ClipKind, ClipDragState, EditorClip, PreviewDragState,
    MontagePreviewSettings, FRAME_CACHE_SIZE,
};
use super::media::MediaItem;
use super::frame_cache::FrameCache;
use super::audio::PlayingAudio;
use super::utils::{probe_duration, uuid_str};

// ─── Стан редактора ───────────────────────────────────────────────────────────

pub struct MontageEditorState {
    pub job_name: String,
    pub save_path: PathBuf,
    pub media_pool: Vec<MediaItem>,
    pub clips: Vec<EditorClip>,
    pub num_tracks: usize,
    pub audio_path: Option<PathBuf>,
    pub audio_start_secs: f32,
    pub audio_duration: f32,
    pub total_duration: f32,
    pub playhead: f32,
    pub is_playing: bool,
    pub last_frame_time: Instant,
    pub timeline_zoom: f32,
    pub selected_clip_id: Option<String>,
    pub dragged_media_id: Option<String>,
    pub drop_target_track: Option<usize>,
    pub clip_drag_state: Option<ClipDragState>,
    /// LRU кеш текстур — заповнюється з попередньо витягнутих JPG файлів
    pub frame_cache: FrameCache,
    /// Налаштування ефектів превью (синхронізуються з JobSettings кожного кадру)
    pub preview_settings: MontagePreviewSettings,
    /// Висота панелі таймлайну (змінюється drag-handle)
    pub timeline_height: f32,
    /// Активні аудіо плеєри
    pub active_audios: Vec<PlayingAudio>,
    /// Стан інтерактивного drag-трансформу на превью
    pub preview_drag: Option<PreviewDragState>,
    /// Виділені медіа в пулі (для групової анімації)
    pub selected_media_ids: HashSet<String>,
    /// Тимчасові шляхи для оживлення (заповнюються draw_*, обробляються в draw_montage_editor_window)
    pub pending_animate_paths: Vec<PathBuf>,
    /// Тимчасова дія перегенерації (заповнюється draw_*, обробляється в draw_montage_editor_window)
    pub pending_regen: Option<(PathBuf, bool)>,
    /// Шлях медіа що зараз відкрите у fullscreen preview
    pub pool_preview: Option<PathBuf>,
    /// Кешована текстура для fullscreen preview (шлях, текстура)
    pub pool_preview_texture: Option<(PathBuf, eframe::egui::TextureHandle)>,
    /// Чи максимізовано вікно редактора на весь екран програми
    pub maximized: bool,
}

impl MontageEditorState {
    pub fn load(save_path: &Path, job_name: &str) -> Self {
        let (mut clips, total_duration, audio_start_secs) = load_timeline_clips(save_path);
        let audio_path = find_audio_file(save_path);
        let audio_duration = if let Some(ref ap) = audio_path {
            probe_duration(ap).unwrap_or(0.0)
        } else {
            0.0
        };
        let mut media_pool = load_media_pool(save_path);

        // Додаємо у пул файли з таймлінії, яких ще немає (захист від розбіжності шляхів)
        for clip in &clips {
            if let Some(ref path) = clip.path {
                if path.exists() && !media_pool.iter().any(|m| m.path == *path) {
                    media_pool.push(MediaItem::new(path.clone(), save_path));
                }
            }
        }

        // Виправляємо кліпи що вказують на .jpg якого вже немає (оживлення → .mp4 є в пулі)
        for clip in &mut clips {
            if let Some(ref path) = clip.path.clone() {
                if !path.exists() {
                    let mp4 = path.with_extension("mp4");
                    if let Some(pm) = media_pool.iter().find(|m| m.path == mp4) {
                        clip.path = Some(pm.path.clone());
                        clip.kind = ClipKind::Video;
                        clip.name = pm.name.clone();
                    }
                }
            }
        }

        // Синхронізуємо media_id кліпів з реальними UUID пулу
        // (UUID змінюються при кожному перезавантаженні, тому ID з JSON вже застарілі)
        for clip in &mut clips {
            if let Some(ref path) = clip.path.clone() {
                if let Some(m) = media_pool.iter().find(|m| m.path == *path) {
                    clip.media_id = m.id.clone();
                }
            }
        }

        let num_tracks = clips.iter().map(|c| c.track_idx + 1).max().unwrap_or(1).max(2);

        Self {
            job_name: job_name.to_string(),
            save_path: save_path.to_path_buf(),
            media_pool,
            clips,
            num_tracks,
            audio_path,
            audio_start_secs,
            audio_duration,
            total_duration: total_duration.max(10.0),
            playhead: 0.0,
            is_playing: false,
            last_frame_time: Instant::now(),
            timeline_zoom: 60.0,
            selected_clip_id: None,
            dragged_media_id: None,
            drop_target_track: None,
            clip_drag_state: None,
            frame_cache: FrameCache::new(FRAME_CACHE_SIZE),
            preview_settings: MontagePreviewSettings::default(),
            timeline_height: 220.0,
            active_audios: Vec::new(),
            preview_drag: None,
            selected_media_ids: HashSet::new(),
            pending_animate_paths: vec![],
            pending_regen: None,
            pool_preview: None,
            pool_preview_texture: None,
            maximized: false,
        }
    }

    pub fn total_dur(&self) -> f32 {
        let clip_end = self.clips.iter().map(|c| c.end_secs()).fold(0.0f32, f32::max);
        clip_end.max(self.total_duration).max(10.0)
    }

    /// Конвертує абсолютний шлях до медіафайлу у відносний (від save_path).
    fn path_to_rel(&self, p: &Path) -> Option<String> {
        if let Ok(rel) = p.strip_prefix(&self.save_path) {
            return Some(rel.to_string_lossy().replace('\\', "/"));
        }
        let canon_clip = std::fs::canonicalize(p).ok();
        let canon_save = std::fs::canonicalize(&self.save_path).ok();
        if let (Some(cc), Some(cs)) = (canon_clip, canon_save) {
            cc.strip_prefix(&cs).ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        } else {
            None
        }
    }

    /// Зберігає поточний стан у timeline.json для run_montage.
    /// Доріжка 0 → "segments" (з null-gap заглушками для чорного екрану).
    /// Доріжки 1+ → "overlay_tracks" (з трансформ-даними scale/pos_x/pos_y).
    pub fn save_to_timeline(&self) -> Result<(), std::io::Error> {
        let total_duration_secs = self.clips.iter()
            .map(|c| c.end_secs() as f64)
            .fold(0.0f64, f64::max)
            .max(self.total_dur() as f64);

        // ── Доріжка 0: основна послідовність ────────────────────────────────
        let mut sorted0: Vec<&EditorClip> = self.clips.iter()
            .filter(|c| c.track_idx == 0 && c.path.is_some())
            .collect();
        sorted0.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));

        let mut main_segments: Vec<serde_json::Value> = Vec::new();
        let mut cursor = 0.0f32;
        for clip in &sorted0 {
            let actual_start = clip.start_secs.max(cursor);
            if actual_start > cursor + 0.01 {
                main_segments.push(serde_json::json!({
                    "start_secs": cursor as f64,
                    "end_secs": actual_start as f64,
                    "media": serde_json::Value::Null,
                }));
            }
            let media_rel = clip.path.as_ref().and_then(|p| self.path_to_rel(p));
            let actual_end = actual_start + clip.duration;
            main_segments.push(serde_json::json!({
                "start_secs": actual_start as f64,
                "end_secs": actual_end as f64,
                "media": media_rel,
                "media_id": clip.media_id,
                "zoom_enabled": clip.zoom_enabled,
                "shake_enabled": clip.shake_enabled,
            }));
            cursor = actual_end;
        }

        // ── Overlay-доріжки (1+) ─────────────────────────────────────────────
        let max_track = self.clips.iter().map(|c| c.track_idx).max().unwrap_or(0);
        let mut overlay_tracks: Vec<serde_json::Value> = Vec::new();
        for t in 1..=max_track {
            let mut segs: Vec<&EditorClip> = self.clips.iter()
                .filter(|c| c.track_idx == t && c.path.is_some())
                .collect();
            if segs.is_empty() { continue; }
            segs.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));
            let segments: Vec<serde_json::Value> = segs.iter().map(|clip| {
                let media_rel = clip.path.as_ref().and_then(|p| self.path_to_rel(p));
                serde_json::json!({
                    "start_secs": clip.start_secs as f64,
                    "end_secs": clip.end_secs() as f64,
                    "media": media_rel,
                    "media_id": clip.media_id,
                    "scale": clip.scale as f64,
                    "pos_x": clip.pos_x as f64,
                    "pos_y": clip.pos_y as f64,
                    "zoom_enabled": clip.zoom_enabled,
                    "shake_enabled": clip.shake_enabled,
                })
            }).collect();
            overlay_tracks.push(serde_json::json!({
                "track_idx": t,
                "segments": segments,
            }));
        }

        let json = serde_json::json!({
            "total_duration_secs": total_duration_secs,
            "audio_start_secs": self.audio_start_secs as f64,
            "segments": main_segments,
            "overlay_tracks": overlay_tracks,
        });

        let timeline_path = self.save_path.join("timeline.json");
        let content = serde_json::to_string_pretty(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(timeline_path, content)
    }
}

// ─── Завантаження даних ───────────────────────────────────────────────────────

/// Будує EditorClip з JSON-сегменту (спільна логіка для track 0 та overlay).
fn clip_from_json_seg(
    seg: &serde_json::Value,
    media_str: &str,
    save_path: &Path,
    track_idx: usize,
) -> EditorClip {
    let full_path: PathBuf = save_path.join(media_str).components().collect();
    let name = full_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(media_str)
        .to_string();
    let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let kind = if matches!(ext.as_str(), "mp4" | "mov" | "webm") {
        ClipKind::Video
    } else if matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac") {
        ClipKind::Audio
    } else {
        ClipKind::Image
    };
    let start = seg["start_secs"].as_f64().unwrap_or(0.0) as f32;
    let end = seg["end_secs"].as_f64().unwrap_or(0.0) as f32;
    let scale = seg["scale"].as_f64().unwrap_or(1.0) as f32;
    let pos_x = seg["pos_x"].as_f64().unwrap_or(0.0) as f32;
    let pos_y = seg["pos_y"].as_f64().unwrap_or(0.0) as f32;
    // Якщо поле відсутнє (старий формат) — вважаємо ефекти увімкненими для сумісності
    let zoom_enabled = seg["zoom_enabled"].as_bool().unwrap_or(true);
    let shake_enabled = seg["shake_enabled"].as_bool().unwrap_or(true);
    let media_id = seg["media_id"].as_str().unwrap_or("").to_string();
    EditorClip {
        id: uuid_str(),
        media_id,
        path: Some(full_path),
        name,
        start_secs: start,
        duration: (end - start).max(0.1),
        track_idx,
        kind,
        scale,
        pos_x,
        pos_y,
        zoom_enabled,
        shake_enabled,
    }
}

fn load_timeline_clips(save_path: &Path) -> (Vec<EditorClip>, f32, f32) {
    let path = save_path.join("timeline.json");
    if !path.exists() { return (Vec::new(), 10.0, 0.0); }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (Vec::new(), 10.0, 0.0),
    };
    let total = v["total_duration_secs"].as_f64().unwrap_or(10.0) as f32;
    let audio_start = v["audio_start_secs"].as_f64().unwrap_or(0.0) as f32;
    let mut clips = Vec::new();

    // Доріжка 0 з "segments"
    if let Some(segs) = v["segments"].as_array() {
        // Збираємо медіафайли для відновлення null-media сегментів
        // (агент міг випадково прибрати шляхи при редагуванні таймінгів)
        let media_dir = save_path.join("media");
        let recovery_files: Vec<String> = if media_dir.exists() {
            let mut files: Vec<String> = std::fs::read_dir(&media_dir)
                .ok().into_iter().flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name();
                    let s = name.to_string_lossy();
                    let ext = s.rsplit('.').next().unwrap_or("").to_lowercase();
                    matches!(ext.as_str(), "jpg"|"jpeg"|"png"|"webp"|"gif"|"mp4"|"mov"|"webm")
                })
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            files.sort();
            files
        } else {
            Vec::new()
        };
        let n_segs = segs.len();
        let n_files = recovery_files.len();

        for (i, seg) in segs.iter().enumerate() {
            let media_str = seg["media"].as_str().map(|s| s.to_string())
                .or_else(|| {
                    // Відновлення лише для pipeline-формату (є поле "text"), не для редакторських gap-сегментів
                    if seg["text"].as_str().is_none() || n_files == 0 { return None; }
                    let file_idx = if n_files <= n_segs {
                        (i as f64 * n_files as f64 / n_segs as f64).floor() as usize
                    } else {
                        i.min(n_files - 1)
                    };
                    recovery_files.get(file_idx).map(|f| format!("media/{}", f))
                });

            if let Some(media) = media_str {
                clips.push(clip_from_json_seg(seg, &media, save_path, 0));
            }
        }
    }

    // Overlay-доріжки з "overlay_tracks"
    if let Some(tracks) = v["overlay_tracks"].as_array() {
        for track in tracks {
            let track_idx = track["track_idx"].as_u64().unwrap_or(1) as usize;
            if let Some(segs) = track["segments"].as_array() {
                for seg in segs {
                    if let Some(media) = seg["media"].as_str() {
                        clips.push(clip_from_json_seg(seg, media, save_path, track_idx));
                    }
                }
            }
        }
    }

    (clips, total, audio_start)
}

fn find_audio_file(save_path: &Path) -> Option<PathBuf> {
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_path.join(name);
        if p.exists() { return Some(p); }
    }
    None
}

fn load_media_pool(save_path: &Path) -> Vec<MediaItem> {
    let media_dir = save_path.join("media");
    let mut items = Vec::new();
    if media_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&media_dir) {
            for entry in entries.filter_map(Result::ok) {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if matches!(ext.as_str(), "mp4" | "mov" | "webm" | "jpg" | "jpeg" | "png" | "webp" | "mp3" | "wav") {
                        items.push(MediaItem::new(p, save_path));
                    }
                }
            }
        }
    }
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_path.join(name);
        if p.exists() { items.push(MediaItem::new(p, save_path)); }
    }
    items
}
