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
pub fn draw_pipeline_panel(
    ui: &mut egui::Ui,
    language: Language,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    template_name_input: &mut String,
    saved_templates: &mut Vec<String>,
    template_status: &mut Option<String>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        ui.heading(translate(language, "pipeline_settings"));
        ui.separator();
        
        ui.add_space(8.0);

        // Форма створення нового шаблону (вгорі панелі пайплайну)
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let text_edit = egui::TextEdit::singleline(template_name_input)
                .hint_text(translate(language, "template_name_hint"))
                .desired_width((available_width - 95.0).max(100.0));
            
            let name_resp = ui.add(text_edit);
            if name_resp.changed() {
                *template_status = None;
            }

            let save_btn = ui.add_sized(
                [75.0, 20.0],
                egui::Button::new(translate(language, "template_save_btn"))
            );

            if save_btn.clicked() {
                let name = template_name_input.trim();
                if name.is_empty() {
                    *template_status = Some(translate(language, "template_status_empty").to_string());
                } else {
                    match crate::gui::settings::storage::save_template(name, openrouter_key) {
                        Ok(_) => {
                            *template_status = Some(format!("{} ✔", translate(language, "template_status_saved")));
                            template_name_input.clear();
                            *saved_templates = crate::gui::settings::storage::load_saved_templates();
                        }
                        Err(err) => {
                            *template_status = Some(format!("❌ Error: {}", err));
                        }
                    }
                }
            }
        });

        // Статус збереження/завантаження шаблону
        if let Some(status) = template_status {
            ui.add_space(2.0);
            let is_success = status.contains('✔') || status.contains('🗑') || status.contains("Завантажено") || status.contains("Loaded") || status.contains("Загружен");
            let color = if is_success {
                egui::Color32::from_rgb(46, 204, 113) // Зелений
            } else {
                egui::Color32::from_rgb(231, 76, 60) // Червоний
            };
            ui.add(egui::Label::new(egui::RichText::new(status.as_str()).color(color).size(11.0)).wrap());
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
 
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 1. Шаблони
                ui.collapsing(translate(language, "templates"), |ui| {
                    templates::draw_templates_section(
                        ui,
                        language,
                        saved_templates,
                        openrouter_key,
                        openrouter_status,
                        template_status,
                    );
                });
                
                ui.add_space(8.0);
                
                // 2. Шлях збереження
                ui.collapsing(translate(language, "storage"), |ui| {
                    storage::draw_storage_section(ui);
                });
                
                ui.add_space(8.0);
                
                // 3. АПІ
                ui.collapsing(translate(language, "api"), |ui| {
                    api::draw_api_section(ui, language, openrouter_key, openrouter_status);
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
