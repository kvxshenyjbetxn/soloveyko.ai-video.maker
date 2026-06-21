use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use super::types::{
    ClipKind, ClipDragState, EditorClip, MontagePreviewSettings, OpacityDragState,
    PreviewDragState, PreviewQuality, PreviewRenderSettings, TimelineSnapshot, TrackDragState, TrackKind, FRAME_CACHE_SIZE,
};
use super::media::MediaItem;
use super::frame_cache::FrameCache;
use super::audio::PlayingAudio;
use super::utils::{frame_cache_dir, probe_duration, sharp_frame_cache_dir, uuid_str};

// ─── Стан редактора ───────────────────────────────────────────────────────────

pub struct MontageEditorState {
    pub job_name: String,
    pub save_path: PathBuf,
    pub media_pool: Vec<MediaItem>,
    pub clips: Vec<EditorClip>,
    pub num_tracks: usize,
    /// Тип кожної доріжки (Video / Audio) у порядку додавання
    pub track_kinds: Vec<TrackKind>,
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
    /// Налаштування якості/FPS тільки для попереднього перегляду редактора.
    /// Не впливає на фінальний FFmpeg/CapCut рендер.
    pub preview_render: PreviewRenderSettings,
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
    /// Шлях, для якого треба перезавантажити preview-текстуру через кадр
    /// (дає GPU час звільнити стару текстуру перед завантаженням нової)
    pub preview_stale_path: Option<PathBuf>,
    /// Спільний стан для випадаючого списку вибору overlap-переходу (clip_id, позиція на екрані)
    pub overlap_transition_popup: Option<(String, eframe::egui::Pos2)>,
    /// Чи максимізовано вікно редактора на весь екран програми
    pub maximized: bool,
    /// Запит на відкриття Stock Picker для сегмента з вказаним індексом
    pub pending_open_stock_picker: Option<usize>,
    /// Прапорець: оновити плейсхолдери з stock_cache.json (встановлюється після підтвердження вибору стоку)
    pub needs_stock_refresh: bool,
    /// Блокує drag на превью (коли поверх відкрито stock picker або інше вікно)
    pub input_blocked: bool,
    /// Нові preview_render, якщо користувач змінив якість/FPS у топбарі
    pub pending_preview_render: Option<PreviewRenderSettings>,
    /// Стан drag смужки прозорості кліпу
    pub opacity_drag: Option<OpacityDragState>,
    /// Кеш мініатюр для медіа-пулу та таймлайну (media_id → текстура першого кадру)
    pub pool_thumbnails: HashMap<String, eframe::egui::TextureHandle>,
    /// Гучність кожної доріжки (індекс = track_idx); 1.0 = норма, 0.0 = тиша
    pub track_volumes: Vec<f32>,
    /// Чи активний інструмент розрізу (як лезо в професійних редакторах)
    pub split_tool_active: bool,
    /// Снапнута позиція курсора в режимі розрізу (секунди). None = без снапу.
    pub split_snap_secs: Option<f32>,
    /// Стан перетягування доріжки для зміни її порядку
    pub track_drag: Option<TrackDragState>,
    /// Поточне горизонтальне зміщення скролу таймлінії (для авто-прокрутки за плейхедом)
    pub timeline_scroll_x: f32,
    /// true поки користувач тягне плейхед (натиснув на лінійці і ще не відпустив)
    pub playhead_dragging: bool,
    /// Стек для скасування дій (Ctrl+Z)
    pub undo_stack: Vec<TimelineSnapshot>,
    /// Стек для повтору дій (Ctrl+Y)
    pub redo_stack: Vec<TimelineSnapshot>,
}

impl MontageEditorState {
    pub fn load(save_path: &Path, job_name: &str, preview_render: PreviewRenderSettings) -> Self {
        let (mut clips, total_duration, audio_start_secs) = load_timeline_clips(save_path);
        let audio_path = find_audio_file(save_path);
        let audio_duration = if let Some(ref ap) = audio_path {
            probe_duration(ap).unwrap_or(0.0)
        } else {
            0.0
        };
        let mut media_pool = load_media_pool(save_path, preview_render);

        // Створюємо візуальний аудіо-кліп для голосової доріжки (без зміни audio_path для рендеру)
        let mut vo_clip_on_track = None;
        if let Some(ref ap) = audio_path {
            let media_id = if let Some(m) = media_pool.iter().find(|m| m.path == *ap) {
                m.id.clone()
            } else {
                let m = MediaItem::new(ap.clone(), save_path, preview_render);
                let id = m.id.clone();
                media_pool.push(m);
                id
            };
            let vo_name = ap.file_name().and_then(|n| n.to_str()).unwrap_or("voice").to_string();
            // Знаходимо існуючу аудіо-доріжку або створюємо нову
            let has_audio_track = clips.iter().any(|c| matches!(c.kind, ClipKind::Audio));
            let num_tracks_now = clips.iter().map(|c| c.track_idx + 1).max().unwrap_or(1).max(2);
            let audio_track_idx = if has_audio_track {
                clips.iter().filter(|c| matches!(c.kind, ClipKind::Audio))
                    .map(|c| c.track_idx).min().unwrap_or(num_tracks_now)
            } else {
                num_tracks_now
            };
            vo_clip_on_track = Some(EditorClip {
                id: uuid_str(),
                media_id,
                path: Some(ap.clone()),
                name: format!("♪ {}", vo_name),
                start_secs: audio_start_secs,
                duration: audio_duration,
                track_idx: audio_track_idx,
                kind: ClipKind::Audio,
                scale: 1.0, pos_x: 0.0, pos_y: 0.0,
                zoom_enabled: false, shake_enabled: false,
                is_placeholder: false, trim_start: 0.0,
                stock_seg_idx: None,
                overlap_transition: "fade".to_string(),
                opacity: 1.0,
                pair_id: None, audio_linked: false,
                is_embedded_audio: false,
            });
        }

        // Додаємо у пул файли з таймлінії, яких ще немає (захист від розбіжності шляхів)
        for clip in &clips {
            if let Some(ref path) = clip.path {
                if path.exists() && !media_pool.iter().any(|m| m.path == *path) {
                    media_pool.push(MediaItem::new(path.clone(), save_path, preview_render));
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

        // Додаємо візуальний кліп голосової доріжки ДО обчислення num_tracks
        if let Some(vo_clip) = vo_clip_on_track {
            clips.push(vo_clip);
        }

        let num_tracks = clips.iter().map(|c| c.track_idx + 1).max().unwrap_or(1).max(2);
        // За замовчуванням всі доріжки — відео (V1, V2, ...), але ті де є аудіо-кліпи — аудіо
        let mut track_kinds: Vec<TrackKind> = (0..num_tracks).map(|_| TrackKind::Video).collect();
        for clip in &clips {
            if matches!(clip.kind, ClipKind::Audio) {
                if clip.track_idx < track_kinds.len() {
                    track_kinds[clip.track_idx] = TrackKind::Audio;
                }
            }
        }
        let mut track_volumes = load_track_volumes(save_path).unwrap_or_default();
        // Доповнюємо до поточної кількості доріжок
        while track_volumes.len() < num_tracks { track_volumes.push(1.0); }

        // Запускаємо витягування WAV для всіх вбудованих аудіо-кліпів без кешу
        for clip in &clips {
            if clip.is_embedded_audio {
                if let Some(ref path) = clip.path {
                    super::audio::extract_embedded_audio_async(path.clone(), save_path.to_path_buf());
                }
            }
        }

        Self {
            job_name: job_name.to_string(),
            save_path: save_path.to_path_buf(),
            media_pool,
            clips,
            num_tracks,
            track_kinds,
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
            preview_render,
            timeline_height: 220.0,
            active_audios: Vec::new(),
            preview_drag: None,
            selected_media_ids: HashSet::new(),
            pending_animate_paths: vec![],
            pending_regen: None,
            pool_preview: None,
            pool_preview_texture: None,
            preview_stale_path: None,
            overlap_transition_popup: None,
            maximized: false,
            pending_open_stock_picker: None,
            needs_stock_refresh: false,
            input_blocked: false,
            pending_preview_render: None,
            opacity_drag: None,
            pool_thumbnails: HashMap::new(),
            track_volumes,
            split_tool_active: false,
            split_snap_secs: None,
            track_drag: None,
            timeline_scroll_x: 0.0,
            playhead_dragging: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Зберігає поточний стан у стек для скасування (Ctrl+Z).
    /// Скидає redo_stack — нова дія скасовує можливість "повтору".
    pub fn push_undo(&mut self) {
        self.undo_stack.push(TimelineSnapshot {
            clips: self.clips.clone(),
            num_tracks: self.num_tracks,
            track_kinds: self.track_kinds.clone(),
            track_volumes: self.track_volumes.clone(),
        });
        self.redo_stack.clear();
        // Обмежуємо глибину стеку (50 кроків)
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// Скасовує останню дію (Ctrl+Z).
    pub fn undo(&mut self) {
        let Some(snapshot) = self.undo_stack.pop() else { return };
        let current = TimelineSnapshot {
            clips: self.clips.clone(),
            num_tracks: self.num_tracks,
            track_kinds: self.track_kinds.clone(),
            track_volumes: self.track_volumes.clone(),
        };
        self.redo_stack.push(current);
        self.apply_snapshot(snapshot);
    }

    /// Повторює скасовану дію (Ctrl+Y).
    pub fn redo(&mut self) {
        let Some(snapshot) = self.redo_stack.pop() else { return };
        let current = TimelineSnapshot {
            clips: self.clips.clone(),
            num_tracks: self.num_tracks,
            track_kinds: self.track_kinds.clone(),
            track_volumes: self.track_volumes.clone(),
        };
        self.undo_stack.push(current);
        self.apply_snapshot(snapshot);
    }

    fn apply_snapshot(&mut self, snapshot: TimelineSnapshot) {
        self.clips = snapshot.clips;
        self.num_tracks = snapshot.num_tracks;
        self.track_kinds = snapshot.track_kinds;
        self.track_volumes = snapshot.track_volumes;
        self.selected_clip_id = None;
        self.clip_drag_state = None;
        self.opacity_drag = None;
        self.save_to_timeline().ok();
    }

    pub fn set_preview_render(&mut self, quality: PreviewQuality, fps: f32) {
        let next = PreviewRenderSettings { quality, fps };
        if self.preview_render == next {
            return;
        }

        self.preview_render = next;
        self.pending_preview_render = Some(next);
        self.frame_cache = FrameCache::new(FRAME_CACHE_SIZE);
        for media in &mut self.media_pool {
            let old_id = media.id.clone();
            let path = media.path.clone();
            *media = MediaItem::new(path, &self.save_path, next);
            media.id = old_id;
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
        // Виключаємо візуальний кліп голосової доріжки (він обробляється окремо)
        let mut sorted0: Vec<&EditorClip> = self.clips.iter()
            .filter(|c| c.track_idx == 0 && c.path.is_some()
                // Не включаємо візуальний голосовий кліп
                && !(matches!(c.kind, ClipKind::Audio) && !c.is_embedded_audio && c.pair_id.is_none()))
            .collect();
        sorted0.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));

        let mut main_segments: Vec<serde_json::Value> = Vec::new();
        let mut cursor = 0.0f32;
        for clip in &sorted0 {
            // Зберігаємо оригінальний start_secs щоб overlap-зони не губились
            let seg_start = clip.start_secs;
            if seg_start > cursor + 0.01 {
                // Проміжок (gap) перед цим кліпом
                main_segments.push(serde_json::json!({
                    "start_secs": cursor as f64,
                    "end_secs": seg_start as f64,
                    "media": serde_json::Value::Null,
                }));
            }
            let media_rel = clip.path.as_ref().map(|p| {
                self.path_to_rel(p)
                    .unwrap_or_else(|| p.to_string_lossy().replace('\\', "/"))
            });
            let seg_end = seg_start + clip.duration;
            main_segments.push(serde_json::json!({
                "start_secs": seg_start as f64,
                "end_secs": seg_end as f64,
                "media": media_rel,
                "media_id": clip.media_id,
                "clip_kind": match clip.kind { ClipKind::Video => "video", ClipKind::Audio => "audio", ClipKind::Image => "image" },
                "zoom_enabled": clip.zoom_enabled,
                "shake_enabled": clip.shake_enabled,
                "trim_start": clip.trim_start as f64,
                "stock_seg_idx": clip.stock_seg_idx,
                "overlap_transition": clip.overlap_transition,
                "opacity": clip.opacity as f64,
                "pair_id": clip.pair_id,
                "audio_linked": clip.audio_linked,
                "is_embedded_audio": clip.is_embedded_audio,
            }));
            cursor = cursor.max(seg_end);
        }

        // ── Overlay-доріжки (1+) ─────────────────────────────────────────────
        let max_track = self.clips.iter().map(|c| c.track_idx).max().unwrap_or(0);
        let mut overlay_tracks: Vec<serde_json::Value> = Vec::new();
        for t in 1..=max_track {
            let mut segs: Vec<&EditorClip> = self.clips.iter()
                .filter(|c| c.track_idx == t && c.path.is_some()
                    && !(matches!(c.kind, ClipKind::Audio) && !c.is_embedded_audio && c.pair_id.is_none()))
                .collect();
            if segs.is_empty() { continue; }
            segs.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));
            let segments: Vec<serde_json::Value> = segs.iter().map(|clip| {
                // Якщо файл поза папкою проєкту — зберігаємо абсолютний шлях,
                // щоб montage.rs міг знайти медіа при рендері.
                let media_rel = clip.path.as_ref().map(|p| {
                    self.path_to_rel(p)
                        .unwrap_or_else(|| p.to_string_lossy().replace('\\', "/"))
                });
                serde_json::json!({
                    "start_secs": clip.start_secs as f64,
                    "end_secs": clip.end_secs() as f64,
                    "media": media_rel,
                    "media_id": clip.media_id,
                    "clip_kind": match clip.kind { ClipKind::Video => "video", ClipKind::Audio => "audio", ClipKind::Image => "image" },
                    "trim_start": clip.trim_start as f64,
                    "scale": clip.scale as f64,
                    "pos_x": clip.pos_x as f64,
                    "pos_y": clip.pos_y as f64,
                    "zoom_enabled": clip.zoom_enabled,
                    "shake_enabled": clip.shake_enabled,
                    "stock_seg_idx": clip.stock_seg_idx,
                    "overlap_transition": clip.overlap_transition,
                    "opacity": clip.opacity as f64,
                    "pair_id": clip.pair_id,
                    "audio_linked": clip.audio_linked,
                    "is_embedded_audio": clip.is_embedded_audio,
                })
            }).collect();
            overlay_tracks.push(serde_json::json!({
                "track_idx": t,
                "segments": segments,
            }));
        }

        let track_vols_json: Vec<f64> = self.track_volumes.iter().map(|&v| v as f64).collect();
        // Синхронізуємо audio_start_secs з позиції візуального кліпу
        let vo_start = self.clips.iter()
            .find(|c| matches!(c.kind, ClipKind::Audio) && !c.is_embedded_audio && c.pair_id.is_none())
            .map(|c| c.start_secs as f64)
            .unwrap_or(0.0);
        let json = serde_json::json!({
            "total_duration_secs": total_duration_secs,
            "audio_start_secs": vo_start as f64,
            "voiceover_volume": 1.0_f64,
            "track_volumes": track_vols_json,
            "segments": main_segments,
            "overlay_tracks": overlay_tracks,
        });

        let timeline_path = self.save_path.join("timeline.json");
        let content = serde_json::to_string_pretty(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(timeline_path, content)
    }

    /// Зберігає тільки гучності у timeline.json — не чіпає segments/overlay_tracks.
    /// Використовується для повзунка гучності щоб не перетирати дані агента.
    pub fn save_volumes_only(&self) -> Result<(), std::io::Error> {
        let timeline_path = self.save_path.join("timeline.json");
        let content = std::fs::read_to_string(&timeline_path).unwrap_or_else(|_| "{}".to_string());
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| serde_json::json!({}));

        let track_vols_json: Vec<f64> = self.track_volumes.iter().map(|&v| v as f64).collect();
        json["voiceover_volume"] = serde_json::json!(1.0_f64);
        json["track_volumes"] = serde_json::json!(track_vols_json);

        let updated = serde_json::to_string_pretty(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(timeline_path, updated)
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
    // Зберігаємо kind явно в JSON щоб відрізнити вбудоване аудіо (.mp4) від відеокліпу
    let kind = match seg["clip_kind"].as_str().unwrap_or("") {
        "audio" => ClipKind::Audio,
        "image" => ClipKind::Image,
        "video" => ClipKind::Video,
        _ => if matches!(ext.as_str(), "mp4" | "mov" | "webm") {
            ClipKind::Video
        } else if matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac") {
            ClipKind::Audio
        } else {
            ClipKind::Image
        }
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
        is_placeholder: false,
        trim_start: seg["trim_start"].as_f64().unwrap_or(0.0) as f32,
        stock_seg_idx: seg["stock_seg_idx"].as_u64().map(|v| v as usize),
        overlap_transition: seg["overlap_transition"].as_str().unwrap_or("fade").to_string(),
        opacity: seg["opacity"].as_f64().unwrap_or(1.0) as f32,
        pair_id: seg["pair_id"].as_str().map(|s| s.to_string()),
        audio_linked: seg["audio_linked"].as_bool().unwrap_or(true),
        is_embedded_audio: seg["is_embedded_audio"].as_bool().unwrap_or(false),
    }
}

fn load_track_volumes(save_path: &Path) -> Option<Vec<f32>> {
    let path = save_path.join("timeline.json");
    if !path.exists() { return None; }
    let content = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v["track_volumes"].as_array().map(|arr| {
        arr.iter().map(|x| x.as_f64().unwrap_or(1.0) as f32).collect()
    })
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
        // stock_cache.json для відновлення медіа в режимі стоків (пріоритет над індексним маппінгом)
        let stock_cache = crate::api::stock::load_cache(save_path);

        // Збираємо медіафайли для fallback-відновлення null-media сегментів
        // (використовується тільки коли немає stock_cache і немає медіа в сегменті)
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
                    if seg["text"].as_str().is_none() { return None; }

                    // Режим стоків: якщо stock_cache.json існує — він є єдиним джерелом медіа.
                    // Сегменти без вибраного стоку залишаються плейсхолдерами (не беремо перший з пулу).
                    if let Some(ref cache) = stock_cache {
                        if let Some(entry) = cache.get(i) {
                            if let Some(sel) = &entry.selected {
                                let fp = format!("media/{}", sel.filename);
                                if save_path.join(&fp).exists() {
                                    return Some(fp);
                                }
                            }
                        }
                        return None; // stocks-режим, але медіа ще не вибрано — плейсхолдер
                    }

                    // Fallback (не stocks-режим): пропорційний індексний маппінг (для 0001.jpg…)
                    if n_files == 0 { return None; }
                    let file_idx = if n_files <= n_segs {
                        (i as f64 * n_files as f64 / n_segs as f64).floor() as usize
                    } else {
                        i.min(n_files - 1)
                    };
                    recovery_files.get(file_idx).map(|f| format!("media/{}", f))
                });

            if let Some(media) = media_str {
                let mut clip = clip_from_json_seg(seg, &media, save_path, 0);
                // Pipeline-формат (є "text") не містить stock_seg_idx у JSON — відновлюємо з індексу сегменту
                if clip.stock_seg_idx.is_none() && seg["text"].as_str().is_some() {
                    clip.stock_seg_idx = Some(i);
                }
                clips.push(clip);
            } else if seg["text"].as_str().is_some() {
                // Сегмент без медіа (media: null) — плейсхолдер для Stock Picker
                let text = seg["text"].as_str().unwrap_or("").to_string();
                let start = seg["start_secs"].as_f64().unwrap_or(0.0) as f32;
                let end   = seg["end_secs"].as_f64().unwrap_or(0.0) as f32;
                clips.push(EditorClip {
                    id: uuid_str(),
                    media_id: format!("placeholder_{}", i),
                    path: None,
                    name: text.chars().take(24).collect::<String>(),
                    start_secs: start,
                    duration: (end - start).max(0.5),
                    track_idx: 0,
                    kind: ClipKind::Image,
                    scale: 1.0, pos_x: 0.0, pos_y: 0.0,
                    zoom_enabled: false, shake_enabled: false,
                    is_placeholder: true, trim_start: 0.0,
                    stock_seg_idx: Some(i),
                    overlap_transition: "fade".to_string(),
                    opacity: 1.0,
                    pair_id: None,
                    audio_linked: false,
                    is_embedded_audio: false,
                });
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

/// Після підтвердження вибору стоку — замінює плейсхолдери реальними кліпами.
/// Повертає true якщо є ще незавантажені файли (треба повторити наступного кадру).
pub fn refresh_placeholder_clips(editor: &mut MontageEditorState) -> bool {
    let cache_path = editor.save_path.join("stock_cache.json");
    let content = match std::fs::read_to_string(&cache_path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let cache: Vec<crate::api::stock::SegmentCache> = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut replacements: Vec<(String, PathBuf, ClipKind, f32, usize)> = Vec::new();
    let mut still_pending = false;

    for clip in &editor.clips {
        // Обробляємо і плейсхолдери (перший вибір), і вже-призначені стокові кліпи (заміна через контекстне меню)
        let seg_idx = if clip.is_placeholder {
            match clip.media_id.strip_prefix("placeholder_").and_then(|s| s.parse::<usize>().ok()) {
                Some(i) => i,
                None => continue,
            }
        } else if let Some(idx) = clip.stock_seg_idx {
            idx
        } else {
            continue
        };
        if let Some(entry) = cache.get(seg_idx) {
            if let Some(sel) = &entry.selected {
                let file_path = editor.save_path.join("media").join(&sel.filename);
                if file_path.exists() {
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let kind = if matches!(ext.as_str(), "mp4"|"mov"|"webm") { ClipKind::Video } else { ClipKind::Image };
                    replacements.push((clip.id.clone(), file_path, kind, sel.trim_start, seg_idx));
                } else {
                    // Файл ще не завантажений — повторимо після наступного repaint
                    still_pending = true;
                }
            }
        }
    }

    let made_replacements = !replacements.is_empty();
    for (clip_id, file_path, kind, trim_start, seg_idx) in replacements {
        // Видаляємо старий запис і кеш кадрів щоб thumbnail перегенерувався з нового відео
        editor.media_pool.retain(|m| m.path != file_path);
        let old_cache = frame_cache_dir(&editor.save_path, &file_path, editor.preview_render);
        let old_sharp_cache = sharp_frame_cache_dir(&editor.save_path, &file_path, editor.preview_render);
        let _ = std::fs::remove_dir_all(&old_cache);
        let _ = std::fs::remove_dir_all(&old_sharp_cache);
        editor.media_pool.push(MediaItem::new(file_path.clone(), &editor.save_path, editor.preview_render));
        let media_id = editor.media_pool.iter()
            .find(|m| m.path == file_path)
            .map(|m| m.id.clone())
            .unwrap_or_default();
        let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if let Some(clip) = editor.clips.iter_mut().find(|c| c.id == clip_id) {
            clip.path = Some(file_path);
            clip.kind = kind;
            clip.name = name;
            clip.is_placeholder = false;
            clip.media_id = media_id;
            clip.trim_start = trim_start;
            clip.stock_seg_idx = Some(seg_idx);
        }
    }

    // Зберігаємо timeline.json щоб stock_seg_idx пережив перезапуск
    if made_replacements {
        editor.save_to_timeline().ok();
    }

    still_pending
}

fn load_media_pool(save_path: &Path, preview: PreviewRenderSettings) -> Vec<MediaItem> {
    let media_dir = save_path.join("media");
    let mut items = Vec::new();
    if media_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&media_dir) {
            for entry in entries.filter_map(Result::ok) {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if matches!(ext.as_str(), "mp4" | "mov" | "webm" | "jpg" | "jpeg" | "png" | "webp" | "mp3" | "wav") {
                        items.push(MediaItem::new(p, save_path, preview));
                    }
                }
            }
        }
    }
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_path.join(name);
        if p.exists() { items.push(MediaItem::new(p, save_path, preview)); }
    }
    items
}
