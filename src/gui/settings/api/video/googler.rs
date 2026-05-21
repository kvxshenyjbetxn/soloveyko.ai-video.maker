use eframe::egui;
use crate::localization::{Language, translate};

/// Малює вкладку налаштувань Googler.
pub fn draw(ui: &mut egui::Ui, language: Language) {
    ui.heading(format!("{}: Googler", translate(language, "settings_api_video")));
    ui.separator();
}
