use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "Монтаж" на панелі пайплайну.
pub fn draw_editing_section(
    ui: &mut egui::Ui,
    language: Language,
    montage_service: &mut String,
    montage_fps: &mut u32,
    montage_preset: &mut String,
    montage_bitrate: &mut u32,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Вибір сервісу
        ui.label(egui::RichText::new(translate(language, "montage_service_label")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("montage_service_combo")
            .selected_text(montage_service.as_str())
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(montage_service, "FFmpeg".to_string(), "FFmpeg");
            });

        // Налаштування FFmpeg
        if montage_service.as_str() == "FFmpeg" {
            ui.add_space(8.0);

            // FPS
            ui.label(egui::RichText::new(translate(language, "montage_fps_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for &val in &[24u32, 30, 60] {
                    ui.radio_value(montage_fps, val, format!("{val}"));
                }
                ui.add(egui::DragValue::new(montage_fps).range(1..=120).suffix(" fps"));
            });

            ui.add_space(8.0);

            // Пресет
            ui.label(egui::RichText::new(translate(language, "montage_preset_label")).strong());
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("montage_preset_combo")
                .selected_text(montage_preset.as_str())
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    for p in &["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"] {
                        ui.selectable_value(montage_preset, p.to_string(), *p);
                    }
                });

            ui.add_space(8.0);

            // Бітрейт
            ui.label(egui::RichText::new(translate(language, "montage_bitrate_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(montage_bitrate).range(1..=100).suffix(" MB/s"));
            });
        }

        ui.add_space(6.0);
    });
}
