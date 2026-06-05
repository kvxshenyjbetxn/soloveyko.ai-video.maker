use std::io::Read as IoRead;
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

// ─── Кліп на таймлінії ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        let kind = if matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm") {
            ClipKind::Video
        } else if matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac") {
            ClipKind::Audio
        } else {
            ClipKind::Image
        };
        Self {
            id: uuid_str(),
            path,
            name,
            duration_secs: 0.0, // заповнюється окремо якщо є
            kind,
        }
    }
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

#[allow(dead_code)]
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
    /// Підсвічення доріжки при drag-over (track_idx)
    pub drop_target_track: Option<usize>,
    // Preview через ffmpeg
    pub preview_texture: Option<egui::TextureHandle>,
    pub preview_pending: Arc<Mutex<Option<egui::TextureHandle>>>,
    pub preview_loading: Arc<Mutex<bool>>,
    /// Ключ кадру що зараз в preview (щоб не перезапускати той самий)
    pub preview_key: String,
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
            preview_texture: None,
            preview_pending: Arc::new(Mutex::new(None)),
            preview_loading: Arc::new(Mutex::new(false)),
            preview_key: String::new(),
        }
    }

    pub fn total_dur(&self) -> f32 {
        let clip_end = self.clips.iter().map(|c| c.end_secs()).fold(0.0f32, f32::max);
        clip_end.max(self.total_duration).max(10.0)
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
    // Також аудіо з кореня
    for name in &["voice.wav", "voice.mp3"] {
        let p = save_path.join(name);
        if p.exists() { items.push(MediaItem::from_path(p)); }
    }
    items
}

// ─── Головне вікно редактора ──────────────────────────────────────────────────

/// Малює плаваюче вікно редактора монтажу.
/// `jobs` потрібний для завантаження стану та отримання `montage_control_resume`.
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

    // Завантажуємо стан якщо ще не завантажено
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

    // Плейбек
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

    // Перевіряємо чи можна показати кнопку "Продовжити" (пайплайн чекає)
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
            // Топ-бар із транспортом та кнопкою продовжити
            draw_topbar(ui, language, editor, is_awaiting, job_id, jobs);
            ui.separator();

            // Основна область: ліво (пул) + центр (preview) + право (inspector)
            let available = ui.available_size();
            let bottom_h = 200.0; // Висота таймлінії

            // Верхня частина — три панелі
            let top_h = (available.y - bottom_h - 8.0).max(100.0);
            ui.horizontal(|ui| {
                ui.set_max_height(top_h);

                // Ліво: медіа-пул
                egui::SidePanel::left("editor_media_pool")
                    .resizable(true)
                    .default_width(220.0)
                    .min_width(160.0)
                    .frame(Frame::none().fill(Color32::from_rgb(18, 18, 20)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_media_pool(ui, language, editor);
                    });

                // Право: інспектор
                egui::SidePanel::right("editor_inspector")
                    .resizable(true)
                    .default_width(240.0)
                    .min_width(180.0)
                    .frame(Frame::none().fill(Color32::from_rgb(18, 18, 20)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_inspector(ui, language, editor);
                    });

                // Центр: preview
                egui::CentralPanel::default()
                    .frame(Frame::none().fill(Color32::from_rgb(10, 10, 12)).inner_margin(6.0))
                    .show_inside(ui, |ui| {
                        draw_preview(ui, editor);
                    });
            });

            ui.separator();

            // Нижня частина: таймлінія
            ui.allocate_ui(Vec2::new(ui.available_width(), bottom_h), |ui| {
                draw_timeline(ui, language, editor);
            });
        });

    if !is_open {
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
) {
    ui.horizontal(|ui| {
        // Стоп
        if ui.button("⏹").on_hover_text(translate(language, "montage_editor_stop")).clicked() {
            editor.is_playing = false;
            editor.playhead = 0.0;
        }

        // Play / Pause
        let play_label = if editor.is_playing { "⏸" } else { "▶" };
        if ui.button(play_label).clicked() {
            editor.is_playing = !editor.is_playing;
            editor.last_frame_time = Instant::now();
        }

        // Таймкод
        let m = (editor.playhead / 60.0) as u32;
        let s = (editor.playhead % 60.0) as u32;
        let cs = ((editor.playhead % 1.0) * 100.0) as u32;
        ui.label(
            egui::RichText::new(format!("{:02}:{:02}.{:02}", m, s, cs)).monospace()
        );

        ui.separator();

        // Масштаб
        ui.label(translate(language, "montage_editor_zoom"));
        ui.add(egui::Slider::new(&mut editor.timeline_zoom, 10.0..=300.0).show_value(false));

        // Кількість кліпів / тривалість
        let total_dur = editor.total_dur();
        let dm = (total_dur / 60.0) as u32;
        let ds = (total_dur % 60.0) as u32;
        ui.label(
            egui::RichText::new(format!("{} кліпів | {:02}:{:02}", editor.clips.len(), dm, ds))
                .weak().size(11.0)
        );

        // Кнопка "Продовжити" — активна тільки якщо пайплайн чекає
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
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    let (lock, cvar) = &*job.montage_control_resume;
                    *lock.lock().unwrap() = true;
                    cvar.notify_one();
                }
            }
        });
    });
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
                let display = if media.name.chars().count() > 20 {
                    format!("{} {}…", icon, media.name.chars().take(17).collect::<String>())
                } else {
                    format!("{} {}", icon, media.name)
                };
                let text_col = if is_dragged { Color32::from_rgb(9, 123, 244) } else { Color32::from_rgb(200, 200, 205) };
                ui.painter().text(
                    Pos2::new(rect.left() + 6.0, rect.top() + 6.0),
                    Align2::LEFT_TOP, &display,
                    egui::FontId::proportional(11.5), text_col,
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

    // Дроп: якщо відпустили drag над таймлінією — додається через drop_target_track
}

// ─── Preview ──────────────────────────────────────────────────────────────────

fn draw_preview(ui: &mut egui::Ui, editor: &mut MontageEditorState) {
    let ph = editor.playhead;

    // Знаходимо активний кліп на доріжці 0
    let active_clip = editor.clips.iter()
        .filter(|c| c.track_idx == 0 && c.start_secs <= ph && ph < c.end_secs())
        .last()
        .map(|c| (c.id.clone(), c.path.clone(), c.start_secs, c.kind.clone()));

    // Ключ кешу: округляємо до 0.25 сек щоб не спамити ffmpeg при smooth playback
    let new_key = match &active_clip {
        Some((id, _, start, _)) => {
            let offset = ((ph - start) * 4.0) as u32; // кратно 0.25 сек
            format!("{}-{}", id, offset)
        }
        None => String::from("empty"),
    };

    // Дренуємо готовий кадр з фонового потоку
    {
        let mut pending = editor.preview_pending.lock().unwrap();
        if let Some(tex) = pending.take() {
            editor.preview_texture = Some(tex);
            *editor.preview_loading.lock().unwrap() = false;
        }
    }

    // Запускаємо витягування якщо ключ змінився і кліп є
    if new_key != editor.preview_key {
        editor.preview_key = new_key;
        if let Some((_, Some(path), start, kind)) = active_clip.clone() {
            let offset_secs = (ph - start).max(0.0);
            let loading = Arc::clone(&editor.preview_loading);
            let pending = Arc::clone(&editor.preview_pending);
            let ctx = ui.ctx().clone();
            if !*loading.lock().unwrap() {
                *loading.lock().unwrap() = true;
                std::thread::spawn(move || {
                    let tex = extract_frame_at(&path, offset_secs, &ctx, &kind);
                    *pending.lock().unwrap() = tex;
                    ctx.request_repaint();
                });
            }
        } else {
            editor.preview_texture = None;
        }
    }

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("📺 Попередній перегляд").strong().size(12.0));

        let avail = ui.available_size() - Vec2::new(0.0, 40.0);
        let h = avail.y.max(120.0);
        let w = (h * 16.0 / 9.0).min(avail.x);
        let preview_size = Vec2::new(w, w * 9.0 / 16.0);

        let (rect, _) = ui.allocate_exact_size(preview_size, Sense::hover());
        ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(6, 6, 8));

        if let Some(tex) = &editor.preview_texture {
            // Показуємо реальний кадр
            let sz = tex.size_vec2();
            let scale = (rect.width() / sz.x).min(rect.height() / sz.y);
            let disp = sz * scale;
            let img_rect = Rect::from_center_size(rect.center(), disp);
            ui.painter().image(tex.id(), img_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        } else if *editor.preview_loading.lock().unwrap() {
            // Завантажується
            ui.put(rect, egui::Spinner::new().size(28.0));
        } else {
            // Нема кліпу
            ui.painter().text(
                rect.center() - Vec2::new(0.0, 8.0),
                Align2::CENTER_CENTER, "🎬",
                egui::FontId::proportional(30.0), Color32::from_rgb(45, 45, 50),
            );
            ui.painter().text(
                rect.center() + Vec2::new(0.0, 20.0),
                Align2::CENTER_CENTER, "Немає відео у цей момент",
                egui::FontId::proportional(10.0), Color32::from_rgb(80, 80, 90),
            );
        }

        // Спіннер завантаження у куті поверх кадру
        if *editor.preview_loading.lock().unwrap() && editor.preview_texture.is_some() {
            ui.painter().rect_filled(
                Rect::from_min_size(Pos2::new(rect.right() - 26.0, rect.top() + 4.0), Vec2::splat(22.0)),
                4.0, Color32::from_black_alpha(120),
            );
            ui.put(
                Rect::from_center_size(Pos2::new(rect.right() - 15.0, rect.top() + 15.0), Vec2::splat(14.0)),
                egui::Spinner::new().size(14.0),
            );
        }

        // Таймкод
        ui.add_space(4.0);
        let m = (ph / 60.0) as u32;
        let s = (ph % 60.0) as u32;
        let cs = ((ph % 1.0) * 100.0) as u32;
        ui.label(
            egui::RichText::new(format!("{:02}:{:02}.{:02}", m, s, cs)).monospace().size(13.0)
        );
    });
}

/// Витягує один кадр через ffmpeg pipe зі вказаного offset_secs.
/// Для зображень — просто читає як картинку.
fn extract_frame_at(path: &Path, offset_secs: f32, ctx: &egui::Context, kind: &ClipKind) -> Option<egui::TextureHandle> {
    const TARGET_W: u32 = 640;

    match kind {
        ClipKind::Image => {
            // Зображення: читаємо через ffmpeg щоб нормалізувати формат → RGBA
            let out = std::process::Command::new("ffmpeg")
                .arg("-i").arg(path)
                .arg("-vf").arg(format!("scale={}:-2", TARGET_W))
                .arg("-frames:v").arg("1")
                .arg("-f").arg("rawvideo")
                .arg("-pix_fmt").arg("rgba")
                .arg("-loglevel").arg("error")
                .arg("-")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn();
            read_ffmpeg_frame(out, path, ctx, TARGET_W, "img")
        }
        ClipKind::Video => {
            // Відео: seek до потрібної секунди
            let out = std::process::Command::new("ffmpeg")
                .arg("-ss").arg(format!("{:.3}", offset_secs))
                .arg("-i").arg(path)
                .arg("-vf").arg(format!("scale={}:-2", TARGET_W))
                .arg("-frames:v").arg("1")
                .arg("-f").arg("rawvideo")
                .arg("-pix_fmt").arg("rgba")
                .arg("-loglevel").arg("error")
                .arg("-")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn();
            read_ffmpeg_frame(out, path, ctx, TARGET_W, "vid")
        }
        ClipKind::Audio => None,
    }
}

fn read_ffmpeg_frame(
    child: Result<std::process::Child, std::io::Error>,
    path: &Path,
    ctx: &egui::Context,
    target_w: u32,
    prefix: &str,
) -> Option<egui::TextureHandle> {
    let mut child = child.ok()?;

    // Дізнаємося реальні розміри через ffprobe
    let (out_w, out_h) = get_preview_dimensions(path, target_w).unwrap_or((target_w, target_w * 9 / 16));
    let frame_bytes = (out_w * out_h * 4) as usize;

    let mut buf = vec![0u8; frame_bytes];
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_exact(&mut buf).ok()?;
    }
    let _ = child.wait();

    let ci = egui::ColorImage::from_rgba_unmultiplied([out_w as usize, out_h as usize], &buf);
    let name = format!("{}_{}", prefix, path.to_string_lossy());
    Some(ctx.load_texture(name, ci, egui::TextureOptions::LINEAR))
}

fn get_preview_dimensions(path: &Path, target_w: u32) -> Option<(u32, u32)> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-select_streams", "v:0",
               "-show_entries", "stream=width,height",
               "-of", "csv=p=0"])
        .arg(path)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = s.trim().split(',').collect();
    if parts.len() < 2 { return None; }
    let w: u32 = parts[0].trim().parse().ok()?;
    let h: u32 = parts[1].trim().parse().ok()?;
    if w == 0 || h == 0 { return None; }
    let scale = target_w as f32 / w as f32;
    let out_h = ((h as f32 * scale) as u32).max(2) & !1;
    Some((target_w, out_h))
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

            // Аудіо трек
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

        // Горизонтальна область таймлінії
        ScrollArea::horizontal().id_salt("editor_timeline_scroll").auto_shrink([false; 2]).show(ui, |ui| {
            let total_audio_tracks = if editor.audio_path.is_some() { 1 } else { 0 };
            let total_tracks_h = ruler_h + (track_h + 2.0) * (editor.num_tracks + total_audio_tracks) as f32;
            let timeline_w = (total_dur + 4.0) * zoom;

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(timeline_w, total_tracks_h), Sense::click_and_drag());
            let painter = ui.painter();

            // Фон
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

            // Фони треків та кліпи
            for track_idx in 0..editor.num_tracks {
                let track_y = rect.top() + ruler_h + (track_h + 2.0) * track_idx as f32;
                let track_row = Rect::from_min_size(Pos2::new(rect.left(), track_y), Vec2::new(rect.width(), track_h));

                // Фон доріжки
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

                // Кліпи на цій доріжці
                for clip in editor.clips.iter().filter(|c| c.track_idx == track_idx) {
                    let cx = rect.left() + clip.start_secs * zoom;
                    let cw = (clip.duration * zoom).max(4.0);
                    let clip_rect = Rect::from_min_size(
                        Pos2::new(cx, track_y + 2.0),
                        Vec2::new(cw, track_h - 4.0),
                    );

                    let is_sel = editor.selected_clip_id.as_deref() == Some(clip.id.as_str());
                    let (bg, border) = match clip.kind {
                        ClipKind::Video => (Color32::from_rgb(18, 32, 55), Color32::from_rgb(9, 100, 220)),
                        ClipKind::Image => (Color32::from_rgb(30, 22, 48), Color32::from_rgb(120, 70, 200)),
                        ClipKind::Audio => (Color32::from_rgb(20, 40, 28), Color32::from_rgb(39, 160, 80)),
                    };
                    let border = if is_sel { Color32::WHITE } else { border };
                    painter.rect(clip_rect, 3.0, bg, Stroke::new(if is_sel { 2.0 } else { 1.2 }, border));

                    if cw > 18.0 {
                        let icon = match clip.kind { ClipKind::Video => "🎬", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                        let label = if clip.name.chars().count() > 16 {
                            format!("{} {}…", icon, clip.name.chars().take(13).collect::<String>())
                        } else {
                            format!("{} {}", icon, clip.name)
                        };
                        painter.text(
                            Pos2::new(clip_rect.left() + 4.0, clip_rect.top() + 5.0),
                            Align2::LEFT_TOP, &label,
                            egui::FontId::proportional(10.0), Color32::from_rgb(200, 200, 215),
                        );
                        if cw > 40.0 {
                            painter.text(
                                Pos2::new(clip_rect.left() + 4.0, clip_rect.top() + 19.0),
                                Align2::LEFT_TOP, format!("{:.1}s", clip.duration),
                                egui::FontId::proportional(9.0), Color32::from_rgb(120, 120, 130),
                            );
                        }
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
                painter.circle_filled(Pos2::new(ph_x, rect.top() + ruler_h - 4.0), 5.0, Color32::from_rgb(9, 123, 244));
            }

            // Клік по лінійці — переміщення плейхеда
            let mouse = ui.input(|i| i.pointer.hover_pos());
            if let Some(pos) = mouse {
                if ruler_rect.contains(pos) && ui.input(|i| i.pointer.primary_down()) {
                    editor.playhead = ((pos.x - rect.left()) / zoom).clamp(0.0, total_dur);
                }
            }

            // Клік по кліпу — виділення; drop з медіа-пулу
            if resp.clicked() {
                let pos = resp.interact_pointer_pos().unwrap_or_default();
                let mut hit = false;
                for clip in &editor.clips {
                    let track_y = rect.top() + ruler_h + (track_h + 2.0) * clip.track_idx as f32;
                    let cx = rect.left() + clip.start_secs * zoom;
                    let cw = clip.duration * zoom;
                    let clip_rect = Rect::from_min_size(Pos2::new(cx, track_y + 2.0), Vec2::new(cw, track_h - 4.0));
                    if clip_rect.contains(pos) {
                        editor.selected_clip_id = Some(clip.id.clone());
                        hit = true;
                        break;
                    }
                }
                if !hit { editor.selected_clip_id = None; }
            }

            // Drag з медіа-пулу: drop на доріжку
            if ui.input(|i| !i.pointer.any_down()) {
                if let Some(drag_id) = editor.dragged_media_id.take() {
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        if rect.contains(pos) {
                            let t_idx = {
                                let rel_y = pos.y - (rect.top() + ruler_h);
                                let idx = (rel_y / (track_h + 2.0)) as usize;
                                idx.min(editor.num_tracks - 1)
                            };
                            let start = ((pos.x - rect.left()) / zoom).max(0.0);
                            if let Some(media) = editor.media_pool.iter().find(|m| m.id == drag_id) {
                                let kind = media.kind.clone();
                                let name = media.name.clone();
                                let path = Some(media.path.clone());
                                editor.clips.push(EditorClip {
                                    id: uuid_str(),
                                    path,
                                    name,
                                    start_secs: start,
                                    duration: 5.0,
                                    track_idx: t_idx,
                                    kind,
                                });
                            }
                        }
                    }
                    editor.drop_target_track = None;
                }
            }

            // Підсвічення target при drag-over
            if editor.dragged_media_id.is_some() {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if rect.contains(pos) {
                        let rel_y = pos.y - (rect.top() + ruler_h);
                        let idx = (rel_y / (track_h + 2.0)) as usize;
                        editor.drop_target_track = Some(idx.min(editor.num_tracks - 1));
                        ui.ctx().request_repaint();
                    }
                }
            }
        });
    });
}
