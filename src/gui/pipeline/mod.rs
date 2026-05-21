pub mod api;
pub mod editing;
pub mod storage;
pub mod templates;
pub mod translation;
pub mod video;
pub mod voiceover;

use eframe::egui;
use crate::localization::{Language, translate};

/// Відображає бічну панель пайплайну із порожніми згорнутими секціями.
/// Секції відсортовані у логічному порядку процесу створення відео:
/// 1. Шаблони
/// 2. Шлях збереження
/// 3. АПІ (API Keys)
/// 4. Переклад
/// 5. Озвучка
/// 6. Відеоряд
/// 7. Монтаж
pub fn draw_pipeline_panel(ui: &mut egui::Ui, language: Language) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.heading(translate(language, "pipeline_settings"));
        ui.separator();
        
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 1. Шаблони
                ui.collapsing(translate(language, "templates"), |ui| {
                    templates::draw_templates_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 2. Шлях збереження
                ui.collapsing(translate(language, "storage"), |ui| {
                    storage::draw_storage_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 3. АПІ
                ui.collapsing(translate(language, "api"), |ui| {
                    api::draw_api_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 4. Переклад
                ui.collapsing(translate(language, "translation"), |ui| {
                    translation::draw_translation_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 5. Озвучка
                ui.collapsing(translate(language, "voiceover"), |ui| {
                    voiceover::draw_voiceover_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 6. Відеоряд
                ui.collapsing(translate(language, "video"), |ui| {
                    video::draw_video_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 7. Монтаж
                ui.collapsing(translate(language, "editing"), |ui| {
                    editing::draw_editing_section(ui);
                });
            });
    });
}
