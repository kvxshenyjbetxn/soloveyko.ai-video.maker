pub mod api;
pub mod control;
pub mod editing;
pub mod resume;
pub mod storage;
pub mod subtitles;
pub mod templates;
pub mod translation;
pub mod translation_control;
pub mod video;
pub mod voiceover;

use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

/// Малює toggle switch (повзунок вмик/вимк) та повертає Response.
pub(crate) fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(26.0, 14.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);

        let bg_color = if *on {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };

        let rounding = egui::Rounding::same(rect.height() / 2.0);
        ui.painter().rect_filled(rect, rounding, bg_color);

        let thumb_r = rect.height() / 2.0 - 2.0;
        let thumb_x = egui::lerp(
            (rect.left() + thumb_r + 2.0)..=(rect.right() - thumb_r - 2.0),
            how_on,
        );
        ui.painter().circle_filled(
            egui::pos2(thumb_x, rect.center().y),
            thumb_r,
            egui::Color32::WHITE,
        );
    }

    response
}

/// Відображає бічну панель пайплайну із секціями.
/// Секції відсортовані у логічному порядку процесу створення відео:
/// 1. Шаблони
/// 2. Шлях збереження
/// 3. АПІ (API Keys)
/// 4. Контроль
/// 5. Переклад
/// 6. Озвучка
/// 7. Відеоряд
/// 8. Субтитри
/// 9. Монтаж
#[allow(clippy::too_many_arguments)]
pub fn draw_pipeline_panel(
    ui: &mut egui::Ui,
    language: Language,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    openrouter_balance: &Arc<Mutex<Option<String>>>,
    voicebot_key: &mut String,
    voicebot_status: &mut Option<String>,
    voicebot_test_result: &Arc<Mutex<Option<String>>>,
    voicebot_balance: &Arc<Mutex<Option<String>>>,
    googler_key: &mut String,
    googler_status: &mut Option<String>,
    googler_test_result: &Arc<Mutex<Option<String>>>,
    googler_balance: &Arc<Mutex<Option<crate::api::googler::GooglerBalance>>>,
    assemblyai_key: &mut String,
    assemblyai_status: &mut Option<String>,
    assemblyai_test_result: &Arc<Mutex<Option<String>>>,
    voiceover_provider: &mut String,
    voiceover_template_uuid: &mut String,
    voicebot_templates: &Arc<Mutex<Option<Result<Vec<voiceover::VoiceBotTemplate>, String>>>>,
    voicebot_loading: &Arc<Mutex<bool>>,
    edge_tts_voice: &mut String,
    edge_tts_rate: &mut String,
    edge_tts_pitch: &mut String,
    edge_tts_volume: &mut String,
    edge_tts_voices: &Arc<Mutex<Option<Result<Vec<crate::api::edgetts::EdgeTTSVoice>, String>>>>,
    edge_tts_loading_voices: &Arc<Mutex<bool>>,
    edge_tts_show_all_languages: &mut bool,
    template_name_input: &mut String,
    saved_templates: &mut Vec<String>,
    template_status: &mut Option<String>,
    pipeline_translation_enabled: &mut bool,
    pipeline_translation_control_enabled: &mut bool,
    pipeline_control_auto_open: &mut bool,
    pipeline_media_control_enabled: &mut bool,
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
    translation_model_agy: &mut String,
    translation_model_search: &mut String,
    openrouter_models: &Arc<Mutex<Option<Result<Vec<translation::OpenRouterModel>, String>>>>,
    openrouter_models_loading: &Arc<Mutex<bool>>,
    video_service: &mut String,
    video_media_type: &mut String,
    text_split_mode: &mut String,
    text_split_mode_openrouter: &mut String,
    text_split_char_limit: &mut usize,
    video_prompt: &mut String,
    video_agent_prompt: &mut String,
    video_style_enabled: &mut bool,
    video_style_prompt: &mut String,
    video_llm_service: &mut String,
    video_llm_model: &mut String,
    video_llm_model_openrouter: &mut String,
    video_llm_model_claude: &mut String,
    video_llm_model_gemini: &mut String,
    video_llm_model_codex: &mut String,
    video_llm_model_agy: &mut String,
    video_llm_temperature: &mut f32,
    video_llm_model_search: &mut String,
    translation_temperature: &mut f32,
    translation_service: &mut String,
    save_path_macos: &mut String,
    save_path_windows: &mut String,
    googler_image_max_threads: &mut usize,
    googler_video_max_threads: &mut usize,
    voiceover_convert_to_wav: &mut bool,
    googler_image_priority: &mut Vec<String>,
    googler_video_priority: &mut Vec<String>,
    googler_video_disabled: &mut Vec<String>,
    subtitles_service: &mut String,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
    whisper_model_download: &std::sync::Arc<std::sync::Mutex<crate::gui::welcome::BinaryDownload>>,
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
    available_subtitle_fonts: &[String],
    capcut_enabled: &mut bool,
    capcut_draft_path: &mut String,
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
    overlay_triggers_enabled: &mut bool,
    overlay_triggers: &mut Vec<crate::core::pipeline::montage::OverlayTrigger>,
    googler_video_upscale_enabled: &mut bool,
    googler_video_upscale_resolution: &mut String,
    googler_video_upscale_quality: &mut String,
    text_input: &str,

    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
    queue_error: &mut Option<String>,
    job_name_dialog_open: &mut bool,
    job_name_input: &mut String,
    resume_dialog_open: &mut bool,
    resume_pending: &mut Option<resume::ResumePendingData>,
) {
    // Забороняємо будь-якому елементу розширювати панель за поточну ширину
    ui.set_max_width(ui.available_width());

    ui.add_space(8.0);

    // Форма створення нового шаблону (вгорі панелі пайплайну)
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(8.0, 0.0))
        .show(ui, |ui| {
            let available_width = ui.available_width();
            ui.horizontal(|ui| {
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
                        match crate::gui::settings::storage::save_template(
                            name,
                            openrouter_key,
                            assemblyai_key,
                            voiceover_provider,
                            voiceover_template_uuid,
                            *pipeline_translation_enabled,
                            *pipeline_translation_control_enabled,
                            *pipeline_control_auto_open,
                            *pipeline_media_control_enabled,
                            *pipeline_montage_control_enabled,
                            *pipeline_voiceover_enabled,
                            *pipeline_video_enabled,
                            *pipeline_subtitles_enabled,
                            *pipeline_editing_enabled,
                            translation_prompt,
                            translation_model,
                            translation_model_openrouter,
                            translation_model_claude,
                            translation_model_gemini,
                            translation_model_codex,
                            translation_model_agy,
                            video_service,
                            text_split_mode,
                            text_split_mode_openrouter,
                            *text_split_char_limit,
                            *translation_temperature,
                            translation_service,
                            edge_tts_voice,
                            edge_tts_rate,
                            edge_tts_pitch,
                            edge_tts_volume,
                            *googler_image_max_threads,
                            *googler_video_max_threads,
                            *voiceover_convert_to_wav,
                            video_prompt,
                            video_agent_prompt,
                            *video_style_enabled,
                            video_style_prompt,
                            googler_image_priority.clone(),
                            googler_video_priority.clone(),
                            googler_video_disabled.clone(),
                            video_media_type,
                            subtitles_service,
                            whisper_language,
                            whisper_model,
                            *whisper_max_line_width,
                            *subtitle_font_size,
                            *subtitle_color,
                            *subtitle_margin_v,
                            *subtitle_karaoke,
                            *subtitle_karaoke_mode,
                            *subtitle_karaoke_highlight_color,
                            *subtitle_karaoke_outline_color,
                            *subtitle_karaoke_bold,
                            *subtitle_karaoke_scale,
                            subtitle_font,
                            montage_service,
                            *montage_fps,
                            montage_preset,
                            *montage_bitrate,
                            montage_transition,
                            *montage_transition_duration,
                            *montage_image_zoom_enabled,
                            *montage_image_zoom_intensity,
                            montage_image_zoom_mode,
                            *montage_image_zoom_scale,
                            *montage_image_shake_enabled,
                            *montage_image_shake_intensity,
                            *capcut_enabled,
                            capcut_draft_path,
                            video_llm_service,
                            video_llm_model,
                            video_llm_model_openrouter,
                            video_llm_model_claude,
                            video_llm_model_gemini,
                            video_llm_model_codex,
                            video_llm_model_agy,
                            *video_llm_temperature,
                            *overlay_triggers_enabled,
                            overlay_triggers.clone(),
                            *googler_video_upscale_enabled,
                            googler_video_upscale_resolution,
                            googler_video_upscale_quality,
                        ) {

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

            if let Some(status) = template_status {
                ui.add_space(2.0);
                let is_success = status.contains('✔') || status.contains('🗑') || status.contains("Завантажено") || status.contains("Loaded") || status.contains("Загружен");
                let color = if is_success {
                    egui::Color32::from_rgb(46, 204, 113)
                } else {
                    egui::Color32::from_rgb(231, 76, 60)
                };
                ui.add(egui::Label::new(egui::RichText::new(status.as_str()).color(color).size(11.0)).wrap());
            }
        });

    ui.add_space(3.0);
    ui.separator();
    ui.add_space(3.0);

        // Залишаємо місце внизу для кнопки "Додати в чергу" та можливої помилки
        let bottom_reserve = 8.0 + 28.0 + 8.0 + 8.0;
        let scroll_height = (ui.available_height() - bottom_reserve).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(8.0, 0.0))
                    .show(ui, |ui| {
                        // 1. Шаблони
                        {
                            let id = ui.make_persistent_id("templates_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "templates")).sense(egui::Sense::click()));
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                templates::draw_templates_section(
                                    ui,
                                    language,
                                    saved_templates,
                                    openrouter_key,
                                    openrouter_status,
                                    assemblyai_key,
                                    assemblyai_status,
                                    voiceover_provider,
                                    voiceover_template_uuid,
                                    template_status,
                                    template_name_input,
                                    pipeline_translation_enabled,
                                    pipeline_translation_control_enabled,
                                    pipeline_control_auto_open,
                                    pipeline_media_control_enabled,
                                    pipeline_montage_control_enabled,
                                    pipeline_voiceover_enabled,
                                    pipeline_video_enabled,
                                    pipeline_subtitles_enabled,
                                    pipeline_editing_enabled,
                                    translation_prompt,
                                    translation_model,
                                    translation_model_openrouter,
                                    translation_model_claude,
                                    translation_model_gemini,
                                    translation_model_codex,
                                    translation_model_agy,
                                    video_service,
                                    video_media_type,
                                    text_split_mode,
                                    text_split_char_limit,
                                    video_prompt,
                                    video_agent_prompt,
                                    video_style_enabled,
                                    video_style_prompt,
                                    video_llm_service,
                                    video_llm_model,
                                    video_llm_model_openrouter,
                                    video_llm_model_claude,
                                    video_llm_model_gemini,
                                    video_llm_model_codex,
                                    video_llm_model_agy,
                                    video_llm_temperature,
                                    translation_temperature,
                                    translation_service,
                                    edge_tts_voice,
                                    edge_tts_rate,
                                    edge_tts_pitch,
                                    edge_tts_volume,
                                    googler_image_max_threads,
                                    googler_video_max_threads,
                                    voiceover_convert_to_wav,
                                    googler_image_priority,
                                    googler_video_priority,
                                    googler_video_disabled,
                                    subtitles_service,
                                    whisper_language,
                                    whisper_model,
                                    whisper_max_line_width,
                                    subtitle_font_size,
                                    subtitle_color,
                                    subtitle_margin_v,
                                    subtitle_karaoke,
                                    subtitle_karaoke_mode,
                                    subtitle_karaoke_highlight_color,
                                    subtitle_karaoke_outline_color,
                                    subtitle_karaoke_bold,
                                    subtitle_karaoke_scale,
                                    subtitle_font,
                                    montage_service,
                                    montage_fps,
                                    montage_preset,
                                    montage_bitrate,
                                    montage_transition,
                                    montage_transition_duration,
                                    montage_image_zoom_enabled,
                                    montage_image_zoom_intensity,
                                    montage_image_zoom_mode,
                                    montage_image_zoom_scale,
                                    montage_image_shake_enabled,
                                    montage_image_shake_intensity,
                                    capcut_enabled,
                                    capcut_draft_path,
                                    overlay_triggers_enabled,
                                    overlay_triggers,
                                    googler_video_upscale_enabled,
                                    googler_video_upscale_resolution,
                                    googler_video_upscale_quality,
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 2. Шлях збереження
                        {
                            let id = ui.make_persistent_id("storage_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "storage")).sense(egui::Sense::click()));
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                storage::draw_storage_section(ui, language, save_path_macos, save_path_windows);
                            });
                        }

                        ui.add_space(8.0);

                        // 3. АПІ
                        {
                            let id = ui.make_persistent_id("api_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "api")).sense(egui::Sense::click()));
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                api::draw_api_section(
                                    ui,
                                    language,
                                    openrouter_key,
                                    openrouter_status,
                                    openrouter_balance,
                                    voicebot_key,
                                    voicebot_status,
                                    voicebot_test_result,
                                    voicebot_balance,
                                    googler_key,
                                    googler_status,
                                    googler_test_result,
                                    googler_balance,
                                    assemblyai_key,
                                    assemblyai_status,
                                    assemblyai_test_result,
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 4. Контроль
                        {
                            let id = ui.make_persistent_id("control_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "control")).sense(egui::Sense::click()));
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                control::draw_control_section(ui, language, pipeline_translation_control_enabled, pipeline_control_auto_open, pipeline_media_control_enabled, pipeline_montage_control_enabled);
                            });
                        }

                        ui.add_space(8.0);

                        // 5. Переклад (з перемикачем)
                        {
                            let id = ui.make_persistent_id("translation_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "translation")).sense(egui::Sense::click()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, pipeline_translation_enabled);
                                });
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                translation::draw_translation_section(
                                    ui,
                                    language,
                                    translation_prompt,
                                    translation_model,
                                    translation_model_search,
                                    openrouter_models,
                                    openrouter_models_loading,
                                    translation_temperature,
                                    translation_service,
                                    translation_model_openrouter,
                                    translation_model_claude,
                                    translation_model_gemini,
                                    translation_model_codex,
                                    translation_model_agy,
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 6. Озвучка (з перемикачем)
                        {
                            let id = ui.make_persistent_id("voiceover_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "voiceover")).sense(egui::Sense::click()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, pipeline_voiceover_enabled);
                                });
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                voiceover::draw_voiceover_section(
                                    ui,
                                    language,
                                    voicebot_key,
                                    voiceover_provider,
                                    voiceover_template_uuid,
                                    voicebot_templates,
                                    voicebot_loading,
                                    edge_tts_voice,
                                    edge_tts_rate,
                                    edge_tts_pitch,
                                    edge_tts_volume,
                                    edge_tts_voices,
                                    edge_tts_loading_voices,
                                    edge_tts_show_all_languages,
                                    voiceover_convert_to_wav,
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 7. Відеоряд (з перемикачем)
                        {
                            let id = ui.make_persistent_id("video_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "video")).sense(egui::Sense::click()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, pipeline_video_enabled);
                                });
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                video::draw_video_section(
                                    ui,
                                    language,
                                    video_service,
                                    video_media_type,
                                    text_split_mode,
                                    text_split_mode_openrouter,
                                    text_split_char_limit,
                                    video_prompt,
                                    googler_image_priority,
                                    googler_video_priority,
                                    googler_video_disabled,
                                    video_llm_service,
                                    video_llm_model,
                                    video_llm_model_openrouter,
                                    video_llm_model_claude,
                                    video_llm_model_gemini,
                                    video_llm_model_codex,
                                    video_llm_model_agy,
                                    video_llm_temperature,
                                    video_agent_prompt,
                                    video_style_enabled,
                                    video_style_prompt,
                                    video_llm_model_search,
                                    openrouter_models,
                                    openrouter_models_loading,
                                    overlay_triggers_enabled,
                                    overlay_triggers,
                                    googler_video_upscale_enabled,
                                    googler_video_upscale_resolution,
                                    googler_video_upscale_quality,
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 8. Субтитри (з перемикачем)
                        {
                            let id = ui.make_persistent_id("subtitles_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "subtitles")).sense(egui::Sense::click()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, pipeline_subtitles_enabled);
                                });
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                subtitles::draw_subtitles_section(
                                    ui,
                                    language,
                                    subtitles_service,
                                    whisper_language,
                                    whisper_model,
                                    whisper_max_line_width,
                                    whisper_model_download,
                                    subtitle_font_size,
                                    subtitle_color,
                                    subtitle_margin_v,
                                    subtitle_karaoke,
                                    subtitle_karaoke_mode,
                                    subtitle_karaoke_highlight_color,
                                    subtitle_karaoke_outline_color,
                                    subtitle_karaoke_bold,
                                    subtitle_karaoke_scale,
                                    subtitle_font,
                                    available_subtitle_fonts,
                                    ui.ctx().clone(),
                                );
                            });
                        }

                        ui.add_space(8.0);

                        // 9. Монтаж (з перемикачем)
                        {
                            let id = ui.make_persistent_id("editing_section");
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(), id, false,
                            );
                            let header = ui.horizontal(|ui| {
                                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                                let label = ui.add(egui::Label::new(translate(language, "editing")).sense(egui::Sense::click()));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, pipeline_editing_enabled);
                                });
                                label
                            });
                            if header.inner.clicked() { state.toggle(ui); }
                            state.store(ui.ctx());
                            state.show_body_indented(&header.response, ui, |ui| {
                                editing::draw_editing_section(
                                    ui,
                                    language,
                                    capcut_enabled,
                                    capcut_draft_path,
                                    montage_service,
                                    montage_fps,
                                    montage_preset,
                                    montage_bitrate,
                                    montage_transition,
                                    montage_transition_duration,
                                    montage_image_zoom_enabled,
                                    montage_image_zoom_mode,
                                    montage_image_zoom_scale,
                                    montage_image_shake_enabled,
                                    montage_image_shake_intensity,
                                );
                            });
                        }
                    });
            });

        ui.add_space(4.0);

        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(8.0, 0.0))
            .show(ui, |ui| {
                // Помилка валідації (якщо є)
                if let Some(err) = queue_error.as_ref() {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(err.as_str())
                                .color(egui::Color32::from_rgb(231, 76, 60))
                                .size(11.0),
                        )
                        .wrap(),
                    );
                    ui.add_space(2.0);
                }

                // Кнопка "Додати в чергу" — в самому низу панелі, завжди видима
                if ui.add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new(
                        egui::RichText::new(translate(language, "queue_add_btn")).strong(),
                    ),
                ).clicked() && !*resume_dialog_open {
                    // Спочатку валідуємо — лише якщо все ок, відкриваємо діалог назви
                    // Текст може бути порожнім якщо задача відновлюється з наявних файлів
                    let error = if effective_save_path(save_path_macos, save_path_windows).trim().is_empty() {
                        Some(translate(language, "queue_error_no_save_path").to_string())
                    } else if *pipeline_translation_enabled && translation_model.is_empty() {
                        Some(translate(language, "queue_error_no_model").to_string())
                    } else if *pipeline_translation_enabled && openrouter_key.is_empty() {
                        Some(translate(language, "queue_error_no_key").to_string())
                    } else if *pipeline_voiceover_enabled && voicebot_key.is_empty() {
                        Some(translate(language, "queue_error_no_voicebot_key").to_string())
                    } else {
                        None
                    };

                    if let Some(err) = error {
                        *queue_error = Some(err);
                    } else {
                        *queue_error = None;
                        *job_name_dialog_open = true;
                    }
                }
            });

        ui.add_space(8.0);

        // Вікно введення назви задачі
        if *job_name_dialog_open {
            egui::Window::new(translate(language, "job_name_dialog_title"))
                .collapsible(false)
                .resizable(false)
                .default_width(280.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(job_name_input)
                            .hint_text(translate(language, "job_name_hint"))
                            .desired_width(f32::INFINITY),
                    );

                    let enter_pressed = response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if let Some(err) = queue_error.as_ref() {
                        ui.add_space(4.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(err.as_str())
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .size(11.0),
                            )
                            .wrap(),
                        );
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(translate(language, "job_name_cancel_btn")).clicked() {
                            *job_name_dialog_open = false;
                            job_name_input.clear();
                            *queue_error = None;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let confirm_clicked = ui.add_sized(
                                [80.0, 22.0],
                                egui::Button::new(
                                    egui::RichText::new(translate(language, "job_name_confirm_btn")).strong(),
                                ),
                            ).clicked();

                            if confirm_clicked || enter_pressed {
                                let name = if job_name_input.trim().is_empty() {
                                    format!("{} {}", translate(language, "job_name_auto"), jobs.len() + 1)
                                } else {
                                    job_name_input.trim().to_string()
                                };
                                *job_name_dialog_open = false;
                                job_name_input.clear();
                                *queue_error = None;

                                // Перевіряємо чи є наявні файли в папці
                                let base = effective_save_path(save_path_macos, save_path_windows)
                                    .trim_end_matches('/')
                                    .trim_end_matches('\\');
                                let actual_path = format!("{}/{}", base, name);
                                let found = resume::FoundFiles::scan(
                                    std::path::Path::new(&actual_path),
                                    &name,
                                );

                                if found.has_any() {
                                    // Є наявні файли — показуємо діалог відновлення
                                    if let Err(e) = std::fs::create_dir_all(&actual_path) {
                                        *queue_error = Some(format!(
                                            "{}: {}",
                                            translate(language, "queue_error_create_dir"),
                                            e
                                        ));
                                    } else {
                                        let settings = build_job_settings(
                                            text_input,
                                            actual_path,
                                            *pipeline_translation_enabled,
                                            *pipeline_translation_control_enabled,
                                            *pipeline_media_control_enabled,
                                            *pipeline_montage_control_enabled,
                                            translation_prompt,
                                            translation_model,
                                            *translation_temperature,
                                            translation_service,
                                            openrouter_key,
                                            *pipeline_voiceover_enabled,
                                            voicebot_key,
                                            voiceover_template_uuid,
                                            voiceover_provider,
                                            edge_tts_voice,
                                            edge_tts_rate,
                                            edge_tts_pitch,
                                            edge_tts_volume,
                                            *voiceover_convert_to_wav,
                                            *pipeline_video_enabled,
                                            video_service,
                                            video_media_type,
                                            video_prompt,
                                            video_agent_prompt,
                                            *video_style_enabled,
                                            video_style_prompt,
                                            video_llm_service,
                                            video_llm_model,
                                            *video_llm_temperature,
                                            text_split_mode,
                                            *text_split_char_limit,
                                            googler_key,
                                            googler_image_priority.clone(),
                                            googler_video_priority.iter().filter(|p| !googler_video_disabled.contains(p)).cloned().collect(),
                                            *googler_image_max_threads,
                                            *googler_video_upscale_enabled,
                                            googler_video_upscale_resolution,
                                            googler_video_upscale_quality,
                                            assemblyai_key,

                                            *pipeline_subtitles_enabled,
                                            subtitles_service,
                                            whisper_language,
                                            whisper_model,
                                            *whisper_max_line_width,
                                            *subtitle_font_size,
                                            *subtitle_color,
                                            *subtitle_margin_v,
                                            *subtitle_karaoke,
                                            *subtitle_karaoke_mode,
                                            *subtitle_karaoke_highlight_color,
                                            *subtitle_karaoke_outline_color,
                                            *subtitle_karaoke_bold,
                                            *subtitle_karaoke_scale,
                                            subtitle_font,
                                            *pipeline_editing_enabled,
                                            montage_service,
                                            *capcut_enabled,
                                            capcut_draft_path,
                                            *montage_fps,
                                            montage_preset,
                                            *montage_bitrate,
                                            montage_transition,
                                            *montage_transition_duration,
                                            *montage_image_zoom_enabled,
                                            *montage_image_zoom_intensity,
                                            montage_image_zoom_mode,
                                            *montage_image_zoom_scale,
                                            *montage_image_shake_enabled,
                                            *montage_image_shake_intensity,
                                            *overlay_triggers_enabled,
                                            overlay_triggers.clone(),
                                        );
                                        *resume_dialog_open = true;
                                        *resume_pending = Some(resume::ResumePendingData::new(
                                            name,
                                            found,
                                            settings,
                                        ));
                                    }
                                } else {
                                    validate_and_enqueue(
                                        language,
                                        text_input,
                                        effective_save_path(save_path_macos, save_path_windows),
                                        &name,
                                        *pipeline_translation_enabled,
                                        *pipeline_translation_control_enabled,
                                        *pipeline_media_control_enabled,
                                        *pipeline_montage_control_enabled,
                                        translation_prompt,
                                        translation_model,
                                        *translation_temperature,
                                        openrouter_key,
                                        jobs,
                                        job_counter,
                                        queue_error,
                                        translation_service,
                                        *pipeline_voiceover_enabled,
                                        voicebot_key,
                                        voiceover_template_uuid,
                                        voiceover_provider,
                                        edge_tts_voice,
                                        edge_tts_rate,
                                        edge_tts_pitch,
                                        edge_tts_volume,
                                        *voiceover_convert_to_wav,
                                        *pipeline_video_enabled,
                                        video_service,
                                        video_media_type,
                                        video_prompt,
                                        video_agent_prompt,
                                        *video_style_enabled,
                                        video_style_prompt,
                                        video_llm_service,
                                        video_llm_model,
                                        *video_llm_temperature,
                                        text_split_mode,
                                        *text_split_char_limit,
                                        googler_key,
                                        googler_image_priority.clone(),
                                        googler_video_priority.clone(),
                                        *googler_image_max_threads,
                                        *googler_video_upscale_enabled,
                                        googler_video_upscale_resolution,
                                        googler_video_upscale_quality,
                                        assemblyai_key,
                                        *pipeline_subtitles_enabled,
                                        subtitles_service,
                                        whisper_language,
                                        whisper_model,
                                        *whisper_max_line_width,
                                        *subtitle_font_size,
                                        *subtitle_color,
                                        *subtitle_margin_v,
                                        *subtitle_karaoke,
                                        *subtitle_karaoke_mode,
                                        *subtitle_karaoke_highlight_color,
                                        *subtitle_karaoke_outline_color,
                                        *subtitle_karaoke_bold,
                                        *subtitle_karaoke_scale,
                                        subtitle_font,
                                        *pipeline_editing_enabled,
                                        montage_service,
                                        *capcut_enabled,
                                        capcut_draft_path,
                                        *montage_fps,
                                        montage_preset,
                                        *montage_bitrate,
                                        montage_transition,
                                        *montage_transition_duration,
                                        *montage_image_zoom_enabled,
                                        *montage_image_zoom_intensity,
                                        montage_image_zoom_mode,
                                        *montage_image_zoom_scale,
                                        *montage_image_shake_enabled,
                                        *montage_image_shake_intensity,
                                        *overlay_triggers_enabled,
                                        overlay_triggers.clone(),
                                    );
                                }
                            }
                        });
                    });
                });
        }
}

/// Повертає активний шлях збереження залежно від поточної ОС.
fn effective_save_path<'a>(save_path_macos: &'a str, save_path_windows: &'a str) -> &'a str {
    if cfg!(target_os = "macos") { save_path_macos } else { save_path_windows }
}

/// Будує знімок налаштувань задачі без створення папки та без додавання в чергу.
#[allow(clippy::too_many_arguments)]
fn build_job_settings(
    text_input: &str,
    actual_path: String,
    translation_enabled: bool,
    translation_control_enabled: bool,
    media_control_enabled: bool,
    montage_control_enabled: bool,
    translation_prompt: &str,
    translation_model: &str,
    translation_temperature: f32,
    translation_service: &str,
    openrouter_key: &str,
    voiceover_enabled: bool,
    voicebot_key: &str,
    voiceover_template_uuid: &str,
    voiceover_provider: &str,
    edge_tts_voice: &str,
    edge_tts_rate: &str,
    edge_tts_pitch: &str,
    edge_tts_volume: &str,
    voiceover_convert_to_wav: bool,
    video_enabled: bool,
    video_service: &str,
    video_media_type: &str,
    video_prompt: &str,
    video_agent_prompt: &str,
    video_style_enabled: bool,
    video_style_prompt: &str,
    video_llm_service: &str,
    video_llm_model: &str,
    video_llm_temperature: f32,
    text_split_mode: &str,
    text_split_char_limit: usize,
    googler_key: &str,
    googler_image_priority: Vec<String>,
    googler_video_priority: Vec<String>,
    googler_image_max_threads: usize,
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: &str,
    googler_video_upscale_quality: &str,
    assemblyai_key: &str,

    subtitles_enabled: bool,
    subtitles_service: &str,
    whisper_language: &str,
    whisper_model: &str,
    whisper_max_line_width: usize,
    subtitle_font_size: u32,
    subtitle_color: [u8; 3],
    subtitle_margin_v: u32,
    subtitle_karaoke: bool,
    subtitle_karaoke_mode: u8,
    subtitle_karaoke_highlight_color: [u8; 3],
    subtitle_karaoke_outline_color: [u8; 3],
    subtitle_karaoke_bold: bool,
    subtitle_karaoke_scale: u32,
    subtitle_font: &str,
    montage_enabled: bool,
    montage_service: &str,
    capcut_enabled: bool,
    capcut_draft_path: &str,
    montage_fps: u32,
    montage_preset: &str,
    montage_bitrate: u32,
    montage_transition: &str,
    montage_transition_duration: f32,
    montage_image_zoom_enabled: bool,
    montage_image_zoom_intensity: f32,
    montage_image_zoom_mode: &str,
    montage_image_zoom_scale: f32,
    montage_image_shake_enabled: bool,
    montage_image_shake_intensity: f32,
    overlay_triggers_enabled: bool,
    overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
) -> crate::queue::JobSettings {
    crate::queue::JobSettings {
        text: text_input.to_string(),
        save_path: actual_path,
        translation_enabled,
        translation_control_enabled,
        translation_prompt: translation_prompt.to_string(),
        translation_model: translation_model.to_string(),
        translation_temperature,
        translation_service: translation_service.to_string(),
        openrouter_key: openrouter_key.to_string(),
        voiceover_enabled,
        voicebot_key: voicebot_key.to_string(),
        voiceover_template_uuid: voiceover_template_uuid.to_string(),
        voiceover_provider: voiceover_provider.to_string(),
        edge_tts_voice: edge_tts_voice.to_string(),
        edge_tts_rate: edge_tts_rate.to_string(),
        edge_tts_pitch: edge_tts_pitch.to_string(),
        edge_tts_volume: edge_tts_volume.to_string(),
        voiceover_convert_to_wav,
        video_enabled,
        video_service: video_service.to_string(),
        video_media_type: video_media_type.to_string(),
        video_prompt: video_prompt.to_string(),
        video_agent_prompt: video_agent_prompt.to_string(),
        video_style_enabled,
        video_style_prompt: video_style_prompt.to_string(),
        video_llm_service: video_llm_service.to_string(),
        video_llm_model: video_llm_model.to_string(),
        video_llm_temperature,
        text_split_mode: text_split_mode.to_string(),
        text_split_char_limit,
        googler_key: googler_key.to_string(),
        googler_image_priority,
        googler_video_priority,
        googler_image_max_threads,
        googler_video_upscale_enabled,
        googler_video_upscale_resolution: googler_video_upscale_resolution.to_string(),
        googler_video_upscale_quality: googler_video_upscale_quality.to_string(),
        assemblyai_key: assemblyai_key.to_string(),

        subtitles_enabled,
        subtitles_service: subtitles_service.to_string(),
        whisper_language: whisper_language.to_string(),
        whisper_model: whisper_model.to_string(),
        whisper_max_line_width,
        subtitle_font_size,
        subtitle_color,
        subtitle_margin_v,
        subtitle_karaoke,
        subtitle_karaoke_mode,
        subtitle_karaoke_highlight_color,
        subtitle_karaoke_outline_color,
        subtitle_karaoke_bold,
        subtitle_karaoke_scale,
        subtitle_font: subtitle_font.to_string(),
        montage_enabled,
        montage_service: montage_service.to_string(),
        capcut_enabled,
        capcut_draft_path: capcut_draft_path.to_string(),
        montage_fps,
        montage_preset: montage_preset.to_string(),
        montage_bitrate,
        montage_transition: montage_transition.to_string(),
        montage_transition_duration,
        montage_image_zoom_enabled,
        montage_image_zoom_intensity,
        montage_image_zoom_mode: montage_image_zoom_mode.to_string(),
        montage_image_zoom_scale,
        montage_image_shake_enabled,
        montage_image_shake_intensity,
        media_control_enabled,
        montage_control_enabled,
        overlay_triggers_enabled,
        overlay_triggers,
        resume_from_stage: None,
    }
}

/// Створює папку задачі та додає її в чергу зі статусом Pending.
#[allow(clippy::too_many_arguments)]
fn validate_and_enqueue(
    language: Language,
    text_input: &str,
    save_path: &str,
    task_name: &str,
    translation_enabled: bool,
    translation_control_enabled: bool,
    media_control_enabled: bool,
    montage_control_enabled: bool,
    translation_prompt: &str,
    translation_model: &str,
    translation_temperature: f32,
    openrouter_key: &str,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
    queue_error: &mut Option<String>,
    translation_service: &str,
    voiceover_enabled: bool,
    voicebot_key: &str,
    voiceover_template_uuid: &str,
    voiceover_provider: &str,
    edge_tts_voice: &str,
    edge_tts_rate: &str,
    edge_tts_pitch: &str,
    edge_tts_volume: &str,
    voiceover_convert_to_wav: bool,
    video_enabled: bool,
    video_service: &str,
    video_media_type: &str,
    video_prompt: &str,
    video_agent_prompt: &str,
    video_style_enabled: bool,
    video_style_prompt: &str,
    video_llm_service: &str,
    video_llm_model: &str,
    video_llm_temperature: f32,
    text_split_mode: &str,
    text_split_char_limit: usize,
    googler_key: &str,
    googler_image_priority: Vec<String>,
    googler_video_priority: Vec<String>,
    googler_image_max_threads: usize,
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: &str,
    googler_video_upscale_quality: &str,
    assemblyai_key: &str,

    subtitles_enabled: bool,
    subtitles_service: &str,
    whisper_language: &str,
    whisper_model: &str,
    whisper_max_line_width: usize,
    subtitle_font_size: u32,
    subtitle_color: [u8; 3],
    subtitle_margin_v: u32,
    subtitle_karaoke: bool,
    subtitle_karaoke_mode: u8,
    subtitle_karaoke_highlight_color: [u8; 3],
    subtitle_karaoke_outline_color: [u8; 3],
    subtitle_karaoke_bold: bool,
    subtitle_karaoke_scale: u32,
    subtitle_font: &str,
    montage_enabled: bool,
    montage_service: &str,
    capcut_enabled: bool,
    capcut_draft_path: &str,
    montage_fps: u32,
    montage_preset: &str,
    montage_bitrate: u32,
    montage_transition: &str,
    montage_transition_duration: f32,
    montage_image_zoom_enabled: bool,
    montage_image_zoom_intensity: f32,
    montage_image_zoom_mode: &str,
    montage_image_zoom_scale: f32,
    montage_image_shake_enabled: bool,
    montage_image_shake_intensity: f32,
    overlay_triggers_enabled: bool,
    overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
) {
    // Будуємо шлях: {save_path}/{task_name}
    let base = save_path.trim_end_matches('/').trim_end_matches('\\');
    let actual_path = format!("{}/{}", base, task_name);

    if let Err(e) = std::fs::create_dir_all(&actual_path) {
        *queue_error = Some(format!("{}: {}", translate(language, "queue_error_create_dir"), e));
        return;
    }

    let settings = build_job_settings(
        text_input,
        actual_path,
        translation_enabled,
        translation_control_enabled,
        media_control_enabled,
        montage_control_enabled,
        translation_prompt,
        translation_model,
        translation_temperature,
        translation_service,
        openrouter_key,
        voiceover_enabled,
        voicebot_key,
        voiceover_template_uuid,
        voiceover_provider,
        edge_tts_voice,
        edge_tts_rate,
        edge_tts_pitch,
        edge_tts_volume,
        voiceover_convert_to_wav,
        video_enabled,
        video_service,
        video_media_type,
        video_prompt,
        video_agent_prompt,
        video_style_enabled,
        video_style_prompt,
        video_llm_service,
        video_llm_model,
        video_llm_temperature,
        text_split_mode,
        text_split_char_limit,
        googler_key,
        googler_image_priority,
        googler_video_priority,
        googler_image_max_threads,
        googler_video_upscale_enabled,
        googler_video_upscale_resolution,
        googler_video_upscale_quality,
        assemblyai_key,

        subtitles_enabled,
        subtitles_service,
        whisper_language,
        whisper_model,
        whisper_max_line_width,
        subtitle_font_size,
        subtitle_color,
        subtitle_margin_v,
        subtitle_karaoke,
        subtitle_karaoke_mode,
        subtitle_karaoke_highlight_color,
        subtitle_karaoke_outline_color,
        subtitle_karaoke_bold,
        subtitle_karaoke_scale,
        subtitle_font,
        montage_enabled,
        montage_service,
        capcut_enabled,
        capcut_draft_path,
        montage_fps,
        montage_preset,
        montage_bitrate,
        montage_transition,
        montage_transition_duration,
        montage_image_zoom_enabled,
        montage_image_zoom_intensity,
        montage_image_zoom_mode,
        montage_image_zoom_scale,
        montage_image_shake_enabled,
        montage_image_shake_intensity,
        overlay_triggers_enabled,
        overlay_triggers,
    );

    let id = *job_counter;
    *job_counter += 1;
    jobs.push(crate::queue::PipelineJob::new(id, task_name.to_string(), settings));
}
