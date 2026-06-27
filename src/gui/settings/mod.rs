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
) -> bool {
    general::draw_general_settings(ui, current_theme, accent_color, language, show_welcome)
}
