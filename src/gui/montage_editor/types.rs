use std::path::PathBuf;
use eframe::egui::{Pos2, Rect};

pub const PREVIEW_FPS: f32 = 30.0;
/// Кількість GPU-текстур у RAM/GPU LRU.
pub const FRAME_CACHE_SIZE: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewQuality {
    Performance,
    Balanced,
    High,
    Ultra,
}

impl PreviewQuality {
    pub fn storage_key(self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Balanced => "balanced",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn from_storage(value: &str) -> Self {
        match value {
            "performance" => Self::Performance,
            "high" => Self::High,
            "ultra" => Self::Ultra,
            _ => Self::Balanced,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Performance => "montage_preview_quality_performance",
            Self::Balanced => "montage_preview_quality_balanced",
            Self::High => "montage_preview_quality_high",
            Self::Ultra => "montage_preview_quality_ultra",
        }
    }

    pub fn scrub_width(self) -> u32 {
        match self {
            Self::Performance => 640,
            Self::Balanced => 960,
            Self::High => 1280,
            Self::Ultra => 1600,
        }
    }

    pub fn sharp_width(self) -> u32 {
        match self {
            Self::Performance => 1280,
            Self::Balanced => 1920,
            Self::High => 1920,
            Self::Ultra => 2560,
        }
    }

    pub fn jpeg_quality(self) -> u8 {
        match self {
            Self::Performance => 82,
            Self::Balanced => 88,
            Self::High => 92,
            Self::Ultra => 94,
        }
    }

    pub fn sharp_jpeg_quality(self) -> u8 {
        match self {
            Self::Performance => 90,
            Self::Balanced => 96,
            Self::High => 97,
            Self::Ultra => 98,
        }
    }

    pub fn ffmpeg_qscale(self) -> &'static str {
        match self {
            Self::Performance => "6",
            Self::Balanced => "4",
            Self::High => "3",
            Self::Ultra => "2",
        }
    }

    pub fn sharp_ffmpeg_qscale(self) -> &'static str {
        match self {
            Self::Performance => "4",
            Self::Balanced => "2",
            Self::High => "2",
            Self::Ultra => "1",
        }
    }

    pub fn cache_tag(self) -> &'static str {
        match self {
            Self::Performance => "perf_w640_q6",
            Self::Balanced => "bal_w960_q4",
            Self::High => "high_w1280_q3",
            Self::Ultra => "ultra_w1600_q2",
        }
    }

    pub fn sharp_cache_tag(self) -> &'static str {
        match self {
            Self::Performance => "still_w1280_q4",
            Self::Balanced => "still_w1920_q2",
            Self::High => "still_w1920_q2",
            Self::Ultra => "still_w2560_q1",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewRenderSettings {
    pub quality: PreviewQuality,
    pub fps: f32,
}

impl Default for PreviewRenderSettings {
    fn default() -> Self {
        Self { quality: PreviewQuality::Balanced, fps: PREVIEW_FPS }
    }
}

impl PreviewRenderSettings {
    pub fn fps_tag(self) -> String {
        format!("f{}", self.fps.round() as u32)
    }
}

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

// ─── Тип доріжки ────────────────────────────────────────────────────────────

/// Визначає тип доріжки на таймлінії: відео або аудіо.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
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
    /// Позиція лінії снапу (секунди), якщо кліп приліп до чогось
    pub snap_line_secs: Option<f32>,
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
    /// Тип xfade-переходу для overlap-зон ("fade", "wipeleft", "dissolve", …).
    /// Визначає як прев'ю рендерить перехід при накладанні, так і FFmpeg-монтаж.
    pub overlap_transition: String,
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
    /// Нові налаштування якості/FPS превʼю, якщо користувач змінив їх у топбарі
    pub preview_render_changed: Option<PreviewRenderSettings>,
}

impl Default for MontageEditorActions {
    fn default() -> Self {
        Self { animate_paths: vec![], regen_action: None, open_stock_picker: None, preview_render_changed: None }
    }
}
