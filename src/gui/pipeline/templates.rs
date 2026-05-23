use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "Шаблони" на панелі пайплайну з можливістю завантаження та видалення.
pub fn draw_templates_section(
    ui: &mut egui::Ui,
    language: Language,
    saved_templates: &mut Vec<String>,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    voiceover_provider: &mut String,
    voiceover_template_uuid: &mut String,
    template_status: &mut Option<String>,
    template_name_input: &mut String,
    pipeline_translation_enabled: &mut bool,
    pipeline_translation_control_enabled: &mut bool,
    pipeline_control_auto_open: &mut bool,
    pipeline_voiceover_enabled: &mut bool,
    pipeline_video_enabled: &mut bool,
    pipeline_subtitles_enabled: &mut bool,
    pipeline_editing_enabled: &mut bool,
    translation_prompt: &mut String,
    translation_model: &mut String,
    translation_model_openrouter: &mut String,
    translation_model_claude: &mut String,
    translation_model_gemini: &mut String,
    video_service: &mut String,
    googler_image_provider: &mut String,
    translation_temperature: &mut f32,
    translation_service: &mut String,
    edge_tts_voice: &mut String,
    edge_tts_rate: &mut String,
    edge_tts_pitch: &mut String,
    edge_tts_volume: &mut String,
    googler_image_max_threads: &mut usize,
    googler_video_max_threads: &mut usize,
    voiceover_convert_to_wav: &mut bool,
) {
    ui.vertical(|ui| {
        ui.add_space(2.0);

        if saved_templates.is_empty() {
            ui.label(
                egui::RichText::new(translate(language, "templates_empty"))
                    .weak()
                    .size(12.0)
            );
        } else {
            // Клонуємо список для безпечного ітерування, оскільки всередині циклу
            // ми можемо модифікувати оригінальний vector при натисканні на видалення.
            for template_name in saved_templates.clone() {
                let btn_width = (ui.available_width() - 30.0).max(50.0);
                ui.horizontal(|ui| {

                    let btn = ui.add_sized(
                        [btn_width, 20.0],
                        egui::Button::new(format!("📄 {}", template_name))
                    );

                    if btn.clicked() {
                        if let Some(template) = crate::gui::settings::storage::load_template(&template_name) {
                            *openrouter_key = template.openrouter_key;
                            *openrouter_status = None;
                            *voiceover_provider = template.voiceover_provider;
                            *voiceover_template_uuid = template.voiceover_template_uuid;
                            *template_name_input = template_name.clone();
                            *pipeline_translation_enabled = template.pipeline_translation_enabled;
                            *pipeline_translation_control_enabled = template.pipeline_translation_control_enabled;
                            *pipeline_control_auto_open = template.pipeline_control_auto_open;
                            *pipeline_voiceover_enabled = template.pipeline_voiceover_enabled;
                            *pipeline_video_enabled = template.pipeline_video_enabled;
                            *pipeline_subtitles_enabled = template.pipeline_subtitles_enabled;
                            *pipeline_editing_enabled = template.pipeline_editing_enabled;
                            *translation_prompt = template.translation_prompt;
                            *translation_model = template.translation_model.clone();
                            
                            *translation_model_openrouter = template.translation_model_openrouter;
                            *translation_model_claude = if template.translation_model_claude.is_empty() { "sonnet".to_string() } else { template.translation_model_claude };
                            *translation_model_gemini = if template.translation_model_gemini.is_empty() { "gemini-2.5-flash".to_string() } else { template.translation_model_gemini };

                            if template.translation_service == "OpenRouter" && translation_model_openrouter.is_empty() {
                                *translation_model_openrouter = template.translation_model.clone();
                            }
                            if template.translation_service == "Claude Code" && translation_model_claude.is_empty() {
                                *translation_model_claude = template.translation_model.clone();
                            }
                            if template.translation_service == "Gemini CLI" && translation_model_gemini.is_empty() {
                                *translation_model_gemini = template.translation_model.clone();
                            }

                            *video_service = template.video_service;
                            *googler_image_provider = template.googler_image_provider;
                            *translation_temperature = template.translation_temperature;
                            *translation_service = template.translation_service;
                            *edge_tts_voice = template.edge_tts_voice;
                            *edge_tts_rate = template.edge_tts_rate;
                            *edge_tts_pitch = template.edge_tts_pitch;
                            *edge_tts_volume = template.edge_tts_volume;
                            *googler_image_max_threads = template.googler_image_max_threads;
                            *googler_video_max_threads = template.googler_video_max_threads;
                            *voiceover_convert_to_wav = template.voiceover_convert_to_wav;
                            *template_status = Some(format!(
                                "{}: {} ✔",
                                translate(language, "template_loaded"),
                                template_name
                            ));
                        }
                    }

                    let del_btn = ui.add_sized(
                        [22.0, 20.0],
                        egui::Button::new(
                            egui::RichText::new("🗑")
                                .color(egui::Color32::from_rgb(231, 76, 60))
                        )
                    );

                    if del_btn.clicked() {
                        if let Some(mut dir) = crate::gui::settings::storage::get_templates_dir() {
                            dir.push(format!("{}.json", template_name));
                            if dir.exists() {
                                let _ = std::fs::remove_file(dir);
                                *template_status = Some(format!(
                                    "{} 🗑",
                                    translate(language, "template_deleted")
                                ));
                                *saved_templates = crate::gui::settings::storage::load_saved_templates();
                            }
                        }
                    }
                });
                ui.add_space(4.0);
            }
        }

        ui.add_space(2.0);
    });
}
