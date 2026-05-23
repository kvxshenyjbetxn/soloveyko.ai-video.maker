use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "Контроль" на панелі пайплайну з налаштуваннями контролю етапів.
pub fn draw_control_section(
    ui: &mut egui::Ui,
    language: Language,
    pipeline_translation_control_enabled: &mut bool,
) {
    ui.vertical(|ui| {
        ui.add_space(2.0);
        
        ui.horizontal(|ui| {
            ui.label(translate(language, "control_translation"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Використовуємо кастомний toggle switch, який визначено в mod.rs.
                // Але оскільки ми не хочемо дублювати toggle_switch, ми можемо або
                // імпортувати його, або просто використати стандартний egui::Checkbox,
                // або викликати toggle_switch із батьківського модуля за допомогою `super::toggle_switch`.
                // Оскільки toggle_switch в mod.rs не є pub, нам треба або зробити його pub,
                // або використати стандартний ui.checkbox, або зробити його pub і викликати як crate::gui::pipeline::toggle_switch.
                // Давайте подивимось, як toggle_switch оголошено в mod.rs: `fn toggle_switch(...)`.
                // Зробимо його `pub(crate) fn toggle_switch` або просто `pub fn toggle_switch`.
                super::toggle_switch(ui, pipeline_translation_control_enabled);
            });
        });
        
        ui.add_space(2.0);
    });
}
