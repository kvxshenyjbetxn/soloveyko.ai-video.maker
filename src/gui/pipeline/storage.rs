use crate::localization::{Language, translate};
use eframe::egui;

/// Малює секцію "Шлях збереження" з окремими полями для macOS та Windows.
/// Активна платформа підсвічується міткою "(активний)".
pub fn draw_storage_section(
    ui: &mut egui::Ui,
    language: Language,
    save_path_macos: &mut String,
    save_path_windows: &mut String,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(translate(language, "storage_path_label")).strong());
        ui.add_space(4.0);

        let is_macos = cfg!(target_os = "macos");
        draw_path_row(
            ui,
            language,
            translate(language, "storage_path_macos"),
            save_path_macos,
            is_macos,
        );
        ui.add_space(4.0);
        draw_path_row(
            ui,
            language,
            translate(language, "storage_path_windows"),
            save_path_windows,
            !is_macos,
        );

        ui.add_space(6.0);
    });
}

fn draw_path_row(
    ui: &mut egui::Ui,
    language: Language,
    platform_label: &str,
    path: &mut String,
    is_active: bool,
) {
    let btn_width = 65.0;
    let item_spacing = ui.spacing().item_spacing.x;
    let available = ui.available_width();

    ui.horizontal(|ui| {
        // Мітка платформи + "(активний)" якщо поточна ОС
        if is_active {
            ui.label(
                egui::RichText::new(format!(
                    "{} {}",
                    platform_label,
                    translate(language, "storage_path_active")
                ))
                .strong(),
            );
        } else {
            ui.label(egui::RichText::new(platform_label).weak());
        }
    });

    ui.horizontal(|ui| {
        ui.add_sized(
            [(available - btn_width - item_spacing).max(60.0), 20.0],
            egui::TextEdit::singleline(path).hint_text(translate(language, "storage_path_hint")),
        );
        if ui
            .add_sized(
                [btn_width, 20.0],
                egui::Button::new(translate(language, "storage_browse_btn")),
            )
            .clicked()
        {
            if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                *path = picked.to_string_lossy().to_string();
            }
        }
    });
}
