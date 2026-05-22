pub mod general;
pub mod storage;

use crate::theme::AppTheme;
use crate::localization::Language;
use eframe::egui;

/// Малює вкладку налаштувань.
pub fn draw_settings(
    ui: &mut egui::Ui,
    current_theme: &mut AppTheme,
    accent_color: &mut egui::Color32,
    language: &mut Language,
) {
    general::draw_general_settings(ui, current_theme, accent_color, language);
}
