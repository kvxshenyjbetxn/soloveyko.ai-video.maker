pub mod icons;
pub mod preview;
pub mod regen;
pub mod tab;
pub mod video_player;

pub use preview::draw_image_preview;
pub use regen::draw_media_regen_window;
pub use tab::draw_gallery_tab;

/// Дія перегенерації: (файл, налаштування задачі, чи кастомна, job_id, job_name).
pub type RegenAction = (
    std::path::PathBuf,
    crate::queue::JobSettings,
    bool,
    u64,
    String,
);
