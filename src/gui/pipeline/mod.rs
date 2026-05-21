pub mod api;
pub mod editing;
pub mod storage;
pub mod templates;
pub mod translation;
pub mod video;
pub mod voiceover;

use eframe::egui;

/// Відображає бічну панель пайплайну із порожніми згорнутими секціями.
/// Секції відсортовані у логічному порядку процесу створення відео:
/// 1. Шаблони
/// 2. Шлях збереження
/// 3. АПІ (API Keys)
/// 4. Переклад
/// 5. Озвучка
/// 6. Відеоряд
/// 7. Монтаж
pub fn draw_pipeline_panel(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.heading("Налаштування пайплайну");
        ui.separator();
        
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 1. Шаблони
                ui.collapsing("Шаблони", |ui| {
                    templates::draw_templates_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 2. Шлях збереження
                ui.collapsing("Шлях збереження", |ui| {
                    storage::draw_storage_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 3. АПІ
                ui.collapsing("АПІ", |ui| {
                    api::draw_api_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 4. Переклад
                ui.collapsing("Переклад", |ui| {
                    translation::draw_translation_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 5. Озвучка
                ui.collapsing("Озвучка", |ui| {
                    voiceover::draw_voiceover_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 6. Відеоряд
                ui.collapsing("Відеоряд", |ui| {
                    video::draw_video_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 7. Монтаж
                ui.collapsing("Монтаж", |ui| {
                    editing::draw_editing_section(ui);
                });
            });
    });
}
