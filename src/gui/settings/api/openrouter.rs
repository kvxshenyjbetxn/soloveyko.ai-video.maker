use eframe::egui;
use crate::localization::{Language, translate};

/// Малює вкладку налаштувань OpenRouter.
pub fn draw(ui: &mut egui::Ui, language: Language) {
    ui.heading(translate(language, "settings_api_openrouter"));
    ui.separator();
}
