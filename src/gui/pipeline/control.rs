use crate::localization::{Language, translate};
use eframe::egui;

/// Малює секцію "Контроль" на панелі пайплайну з налаштуваннями контролю етапів.
pub fn draw_control_section(
    ui: &mut egui::Ui,
    language: Language,
    pipeline_translation_control_enabled: &mut bool,
    pipeline_control_auto_open: &mut bool,
    pipeline_media_control_enabled: &mut bool,
    pipeline_montage_control_enabled: &mut bool,
) {
    ui.vertical(|ui| {
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.label(translate(language, "control_translation"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::toggle_switch(ui, pipeline_translation_control_enabled);
            });
        });

        if *pipeline_translation_control_enabled {
            ui.add_space(4.0);
            ui.checkbox(
                pipeline_control_auto_open,
                translate(language, "control_auto_open"),
            );
        }

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(translate(language, "control_media"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::toggle_switch(ui, pipeline_media_control_enabled);
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(translate(language, "control_montage"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::toggle_switch(ui, pipeline_montage_control_enabled);
            });
        });

        ui.add_space(2.0);
    });
}
