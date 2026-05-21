pub mod api;
pub mod storage;
pub mod templates;
pub mod voiceover;
pub mod editing;
pub mod video;

use eframe::egui;

/// Відображає бічну панель пайплайну із порожніми згорнутими секціями.
pub fn draw_pipeline_panel(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.heading("Налаштування пайплайну");
        ui.separator();
        
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.collapsing("АПІ (API Keys)", |ui| {
                    api::draw_api_section(ui);
                });
                
                ui.add_space(8.0);
                
                ui.collapsing("Шлях збереження (Storage)", |ui| {
                    storage::draw_storage_section(ui);
                });
                
                ui.add_space(8.0);
                
                ui.collapsing("Шаблони (Templates)", |ui| {
                    templates::draw_templates_section(ui);
                });
                
                ui.add_space(8.0);
                
                ui.collapsing("Озвучка (Voiceover)", |ui| {
                    voiceover::draw_voiceover_section(ui);
                });
                
                ui.add_space(8.0);
                
                ui.collapsing("Монтаж (Editing)", |ui| {
                    editing::draw_editing_section(ui);
                });
                
                ui.add_space(8.0);
                
                ui.collapsing("Відеоряд (Video Frames)", |ui| {
                    video::draw_video_section(ui);
                });
            });
    });
}
