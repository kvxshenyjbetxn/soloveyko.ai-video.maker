use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "Шаблони" на панелі пайплайну з можливістю завантаження та видалення.
pub fn draw_templates_section(
    ui: &mut egui::Ui,
    language: Language,
    saved_templates: &mut Vec<String>,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    assemblyai_key: &mut String,
    assemblyai_status: &mut Option<String>,
    voiceover_provider: &mut String,
    voiceover_template_uuid: &mut String,
    template_status: &mut Option<String>,
    template_name_input: &mut String,
    pipeline_translation_enabled: &mut bool,
    pipeline_translation_control_enabled: &mut bool,
    pipeline_control_auto_open: &mut bool,
    pipeline_media_control_enabled: &mut bool,
    pipeline_agent_control_enabled: &mut bool,
    pipeline_montage_control_enabled: &mut bool,
    pipeline_voiceover_enabled: &mut bool,
    pipeline_video_enabled: &mut bool,
    pipeline_subtitles_enabled: &mut bool,
    pipeline_editing_enabled: &mut bool,
    translation_prompt: &mut String,
    translation_model: &mut String,
    translation_model_openrouter: &mut String,
    translation_model_claude: &mut String,
    translation_model_gemini: &mut String,
    translation_model_codex: &mut String,
    video_service: &mut String,
    video_media_type: &mut String,
    text_split_mode: &mut String,
    text_split_char_limit: &mut usize,
    video_prompt: &mut String,
    video_agent_prompt: &mut String,
    video_llm_service: &mut String,
    video_llm_model: &mut String,
    video_llm_model_openrouter: &mut String,
    video_llm_model_claude: &mut String,
    video_llm_model_gemini: &mut String,
    video_llm_model_codex: &mut String,
    video_llm_temperature: &mut f32,
    translation_temperature: &mut f32,
    translation_service: &mut String,
    edge_tts_voice: &mut String,
    edge_tts_rate: &mut String,
    edge_tts_pitch: &mut String,
    edge_tts_volume: &mut String,
    googler_image_max_threads: &mut usize,
    googler_video_max_threads: &mut usize,
    voiceover_convert_to_wav: &mut bool,
    googler_image_priority: &mut Vec<String>,
    googler_video_priority: &mut Vec<String>,
    subtitles_service: &mut String,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
    subtitle_font_size: &mut u32,
    subtitle_color: &mut [u8; 3],
    subtitle_margin_v: &mut u32,
    subtitle_karaoke: &mut bool,
    subtitle_karaoke_mode: &mut u8,
    subtitle_karaoke_highlight_color: &mut [u8; 3],
    subtitle_karaoke_outline_color: &mut [u8; 3],
    subtitle_karaoke_bold: &mut bool,
    subtitle_karaoke_scale: &mut u32,
    subtitle_font: &mut String,
    montage_service: &mut String,
    montage_fps: &mut u32,
    montage_preset: &mut String,
    montage_bitrate: &mut u32,
    montage_transition: &mut String,
    montage_transition_duration: &mut f32,
    montage_image_zoom_enabled: &mut bool,
    montage_image_zoom_intensity: &mut f32,
    montage_image_zoom_mode: &mut String,
    montage_image_zoom_scale: &mut f32,
    montage_image_shake_enabled: &mut bool,
    montage_image_shake_intensity: &mut f32,
    capcut_enabled: &mut bool,
    capcut_draft_path: &mut String,
    overlay_triggers_enabled: &mut bool,
    overlay_triggers: &mut Vec<crate::core::pipeline::montage::OverlayTrigger>,
    googler_video_upscale_enabled: &mut bool,
    googler_video_upscale_resolution: &mut String,
    googler_video_upscale_quality: &mut String,
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
                            *assemblyai_key = template.assemblyai_key;
                            *assemblyai_status = None;
                            *voiceover_provider = template.voiceover_provider;
                            *voiceover_template_uuid = template.voiceover_template_uuid;
                            *template_name_input = template_name.clone();
                            *pipeline_translation_enabled = template.pipeline_translation_enabled;
                            *pipeline_translation_control_enabled = template.pipeline_translation_control_enabled;
                            *pipeline_control_auto_open = template.pipeline_control_auto_open;
                            *pipeline_media_control_enabled = template.pipeline_media_control_enabled;
                            *pipeline_agent_control_enabled = template.pipeline_agent_control_enabled;
                            *pipeline_montage_control_enabled = template.pipeline_montage_control_enabled;
                            *pipeline_voiceover_enabled = template.pipeline_voiceover_enabled;
                            *pipeline_video_enabled = template.pipeline_video_enabled;
                            *pipeline_subtitles_enabled = template.pipeline_subtitles_enabled;
                            *pipeline_editing_enabled = template.pipeline_editing_enabled;
                            *translation_prompt = template.translation_prompt;
                            *translation_model = template.translation_model.clone();
                            
                            *translation_model_openrouter = template.translation_model_openrouter;
                            *translation_model_claude = if template.translation_model_claude.is_empty() { "sonnet".to_string() } else { template.translation_model_claude };
                            *translation_model_gemini = if template.translation_model_gemini.is_empty() { "gemini-2.5-flash".to_string() } else { template.translation_model_gemini };
                            *translation_model_codex = if template.translation_model_codex.is_empty() { "o3-mini".to_string() } else { template.translation_model_codex };

                            if template.translation_service == "OpenRouter" && translation_model_openrouter.is_empty() {
                                *translation_model_openrouter = template.translation_model.clone();
                            }
                            if template.translation_service == "Claude Code" && translation_model_claude.is_empty() {
                                *translation_model_claude = template.translation_model.clone();
                            }
                            if template.translation_service == "Gemini CLI" && translation_model_gemini.is_empty() {
                                *translation_model_gemini = template.translation_model.clone();
                            }
                            if template.translation_service == "Codex CLI" && translation_model_codex.is_empty() {
                                *translation_model_codex = template.translation_model.clone();
                            }

                            *video_service = template.video_service;
                            *video_media_type = template.video_media_type;
                            *text_split_mode = template.text_split_mode;
                            *text_split_char_limit = template.text_split_char_limit;
                            *video_prompt = template.video_prompt;
                            *video_agent_prompt = template.video_agent_prompt;
                            *video_llm_service = template.video_llm_service.clone();
                            *video_llm_model_openrouter = template.video_llm_model_openrouter.clone();
                            *video_llm_model_claude = if template.video_llm_model_claude.is_empty() { "sonnet".to_string() } else { template.video_llm_model_claude.clone() };
                            *video_llm_model_gemini = if template.video_llm_model_gemini.is_empty() { "gemini-2.5-flash".to_string() } else { template.video_llm_model_gemini.clone() };
                            *video_llm_model_codex = if template.video_llm_model_codex.is_empty() { "o3-mini".to_string() } else { template.video_llm_model_codex.clone() };
                            *video_llm_temperature = template.video_llm_temperature;
                            *video_llm_model = match template.video_llm_service.as_str() {
                                "OpenRouter" => template.video_llm_model_openrouter.clone(),
                                "Claude Code" => video_llm_model_claude.clone(),
                                "Gemini CLI" => video_llm_model_gemini.clone(),
                                "Codex CLI" => video_llm_model_codex.clone(),
                                _ => template.video_llm_model.clone(),
                            };
                            *translation_temperature = template.translation_temperature;
                            *translation_service = template.translation_service;
                            *edge_tts_voice = template.edge_tts_voice;
                            *edge_tts_rate = template.edge_tts_rate;
                            *edge_tts_pitch = template.edge_tts_pitch;
                            *edge_tts_volume = template.edge_tts_volume;
                            *googler_image_max_threads = template.googler_image_max_threads;
                            *googler_video_max_threads = template.googler_video_max_threads;
                            *voiceover_convert_to_wav = template.voiceover_convert_to_wav;
                            *googler_image_priority = template.googler_image_priority;
                            *googler_video_priority = template.googler_video_priority;
                            *subtitles_service = template.subtitles_service;
                            *whisper_language = template.whisper_language;
                            *whisper_model = template.whisper_model;
                            *whisper_max_line_width = template.whisper_max_line_width;
                            *subtitle_font_size = template.subtitle_font_size;
                            *subtitle_color = template.subtitle_color;
                            *subtitle_margin_v = template.subtitle_margin_v;
                            *subtitle_karaoke = template.subtitle_karaoke;
                            *subtitle_karaoke_mode = template.subtitle_karaoke_mode;
                            *subtitle_karaoke_highlight_color = template.subtitle_karaoke_highlight_color;
                            *subtitle_karaoke_outline_color = template.subtitle_karaoke_outline_color;
                            *subtitle_karaoke_bold = template.subtitle_karaoke_bold;
                            *subtitle_karaoke_scale = template.subtitle_karaoke_scale;
                            *subtitle_font = template.subtitle_font;
                            *montage_service = template.montage_service;
                            *montage_fps = template.montage_fps;
                            *montage_preset = template.montage_preset;
                            *montage_bitrate = template.montage_bitrate;
                            *montage_transition = template.montage_transition;
                            *montage_transition_duration = template.montage_transition_duration;
                            *montage_image_zoom_enabled = template.montage_image_zoom_enabled;
                            *montage_image_zoom_intensity = template.montage_image_zoom_intensity;
                            *montage_image_zoom_mode = template.montage_image_zoom_mode;
                            *montage_image_zoom_scale = template.montage_image_zoom_scale;
                            *montage_image_shake_enabled = template.montage_image_shake_enabled;
                            *montage_image_shake_intensity = template.montage_image_shake_intensity;
                            *capcut_enabled = template.capcut_enabled;
                            *capcut_draft_path = template.capcut_draft_path;
                            *overlay_triggers_enabled = template.overlay_triggers_enabled;
                            *overlay_triggers = template.overlay_triggers;
                            *googler_video_upscale_enabled = template.googler_video_upscale_enabled;
                            *googler_video_upscale_resolution = if template.googler_video_upscale_resolution.is_empty() { "1080p".to_string() } else { template.googler_video_upscale_resolution };
                            *googler_video_upscale_quality = if template.googler_video_upscale_quality.is_empty() { "balanced".to_string() } else { template.googler_video_upscale_quality };
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
