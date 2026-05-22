use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "Шлях збереження" на панелі пайплайну.
pub fn draw_storage_section(ui: &mut egui::Ui, language: Language, save_path: &mut String) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(translate(language, "storage_path_label")).strong());
        ui.add_space(4.0);

        let btn_width = 65.0;
        let item_spacing = ui.spacing().item_spacing.x;
        let available = ui.available_width();
        ui.horizontal(|ui| {
            ui.add_sized(
                [(available - btn_width - item_spacing).max(60.0), 20.0],
                egui::TextEdit::singleline(save_path)
                    .hint_text(translate(language, "storage_path_hint")),
            );
            if ui.add_sized([btn_width, 20.0], egui::Button::new(translate(language, "storage_browse_btn"))).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    *save_path = path.to_string_lossy().to_string();
                }
            }
        });

        ui.add_space(6.0);
    });
}
