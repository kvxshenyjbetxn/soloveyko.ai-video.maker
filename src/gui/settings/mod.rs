pub mod general;
pub mod storage;

use crate::localization::Language;
use crate::theme::AppTheme;
use eframe::egui;

/// Малює вкладку налаштувань. Повертає true, якщо змінилось show_welcome.
pub fn draw_settings(
    ui: &mut egui::Ui,
    current_theme: &mut AppTheme,
    accent_color: &mut egui::Color32,
    language: &mut Language,
    show_welcome: &mut bool,
    shared_stock_cache_enabled: &mut bool,
    shared_stock_cache_dir: &mut String,
) -> bool {
    general::draw_general_settings(
        ui,
        current_theme,
        accent_color,
        language,
        show_welcome,
        shared_stock_cache_enabled,
        shared_stock_cache_dir,
    )
}
