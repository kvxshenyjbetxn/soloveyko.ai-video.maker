pub mod api;
pub mod editing;
pub mod storage;
pub mod subtitles;
pub mod templates;
pub mod translation;
pub mod video;
pub mod voiceover;

use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

/// Малює toggle switch (повзунок вмик/вимк) та повертає Response.
fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(26.0, 14.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        // Плавна анімація переходу між станами
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);

        let bg_color = if *on {
            // Колір акценту з теми застосунку
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };

        let rounding = egui::Rounding::same(rect.height() / 2.0);
        ui.painter().rect_filled(rect, rounding, bg_color);

        // Thumb (коло перемикача)
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
    openrouter_balance: &Arc<Mutex<Option<String>>>,
    voicebot_key: &mut String,
    voicebot_status: &mut Option<String>,
    voicebot_test_result: &Arc<Mutex<Option<String>>>,
    voicebot_balance: &Arc<Mutex<Option<String>>>,
    googler_key: &mut String,
    googler_status: &mut Option<String>,
    googler_test_result: &Arc<Mutex<Option<String>>>,
    googler_balance: &Arc<Mutex<Option<crate::api::googler::GooglerBalance>>>,
    voiceover_provider: &mut String,
    voiceover_template_uuid: &mut String,
    voicebot_templates: &Arc<Mutex<Option<Result<Vec<voiceover::VoiceBotTemplate>, String>>>>,
    voicebot_loading: &Arc<Mutex<bool>>,
    template_name_input: &mut String,
    saved_templates: &mut Vec<String>,
    template_status: &mut Option<String>,
    pipeline_translation_enabled: &mut bool,
    pipeline_voiceover_enabled: &mut bool,
    pipeline_video_enabled: &mut bool,
    pipeline_subtitles_enabled: &mut bool,
    pipeline_editing_enabled: &mut bool,
    translation_prompt: &mut String,
    translation_model: &mut String,
    translation_model_search: &mut String,
    openrouter_models: &Arc<Mutex<Option<Result<Vec<translation::OpenRouterModel>, String>>>>,
    openrouter_models_loading: &Arc<Mutex<bool>>,
    video_service: &mut String,
    googler_image_provider: &mut String,
    translation_temperature: &mut f32,
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
                    match crate::gui::settings::storage::save_template(
                        name,
                        openrouter_key,
                        voiceover_provider,
                        voiceover_template_uuid,
                        *pipeline_translation_enabled,
                        *pipeline_voiceover_enabled,
                        *pipeline_video_enabled,
                        *pipeline_subtitles_enabled,
                        *pipeline_editing_enabled,
                        translation_prompt,
                        translation_model,
                        video_service,
                        googler_image_provider,
                        *translation_temperature,
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

        // Статус збереження/завантаження шаблону
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

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
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
                            voiceover_provider,
                            voiceover_template_uuid,
                            template_status,
                            template_name_input,
                            pipeline_translation_enabled,
                            pipeline_voiceover_enabled,
                            pipeline_video_enabled,
                            pipeline_subtitles_enabled,
                            pipeline_editing_enabled,
                            translation_prompt,
                            translation_model,
                            video_service,
                            googler_image_provider,
                            translation_temperature,
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
                        storage::draw_storage_section(ui);
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
                        );
                    });
                }

                ui.add_space(8.0);

                // 4. Переклад (з перемикачем)
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
                        );
                    });
                }

                ui.add_space(8.0);

                // 5. Озвучка (з перемикачем)
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
                        );
                    });
                }

                ui.add_space(8.0);

                // 6. Відеоряд (з перемикачем)
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
                            googler_image_provider,
                        );
                    });
                }

                ui.add_space(8.0);

                // 7. Субтитри (з перемикачем)
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
                        subtitles::draw_subtitles_section(ui);
                    });
                }

                ui.add_space(8.0);

                // 8. Монтаж (з перемикачем)
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
                        editing::draw_editing_section(ui);
                    });
                }
            });
    });
}
