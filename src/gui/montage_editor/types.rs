use std::path::PathBuf;
use eframe::egui::{Pos2, Rect};

pub const PREVIEW_FPS: f32 = 15.0;
pub const PREVIEW_WIDTH: u32 = 640;
pub const FRAME_CACHE_SIZE: usize = 200;

// ─── Налаштування превью (синхронізуються з JobSettings) ─────────────────────

#[derive(Clone)]
pub struct MontagePreviewSettings {
    pub zoom_enabled: bool,
    pub zoom_mode: String,
    pub zoom_scale: f32,
    pub shake_enabled: bool,
    pub shake_intensity: f32,
    pub transition: String,
    pub transition_duration: f32,
}

impl Default for MontagePreviewSettings {
    fn default() -> Self {
        Self {
            zoom_enabled: false,
            zoom_mode: "alternate".to_string(),
            zoom_scale: 1.2,
            shake_enabled: false,
            shake_intensity: 0.3,
            transition: "none".to_string(),
            transition_duration: 0.5,
        }
    }
}

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
    /// Стабільний ID медіа-елементу в пулі (не залежить від шляху файлу)
    pub media_id: String,
    pub path: Option<PathBuf>,
    pub name: String,
    pub start_secs: f32,
    pub duration: f32,
    pub track_idx: usize,
    pub kind: ClipKind,
    /// Масштаб кліпу (1.0 = повний кадр). Впливає на превью та FFmpeg overlay.
    pub scale: f32,
    /// Горизонтальне зміщення центру кліпу в нормалізованих координатах (-1..1).
    pub pos_x: f32,
    /// Вертикальне зміщення центру кліпу в нормалізованих координатах (-1..1).
    pub pos_y: f32,
    /// Чи застосовувати ефект зуму до цього кліпу у превью.
    pub zoom_enabled: bool,
    /// Чи застосовувати ефект покачування до цього кліпу у превью.
    pub shake_enabled: bool,
    /// Чи є кліп плейсхолдером (медіа ще не обрано — чекає вибору стоку).
    pub is_placeholder: bool,
    /// Початок обрізки у вихідному файлі (секунди). 0.0 = з початку.
    pub trim_start: f32,
    /// Індекс сегменту в stock_cache.json (для кліпів обраних зі стоків; None = не стокове медіа).
    pub stock_seg_idx: Option<usize>,
}

impl EditorClip {
    pub fn end_secs(&self) -> f32 {
        self.start_secs + self.duration
    }
}

// ─── Режими перетягування на превью ──────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreviewDragMode {
    Move,
    Scale,
}

/// Стан інтерактивного drag-трансформу прямо на превью
pub struct PreviewDragState {
    pub clip_id: String,
    pub mode: PreviewDragMode,
    pub initial_pos_x: f32,
    pub initial_pos_y: f32,
    pub initial_scale: f32,
    pub initial_mouse: Pos2,
    pub frame_rect: Rect,
}

// ─── Дії редактора монтажу ────────────────────────────────────────────────────

/// Дії що редактор монтажу повертає для обробки в app.rs за кожен кадр
pub struct MontageEditorActions {
    /// Шляхи зображень для оживлення (image → video)
    pub animate_paths: Vec<PathBuf>,
    /// Дія перегенерації (файл, налаштування, is_custom, job_id, job_name)
    pub regen_action: Option<crate::gui::gallery::RegenAction>,
    /// Запит на відкриття Stock Picker для вказаного індексу сегмента
    pub open_stock_picker: Option<usize>,
}

impl Default for MontageEditorActions {
    fn default() -> Self {
        Self { animate_paths: vec![], regen_action: None, open_stock_picker: None }
    }
}
