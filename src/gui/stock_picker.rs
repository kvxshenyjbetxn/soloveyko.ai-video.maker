use eframe::egui;
use crate::localization::{Language, translate};
use crate::api::stock::{SegmentCache, SelectedMedia, load_cache, save_cache};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Arc на результат фонового Pexels пошуку для одного сегменту.
/// None = ще виконується; Some(Ok(...)) = готово; Some(Err(...)) = помилка.
type SegmentSearchArc = Arc<Mutex<Option<Result<
    (Vec<crate::api::stock::CachedPhoto>, Vec<crate::api::stock::CachedVideo>),
    String,
>>>>;


// ─── Структури стану ──────────────────────────────────────────────────────────

pub struct StockPickerState {
    pub save_path: String,
    pub cache: Vec<SegmentCache>,
    pub active_segment: usize,
    /// Показувати відео (true) чи фото (false); None = auto з кешу
    pub show_videos: Option<bool>,
    /// Текстури мініатюр: preview_url → TextureHandle
    pub thumbnails: std::collections::HashMap<String, Option<egui::TextureHandle>>,
    /// Завантаження мініатюри у фоні
    pub thumb_loading: std::collections::HashMap<String, Arc<Mutex<Option<Option<egui::ColorImage>>>>>,
    /// Активне завантаження відео з відстеженням прогресу
    pub video_download: Option<VideoDownloadState>,
    /// Відкритий міні-редактор нарізки (після завантаження)
    pub trim_edit: Option<TrimEditState>,
    /// Збережений API ключ Pexels
    pub pexels_key: String,
    /// Збережений API ключ Pixabay
    pub pixabay_key: String,
    /// Чи використовувати Pexels для пошуку
    pub use_pexels: bool,
    /// Чи використовувати Pixabay для пошуку
    pub use_pixabay: bool,
    /// Чи показувати панель вибору сервісів (фільтри)
    pub show_services_filter: bool,
    /// Фоновий пошук для кожного сегмента: seg_idx → arc результату
    pub segment_search: std::collections::HashMap<usize, SegmentSearchArc>,
    /// Поточний текст промту для активного сегмента (редагується через TextEdit)
    pub edit_keyword: String,
}

/// Стан завантаження відео
pub struct VideoDownloadState {
    pub segment_idx: usize,
    pub video_id: String,
    pub video_duration: f32,
    pub filename: String,
    pub dest_path: PathBuf,
    /// Прогрес 0.0..1.0; -1.0 = помилка
    pub progress: Arc<Mutex<f32>>,
}

/// Стан міні-редактора вибору фрагменту відео
pub struct TrimEditState {
    pub segment_idx: usize,
    pub video_id: String,
    pub filename: String,
    pub video_path: PathBuf,
    pub video_duration: f32,
    /// Тривалість фрагменту, який треба вставити
    pub segment_duration: f32,
    /// Поточна позиція початку (перетягується користувачем)
    pub trim_start: f32,
    /// Кадри стрічки (8 рівномірних по всьому відео)
    pub preview_frames: Vec<(f32, egui::TextureHandle)>,
    pub frames_raw: Arc<Mutex<Vec<(f32, Vec<u8>)>>>,
    /// Кадри для відтворення обраного фрагменту (640px, 10fps)
    pub playback_frames: Vec<egui::TextureHandle>,
    pub playback_raw: Arc<Mutex<Option<Vec<Vec<u8>>>>>,
    pub playback_fps: f32,
    pub playback_frame_idx: usize,
    pub playback_last_tick: Option<std::time::Instant>,
    /// trim_start при якому були витягнуті playback_frames
    pub playback_for_trim: f32,
}

impl StockPickerState {
    pub fn new(
        save_path: String,
        pexels_key: String,
        pixabay_key: String,
        stock_service: String,
    ) -> Option<Self> {
        let path = Path::new(&save_path);
        let mut cache = load_cache(path)
            .or_else(|| build_skeleton_cache_from_timeline(path))?;
        // Оновлюємо segment_duration якщо вони були 0.0 (timeline ще не існував при запуску пайплайну)
        sync_segment_durations_from_timeline(path, &mut cache);
        let edit_keyword = cache.first().map(|s| s.keyword.clone()).unwrap_or_default();

        let has_pexels_key = !pexels_key.is_empty();
        let has_pixabay_key = !pixabay_key.is_empty();

        let (use_pexels, use_pixabay) = if has_pexels_key || has_pixabay_key {
            (has_pexels_key, has_pixabay_key)
        } else {
            (stock_service != "Pixabay", stock_service == "Pixabay")
        };

        Some(Self {
            save_path,
            active_segment: 0,
            show_videos: None,
            cache,
            thumbnails: Default::default(),
            thumb_loading: Default::default(),
            video_download: None,
            trim_edit: None,
            edit_keyword,
            pexels_key,
            pixabay_key,
            use_pexels,
            use_pixabay,
            show_services_filter: false,
            segment_search: Default::default(),
        })
    }

    #[allow(dead_code)]
    pub fn reload_cache(&mut self) {
        if let Some(cache) = load_cache(Path::new(&self.save_path)) {
            self.cache = cache;
        }
    }
}

// ─── Допоміжні функції ────────────────────────────────────────────────────────

/// Якщо у кеші segment_duration == 0.0 (timeline ще не існував при запуску пайплайну),
/// оновлюємо значення з поточного timeline.json.
fn sync_segment_durations_from_timeline(save_dir: &Path, cache: &mut Vec<SegmentCache>) {
    let content = match std::fs::read_to_string(save_dir.join("timeline.json")) {
        Ok(s) => s,
        Err(_) => return,
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };
    let segs = match v["segments"].as_array() {
        Some(s) => s,
        None => return,
    };
    for (entry, seg) in cache.iter_mut().zip(segs.iter()) {
        if entry.segment_duration <= 0.0 {
            let start = seg["start_secs"].as_f64().unwrap_or(0.0);
            let end = seg["end_secs"].as_f64().unwrap_or(0.0);
            let dur = (end - start).max(0.0) as f32;
            if dur > 0.0 {
                entry.segment_duration = dur;
            }
        }
    }
}

fn build_skeleton_cache_from_timeline(save_dir: &Path) -> Option<Vec<SegmentCache>> {
    let content = std::fs::read_to_string(save_dir.join("timeline.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let segs = v["segments"].as_array()?;
    let result: Vec<SegmentCache> = segs.iter().enumerate()
        .filter_map(|(i, s)| {
            let text = s["text"].as_str()?.to_string();
            let start = s["start_secs"].as_f64().unwrap_or(0.0);
            let end = s["end_secs"].as_f64().unwrap_or(0.0);
            let duration = (end - start).max(0.0) as f32;
            Some(SegmentCache {
                index: i,
                keyword: text.clone(),
                segment_text: text,
                segment_duration: duration,
                photos: vec![],
                videos: vec![],
                selected: None,
            })
        })
        .collect();
    if result.is_empty() { None } else { Some(result) }
}

// ─── Lazy search ──────────────────────────────────────────────────────────────

/// Запускає фоновий пошук стоку для сегмента якщо він ще не має результатів.
fn trigger_segment_search(state: &mut StockPickerState, seg_idx: usize, ctx: &egui::Context) {
    if state.segment_search.contains_key(&seg_idx) { return; }
    let Some(seg) = state.cache.get(seg_idx) else { return };
    if !seg.photos.is_empty() || !seg.videos.is_empty() { return; }
    if !state.use_pexels && !state.use_pixabay { return; }

    let keyword = seg.keyword.clone();
    let pexels_key = state.pexels_key.clone();
    let pixabay_key = state.pixabay_key.clone();
    let use_pexels = state.use_pexels;
    let use_pixabay = state.use_pixabay;
    let arc: SegmentSearchArc = Arc::new(Mutex::new(None));
    let arc_c = Arc::clone(&arc);
    let ctx_c = ctx.clone();

    std::thread::spawn(move || {
        use crate::api::stock::StockProvider;

        let mut all_photos = Vec::new();
        let mut all_videos = Vec::new();

        // Пошук у Pexels
        if use_pexels && !pexels_key.is_empty() {
            let provider = crate::api::stock::pexels::PexelsProvider;
            if let Ok(p) = provider.search_photos(&pexels_key, &keyword, 15) {
                all_photos.extend(p.iter().map(|photo| {
                    let mut cp = crate::api::stock::CachedPhoto::from(photo);
                    cp.provider = "Pexels".to_string();
                    cp
                }));
            }
            if let Ok(v) = provider.search_videos(&pexels_key, &keyword, 15) {
                all_videos.extend(v.iter().map(|video| {
                    let mut cv = crate::api::stock::CachedVideo::from(video);
                    cv.provider = "Pexels".to_string();
                    cv
                }));
            }
        }

        // Пошук у Pixabay
        if use_pixabay && !pixabay_key.is_empty() {
            let provider = crate::api::stock::pixabay::PixabayProvider;
            if let Ok(p) = provider.search_photos(&pixabay_key, &keyword, 15) {
                all_photos.extend(p.iter().map(|photo| {
                    let mut cp = crate::api::stock::CachedPhoto::from(photo);
                    cp.provider = "Pixabay".to_string();
                    cp
                }));
            }
            if let Ok(v) = provider.search_videos(&pixabay_key, &keyword, 15) {
                all_videos.extend(v.iter().map(|video| {
                    let mut cv = crate::api::stock::CachedVideo::from(video);
                    cv.provider = "Pixabay".to_string();
                    cv
                }));
            }
        }

        *arc_c.lock().unwrap() = Some(Ok((all_photos, all_videos)));
        ctx_c.request_repaint();
    });

    state.segment_search.insert(seg_idx, arc);
}

/// Переносить готові результати пошуку з фонових тредів у кеш.
fn flush_segment_search(state: &mut StockPickerState) {
    let idxs: Vec<usize> = state.segment_search.keys().copied().collect();
    let mut changed = false;
    for idx in idxs {
        let arc = Arc::clone(&state.segment_search[&idx]);
        let done = arc.try_lock().ok().and_then(|mut g| g.take());
        match done {
            Some(Ok((photos, videos))) => {
                state.segment_search.remove(&idx);
                if let Some(seg) = state.cache.get_mut(idx) {
                    seg.photos = photos;
                    seg.videos = videos;
                }
                changed = true;
            }
            Some(Err(_)) => {
                state.segment_search.remove(&idx);
            }
            None => {}
        }
    }
    if changed {
        let _ = save_cache(Path::new(&state.save_path), &state.cache);
    }
}

// ─── Головна точка входу ──────────────────────────────────────────────────────

pub enum StockPickerAction {
    None,
    Close,
    Confirmed,
}

pub fn draw_stock_picker(
    ctx: &egui::Context,
    language: Language,
    state: &mut StockPickerState,
) -> StockPickerAction {
    let mut action = StockPickerAction::None;
    let screen = ctx.screen_rect();

    flush_thumb_loading(ctx, state);
    flush_segment_search(state);

    // Перевіряємо чи завершилось завантаження → переходимо до trim редактора
    check_download_complete(state, ctx);

    // Пікер рендерується першим
    draw_single_mode(ctx, language, state, screen, &mut action);

    // Trim editor рендерується ПІСЛЯ пікера — поверх нього
    if state.trim_edit.is_some() {
        draw_trim_editor(ctx, language, state, &mut action);
    }

    action
}

/// Перевіряє прогрес завантаження і переходить до trim_edit якщо готово
fn check_download_complete(state: &mut StockPickerState, ctx: &egui::Context) {
    if state.trim_edit.is_some() { return; }
    let done = if let Some(dl) = &state.video_download {
        let p = *dl.progress.lock().unwrap();
        p >= 1.0 && dl.dest_path.exists()
    } else {
        false
    };
    if done {
        if let Some(dl) = state.video_download.take() {
            let seg_dur = state.cache.get(dl.segment_idx)
                .map(|s| s.segment_duration)
                .unwrap_or(0.0);
            let segment_duration = if seg_dur > 0.0 {
                seg_dur.min(dl.video_duration)
            } else {
                dl.video_duration
            };
            let frames_raw = Arc::new(Mutex::new(Vec::new()));
            spawn_frame_extraction(
                dl.dest_path.clone(), dl.video_duration, 8,
                Arc::clone(&frames_raw), ctx.clone(),
            );
            let playback_raw = Arc::new(Mutex::new(None));
            spawn_playback_extraction(
                dl.dest_path.clone(), 0.0, segment_duration,
                Arc::clone(&playback_raw), ctx.clone(),
            );
            state.trim_edit = Some(TrimEditState {
                segment_idx: dl.segment_idx,
                video_id: dl.video_id,
                filename: dl.filename,
                video_path: dl.dest_path,
                video_duration: dl.video_duration,
                segment_duration,
                trim_start: 0.0,
                preview_frames: Vec::new(),
                frames_raw,
                playback_frames: Vec::new(),
                playback_raw,
                playback_fps: 10.0,
                playback_frame_idx: 0,
                playback_last_tick: None,
                playback_for_trim: 0.0,
            });
        }
    }
}

// ─── Видобування кадрів у фоні ────────────────────────────────────────────────

/// Запускає ffmpeg у фоновому потоці; записує (t_secs, jpeg_bytes) у `out`
fn spawn_frame_extraction(
    video_path: PathBuf,
    video_duration: f32,
    n_frames: usize,
    out: Arc<Mutex<Vec<(f32, Vec<u8>)>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut h = DefaultHasher::new();
        video_path.hash(&mut h);
        let hash = h.finish();

        let tmp_dir = std::env::temp_dir().join(format!("soloveyko_trim_{:x}", hash));
        let _ = std::fs::create_dir_all(&tmp_dir);

        if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
            for e in entries.flatten() { let _ = std::fs::remove_file(e.path()); }
        }

        let out_pattern = tmp_dir.join("frame_%03d.jpg");
        let fps_val = if video_duration > 0.0 { n_frames as f32 / video_duration } else { 1.0 };

        let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
        cmd.arg("-i").arg(&video_path)
           .arg("-vf").arg(format!("fps={:.4},scale=320:-2", fps_val))
           .arg("-q:v").arg("4")
           .arg("-frames:v").arg(n_frames.to_string())
           .arg("-y")
           .arg("-loglevel").arg("error")
           .arg(&out_pattern);
        crate::bundle::set_no_window(&mut cmd);

        if cmd.status().map(|s| s.success()).unwrap_or(false) {
            let step = if n_frames > 1 && video_duration > 0.0 {
                video_duration / (n_frames as f32 - 1.0)
            } else {
                1.0
            };
            let mut entries: Vec<_> = std::fs::read_dir(&tmp_dir).ok()
                .into_iter().flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jpg"))
                .collect();
            entries.sort_by_key(|e| e.path());

            let results: Vec<(f32, Vec<u8>)> = entries.iter().enumerate()
                .filter_map(|(i, entry)| {
                    let bytes = std::fs::read(entry.path()).ok()?;
                    Some((i as f32 * step, bytes))
                })
                .collect();

            *out.lock().unwrap() = results;
            ctx.request_repaint();
        }
    });
}

/// Витягує кадри конкретного фрагменту відео для відтворення (640px, 10fps)
fn spawn_playback_extraction(
    video_path: PathBuf,
    trim_start: f32,
    duration: f32,
    out: Arc<Mutex<Option<Vec<Vec<u8>>>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut h = DefaultHasher::new();
        video_path.hash(&mut h);
        let hash = h.finish();

        let tmp_dir = std::env::temp_dir().join(format!("soloveyko_play_{:x}", hash));
        let _ = std::fs::create_dir_all(&tmp_dir);

        if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
            for e in entries.flatten() { let _ = std::fs::remove_file(e.path()); }
        }

        let fps = 10.0f32;
        let n_frames = ((duration * fps).ceil() as usize).max(1).min(80);
        let out_pattern = tmp_dir.join("frame_%03d.jpg");

        let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
        cmd.arg("-ss").arg(format!("{:.3}", trim_start))
           .arg("-i").arg(&video_path)
           .arg("-t").arg(format!("{:.3}", duration))
           .arg("-vf").arg("fps=10,scale=640:-2")
           .arg("-q:v").arg("2")
           .arg("-frames:v").arg(n_frames.to_string())
           .arg("-y")
           .arg("-loglevel").arg("error")
           .arg(&out_pattern);
        crate::bundle::set_no_window(&mut cmd);

        if cmd.status().map(|s| s.success()).unwrap_or(false) {
            let mut entries: Vec<_> = std::fs::read_dir(&tmp_dir).ok()
                .into_iter().flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jpg"))
                .collect();
            entries.sort_by_key(|e| e.path());

            let results: Vec<Vec<u8>> = entries.iter()
                .filter_map(|entry| std::fs::read(entry.path()).ok())
                .collect();

            *out.lock().unwrap() = Some(results);
            ctx.request_repaint();
        }
    });
}

/// Запускає нове витягування playback кадрів для поточного trim_start
fn retrigger_playback(trim: &mut TrimEditState, ctx: &egui::Context) {
    let raw = Arc::new(Mutex::new(None));
    spawn_playback_extraction(
        trim.video_path.clone(),
        trim.trim_start,
        trim.segment_duration,
        Arc::clone(&raw),
        ctx.clone(),
    );
    trim.playback_raw = raw;
    trim.playback_for_trim = trim.trim_start;
    trim.playback_frame_idx = 0;
    trim.playback_last_tick = None;
}

/// Переносить готові сирі байти кадрів у текстури (викликається кожен кадр UI)
fn flush_trim_frames(ctx: &egui::Context, trim: &mut TrimEditState) {
    if let Ok(mut guard) = trim.frames_raw.try_lock() {
        if guard.is_empty() { return; }
        let raw = std::mem::take(&mut *guard);
        for (t, bytes) in raw {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let ci = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                let tex = ctx.load_texture(
                    format!("trim_frame_{:.3}", t),
                    ci,
                    egui::TextureOptions::LINEAR,
                );
                trim.preview_frames.push((t, tex));
            }
        }
        trim.preview_frames.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }
}

/// Переносить готові playback кадри у текстури (викликається кожен кадр UI)
fn flush_playback_frames(ctx: &egui::Context, trim: &mut TrimEditState) {
    if let Ok(mut guard) = trim.playback_raw.try_lock() {
        if let Some(raw_frames) = guard.take() {
            trim.playback_frames.clear();
            trim.playback_frame_idx = 0;
            trim.playback_last_tick = None;
            for (i, bytes) in raw_frames.iter().enumerate() {
                if let Ok(img) = image::load_from_memory(bytes) {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let ci = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                    let tex = ctx.load_texture(
                        format!("play_frame_{}", i),
                        ci,
                        egui::TextureOptions::LINEAR,
                    );
                    trim.playback_frames.push(tex);
                }
            }
        }
    }
}

// ─── Міні-редактор нарізки ────────────────────────────────────────────────────

fn draw_trim_editor(
    ctx: &egui::Context,
    language: Language,
    state: &mut StockPickerState,
    action: &mut StockPickerAction,
) {
    let Some(trim) = &mut state.trim_edit else { return };

    flush_trim_frames(ctx, trim);
    flush_playback_frames(ctx, trim);

    let max_start = (trim.video_duration - trim.segment_duration).max(0.0);
    trim.trim_start = trim.trim_start.clamp(0.0, max_start);
    let trim_end = trim.trim_start + trim.segment_duration;

    // Анімуємо playback кадри; якщо не готові — використовуємо найближчий кадр стрічки
    let preview_tex = if !trim.playback_frames.is_empty() {
        let now = std::time::Instant::now();
        if let Some(last) = trim.playback_last_tick {
            let elapsed = now.duration_since(last).as_secs_f32();
            if elapsed >= 1.0 / trim.playback_fps {
                let advance = (elapsed * trim.playback_fps) as usize;
                trim.playback_frame_idx = (trim.playback_frame_idx + advance) % trim.playback_frames.len();
                trim.playback_last_tick = Some(now);
            }
        } else {
            trim.playback_last_tick = Some(now);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        trim.playback_frames.get(trim.playback_frame_idx).cloned()
    } else {
        trim.preview_frames.iter()
            .min_by(|a, b| {
                (a.0 - trim.trim_start).abs()
                    .partial_cmp(&(b.0 - trim.trim_start).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, tex)| tex.clone())
    };

    // Клонуємо кадри для малювання смуги (уникаємо подвійного borrow)
    let strip_frames: Vec<(f32, egui::TextureHandle)> = trim.preview_frames
        .iter().map(|(t, tex)| (*t, tex.clone())).collect();

    let title = format!("✂ {}", trim.filename);
    let mut confirmed = false;
    let mut cancelled = false;
    let screen = ctx.screen_rect();

    // Order::Debug — вище за все (Foreground → Tooltip → Debug)
    egui::Window::new(title)
        .id(egui::Id::new("stock_trim_editor"))
        .default_size([860.0, 620.0])
        .min_size([420.0, 300.0])
        .max_size([screen.width() - 40.0, screen.height() - 40.0])
        .resizable(true)
        .collapsible(false)
        .constrain(true)
        .order(egui::Order::Debug)
        .show(ctx, |ui| {
            // Рядок інформації
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("📹 {:.1}с", trim.video_duration)).weak());
                ui.separator();
                ui.label(egui::RichText::new(format!("▶ Фрагмент: {:.1}с", trim.segment_duration)).strong());
                ui.separator();
                ui.label(format!("{:.1}с → {:.1}с", trim.trim_start, trim_end));
            });
            ui.add_space(6.0);

            // Великий превью кадр
            {
                let avail_w = ui.available_width();
                let avail_h = ui.available_height();
                // Залишаємо щонайменше 180 пікселів для інших елементів (інфо-рядок, таймлайн-стрип, мітки, кнопки)
                let max_h = (avail_h - 180.0).max(100.0);
                let preview_w = avail_w.min(820.0).min(max_h * 16.0 / 9.0);
                let preview_h = (preview_w * 9.0 / 16.0).min(max_h);

                // Центруємо горизонтально
                let indent = ((avail_w - preview_w) * 0.5).max(0.0);
                ui.add_space(0.0);
                let start_x = ui.cursor().min.x + indent;
                let (preview_rect, _) = ui.allocate_exact_size(
                    egui::vec2(avail_w, preview_h),
                    egui::Sense::hover(),
                );
                let actual_rect = egui::Rect::from_min_size(
                    egui::pos2(start_x, preview_rect.min.y),
                    egui::vec2(preview_w, preview_h),
                );

                let painter = ui.painter();
                painter.rect_filled(actual_rect, 4.0, egui::Color32::from_gray(20));

                if let Some(tex) = &preview_tex {
                    painter.image(
                        tex.id(), actual_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                } else {
                    let icon = if strip_frames.is_empty() { "⏳" } else { "🎬" };
                    painter.text(
                        actual_rect.center(), egui::Align2::CENTER_CENTER,
                        icon, egui::FontId::proportional(36.0), egui::Color32::GRAY,
                    );
                }
            }

            ui.add_space(8.0);

            // Таймлайн-стрип зі мініатюрами кадрів
            let strip_height = 76.0;
            let (strip_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), strip_height),
                egui::Sense::hover(),
            );

            let painter = ui.painter();
            painter.rect_filled(strip_rect, 4.0, egui::Color32::from_gray(30));

            // Мініатюри кадрів як підкладка смуги
            if !strip_frames.is_empty() {
                let frame_w = strip_rect.width() / strip_frames.len() as f32;
                for (i, (_, tex)) in strip_frames.iter().enumerate() {
                    let x = strip_rect.min.x + i as f32 * frame_w;
                    let fr = egui::Rect::from_min_size(
                        egui::pos2(x, strip_rect.min.y),
                        egui::vec2(frame_w, strip_height),
                    );
                    painter.image(
                        tex.id(), fr,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }

            if trim.video_duration > 0.0 {
                let w = strip_rect.width();
                let handle_w = (trim.segment_duration / trim.video_duration * w).max(8.0);
                let handle_x = strip_rect.min.x + trim.trim_start / trim.video_duration * w;
                let handle_rect = egui::Rect::from_min_size(
                    egui::pos2(handle_x, strip_rect.min.y + 2.0),
                    egui::vec2(handle_w, strip_height - 4.0),
                );

                // Затемнення поза виділенням
                painter.rect_filled(
                    egui::Rect::from_min_max(strip_rect.min, egui::pos2(handle_rect.min.x, strip_rect.max.y)),
                    0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 140),
                );
                painter.rect_filled(
                    egui::Rect::from_min_max(egui::pos2(handle_rect.max.x, strip_rect.min.y), strip_rect.max),
                    0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 140),
                );

                // Виділений фрагмент
                painter.rect_filled(handle_rect, 4.0, egui::Color32::from_rgba_unmultiplied(52, 152, 219, 80));
                painter.rect_stroke(handle_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                // Ручки по краях
                let lx = handle_rect.min.x + 4.0;
                let rx = handle_rect.max.x - 4.0;
                let cy = handle_rect.center().y;
                painter.line_segment([egui::pos2(lx, cy - 8.0), egui::pos2(lx, cy + 8.0)], egui::Stroke::new(2.0, egui::Color32::WHITE));
                painter.line_segment([egui::pos2(rx, cy - 8.0), egui::pos2(rx, cy + 8.0)], egui::Stroke::new(2.0, egui::Color32::WHITE));

                // Мітки часу
                let label_y = strip_rect.max.y + 4.0;
                let fmt_time = |s: f32| -> String {
                    let m = (s as u32) / 60;
                    let sec = s as u32 % 60;
                    if m > 0 { format!("{m}:{sec:02}") } else { format!("{sec}с") }
                };
                for frac in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
                    let t = frac * trim.video_duration;
                    let x = strip_rect.min.x + frac * w;
                    let align = if frac == 0.0 { egui::Align2::LEFT_TOP } else if frac == 1.0 { egui::Align2::RIGHT_TOP } else { egui::Align2::CENTER_TOP };
                    painter.text(egui::pos2(x, label_y), align, fmt_time(t),
                        egui::FontId::proportional(10.0), egui::Color32::GRAY);
                }

                // Drag взаємодія
                let drag_resp = ui.interact(strip_rect, egui::Id::new("trim_strip_drag"), egui::Sense::click_and_drag());
                if drag_resp.dragged() {
                    let delta_secs = drag_resp.drag_delta().x / w * trim.video_duration;
                    trim.trim_start = (trim.trim_start + delta_secs).clamp(0.0, max_start);
                }
                if drag_resp.drag_stopped() {
                    retrigger_playback(trim, ctx);
                }
                if drag_resp.clicked() {
                    if let Some(pos) = drag_resp.interact_pointer_pos() {
                        let clicked_t = ((pos.x - strip_rect.min.x) / w * trim.video_duration)
                            - trim.segment_duration * 0.5;
                        trim.trim_start = clicked_t.clamp(0.0, max_start);
                    }
                    retrigger_playback(trim, ctx);
                }
            }

            ui.add_space(20.0); // місце для міток часу

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(
                    translate(language, "stock_trim_label")
                ).weak().size(11.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(translate(language, "stock_trim_confirm")).strong()).clicked() {
                        confirmed = true;
                    }
                    ui.add_space(8.0);
                    if ui.button(translate(language, "stock_trim_cancel")).clicked() {
                        cancelled = true;
                    }
                });
            });
        });

    if confirmed {
        let trim = state.trim_edit.take().unwrap();
        let trim_end = trim.trim_start + trim.segment_duration;
        if let Some(seg) = state.cache.get_mut(trim.segment_idx) {
            seg.selected = Some(SelectedMedia {
                kind: "video".to_string(),
                id: trim.video_id,
                url: String::new(), // вже завантажено
                filename: trim.filename,
                trim_start: trim.trim_start,
                trim_end,
            });
            let _ = save_cache(Path::new(&state.save_path), &state.cache);
        }
        *action = StockPickerAction::Confirmed;
    } else if cancelled {
        state.trim_edit = None;
    }
}

// ─── Single-mode (один сегмент) ───────────────────────────────────────────────

fn draw_single_mode(
    ctx: &egui::Context,
    language: Language,
    state: &mut StockPickerState,
    screen: egui::Rect,
    action: &mut StockPickerAction,
) {
    let seg_idx = state.active_segment;
    let title = if let Some(seg) = state.cache.get(seg_idx) {
        let kw: String = seg.keyword.chars().take(50).collect();
        let sfx = if seg.keyword.chars().count() > 50 { "…" } else { "" };
        format!("🖼 {}{}", kw, sfx)
    } else {
        "🖼 Stock Picker".to_string()
    };

    // Запускаємо пошук якщо потрібно (перед рендером вікна, щоб уникнути borrow conflicts)
    trigger_segment_search(state, seg_idx, ctx);

    let mut retrigger_search = false;

    egui::Window::new(title)
        .id(egui::Id::new("stock_picker_single"))
        .default_size([750.0, 520.0])
        .min_size([420.0, 300.0])
        .max_size([screen.width() - 40.0, screen.height() - 40.0])
        .resizable(true)
        .collapsible(false)
        .constrain(true)
        .order(egui::Order::Tooltip)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(screen.center())
        .show(ctx, |ui| {
            // Клонуємо дані сегмента щоб уникнути borrow conflicts з edit_keyword
            let (seg_text, selected, photos, videos) = if let Some(seg) = state.cache.get(seg_idx) {
                (seg.segment_text.clone(), seg.selected.clone(), seg.photos.clone(), seg.videos.clone())
            } else {
                return;
            };

            // Рядок редагування keyword + кнопка перезапуску пошуку
            ui.horizontal(|ui| {
                // Кнопка розгортання фільтрів
                let filter_btn_text = if state.show_services_filter {
                    format!("🔽 {}", translate(language, "stock_services_btn"))
                } else {
                    format!("▶ {}", translate(language, "stock_services_btn"))
                };
                if ui.button(filter_btn_text).clicked() {
                    state.show_services_filter = !state.show_services_filter;
                }

                ui.add(egui::TextEdit::singleline(&mut state.edit_keyword)
                    .desired_width(ui.available_width() - 44.0));
                if ui.button("🔄").on_hover_text(translate(language, "stock_search_retrigger")).clicked() {
                    retrigger_search = true;
                }
            });

            // Якщо увімкнено показ фільтрів - показуємо чекбокси
            if state.show_services_filter {
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(translate(language, "stock_search_in"));
                    let prev_pexels = state.use_pexels;
                    let prev_pixabay = state.use_pixabay;

                    ui.checkbox(&mut state.use_pexels, "Pexels");
                    ui.checkbox(&mut state.use_pixabay, "Pixabay");

                    // Запобігаємо вимиканню обох чекбоксів
                    if !state.use_pexels && !state.use_pixabay {
                        if !prev_pexels {
                            state.use_pexels = true;
                        } else {
                            state.use_pixabay = true;
                        }
                    }

                    if state.use_pexels != prev_pexels || state.use_pixabay != prev_pixabay {
                        retrigger_search = true;
                    }
                });
            }
            ui.add_space(4.0);

            // Опис сцени
            if !seg_text.is_empty() {
                let preview: String = seg_text.chars().take(120).collect();
                let sfx = if seg_text.chars().count() > 120 { "…" } else { "" };
                ui.label(egui::RichText::new(format!("{}{}", preview, sfx)).weak().size(11.0));
                ui.add_space(4.0);
            }

            // Прогрес бар завантаження відео
            if let Some(dl) = &state.video_download {
                if dl.segment_idx == seg_idx {
                    let p = *dl.progress.lock().unwrap();
                    ui.horizontal(|ui| {
                        if p < 0.0 {
                            ui.label(egui::RichText::new("❌ Помилка завантаження").color(egui::Color32::RED).size(12.0));
                        } else {
                            ui.label(egui::RichText::new(format!("⬇ {} {:.0}%", dl.filename, p * 100.0)).size(12.0));
                            let bar_w = ui.available_width().min(300.0);
                            let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, 10.0), egui::Sense::hover());
                            ui.painter().rect_filled(bar_rect, 5.0, egui::Color32::from_gray(50));
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(bar_rect.min, egui::vec2(bar_w * p.clamp(0.0, 1.0), 10.0)),
                                5.0, egui::Color32::from_rgb(52, 152, 219),
                            );
                        }
                    });
                    ui.add_space(4.0);
                }
            }

            // Статус вибору
            if let Some(sel) = &selected {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("✔ {}", sel.filename))
                            .color(egui::Color32::from_rgb(46, 204, 113))
                            .size(12.0)
                    );
                });
                ui.add_space(4.0);
            }

            // Сітка результатів
            let is_searching = state.segment_search.contains_key(&seg_idx);

            let has_videos = !videos.is_empty();
            let has_photos = !photos.is_empty();
            if state.show_videos.is_none() {
                state.show_videos = Some(has_videos || !has_photos);
            }
            let is_video_mode = state.show_videos.unwrap_or(true);

            let has_any_working_key = (state.use_pexels && !state.pexels_key.is_empty())
                || (state.use_pixabay && !state.pixabay_key.is_empty());
            let is_any_key_missing = (state.use_pexels && state.pexels_key.is_empty())
                || (state.use_pixabay && state.pixabay_key.is_empty());

            if is_any_key_missing {
                ui.colored_label(egui::Color32::KHAKI, translate(language, "stock_key_missing_warning"));
                ui.add_space(2.0);
            }

            if !has_any_working_key {
                ui.label(egui::RichText::new("Потрібно вказати хоча б один API-ключ для активних сервісів").weak());
            } else if is_searching && photos.is_empty() && videos.is_empty() {
                ui.label(egui::RichText::new("🔍 Шукаємо…").weak().size(13.0));
            } else {
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_video_mode,
                        egui::RichText::new(format!("🎬 Відео ({})", videos.len()))).clicked() {
                        state.show_videos = Some(true);
                    }
                    if ui.selectable_label(!is_video_mode,
                        egui::RichText::new(format!("📷 Фото ({})", photos.len()))).clicked() {
                        state.show_videos = Some(false);
                    }
                });
                ui.add_space(4.0);

                let thumb_size = egui::vec2(160.0, 104.0);
                let max_scroll_h = (screen.height() - 200.0).max(150.0);

                egui::ScrollArea::vertical()
                    .id_salt("stock_single_scroll")
                    .max_height(max_scroll_h)
                    .show(ui, |ui| {
                        let avail_w = ui.available_width();
                        let spacing = 8.0;
                        let col_w = thumb_size.x + spacing;
                        let cols = ((avail_w / col_w).floor() as usize).max(1);

                        if is_video_mode {
                            let pexels_videos: Vec<_> = videos.iter().filter(|v| v.provider == "Pexels" || v.provider.is_empty()).cloned().collect();
                            let pixabay_videos: Vec<_> = videos.iter().filter(|v| v.provider == "Pixabay").cloned().collect();

                            if !pexels_videos.is_empty() {
                                ui.label(egui::RichText::new("🎬 Pexels Videos").strong().size(13.0));
                                ui.add_space(4.0);
                                draw_video_grid(ui, ctx, state, "pexels_v", seg_idx, &pexels_videos, thumb_size, cols, action);
                                ui.add_space(12.0);
                            }

                            if !pixabay_videos.is_empty() {
                                ui.label(egui::RichText::new("🎬 Pixabay Videos").strong().size(13.0));
                                ui.add_space(4.0);
                                draw_video_grid(ui, ctx, state, "pixabay_v", seg_idx, &pixabay_videos, thumb_size, cols, action);
                            }

                            if pexels_videos.is_empty() && pixabay_videos.is_empty() {
                                ui.label(egui::RichText::new("Немає результатів").weak());
                            }
                        } else {
                            let pexels_photos: Vec<_> = photos.iter().filter(|p| p.provider == "Pexels" || p.provider.is_empty()).cloned().collect();
                            let pixabay_photos: Vec<_> = photos.iter().filter(|p| p.provider == "Pixabay").cloned().collect();

                            if !pexels_photos.is_empty() {
                                ui.label(egui::RichText::new("📷 Pexels Photos").strong().size(13.0));
                                ui.add_space(4.0);
                                draw_photo_grid(ui, ctx, state, "pexels_p", seg_idx, &pexels_photos, thumb_size, cols, action);
                                ui.add_space(12.0);
                            }

                            if !pixabay_photos.is_empty() {
                                ui.label(egui::RichText::new("📷 Pixabay Photos").strong().size(13.0));
                                ui.add_space(4.0);
                                draw_photo_grid(ui, ctx, state, "pixabay_p", seg_idx, &pixabay_photos, thumb_size, cols, action);
                            }

                            if pexels_photos.is_empty() && pixabay_photos.is_empty() {
                                ui.label(egui::RichText::new("Немає результатів").weak());
                            }
                        }
                    });
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(translate(language, "stock_picker_close")).clicked() {
                        *action = StockPickerAction::Close;
                    }
                });
            });
        });

    if retrigger_search {
        let new_kw = state.edit_keyword.clone();
        if let Some(seg) = state.cache.get_mut(seg_idx) {
            seg.keyword = new_kw;
            seg.photos.clear();
            seg.videos.clear();
        }
        state.segment_search.remove(&seg_idx);
        state.show_videos = None;
        let _ = save_cache(Path::new(&state.save_path), &state.cache);
        trigger_segment_search(state, seg_idx, ctx);
    }
}

// ─── Сітки результатів ────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_video_grid(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut StockPickerState,
    id_prefix: &str,
    seg_idx: usize,
    videos: &[crate::api::stock::CachedVideo],
    thumb_size: egui::Vec2,
    cols: usize,
    action: &mut StockPickerAction,
) {
    if videos.is_empty() {
        ui.label(egui::RichText::new("Немає результатів").weak());
        return;
    }
    egui::Grid::new(format!("{id_prefix}_grid"))
        .num_columns(cols)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            for (col_idx, vid) in videos.iter().enumerate() {
                ui.vertical(|ui| {
                    let thumb_url = &vid.thumbnail_url;
                    let (rect, resp) = ui.allocate_exact_size(thumb_size, egui::Sense::click());
                    draw_thumb(ui, ctx, state, rect, &resp, thumb_url);
                    if ui.is_rect_visible(rect) {
                        let dur_text = format!("{}с", vid.duration_secs);
                        let tp = egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0);
                        ui.painter().text(tp, egui::Align2::RIGHT_BOTTOM, &dur_text,
                            egui::FontId::proportional(11.0), egui::Color32::WHITE);
                    }
                    if resp.clicked() && state.video_download.is_none() {
                        start_video_download_if_needed(ctx, state, seg_idx, vid, action);
                    }
                    let author: String = vid.author.chars().take(18).collect();
                    ui.label(egui::RichText::new(author).size(10.0).weak());
                });
                if (col_idx + 1) % cols == 0 { ui.end_row(); }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_photo_grid(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut StockPickerState,
    id_prefix: &str,
    seg_idx: usize,
    photos: &[crate::api::stock::CachedPhoto],
    thumb_size: egui::Vec2,
    cols: usize,
    action: &mut StockPickerAction,
) {
    if photos.is_empty() {
        ui.label(egui::RichText::new("Немає результатів").weak());
        return;
    }
    egui::Grid::new(format!("{id_prefix}_grid"))
        .num_columns(cols)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            for (col_idx, photo) in photos.iter().enumerate() {
                ui.vertical(|ui| {
                    let thumb_url = &photo.preview_url;
                    let (rect, resp) = ui.allocate_exact_size(thumb_size, egui::Sense::click());
                    draw_thumb(ui, ctx, state, rect, &resp, thumb_url);
                    if ui.is_rect_visible(rect) {
                        let dim_text = format!("{}×{}", photo.width, photo.height);
                        let tp = egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0);
                        ui.painter().text(tp, egui::Align2::RIGHT_BOTTOM, &dim_text,
                            egui::FontId::proportional(10.0), egui::Color32::WHITE);
                    }
                    if resp.clicked() {
                        start_photo_download(ctx, state, seg_idx, photo, action);
                    }
                    let author: String = photo.author.chars().take(18).collect();
                    ui.label(egui::RichText::new(author).size(10.0).weak());
                });
                if (col_idx + 1) % cols == 0 { ui.end_row(); }
            }
        });
}

// ─── Завантаження ────────────────────────────────────────────────────────────

fn start_video_download_if_needed(
    ctx: &egui::Context,
    state: &mut StockPickerState,
    seg_idx: usize,
    vid: &crate::api::stock::CachedVideo,
    action: &mut StockPickerAction,
) {
    let filename = format!("{:04}.mp4", seg_idx + 1);
    let dest = Path::new(&state.save_path).join("media").join(&filename);

    // Якщо файл є, але належить іншому відео — видаляємо і завантажуємо заново
    if dest.exists() {
        let same_video = state.cache.get(seg_idx)
            .and_then(|s| s.selected.as_ref())
            .map(|sel| sel.id == vid.id)
            .unwrap_or(false);
        if !same_video {
            let _ = std::fs::remove_file(&dest);
        }
    }

    // Якщо файл вже є (і належить потрібному відео) — одразу відкриваємо trim редактор
    if dest.exists() {
        let seg_dur = state.cache.get(seg_idx).map(|s| s.segment_duration).unwrap_or(0.0);
        let video_dur = vid.duration_secs as f32;
        let segment_duration = if seg_dur > 0.0 { seg_dur.min(video_dur) } else { video_dur };
        let frames_raw = Arc::new(Mutex::new(Vec::new()));
        spawn_frame_extraction(
            dest.clone(), video_dur, 8,
            Arc::clone(&frames_raw), ctx.clone(),
        );
        let playback_raw = Arc::new(Mutex::new(None));
        spawn_playback_extraction(
            dest.clone(), 0.0, segment_duration,
            Arc::clone(&playback_raw), ctx.clone(),
        );
        state.trim_edit = Some(TrimEditState {
            segment_idx: seg_idx,
            video_id: vid.id.clone(),
            filename,
            video_path: dest,
            video_duration: video_dur,
            segment_duration,
            trim_start: 0.0,
            preview_frames: Vec::new(),
            frames_raw,
            playback_frames: Vec::new(),
            playback_raw,
            playback_fps: 10.0,
            playback_frame_idx: 0,
            playback_last_tick: None,
            playback_for_trim: 0.0,
        });
        return;
    }

    // Закриваємо попередній trim редактор якщо він був відкритий
    state.trim_edit = None;

    // Починаємо завантаження
    let progress = Arc::new(Mutex::new(0.0f32));
    let progress_c = Arc::clone(&progress);
    let url = vid.download_url.clone();
    let dest_path = dest.clone();
    let ctx_c = ctx.clone();

    let _ = std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")));
    std::thread::spawn(move || {
        let _ = crate::api::stock::download_file_with_progress(&url, &dest_path, Some(&progress_c));
        ctx_c.request_repaint();
    });

    state.video_download = Some(VideoDownloadState {
        segment_idx: seg_idx,
        video_id: vid.id.clone(),
        video_duration: vid.duration_secs as f32,
        filename,
        dest_path: dest,
        progress,
    });

    let _ = action;
}

fn start_photo_download(
    ctx: &egui::Context,
    state: &mut StockPickerState,
    seg_idx: usize,
    photo: &crate::api::stock::CachedPhoto,
    action: &mut StockPickerAction,
) {
    let ext = photo.original_url
        .split('?').next().unwrap_or("")
        .rsplit('.').next()
        .filter(|e| e.len() <= 4)
        .unwrap_or("jpg");
    let filename = format!("{:04}.{}", seg_idx + 1, ext);
    let dest = Path::new(&state.save_path).join("media").join(&filename);
    let url = photo.original_url.clone();
    let id = photo.id.clone();
    let save_path = state.save_path.clone();
    let ctx_c = ctx.clone();
    let fname = filename.clone();

    if let Some(s) = state.cache.get_mut(seg_idx) {
        s.selected = Some(SelectedMedia {
            kind: "photo".to_string(),
            id,
            url: url.clone(),
            filename: fname.clone(),
            trim_start: 0.0,
            trim_end: 0.0,
        });
        let _ = save_cache(Path::new(&save_path), &state.cache);
    }

    std::thread::spawn(move || {
        let _ = std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")));
        let _ = crate::api::stock::download_file(&url, &dest);
        ctx_c.request_repaint();
    });

    *action = StockPickerAction::Confirmed;
}

// ─── Thumbnail ───────────────────────────────────────────────────────────────

fn draw_thumb(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut StockPickerState,
    rect: egui::Rect,
    resp: &egui::Response,
    thumb_url: &str,
) {
    if !ui.is_rect_visible(rect) { return; }
    let tex_opt = state.thumbnails.get(thumb_url);
    match tex_opt {
        Some(Some(tex)) => {
            ui.painter().image(tex.id(), rect,
                egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                egui::Color32::WHITE);
        }
        Some(None) => {
            ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(60));
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "✗",
                egui::FontId::default(), egui::Color32::GRAY);
        }
        None => {
            ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(40));
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "⏳",
                egui::FontId::default(), egui::Color32::GRAY);
            if !state.thumb_loading.contains_key(thumb_url) {
                let arc = Arc::new(Mutex::new(None::<Option<egui::ColorImage>>));
                let arc_c = Arc::clone(&arc);
                let url_c = thumb_url.to_string();
                let ctx_c = ctx.clone();
                std::thread::spawn(move || {
                    let res = (|| -> Result<egui::ColorImage, String> {
                        let r = ureq::get(&url_c).call()
                            .map_err(|e| e.to_string())?;
                        use std::io::Read;
                        let mut bytes = Vec::new();
                        r.into_reader().read_to_end(&mut bytes)
                            .map_err(|e| e.to_string())?;
                        let img = image::load_from_memory(&bytes)
                            .map_err(|e| e.to_string())?;
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        Ok(egui::ColorImage::from_rgba_unmultiplied(size, &rgba))
                    })();

                    match res {
                        Ok(ci) => {
                            *arc_c.lock().unwrap() = Some(Some(ci));
                        }
                        Err(_) => {
                            *arc_c.lock().unwrap() = Some(None);
                        }
                    }
                    ctx_c.request_repaint();
                });
                state.thumb_loading.insert(thumb_url.to_string(), arc);
            }
        }
    }
    if resp.hovered() {
        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
        // Збільшений preview у tooltip
        if let Some(Some(tex)) = state.thumbnails.get(thumb_url) {
            let tex = tex.clone();
            let layer_id = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("stock_preview_layer"));
            egui::show_tooltip_at_pointer(ctx, layer_id, egui::Id::new("stock_thumb_preview"), |ui: &mut egui::Ui| {
                let preview_w = 320.0_f32;
                let (w, h) = (tex.size()[0] as f32, tex.size()[1] as f32);
                let aspect = if h > 0.0 { w / h } else { 16.0 / 9.0 };
                ui.image(egui::load::SizedTexture::new(tex.id(), egui::vec2(preview_w, preview_w / aspect)));
            });
        }
    }
}

fn flush_thumb_loading(ctx: &egui::Context, state: &mut StockPickerState) {
    let ready: Vec<(String, Option<egui::ColorImage>)> = state.thumb_loading.iter()
        .filter_map(|(url, arc)| {
            let mut guard = arc.try_lock().ok()?;
            let opt = guard.take()?;
            Some((url.clone(), opt))
        })
        .collect();
    for (url, opt_ci) in ready {
        state.thumb_loading.remove(&url);
        if let Some(ci) = opt_ci {
            let tex = ctx.load_texture(&url, ci, egui::TextureOptions::default());
            state.thumbnails.insert(url, Some(tex));
        } else {
            state.thumbnails.insert(url, None);
        }
    }
}
