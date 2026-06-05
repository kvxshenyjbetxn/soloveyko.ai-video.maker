use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use eframe::egui;
use egui::{Align2, Color32, Frame, Layout, Pos2, Rect, ScrollArea, Sense, Stroke, Vec2};
use crate::localization::{Language, translate};

// ─── Типи кліпів ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum ClipKind {
    Video,
    Image,
    Audio,
}

// ─── Режим перетягування кліпу ───────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragMode {
    Move,
    TrimLeft,
    TrimRight,
}

// ─── Стан перетягування кліпу ────────────────────────────────────────────────

pub struct ClipDragState {
    pub clip_id: String,
    pub mode: DragMode,
    pub initial_start: f32,
    pub initial_duration: f32,
    pub initial_mouse_x: f32,
    pub initial_track_idx: usize,
}

// ─── Кліп на таймлінії ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct EditorClip {
    pub id: String,
    pub path: Option<PathBuf>,
    pub name: String,
    pub start_secs: f32,
    pub duration: f32,
    pub track_idx: usize,
    pub kind: ClipKind,
}

impl EditorClip {
    pub fn end_secs(&self) -> f32 {
        self.start_secs + self.duration
    }
}

// ─── Медіа-файл у пулі ───────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MediaItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub duration_secs: f32,
    pub kind: ClipKind,
}

impl MediaItem {
    pub fn from_path(path: PathBuf) -> Self {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_image = matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp");
        let is_video = matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm");
        let is_audio = matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac");

        let kind = if is_video {
            ClipKind::Video
        } else if is_audio {
            ClipKind::Audio
        } else {
            ClipKind::Image
        };

        // Зображення завжди 5 секунд; для відео/аудіо пробуємо ffprobe
        let duration_secs = if is_image {
            5.0
        } else if is_video || is_audio {
            probe_duration(&path).unwrap_or(10.0)
        } else {
            5.0
        };

        Self {
            id: uuid_str(),
            path,
            name,
            duration_secs,
            kind,
        }
    }
}

/// Отримує тривалість медіафайлу через ffprobe
fn probe_duration(path: &Path) -> Option<f32> {
    let out = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration",
               "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(path)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<f32>().ok()
}

fn uuid_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", t.as_nanos(), rand_u32())
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    Instant::now().hash(&mut h);
    h.finish() as u32
}

// ─── Стан редактора ───────────────────────────────────────────────────────────

pub struct MontageEditorState {
    pub job_name: String,
    pub save_path: PathBuf,
    pub media_pool: Vec<MediaItem>,
    pub clips: Vec<EditorClip>,
    pub num_tracks: usize,
    pub audio_path: Option<PathBuf>,
    pub total_duration: f32,
    pub playhead: f32,
    pub is_playing: bool,
    pub last_frame_time: Instant,
    pub timeline_zoom: f32,
    pub selected_clip_id: Option<String>,
    pub dragged_media_id: Option<String>,
    pub drop_target_track: Option<usize>,
    pub clip_drag_state: Option<ClipDragState>,
    // Preview — LRU кеш текстур + фоновий loader
    pub frame_cache: std::collections::HashMap<String, egui::TextureHandle>,
    /// (cache_key, Result) де None = завантаження провалилось
    pub pending_frame: Arc<Mutex<Option<(String, Option<egui::TextureHandle>)>>>,
    /// Ключ що зараз завантажується; None = вільно
    pub loading_key: Option<String>,
}

impl MontageEditorState {
    pub fn load(save_path: &Path, job_name: &str) -> Self {
        let (clips, total_duration) = load_timeline_clips(save_path);
        let audio_path = find_audio_file(save_path);
        let media_pool = load_media_pool(save_path);
        let num_tracks = clips.iter().map(|c| c.track_idx + 1).max().unwrap_or(1).max(2);

        Self {
            job_name: job_name.to_string(),
            save_path: save_path.to_path_buf(),
            media_pool,
            clips,
            num_tracks,
            audio_path,
            total_duration: total_duration.max(10.0),
            playhead: 0.0,
            is_playing: false,
            last_frame_time: Instant::now(),
            timeline_zoom: 60.0,
            selected_clip_id: None,
            dragged_media_id: None,
            drop_target_track: None,
            clip_drag_state: None,
            frame_cache: std::collections::HashMap::new(),
            pending_frame: Arc::new(Mutex::new(None)),
            loading_key: None,
        }
    }

    pub fn total_dur(&self) -> f32 {
        let clip_end = self.clips.iter().map(|c| c.end_secs()).fold(0.0f32, f32::max);
        clip_end.max(self.total_duration).max(10.0)
    }

    /// Зберігає поточний стан кліпів у timeline.json для run_montage.
    /// Вставляє null-сегменти для проміжків між кліпами, щоб run_montage
    /// правильно відтворював абсолютні позиції (hold last frame для пустих місць).
    pub fn save_to_timeline(&self) -> Result<(), std::io::Error> {
        // Беремо тільки кліпи трека 0 із встановленим шляхом, сортуємо за стартом
        let mut sorted: Vec<&EditorClip> = self.clips.iter()
            .filter(|c| c.track_idx == 0 && c.path.is_some())
            .collect();
        sorted.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));

        let total_duration_secs = sorted.iter()
            .map(|c| c.end_secs() as f64)
            .fold(0.0f64, f64::max)
            .max(self.total_dur() as f64);

        let mut segments: Vec<serde_json::Value> = Vec::new();
        // cursor — поточна позиція на таймлінії після останнього записаного сегменту
        let mut cursor = 0.0f32;

        for clip in &sorted {
            // Якщо кліп перекриває попередній — починаємо його з кінця попереднього (зберігаємо тривалість)
            let actual_start = clip.start_secs.max(cursor);

            // Проміжок (gap) між попереднім і поточним кліпом → null-сегмент (run_montage утримає останній кадр)
            if actual_start > cursor + 0.01 {
                segments.push(serde_json::json!({
                    "start_secs": cursor as f64,
                    "end_secs": actual_start as f64,
                    "media": serde_json::Value::Null,
                }));
            }

            let media_rel = clip.path.as_ref()
                .and_then(|p| {
                    // Спочатку пробуємо strip_prefix без канонізації
                    if let Ok(rel) = p.strip_prefix(&self.save_path) {
                        return Some(rel.to_string_lossy().replace('\\', "/"));
                    }
                    // Fallback: канонізуємо обидва шляхи та пробуємо знову
                    let canon_clip = std::fs::canonicalize(p).ok();
                    let canon_save = std::fs::canonicalize(&self.save_path).ok();
                    if let (Some(cc), Some(cs)) = (canon_clip, canon_save) {
                        cc.strip_prefix(&cs).ok()
                            .map(|r| r.to_string_lossy().replace('\\', "/"))
                    } else {
                        None
                    }
                });

            let actual_end = actual_start + clip.duration;
            segments.push(serde_json::json!({
                "start_secs": actual_start as f64,
                "end_secs": actual_end as f64,
                "media": media_rel,
            }));

            cursor = actual_end;
        }

        let json = serde_json::json!({
            "total_duration_secs": total_duration_secs,
            "segments": segments,
        });

        let timeline_path = self.save_path.join("timeline.json");
        let content = serde_json::to_string_pretty(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(timeline_path, content)
    }
}

// ─── Завантаження даних ───────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct SegTiming {
    start_secs: f64,
    end_secs: f64,
    media: Option<String>,
}

#[derive(serde::Deserialize)]
struct TimelineJson {
    total_duration_secs: f64,
    segments: Vec<SegTiming>,
}

fn load_timeline_clips(save_path: &Path) -> (Vec<EditorClip>, f32) {
    let path = save_path.join("timeline.json");
    if !path.exists() { return (Vec::new(), 10.0); }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let tl: TimelineJson = match serde_json::from_str(&content) {
        Ok(t) => t,
        Err(_) => return (Vec::new(), 10.0),
    };
    let total = tl.total_duration_secs as f32;
    let mut clips = Vec::new();
    for seg in &tl.segments {
        if let Some(ref media) = seg.media {
            let full_path = save_path.join(media);
            let name = full_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(media)
                .to_string();
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            let kind = if matches!(ext.as_str(), "mp4" | "mov" | "webm") {
                ClipKind::Video
            } else {
                ClipKind::Image
            };
            let start = seg.start_secs as f32;
            let dur = (seg.end_secs - seg.start_secs) as f32;
            clips.push(EditorClip {
                id: uuid_str(),
                path: Some(full_path),
                name,
                start_secs: start,
                duration: dur.max(0.1),
                track_idx: 0,
                kind,
            });
        }
    }
    (clips, total)
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
    if !media_dir.exists() { return Vec::new(); }
    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&media_dir) {
        for entry in entries.filter_map(Result::ok) {
            let p = entry.path();
            if p.is_file() {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if matches!(ext.as_str(), "mp4" | "mov" | "webm" | "jpg" | "jpeg" | "png" | "webp" | "mp3" | "wav") {
                    items.push(MediaItem::from_path(p));
                }
            }
        }
    }
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_path.join(name);
        if p.exists() { items.push(MediaItem::from_path(p)); }
    }
    items
}

// ─── Головне вікно редактора ──────────────────────────────────────────────────

pub fn draw_montage_editor_window(
    ctx: &egui::Context,
    language: Language,
    open_job: &mut Option<u64>,
    state: &mut Option<MontageEditorState>,
    jobs: &[crate::queue::PipelineJob],
) {
    let job_id = match *open_job {
        Some(id) => id,
        None => return,
    };

    if state.is_none() {
        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
            *state = Some(MontageEditorState::load(
                std::path::Path::new(&job.settings.save_path),
                &job.name,
            ));
        } else {
            *open_job = None;
            return;
        }
    }

    let editor = match state {
        Some(s) => s,
        None => return,
    };

    if editor.is_playing {
        let elapsed = editor.last_frame_time.elapsed().as_secs_f32();
        editor.playhead = (editor.playhead + elapsed).min(editor.total_dur());
        editor.last_frame_time = Instant::now();
        if editor.playhead >= editor.total_dur() {
            editor.is_playing = false;
            editor.playhead = 0.0;
        }
        ctx.request_repaint();
    }

    let title = format!(
        "{}: {} #{}", translate(language, "montage_editor_title"),
        editor.job_name, job_id + 1
    );
    let mut is_open = true;
    let mut close_after = false;

    let is_awaiting = jobs.iter().find(|j| j.id == job_id).map(|j| {
        *j.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMontageControl
    }).unwrap_or(false);

    egui::Window::new(&title)
        .id(egui::Id::new("montage_editor_window"))
        .open(&mut is_open)
        .resizable(true)
        .default_size([1100.0, 680.0])
        .min_size([700.0, 480.0])
        .collapsible(false)
        .show(ctx, |ui| {
            if draw_topbar(ui, language, editor, is_awaiting, job_id, jobs) {
                close_after = true;
            }
            ui.separator();

            let available = ui.available_size();
            let bottom_h = 200.0;
            let top_h = (available.y - bottom_h - 8.0).max(100.0);

            ui.horizontal(|ui| {
                ui.set_max_height(top_h);

                egui::SidePanel::left("editor_media_pool")
                    .resizable(true)
                    .default_width(220.0)
                    .min_width(160.0)
                    .frame(Frame::none().fill(Color32::from_rgb(18, 18, 20)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_media_pool(ui, language, editor);
                    });

                egui::SidePanel::right("editor_inspector")
                    .resizable(true)
                    .default_width(240.0)
                    .min_width(180.0)
                    .frame(Frame::none().fill(Color32::from_rgb(18, 18, 20)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_inspector(ui, language, editor);
                    });

                egui::CentralPanel::default()
                    .frame(Frame::none().fill(Color32::from_rgb(10, 10, 12)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_preview(ui, editor);
                    });
            });

            ui.separator();

            ui.allocate_ui(Vec2::new(ui.available_width(), bottom_h), |ui| {
                draw_timeline(ui, language, editor);
            });
        });

    if !is_open || close_after {
        *open_job = None;
        *state = None;
    }
}

// ─── Топ-бар ─────────────────────────────────────────────────────────────────

fn draw_topbar(
    ui: &mut egui::Ui,
    language: Language,
    editor: &mut MontageEditorState,
    is_awaiting: bool,
    job_id: u64,
    jobs: &[crate::queue::PipelineJob],
) -> bool {
    let mut continue_clicked = false;
    ui.horizontal(|ui| {
        ui.label(translate(language, "montage_editor_zoom"));
        ui.add(egui::Slider::new(&mut editor.timeline_zoom, 10.0..=300.0).show_value(false));

        let total_dur = editor.total_dur();
        let dm = (total_dur / 60.0) as u32;
        let ds = (total_dur % 60.0) as u32;
        ui.label(
            egui::RichText::new(format!("{} кліпів | {:02}:{:02}", editor.clips.len(), dm, ds))
                .weak().size(11.0)
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let btn = ui.add_enabled(
                is_awaiting,
                egui::Button::new(
                    egui::RichText::new(translate(language, "montage_editor_continue"))
                        .strong()
                        .color(if is_awaiting { Color32::WHITE } else { Color32::GRAY }),
                )
                .fill(if is_awaiting { Color32::from_rgb(39, 174, 96) } else { Color32::from_rgb(30, 30, 35) })
            );
            if btn.clicked() {
                let job_name = editor.job_name.clone();
                let save_path_str = editor.save_path.display().to_string();
                let clip_count = editor.clips.iter().filter(|c| c.track_idx == 0).count();
                match editor.save_to_timeline() {
                    Ok(_) => crate::logger::log_job(
                        job_id, &job_name,
                        &format!("Montage editor: timeline saved ({} clips, path: {})", clip_count, save_path_str),
                    ),
                    Err(e) => crate::logger::log_job(
                        job_id, &job_name,
                        &format!("Montage editor: SAVE FAILED: {} (path: {})", e, save_path_str),
                    ),
                }
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    let (lock, cvar) = &*job.montage_control_resume;
                    *lock.lock().unwrap() = true;
                    cvar.notify_one();
                }
                continue_clicked = true;
            }
        });
    });
    continue_clicked
}

// ─── Медіа-пул ───────────────────────────────────────────────────────────────

fn draw_media_pool(ui: &mut egui::Ui, language: Language, editor: &mut MontageEditorState) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("📁 {}", translate(language, "montage_editor_media_pool"))).strong());
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(translate(language, "montage_editor_add_media")).clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Media", &["mp4", "mov", "webm", "jpg", "jpeg", "png", "webp", "mp3", "wav"])
                    .pick_files()
                {
                    for path in paths {
                        if !editor.media_pool.iter().any(|m| m.path == path) {
                            editor.media_pool.push(MediaItem::from_path(path));
                        }
                    }
                }
            }
        });
    });
    ui.separator();

    ScrollArea::vertical().id_salt("editor_pool_scroll").show(ui, |ui| {
        if editor.media_pool.is_empty() {
            ui.weak("Медіа файли відсутні");
            return;
        }
        let mut to_remove: Option<usize> = None;
        for (idx, media) in editor.media_pool.iter().enumerate() {
            let item_w = (ui.available_width() - 30.0).max(80.0);
            ui.horizontal(|ui| {
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_w, 26.0), Sense::click_and_drag());

                if resp.drag_started() {
                    editor.dragged_media_id = Some(media.id.clone());
                }
                if resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }

                let is_dragged = editor.dragged_media_id.as_deref() == Some(media.id.as_str());
                let is_hovered = resp.hovered();

                let bg = if is_dragged {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 40)
                } else if is_hovered {
                    Color32::from_rgb(38, 38, 42)
                } else {
                    Color32::from_rgb(28, 28, 30)
                };
                let stroke_col = if is_dragged { Color32::from_rgb(9, 123, 244) } else { Color32::from_rgb(45, 45, 50) };
                ui.painter().rect(rect, 4.0, bg, Stroke::new(1.0, stroke_col));

                let icon = match media.kind {
                    ClipKind::Video => "🎥",
                    ClipKind::Image => "🖼",
                    ClipKind::Audio => "🎵",
                };
                let dur_text = format!("{:.1}s", media.duration_secs);
                let display = if media.name.chars().count() > 16 {
                    format!("{} {}… {}", icon, media.name.chars().take(13).collect::<String>(), dur_text)
                } else {
                    format!("{} {} {}", icon, media.name, dur_text)
                };
                let text_col = if is_dragged { Color32::from_rgb(9, 123, 244) } else { Color32::from_rgb(200, 200, 205) };
                ui.painter().text(
                    Pos2::new(rect.left() + 6.0, rect.top() + 6.0),
                    Align2::LEFT_TOP, &display,
                    egui::FontId::proportional(11.0), text_col,
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").clicked() {
                        to_remove = Some(idx);
                    }
                });
            });
            ui.add_space(2.0);
        }
        if let Some(idx) = to_remove {
            editor.media_pool.remove(idx);
        }
    });

    // Плаваюча картка при drag
    if let Some(ref drag_id) = editor.dragged_media_id.clone() {
        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag_card"));
                let painter = ui.ctx().layer_painter(layer);
                let card_w = 140.0;
                let card_h = 28.0;
                let card_rect = Rect::from_min_size(
                    Pos2::new(pos.x + 10.0, pos.y - card_h / 2.0),
                    Vec2::new(card_w, card_h),
                );
                painter.rect(card_rect, 4.0,
                    Color32::from_rgb(28, 36, 52),
                    Stroke::new(1.5, Color32::from_rgb(9, 123, 244)));
                let icon = match media.kind { ClipKind::Video => "🎥", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                let label = if media.name.chars().count() > 12 {
                    format!("{} {}…", icon, media.name.chars().take(10).collect::<String>())
                } else {
                    format!("{} {}", icon, media.name)
                };
                painter.text(
                    card_rect.center(),
                    Align2::CENTER_CENTER, &label,
                    egui::FontId::proportional(11.0), Color32::WHITE,
                );
            }
        }
    }
}

// ─── Preview ──────────────────────────────────────────────────────────────────

/// Обчислює ключ кешу для поточної позиції плейхеду на кліпі.
/// Зображення: один ключ на кліп. Відео: ключ з кроком 0.1s.
fn preview_cache_key(clip: &EditorClip, clip_offset: f32) -> String {
    match clip.kind {
        ClipKind::Image => format!("img-{}", clip.id),
        ClipKind::Video => format!("vid-{}-{}", clip.id, (clip_offset * 10.0) as u32),
        ClipKind::Audio => String::new(),
    }
}

fn draw_preview(ui: &mut egui::Ui, editor: &mut MontageEditorState) {
    let ph = editor.playhead;

    // Активний кліп трека 0 під плейхедом
    let active = editor.clips.iter()
        .filter(|c| c.track_idx == 0 && c.start_secs <= ph && ph < c.end_secs())
        .last()
        .cloned();

    let cache_key = active.as_ref().map(|c| {
        let off = (ph - c.start_secs).max(0.0);
        preview_cache_key(c, off)
    }).unwrap_or_default();

    // ── Дренуємо pending результат із фонового потоку ────────────────────────
    {
        let mut pend = editor.pending_frame.lock().unwrap();
        if let Some((key, tex_opt)) = pend.take() {
            if let Some(tex) = tex_opt {
                // LRU: обмежуємо кеш 100 записами
                if editor.frame_cache.len() >= 100 {
                    if let Some(oldest) = editor.frame_cache.keys().next().cloned() {
                        editor.frame_cache.remove(&oldest);
                    }
                }
                editor.frame_cache.insert(key, tex);
            }
            editor.loading_key = None;
        }
    }

    // ── Тригеримо завантаження якщо кліп є, кеш-міс, і не зайняте ──────────
    if !cache_key.is_empty()
        && !editor.frame_cache.contains_key(&cache_key)
        && editor.loading_key.is_none()
    {
        if let Some(ref clip) = active {
            if let Some(path) = clip.path.clone() {
                let off = (ph - clip.start_secs).max(0.0);
                let kind = clip.kind.clone();
                let key = cache_key.clone();
                let pending = Arc::clone(&editor.pending_frame);
                let ctx = ui.ctx().clone();
                editor.loading_key = Some(key.clone());
                std::thread::spawn(move || {
                    let tex = load_frame_bg(&path, off, &kind, &ctx);
                    *pending.lock().unwrap() = Some((key, tex));
                    ctx.request_repaint();
                });
            }
        }
    }

    let current_tex = editor.frame_cache.get(&cache_key).cloned();
    let is_loading = editor.loading_key.is_some();

    // ── Layout ───────────────────────────────────────────────────────────────
    const TRANSPORT_H: f32 = 44.0;
    const LABEL_H: f32 = 18.0;

    let avail_w = ui.available_width();
    let avail_h = (ui.available_height() - TRANSPORT_H - LABEL_H - 10.0).max(40.0);
    let frame_w = (avail_h * 16.0 / 9.0).min(avail_w - 4.0);
    let frame_h = frame_w * 9.0 / 16.0;
    let pad_x = ((avail_w - frame_w) / 2.0).max(0.0);

    // Заголовок
    ui.label(egui::RichText::new("📺 Попередній перегляд").size(11.0).weak());

    // Відео-фрейм (горизонтальний відступ = center)
    ui.horizontal(|ui| {
        ui.add_space(pad_x);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(frame_w, frame_h), Sense::hover());

        ui.painter_at(rect).rect_filled(rect, 4.0, Color32::from_rgb(5, 5, 7));

        if let Some(ref tex) = current_tex {
            let sz = tex.size_vec2();
            let scale = (rect.width() / sz.x).min(rect.height() / sz.y);
            let img_rect = Rect::from_center_size(rect.center(), sz * scale);
            ui.painter_at(rect).image(
                tex.id(), img_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            // Маленький індикатор оновлення у куті (не перекриває кадр)
            if is_loading {
                let dot = Pos2::new(rect.right() - 10.0, rect.top() + 10.0);
                ui.painter_at(rect).circle_filled(dot, 5.0, Color32::from_rgba_unmultiplied(9, 123, 244, 220));
            }
        } else if is_loading {
            // Перше завантаження — спіннер у центрі
            ui.put(rect, egui::Spinner::new().size(36.0));
        } else {
            // Нема кліпу під плейхедом
            ui.painter_at(rect).text(
                rect.center() - Vec2::new(0.0, 12.0), Align2::CENTER_CENTER,
                "🎬", egui::FontId::proportional(36.0), Color32::from_rgb(40, 40, 52),
            );
            ui.painter_at(rect).text(
                rect.center() + Vec2::new(0.0, 24.0), Align2::CENTER_CENTER,
                "Немає медіа під плейхедом",
                egui::FontId::proportional(10.0), Color32::from_rgb(70, 70, 88),
            );
        }
    });

    ui.add_space(4.0);

    // ── Транспортні кнопки (під відео, по центру) ────────────────────────────
    let total = editor.total_dur();
    ui.horizontal(|ui| {
        // Центруємо групу кнопок
        let group_w = 260.0;
        ui.add_space(((avail_w - group_w) / 2.0).max(0.0));

        // ⏮ -0.1s
        if ui.button("⏮").on_hover_text("-0.1s").clicked() {
            editor.playhead = (editor.playhead - 0.1).max(0.0);
        }
        // ⏹ Стоп
        if ui.button("⏹").clicked() {
            editor.is_playing = false;
            editor.playhead = 0.0;
        }
        // ▶ / ⏸
        let play_lbl = egui::RichText::new(if editor.is_playing { "⏸" } else { "▶" }).size(16.0);
        if ui.button(play_lbl).clicked() {
            editor.is_playing = !editor.is_playing;
            editor.last_frame_time = Instant::now();
        }
        // ⏭ +0.1s
        if ui.button("⏭").on_hover_text("+0.1s").clicked() {
            editor.playhead = (editor.playhead + 0.1).min(total);
        }

        ui.add_space(8.0);

        // Таймкод
        let m = (editor.playhead / 60.0) as u32;
        let s = (editor.playhead % 60.0) as u32;
        let cs = ((editor.playhead % 1.0) * 100.0) as u32;
        ui.label(
            egui::RichText::new(format!("{:02}:{:02}.{:02}", m, s, cs))
                .monospace().size(12.0).color(Color32::from_rgb(210, 210, 230))
        );
        let tm = (total / 60.0) as u32;
        let ts = (total % 60.0) as u32;
        ui.label(egui::RichText::new(format!("/ {:02}:{:02}", tm, ts)).size(10.0).weak());
    });
}

/// Завантажує один кадр у фоновому потоці.
/// Зображення → `image` crate (без ffmpeg). Відео → ffmpeg PPM.
fn load_frame_bg(
    path: &Path,
    offset_secs: f32,
    kind: &ClipKind,
    ctx: &egui::Context,
) -> Option<egui::TextureHandle> {
    match kind {
        ClipKind::Image => {
            let img = image::open(path).ok()?;
            let img = img.thumbnail(800, 800);
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let ci = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize], &rgba.into_raw(),
            );
            Some(ctx.load_texture(
                format!("img_{}", path.to_string_lossy()),
                ci, egui::TextureOptions::LINEAR,
            ))
        }
        ClipKind::Video => {
            // ffmpeg → PPM (самоописний формат, не потрібно знати розмір заздалегідь)
            let out = std::process::Command::new("ffmpeg")
                .arg("-ss").arg(format!("{:.3}", offset_secs))
                .arg("-i").arg(path)
                .arg("-vf").arg("scale=800:-2")
                .arg("-frames:v").arg("1")
                .arg("-f").arg("image2pipe")
                .arg("-vcodec").arg("ppm")
                .arg("-loglevel").arg("error")
                .arg("-")
                .output()
                .ok()?;
            if out.stdout.is_empty() { return None; }
            let (w, h, offset) = parse_ppm_header(&out.stdout)?;
            let pixels = &out.stdout[offset..];
            if pixels.len() < w * h * 3 { return None; }
            let rgba: Vec<u8> = pixels[..w * h * 3].chunks_exact(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255u8])
                .collect();
            let ci = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);
            Some(ctx.load_texture(
                format!("vid_{}", path.to_string_lossy()),
                ci, egui::TextureOptions::LINEAR,
            ))
        }
        ClipKind::Audio => None,
    }
}

fn parse_ppm_header(data: &[u8]) -> Option<(usize, usize, usize)> {
    let mut i = 0;
    if data.get(i..i + 2)? != b"P6" { return None; }
    i += 2;
    i = ppm_skip(data, i);
    let (w, ni) = ppm_num(data, i)?; i = ni;
    i = ppm_skip(data, i);
    let (h, ni) = ppm_num(data, i)?; i = ni;
    i = ppm_skip(data, i);
    let (_, ni) = ppm_num(data, i)?; i = ni;   // max value
    if i < data.len() { i += 1; }               // один whitespace після max
    Some((w, h, i))
}

fn ppm_skip(data: &[u8], mut i: usize) -> usize {
    loop {
        while i < data.len() && data[i].is_ascii_whitespace() { i += 1; }
        if i < data.len() && data[i] == b'#' {
            while i < data.len() && data[i] != b'\n' { i += 1; }
        } else { break; }
    }
    i
}

fn ppm_num(data: &[u8], mut i: usize) -> Option<(usize, usize)> {
    let s = i;
    while i < data.len() && data[i].is_ascii_digit() { i += 1; }
    if i == s { return None; }
    Some((std::str::from_utf8(&data[s..i]).ok()?.parse().ok()?, i))
}

// ─── Інспектор ───────────────────────────────────────────────────────────────

fn draw_inspector(ui: &mut egui::Ui, language: Language, editor: &mut MontageEditorState) {
    ui.label(egui::RichText::new(format!("⚙ {}", translate(language, "montage_editor_inspector"))).strong());
    ui.separator();

    let sel_id = editor.selected_clip_id.clone();
    if let Some(ref id) = sel_id {
        if let Some(idx) = editor.clips.iter().position(|c| c.id == *id) {
            let num_tracks = editor.num_tracks;
            let clip = &mut editor.clips[idx];

            ui.label(egui::RichText::new(&clip.name).size(12.0).strong());
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_start"));
                ui.add(egui::DragValue::new(&mut clip.start_secs).speed(0.05).range(0.0..=3600.0));
            });

            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_dur"));
                ui.add(egui::DragValue::new(&mut clip.duration).speed(0.05).range(0.1..=3600.0));
            });

            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_track"));
                let mut t = clip.track_idx as i32;
                if ui.add(egui::DragValue::new(&mut t).speed(1.0).range(0..=(num_tracks as i32 - 1))).changed() {
                    clip.track_idx = t as usize;
                }
            });

            ui.add_space(8.0);
            if ui.button(translate(language, "montage_editor_delete_clip")).clicked() {
                editor.clips.remove(idx);
                editor.selected_clip_id = None;
            }
        } else {
            editor.selected_clip_id = None;
        }
    } else {
        ui.weak(translate(language, "montage_editor_no_selection"));
    }

    ui.add_space(12.0);
    ui.separator();

    if ui.button(translate(language, "montage_editor_add_track")).clicked() {
        editor.num_tracks += 1;
    }
}

// ─── Таймлінія ───────────────────────────────────────────────────────────────

fn draw_timeline(ui: &mut egui::Ui, _language: Language, editor: &mut MontageEditorState) {
    let track_h = 40.0;
    let ruler_h = 22.0;
    let label_w = 70.0;
    let total_dur = editor.total_dur();
    let zoom = editor.timeline_zoom;

    ui.horizontal(|ui| {
        // Ліва колонка: назви треків
        ui.vertical(|ui| {
            ui.set_max_width(label_w);
            ui.add_space(ruler_h);

            for track_idx in 0..editor.num_tracks {
                let label = format!("V{}", track_idx + 1);
                ui.allocate_ui(Vec2::new(label_w, track_h), |ui| {
                    Frame::none()
                        .fill(Color32::from_rgb(28, 28, 32))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(42, 42, 48)))
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| { ui.small(&label); });
                        });
                });
                ui.add_space(2.0);
            }

            if editor.audio_path.is_some() {
                ui.add_space(2.0);
                ui.allocate_ui(Vec2::new(label_w, track_h), |ui| {
                    Frame::none()
                        .fill(Color32::from_rgb(22, 32, 26))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(35, 52, 40)))
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| { ui.small("♪ Audio"); });
                        });
                });
            }
        });

        ScrollArea::horizontal().id_salt("editor_timeline_scroll").auto_shrink([false; 2]).show(ui, |ui| {
            let total_audio_tracks = if editor.audio_path.is_some() { 1 } else { 0 };
            let total_tracks_h = ruler_h + (track_h + 2.0) * (editor.num_tracks + total_audio_tracks) as f32;
            let timeline_w = (total_dur + 10.0) * zoom;

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(timeline_w, total_tracks_h), Sense::click_and_drag());
            // painter_at повертає owned Painter (не позичає ui), тому ui.allocate_rect() далі не конфліктує
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 0.0, Color32::from_rgb(14, 14, 17));

            // Лінійка часу
            let ruler_rect = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + ruler_h));
            painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(20, 20, 24));

            let secs_visible = (timeline_w / zoom) as i32 + 2;
            for sec in 0..=secs_visible {
                let x = rect.left() + sec as f32 * zoom;
                let major = sec % 5 == 0;
                let tick_h = if major { 12.0 } else { 6.0 };
                painter.line_segment(
                    [Pos2::new(x, rect.top() + ruler_h - tick_h), Pos2::new(x, rect.top() + ruler_h)],
                    Stroke::new(1.0, Color32::from_rgb(60, 60, 70)),
                );
                if major {
                    let m = (sec / 60) as u32; let s = (sec % 60) as u32;
                    painter.text(
                        Pos2::new(x, rect.top() + 4.0), Align2::CENTER_TOP,
                        format!("{:02}:{:02}", m, s),
                        egui::FontId::proportional(9.0), Color32::from_rgb(110, 110, 120),
                    );
                }
            }

            // Фони треків
            for track_idx in 0..editor.num_tracks {
                let track_y = rect.top() + ruler_h + (track_h + 2.0) * track_idx as f32;
                let track_row = Rect::from_min_size(Pos2::new(rect.left(), track_y), Vec2::new(rect.width(), track_h));
                let track_bg = if editor.drop_target_track == Some(track_idx) {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 20)
                } else {
                    Color32::from_rgb(16, 16, 20)
                };
                painter.rect_filled(track_row, 0.0, track_bg);
                painter.line_segment(
                    [Pos2::new(rect.left(), track_y + track_h), Pos2::new(rect.right(), track_y + track_h)],
                    Stroke::new(1.0, Color32::from_rgb(28, 28, 33)),
                );
            }

            let mouse_pos = ui.input(|i| i.pointer.hover_pos());

            // Ghost-preview при drag з медіа-пулу
            if let Some(ref drag_id) = editor.dragged_media_id.clone() {
                if let Some(pos) = mouse_pos {
                    if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                        let click_t = ((pos.x - rect.left()) / zoom).max(0.0);
                        let rel_y = pos.y - (rect.top() + ruler_h);
                        let t_idx = ((rel_y / (track_h + 2.0)) as usize).min(editor.num_tracks - 1);
                        editor.drop_target_track = Some(t_idx);

                        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
                            let preview_x = rect.left() + click_t * zoom;
                            let preview_w = (media.duration_secs * zoom).max(4.0);
                            let preview_y = rect.top() + ruler_h + (track_h + 2.0) * t_idx as f32 + 2.0;
                            let preview_rect = Rect::from_min_size(
                                Pos2::new(preview_x, preview_y),
                                Vec2::new(preview_w, track_h - 4.0),
                            );
                            painter.rect_filled(preview_rect, 4.0, Color32::from_rgba_unmultiplied(9, 123, 244, 45));
                            painter.rect_stroke(preview_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(9, 123, 244)));
                            painter.text(
                                Pos2::new(preview_rect.left() + 6.0, preview_rect.top() + 4.0),
                                Align2::LEFT_TOP,
                                format!("{:.1}s → V{}", click_t, t_idx + 1),
                                egui::FontId::proportional(10.0), Color32::WHITE,
                            );
                        }
                        ui.ctx().request_repaint();
                    } else {
                        editor.drop_target_track = None;
                    }
                }
            }

            // Drop з медіа-пулу на таймлінію
            if ui.input(|i| !i.pointer.any_down()) {
                if let Some(drag_id) = editor.dragged_media_id.take() {
                    if let Some(pos) = mouse_pos {
                        if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                            let t_idx = {
                                let rel_y = pos.y - (rect.top() + ruler_h);
                                ((rel_y / (track_h + 2.0)) as usize).min(editor.num_tracks - 1)
                            };
                            let start = ((pos.x - rect.left()) / zoom).max(0.0);
                            if let Some(media) = editor.media_pool.iter().find(|m| m.id == drag_id) {
                                let kind = media.kind.clone();
                                let name = media.name.clone();
                                let path = Some(media.path.clone());
                                let duration = media.duration_secs;
                                let new_id = uuid_str();
                                editor.selected_clip_id = Some(new_id.clone());
                                editor.clips.push(EditorClip {
                                    id: new_id,
                                    path,
                                    name,
                                    start_secs: start,
                                    duration,
                                    track_idx: t_idx,
                                    kind,
                                });
                            }
                        }
                    }
                    editor.drop_target_track = None;
                }
            }

            // Обробка перетягування кліпів (move / trim)
            update_clip_drag(ui.ctx(), editor, rect, ruler_h, track_h, zoom);

            // Кліпи
            let clips_snapshot: Vec<EditorClip> = editor.clips.clone();
            for clip in &clips_snapshot {
                let track_y = rect.top() + ruler_h + (track_h + 2.0) * clip.track_idx as f32;
                let cx = rect.left() + clip.start_secs * zoom;
                let cw = (clip.duration * zoom).max(4.0);
                let clip_rect = Rect::from_min_size(
                    Pos2::new(cx, track_y + 2.0),
                    Vec2::new(cw, track_h - 4.0),
                );

                let is_sel = editor.selected_clip_id.as_deref() == Some(clip.id.as_str());
                let (bg, accent) = match clip.kind {
                    ClipKind::Video => (Color32::from_rgb(18, 32, 55), Color32::from_rgb(9, 100, 220)),
                    ClipKind::Image => (Color32::from_rgb(30, 22, 48), Color32::from_rgb(120, 70, 200)),
                    ClipKind::Audio => (Color32::from_rgb(20, 40, 28), Color32::from_rgb(39, 160, 80)),
                };
                let border = if is_sel { Color32::WHITE } else { accent };
                painter.rect(clip_rect, 3.0, bg, Stroke::new(if is_sel { 2.0 } else { 1.2 }, border));

                // Trim-ручки (видимі завжди, акцентовані при виділенні)
                let handle_w = 6.0;
                let handle_col = if is_sel { Color32::WHITE } else { accent.linear_multiply(0.6) };
                // Ліва ручка
                painter.rect_filled(
                    Rect::from_min_size(clip_rect.min, Vec2::new(handle_w, clip_rect.height())),
                    2.0, handle_col,
                );
                // Права ручка
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(clip_rect.max.x - handle_w, clip_rect.min.y),
                        Vec2::new(handle_w, clip_rect.height()),
                    ),
                    2.0, handle_col,
                );

                if cw > 18.0 {
                    let icon = match clip.kind { ClipKind::Video => "🎬", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                    let label = if clip.name.chars().count() > 16 {
                        format!("{} {}…", icon, clip.name.chars().take(13).collect::<String>())
                    } else {
                        format!("{} {}", icon, clip.name)
                    };
                    painter.text(
                        Pos2::new(clip_rect.left() + handle_w + 2.0, clip_rect.top() + 5.0),
                        Align2::LEFT_TOP, &label,
                        egui::FontId::proportional(10.0), Color32::from_rgb(200, 200, 215),
                    );
                    if cw > 50.0 {
                        painter.text(
                            Pos2::new(clip_rect.left() + handle_w + 2.0, clip_rect.top() + 19.0),
                            Align2::LEFT_TOP, format!("{:.1}s", clip.duration),
                            egui::FontId::proportional(9.0), Color32::from_rgb(120, 120, 130),
                        );
                    }
                }

                // Курсор resize біля країв
                if let Some(pos) = mouse_pos {
                    if clip_rect.contains(pos) {
                        let rx = pos.x - clip_rect.left();
                        if rx < 8.0 || rx > clip_rect.width() - 8.0 {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                    }
                }

                // Початок перетягування
                let clip_resp = ui.allocate_rect(clip_rect, Sense::click_and_drag());
                if clip_resp.clicked() {
                    editor.selected_clip_id = Some(clip.id.clone());
                }
                if clip_resp.drag_started() {
                    if let Some(pos) = mouse_pos {
                        let rx = pos.x - clip_rect.left();
                        let mode = if rx < 8.0 {
                            DragMode::TrimLeft
                        } else if rx > clip_rect.width() - 8.0 {
                            DragMode::TrimRight
                        } else {
                            DragMode::Move
                        };
                        editor.clip_drag_state = Some(ClipDragState {
                            clip_id: clip.id.clone(),
                            mode,
                            initial_start: clip.start_secs,
                            initial_duration: clip.duration,
                            initial_mouse_x: pos.x,
                            initial_track_idx: clip.track_idx,
                        });
                        editor.selected_clip_id = Some(clip.id.clone());
                    }
                }
            }

            // Аудіо трек
            if let Some(ref ap) = editor.audio_path {
                let audio_y = rect.top() + ruler_h + (track_h + 2.0) * editor.num_tracks as f32 + 2.0;
                let audio_row = Rect::from_min_size(Pos2::new(rect.left(), audio_y - 2.0), Vec2::new(rect.width(), track_h));
                painter.rect_filled(audio_row, 0.0, Color32::from_rgb(14, 22, 16));
                let audio_w = total_dur * zoom;
                let audio_rect = Rect::from_min_size(Pos2::new(rect.left(), audio_y), Vec2::new(audio_w, track_h - 4.0));
                painter.rect(audio_rect, 3.0, Color32::from_rgb(20, 48, 30), Stroke::new(1.2, Color32::from_rgb(39, 174, 96)));
                if audio_w > 20.0 {
                    let aname = ap.file_name().and_then(|n| n.to_str()).unwrap_or("audio");
                    painter.text(
                        Pos2::new(audio_rect.left() + 6.0, audio_rect.top() + 8.0),
                        Align2::LEFT_TOP, format!("♪ {}", aname),
                        egui::FontId::proportional(11.0), Color32::from_rgb(150, 210, 160),
                    );
                }
            }

            // Плейхед
            let ph_x = rect.left() + editor.playhead * zoom;
            if ph_x >= rect.left() && ph_x <= rect.right() {
                painter.line_segment(
                    [Pos2::new(ph_x, rect.top()), Pos2::new(ph_x, rect.bottom())],
                    Stroke::new(2.0, Color32::from_rgb(9, 123, 244)),
                );
                let ph_top = rect.top() + ruler_h;
                let p1 = Pos2::new(ph_x - 6.0, ph_top - 6.0);
                let p2 = Pos2::new(ph_x + 6.0, ph_top - 6.0);
                let p3 = Pos2::new(ph_x, ph_top);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    Color32::from_rgb(9, 123, 244),
                    Stroke::NONE,
                ));
            }

            // Скраб плейхеда кліком по лінійці
            if let Some(pos) = mouse_pos {
                if ruler_rect.contains(pos) && ui.input(|i| i.pointer.primary_down()) {
                    editor.playhead = ((pos.x - rect.left()) / zoom).clamp(0.0, total_dur);
                }
            }

            // Клік по порожньому місцю — знімаємо виділення
            if resp.clicked() {
                let pos = resp.interact_pointer_pos().unwrap_or_default();
                let hit = editor.clips.iter().any(|clip| {
                    let track_y = rect.top() + ruler_h + (track_h + 2.0) * clip.track_idx as f32;
                    let cx = rect.left() + clip.start_secs * zoom;
                    let cw = clip.duration * zoom;
                    let cr = Rect::from_min_size(Pos2::new(cx, track_y + 2.0), Vec2::new(cw, track_h - 4.0));
                    cr.contains(pos)
                });
                if !hit { editor.selected_clip_id = None; }
            }
        });
    });
}

// ─── Логіка перетягування кліпів (move / trim) ───────────────────────────────

fn update_clip_drag(
    ctx: &egui::Context,
    editor: &mut MontageEditorState,
    rect: Rect,
    ruler_h: f32,
    track_h: f32,
    zoom: f32,
) {
    if ctx.input(|i| i.pointer.any_released()) {
        editor.clip_drag_state = None;
        return;
    }

    let drag = match &editor.clip_drag_state {
        Some(d) => ClipDragState {
            clip_id: d.clip_id.clone(),
            mode: d.mode,
            initial_start: d.initial_start,
            initial_duration: d.initial_duration,
            initial_mouse_x: d.initial_mouse_x,
            initial_track_idx: d.initial_track_idx,
        },
        None => return,
    };

    let pos = match ctx.input(|i| i.pointer.hover_pos()) {
        Some(p) => p,
        None => return,
    };

    let dx = (pos.x - drag.initial_mouse_x) / zoom;

    let clip_idx = match editor.clips.iter().position(|c| c.id == drag.clip_id) {
        Some(idx) => idx,
        None => return,
    };

    let dy = pos.y - (rect.top() + ruler_h);
    let row_idx = (dy / (track_h + 2.0)).max(0.0) as usize;
    let new_track = row_idx.min(editor.num_tracks - 1);

    let raw_start = (drag.initial_start + dx).max(0.0);
    let clip_dur = drag.initial_duration;
    let raw_end = raw_start + clip_dur;

    // Магнітний snap до сусідніх кліпів
    let snap_threshold = 8.0 / zoom;
    let mut best_snap: Option<f32> = None;
    let mut best_dist = snap_threshold;

    for (i, other) in editor.clips.iter().enumerate() {
        if i == clip_idx { continue; }

        let other_end = other.start_secs + other.duration;

        let d1 = (raw_start - other_end).abs();
        if d1 < best_dist { best_dist = d1; best_snap = Some(other_end); }

        let d2 = (raw_start - other.start_secs).abs();
        if d2 < best_dist { best_dist = d2; best_snap = Some(other.start_secs); }

        let d3 = (raw_end - other.start_secs).abs();
        if d3 < best_dist { best_dist = d3; best_snap = Some((other.start_secs - clip_dur).max(0.0)); }
    }

    let snapped_start = best_snap.unwrap_or(raw_start);

    let clip = &mut editor.clips[clip_idx];
    match drag.mode {
        DragMode::Move => {
            clip.start_secs = snapped_start;
            clip.track_idx = new_track;
        }
        DragMode::TrimLeft => {
            let max_dx = drag.initial_duration - 0.1;
            let bounded_dx = dx.clamp(-drag.initial_start, max_dx);
            clip.start_secs = drag.initial_start + bounded_dx;
            clip.duration = drag.initial_duration - bounded_dx;
        }
        DragMode::TrimRight => {
            clip.duration = (drag.initial_duration + dx).max(0.1);
        }
    }

    ctx.request_repaint();
}
