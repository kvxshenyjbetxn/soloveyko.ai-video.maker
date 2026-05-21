use eframe::egui;
use crate::localization::{Language, translate};

/// Малює вкладку налаштувань Voice Bot.
pub fn draw(ui: &mut egui::Ui, language: Language) {
    ui.heading(format!("{}: Voice Bot", translate(language, "settings_api_voiceover")));
    ui.separator();
}
