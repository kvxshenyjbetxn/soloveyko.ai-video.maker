use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

/// Кнопка "дублювати" — коло з плюсом (⊕), намальоване через Painter (незалежно від шрифту).
pub(crate) fn duplicate_button(ui: &mut egui::Ui) -> egui::Response {
    let size = egui::vec2(16.0, 16.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = if response.is_pointer_button_down_on() {
            ui.visuals().widgets.active
        } else if response.hovered() {
            ui.visuals().widgets.hovered
        } else {
            ui.visuals().widgets.inactive
        };

        ui.painter().rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

        let c = rect.center();
        let r = 5.0_f32;
        let stroke = egui::Stroke::new(1.2, visuals.fg_stroke.color);
        let arm = r * 0.55;

        ui.painter().circle_stroke(c, r, stroke);
        ui.painter().line_segment(
            [egui::pos2(c.x, c.y - arm), egui::pos2(c.x, c.y + arm)],
            stroke,
        );
        ui.painter().line_segment(
            [egui::pos2(c.x - arm, c.y), egui::pos2(c.x + arm, c.y)],
            stroke,
        );
    }

    response
}

/// Кнопка зі стрілкою вгору або вниз, намальованою через Painter (незалежно від шрифту).
pub(crate) fn arrow_button(ui: &mut egui::Ui, up: bool, enabled: bool) -> egui::Response {
    let size = egui::vec2(20.0, 20.0);
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);

    if ui.is_rect_visible(rect) {
        let visuals = if !enabled {
            ui.visuals().widgets.noninteractive
        } else if response.is_pointer_button_down_on() {
            ui.visuals().widgets.active
        } else if response.hovered() {
            ui.visuals().widgets.hovered
        } else {
            ui.visuals().widgets.inactive
        };

        ui.painter().rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

        let c = rect.center();
        let h: f32 = 5.0;
        let w: f32 = 5.0;
        let points = if up {
            vec![
                egui::pos2(c.x,     c.y - h * 0.6),
                egui::pos2(c.x - w, c.y + h * 0.4),
                egui::pos2(c.x + w, c.y + h * 0.4),
            ]
        } else {
            vec![
                egui::pos2(c.x,     c.y + h * 0.6),
                egui::pos2(c.x - w, c.y - h * 0.4),
                egui::pos2(c.x + w, c.y - h * 0.4),
            ]
        };
        let color = visuals.fg_stroke.color;
        ui.painter().add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
    }

    response
}

/// Вставляє плейсхолдер у TextEdit за поточною позицією курсора.
fn insert_at_cursor(
    ui: &mut egui::Ui,
    edit_id: egui::Id,
    text: &mut String,
    placeholder: &str,
) {
    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), edit_id) {
        if let Some(cursor_range) = state.cursor.char_range() {
            let cursor_idx = cursor_range.primary.index;
            let byte_idx = text
                .char_indices()
                .map(|(b_idx, _)| b_idx)
                .nth(cursor_idx)
                .unwrap_or(text.len());
            text.insert_str(byte_idx, placeholder);

            let new_char_idx = cursor_idx + placeholder.chars().count();
            let new_cursor = egui::text::CCursor::new(new_char_idx);
            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
            state.store(ui.ctx(), edit_id);
        } else {
            text.push_str(placeholder);
        }
    } else {
        text.push_str(placeholder);
    }
}

/// Повертає відображувану назву та вартість провайдера зображень.
pub(crate) fn image_provider_info(key: &str) -> (&'static str, &'static str) {
    match key {
        "flow_IMAGEN_3_5" => ("Imagen 4 (Flow)",        "4 кр."),
        "flow_GEM_PIX_2"  => ("Nano Banana Pro (Flow)", "4 кр."),
        "flow_NARWHAL"    => ("Nano Banana 2 (Flow)",   "4 кр."),
        "flower"          => ("Nano Banana 2 (Flower)", "1 кр."),
        "grok"            => ("Grok",                    "1 кр."),
        "openai"          => ("ChatGPT Images 2.0",      "1 кр."),
        _                 => ("Unknown",                 ""),
    }
}

/// Повертає відображувану назву та вартість провайдера відео.
pub(crate) fn video_provider_info(key: &str) -> (&'static str, &'static str) {
    match key {
        "flow"            => ("Flow (VEO)",         "1 кр."),
        "flower"          => ("Flower (Veo 3.1)",  "1 кр."),
        "grok"            => ("Grok",              "1 кр."),
        "flow_omni_flash" => ("Omni Flash (Flow)",     "1 кр."),
        "flow_fast"       => ("Veo 3.1 Fast (Flow)",  "1 кр."),
        "flow_light"      => ("Veo 3.1 Light (Flow)",    "1 кр."),
        "flow_quality"    => ("Veo 3.1 Quality (Flow)", "10 кр."),
        _                 => ("Unknown",               ""),
    }
}

/// Малює секцію "Відеоряд" на панелі пайплайну.
pub fn draw_video_section(
    ui: &mut egui::Ui,
    language: Language,
    video_service: &mut String,
    video_media_type: &mut String,
    text_split_mode: &mut String,
    text_split_mode_openrouter: &mut String,
    text_split_char_limit: &mut usize,
    video_prompt: &mut String,
    video_context_enabled: &mut bool,
    video_context_mode: &mut String,
    video_context_chars: &mut usize,
    googler_image_priority: &mut Vec<String>,
    googler_video_priority: &mut Vec<String>,
    googler_video_disabled: &mut Vec<String>,
    video_llm_service: &mut String,
    video_llm_model: &mut String,
    video_llm_model_openrouter: &mut String,
    video_llm_model_claude: &mut String,
    video_llm_model_gemini: &mut String,
    video_llm_model_codex: &mut String,
    video_llm_model_agy: &mut String,
    video_llm_model_pi: &mut String,
    video_llm_temperature: &mut f32,
    video_agent_prompt: &mut String,
    video_style_enabled: &mut bool,
    video_style_prompt: &mut String,
    video_llm_model_search: &mut String,
    openrouter_models: &Arc<Mutex<Option<Result<Vec<crate::gui::pipeline::translation::OpenRouterModel>, String>>>>,
    openrouter_models_loading: &Arc<Mutex<bool>>,
    overlay_triggers_enabled: &mut bool,
    overlay_triggers: &mut Vec<crate::core::pipeline::montage::OverlayTrigger>,
    googler_video_upscale_enabled: &mut bool,
    googler_video_upscale_resolution: &mut String,
    googler_video_upscale_quality: &mut String,
) {

    ui.vertical(|ui| {
        ui.add_space(4.0);

        let is_cli_service = video_llm_service == "Claude Code" || video_llm_service == "Gemini CLI" || video_llm_service == "Codex CLI" || video_llm_service == "AGY CLI" || video_llm_service == "Pi CLI";
        if is_cli_service {
            *text_split_mode = "full".to_string();
        } else {
            // Режим нарізання тексту — вгорі, бо визначає логіку передачі тексту в промт
            ui.label(egui::RichText::new(translate(language, "text_split_mode_label")).strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.radio_value(text_split_mode, "paragraphs".to_string(), translate(language, "text_split_paragraphs"))
                    .on_hover_text(translate(language, "text_split_paragraphs_hint"));
                ui.radio_value(text_split_mode, "sentences".to_string(), translate(language, "text_split_sentences"))
                    .on_hover_text(translate(language, "text_split_sentences_hint"));
            });
            ui.horizontal(|ui| {
                ui.radio_value(text_split_mode, "char_limit".to_string(), translate(language, "text_split_char_limit"))
                    .on_hover_text(translate(language, "text_split_char_limit_hint"));
                if text_split_mode.as_str() == "char_limit" {
                    ui.add(egui::DragValue::new(text_split_char_limit).range(50..=5000).suffix(" симв."));
                }
            });
            ui.radio_value(text_split_mode, "full".to_string(), translate(language, "text_split_full"))
                .on_hover_text(translate(language, "text_split_full_hint"));

            ui.add_space(8.0);
        }

        // Перемикач режиму роботи: API / Agent
        let previous_llm_service = video_llm_service.clone();
        let mut switched_llm_mode = false;

        let mut api_mode = *video_llm_service == "OpenRouter" || *video_llm_service == "None";
        let mut agent_mode = !api_mode && !video_llm_service.is_empty();

        ui.label(egui::RichText::new(translate(language, "video_llm_mode_label")).strong());
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let was_api = api_mode;
            let was_agent = agent_mode;
            if ui.radio_value(&mut api_mode, true, translate(language, "video_llm_mode_api")).changed() && !was_api {
                // Перехід з Agent → API: зберігаємо модель, встановлюємо OpenRouter
                if previous_llm_service == "Claude Code" {
                    *video_llm_model_claude = video_llm_model.clone();
                } else if previous_llm_service == "Gemini CLI" {
                    *video_llm_model_gemini = video_llm_model.clone();
                } else if previous_llm_service == "Codex CLI" {
                    *video_llm_model_codex = video_llm_model.clone();
                } else if previous_llm_service == "AGY CLI" {
                    *video_llm_model_agy = video_llm_model.clone();
                } else if previous_llm_service == "Pi CLI" {
                    *video_llm_model_pi = video_llm_model.clone();
                }
                *video_llm_service = "OpenRouter".to_string();
                *video_llm_model = video_llm_model_openrouter.clone();
                switched_llm_mode = true;
                agent_mode = false;
                // Відновлюємо режим нарізки
                *text_split_mode = text_split_mode_openrouter.clone();
            }
            if ui.radio_value(&mut agent_mode, true, translate(language, "video_llm_mode_agent")).changed() && !was_agent {
                // Перехід з API → Agent: зберігаємо модель OpenRouter, встановлюємо Claude Code
                if previous_llm_service == "OpenRouter" {
                    *video_llm_model_openrouter = video_llm_model.clone();
                }
                *video_llm_service = "Claude Code".to_string();
                *video_llm_model = if video_llm_model_claude.is_empty() { "sonnet".to_string() } else { video_llm_model_claude.clone() };
                switched_llm_mode = true;
                api_mode = false;
                // Зберігаємо режим нарізки та встановлюємо full для агента
                *text_split_mode_openrouter = text_split_mode.clone();
                *text_split_mode = "full".to_string();
            }
        });

        ui.add_space(6.0);

        // Вибір ЛЛМ-сервісу залежно від режиму
        ui.label(egui::RichText::new(translate(language, "video_llm_service_label")).strong());
        ui.add_space(4.0);

        if api_mode {
            // Режим API: OpenRouter або Без ЛЛМ
            egui::ComboBox::from_id_salt("video_llm_service_combo")
                .selected_text(
                    if *video_llm_service == "OpenRouter" {
                        translate(language, "translation_service_openrouter")
                    } else {
                        translate(language, "video_llm_service_none")
                    }
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(video_llm_service, "None".to_string(), translate(language, "video_llm_service_none"));
                    ui.selectable_value(video_llm_service, "OpenRouter".to_string(), translate(language, "translation_service_openrouter"));
                });
        } else if agent_mode {
            // Режим Agent: Claude Code / Gemini CLI / Codex CLI / AGY CLI
            egui::ComboBox::from_id_salt("video_llm_service_combo")
                .selected_text(
                    if *video_llm_service == "Claude Code" {
                        translate(language, "translation_service_claude_code")
                    } else if *video_llm_service == "Gemini CLI" {
                        translate(language, "translation_service_gemini_cli")
                    } else if *video_llm_service == "Codex CLI" {
                        translate(language, "translation_service_codex_cli")
                    } else if *video_llm_service == "Pi CLI" {
                        translate(language, "translation_service_pi_cli")
                    } else {
                        translate(language, "translation_service_agy_cli")
                    }
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(video_llm_service, "Claude Code".to_string(), translate(language, "translation_service_claude_code"));
                    ui.selectable_value(video_llm_service, "Gemini CLI".to_string(), translate(language, "translation_service_gemini_cli"));
                    ui.selectable_value(video_llm_service, "Codex CLI".to_string(), translate(language, "translation_service_codex_cli"));
                    ui.selectable_value(video_llm_service, "AGY CLI".to_string(), translate(language, "translation_service_agy_cli"));
                    ui.selectable_value(video_llm_service, "Pi CLI".to_string(), translate(language, "translation_service_pi_cli"));
                });
        }

        // При зміні сервісу (всередині одного режиму) — відновлюємо відповідну збережену модель
        if !switched_llm_mode && *video_llm_service != previous_llm_service {
            if previous_llm_service == "OpenRouter" {
                *video_llm_model_openrouter = video_llm_model.clone();
            } else if previous_llm_service == "Claude Code" {
                *video_llm_model_claude = video_llm_model.clone();
            } else if previous_llm_service == "Gemini CLI" {
                *video_llm_model_gemini = video_llm_model.clone();
            } else if previous_llm_service == "Codex CLI" {
                *video_llm_model_codex = video_llm_model.clone();
            } else if previous_llm_service == "AGY CLI" {
                *video_llm_model_agy = video_llm_model.clone();
            } else if previous_llm_service == "Pi CLI" {
                *video_llm_model_pi = video_llm_model.clone();
            }
            if video_llm_service == "OpenRouter" {
                *video_llm_model = video_llm_model_openrouter.clone();
            } else if video_llm_service == "Claude Code" {
                *video_llm_model = if video_llm_model_claude.is_empty() { "sonnet".to_string() } else { video_llm_model_claude.clone() };
            } else if video_llm_service == "Gemini CLI" {
                *video_llm_model = if video_llm_model_gemini.is_empty() { "gemini-2.5-flash".to_string() } else { video_llm_model_gemini.clone() };
            } else if video_llm_service == "Codex CLI" {
                *video_llm_model = if video_llm_model_codex.is_empty() { "gpt-5.4-mini".to_string() } else { video_llm_model_codex.clone() };
            } else if video_llm_service == "AGY CLI" {
                *video_llm_model = if video_llm_model_agy.is_empty() { "default".to_string() } else { video_llm_model_agy.clone() };
            } else if video_llm_service == "Pi CLI" {
                *video_llm_model = if video_llm_model_pi.is_empty() { "gemini-2.5-flash".to_string() } else { video_llm_model_pi.clone() };
            }

            // text_split_mode зберігається/відновлюється при перемиканні режиму через радіо-кнопки вище
        }

        let is_agent_mode = video_llm_service == "Claude Code" || video_llm_service == "Gemini CLI" || video_llm_service == "Codex CLI" || video_llm_service == "AGY CLI" || video_llm_service == "Pi CLI";

        // Промт для генерації зображень — прихований в агентному режимі
        let expand_id = ui.make_persistent_id("video_prompt_expand");
        let mut expand_open: bool = ui.data_mut(|d| d.get_persisted(expand_id).unwrap_or(false));

        if !is_agent_mode {

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(translate(language, "video_prompt_label")).strong());
            if ui.small_button("⛶")
                .on_hover_text(translate(language, "prompt_expand_hint"))
                .clicked()
            {
                expand_open = !expand_open;
                ui.data_mut(|d| d.insert_persisted(expand_id, expand_open));
            }
        });
        ui.add_space(4.0);

        let available_width = ui.available_width();
        let te_resp = egui::ScrollArea::vertical()
            .max_height(60.0)
            .id_salt("video_prompt_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(video_prompt)
                        .desired_width(available_width)
                        .hint_text(translate(language, "video_prompt_hint")),
                )
            })
            .inner;

        ui.add_space(4.0);

        // Кнопки швидкої вставки плейсхолдерів за поточним положенням курсора.
        ui.horizontal(|ui| {
            if ui.button(translate(language, "video_insert_placeholder")).clicked() {
                insert_at_cursor(ui, te_resp.id, video_prompt, "{{text}}");
                te_resp.request_focus();
            }
            if *video_context_enabled
                && ui.button(translate(language, "video_insert_context_placeholder")).clicked()
            {
                insert_at_cursor(ui, te_resp.id, video_prompt, "{{context}}");
                te_resp.request_focus();
            }
        });

        ui.add_space(2.0);
        let placeholder_hint = if *video_context_enabled {
            translate(language, "video_placeholder_hint")
        } else {
            translate(language, "video_placeholder_hint_basic")
        };
        ui.label(egui::RichText::new(placeholder_hint).weak().size(11.0));

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(translate(language, "video_context_label")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::toggle_switch(ui, video_context_enabled);
            });
        });

        if *video_context_enabled {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.radio_value(video_context_mode, "around".to_string(), translate(language, "video_context_around"))
                    .on_hover_text(translate(language, "video_context_around_hint"));
                ui.radio_value(video_context_mode, "full".to_string(), translate(language, "video_context_full"))
                    .on_hover_text(translate(language, "video_context_full_hint"));
            });
            if video_context_mode.as_str() == "around" {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "video_context_chars_label"));
                    ui.add(egui::DragValue::new(video_context_chars).range(10..=20000).suffix(" симв."));
                });
            }
            ui.label(egui::RichText::new(translate(language, "video_context_hint")).weak().size(11.0));
        }

        } // !is_agent_mode

        // Поле інструкції агенту — лише при Claude Code або Gemini CLI або Codex CLI
        if is_agent_mode {
            ui.add_space(8.0);

            let agent_expand_id = ui.make_persistent_id("video_agent_prompt_expand");
            let mut agent_expand_open: bool = ui.data_mut(|d| d.get_persisted(agent_expand_id).unwrap_or(false));

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(translate(language, "video_agent_prompt_label")).strong());
                if ui.small_button("⛶")
                    .on_hover_text(translate(language, "prompt_expand_hint"))
                    .clicked()
                {
                    agent_expand_open = !agent_expand_open;
                    ui.data_mut(|d| d.insert_persisted(agent_expand_id, agent_expand_open));
                }
            });
            ui.add_space(4.0);

            let agent_available_width = ui.available_width();
            egui::ScrollArea::vertical()
                .max_height(60.0)
                .id_salt("video_agent_prompt_scroll")
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(video_agent_prompt)
                            .desired_width(agent_available_width),
                    )
                });

            ui.add_space(4.0);

            // Розгорнуте вікно редагування промту агента
            if agent_expand_open {
                let mut still_open = true;
                egui::Window::new(translate(language, "video_agent_prompt_label"))
                    .id(egui::Id::new("video_agent_prompt_window"))
                    .open(&mut still_open)
                    .resizable(true)
                    .collapsible(false)
                    .constrain(true)
                    .default_size([600.0, 400.0])
                    .show(ui.ctx(), |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(ui.ctx().screen_rect().height() * 0.7)
                            .id_salt("win_video_agent_prompt_scroll")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(video_agent_prompt)
                                        .desired_width(f32::INFINITY),
                                )
                            });
                    });
                if !still_open {
                    ui.data_mut(|d| d.insert_persisted(agent_expand_id, false));
                }
            }

            // Toggle "Вказати стиль" + поле промту стилю
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(translate(language, "video_style_label")).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    super::toggle_switch(ui, video_style_enabled);
                });
            });

            if *video_style_enabled {
                ui.add_space(4.0);

                let style_expand_id = ui.make_persistent_id("video_style_prompt_expand");
                let mut style_expand_open: bool = ui.data_mut(|d| d.get_persisted(style_expand_id).unwrap_or(false));

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(translate(language, "video_style_prompt_label")).strong());
                    if ui.small_button("⛶")
                        .on_hover_text(translate(language, "prompt_expand_hint"))
                        .clicked()
                    {
                        style_expand_open = !style_expand_open;
                        ui.data_mut(|d| d.insert_persisted(style_expand_id, style_expand_open));
                    }
                });
                ui.add_space(4.0);

                let style_avail_width = ui.available_width();
                let style_te_resp = egui::ScrollArea::vertical()
                    .max_height(60.0)
                    .id_salt("video_style_prompt_scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(video_style_prompt)
                                .desired_width(style_avail_width)
                                .hint_text(translate(language, "video_style_prompt_hint")),
                        )
                    })
                    .inner;

                ui.add_space(4.0);

                // Кнопка вставки {{text}}
                if ui.button(translate(language, "video_style_insert_placeholder")).clicked() {
                    let te_id = style_te_resp.id;
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), te_id) {
                        let to_insert = "{{text}}";
                        if let Some(cursor_range) = state.cursor.char_range() {
                            let cursor_idx = cursor_range.primary.index;
                            let byte_idx = video_style_prompt
                                .char_indices()
                                .map(|(b, _)| b)
                                .nth(cursor_idx)
                                .unwrap_or(video_style_prompt.len());
                            video_style_prompt.insert_str(byte_idx, to_insert);
                            let new_char_idx = cursor_idx + to_insert.chars().count();
                            let new_cursor = egui::text::CCursor::new(new_char_idx);
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                            state.store(ui.ctx(), te_id);
                        } else {
                            video_style_prompt.push_str(to_insert);
                        }
                    } else {
                        video_style_prompt.push_str("{{text}}");
                    }
                    style_te_resp.request_focus();
                }

                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(translate(language, "video_style_prompt_hint"))
                        .weak()
                        .size(11.0)
                );

                // Розгорнуте вікно редагування промту стилю
                if style_expand_open {
                    let mut still_open = true;
                    egui::Window::new(translate(language, "video_style_prompt_label"))
                        .id(egui::Id::new("video_style_prompt_window"))
                        .open(&mut still_open)
                        .resizable(true)
                        .collapsible(false)
                        .constrain(true)
                        .default_size([600.0, 400.0])
                        .show(ui.ctx(), |ui| {
                            let win_te_resp = egui::ScrollArea::vertical()
                                .max_height(ui.ctx().screen_rect().height() * 0.7)
                                .id_salt("win_video_style_prompt_scroll")
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(video_style_prompt)
                                            .desired_width(f32::INFINITY)
                                            .hint_text(translate(language, "video_style_prompt_hint")),
                                    )
                                })
                                .inner;
                            let win_te_id = win_te_resp.id;
                            ui.add_space(4.0);
                            if ui.button(translate(language, "video_style_insert_placeholder")).clicked() {
                                let to_insert = "{{text}}";
                                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), win_te_id) {
                                    if let Some(cursor_range) = state.cursor.char_range() {
                                        let cursor_idx = cursor_range.primary.index;
                                        let byte_idx = video_style_prompt
                                            .char_indices()
                                            .map(|(b, _)| b)
                                            .nth(cursor_idx)
                                            .unwrap_or(video_style_prompt.len());
                                        video_style_prompt.insert_str(byte_idx, to_insert);
                                        let new_char_idx = cursor_idx + to_insert.chars().count();
                                        let new_cursor = egui::text::CCursor::new(new_char_idx);
                                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                                        state.store(ui.ctx(), win_te_id);
                                    } else {
                                        video_style_prompt.push_str(to_insert);
                                    }
                                } else {
                                    video_style_prompt.push_str(to_insert);
                                }
                                ui.ctx().memory_mut(|m| m.request_focus(win_te_id));
                            }
                        });
                    if !still_open {
                        ui.data_mut(|d| d.insert_persisted(style_expand_id, false));
                    }
                }
            }
        }

        // Розгорнуте вікно редагування промту
        if expand_open {
            let mut still_open = true;
            egui::Window::new(translate(language, "video_prompt_label"))
                .open(&mut still_open)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .default_size([600.0, 400.0])
                .show(ui.ctx(), |ui| {
                    let win_te_resp = egui::ScrollArea::vertical()
                        .max_height(ui.ctx().screen_rect().height() * 0.7)
                        .id_salt("win_video_prompt_scroll")
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(video_prompt)
                                    .desired_width(f32::INFINITY)
                                    .hint_text(translate(language, "video_prompt_hint")),
                            )
                        })
                        .inner;
                    let win_te_id = win_te_resp.id;
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button(translate(language, "video_insert_placeholder")).clicked() {
                            insert_at_cursor(ui, win_te_id, video_prompt, "{{text}}");
                            ui.ctx().memory_mut(|m| m.request_focus(win_te_id));
                        }
                        if *video_context_enabled
                            && ui.button(translate(language, "video_insert_context_placeholder")).clicked()
                        {
                            insert_at_cursor(ui, win_te_id, video_prompt, "{{context}}");
                            ui.ctx().memory_mut(|m| m.request_focus(win_te_id));
                        }
                    });
                });
            if !still_open {
                ui.data_mut(|d| d.insert_persisted(expand_id, false));
            }
        }

        // Вибір моделі та температури якщо обрано ЛЛМ-сервіс
        if video_llm_service != "None" {
            ui.add_space(8.0);

            if video_llm_service == "Claude Code" {
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("video_llm_claude_model")
                    .selected_text(if video_llm_model.is_empty() { "sonnet" } else { video_llm_model.as_str() })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(video_llm_model, "sonnet".to_string(), "sonnet");
                        ui.selectable_value(video_llm_model, "opus".to_string(), "opus");
                        ui.selectable_value(video_llm_model, "haiku".to_string(), "haiku");
                    });
                *video_llm_model_claude = video_llm_model.clone();
            } else if video_llm_service == "Gemini CLI" {
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("video_llm_gemini_model")
                    .selected_text(if video_llm_model.is_empty() { "gemini-2.5-flash" } else { video_llm_model.as_str() })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(video_llm_model, "gemini-2.5-flash".to_string(), "gemini-2.5-flash");
                        ui.selectable_value(video_llm_model, "gemini-2.5-pro".to_string(), "gemini-2.5-pro");
                        ui.selectable_value(video_llm_model, "gemini-3-flash-preview".to_string(), "gemini-3-flash-preview");
                        ui.selectable_value(video_llm_model, "gemini-3.1-pro-preview".to_string(), "gemini-3.1-pro-preview");
                        ui.selectable_value(video_llm_model, "gemini-2.5-flash-lite".to_string(), "gemini-2.5-flash-lite");
                    });
                *video_llm_model_gemini = video_llm_model.clone();
            } else if video_llm_service == "Codex CLI" {
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("video_llm_codex_model")
                    .selected_text(if video_llm_model.is_empty() { "gpt-5.4-mini" } else { video_llm_model.as_str() })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(video_llm_model, "gpt-5.5".to_string(), "gpt-5.5");
                        ui.selectable_value(video_llm_model, "gpt-5.4-mini".to_string(), "gpt-5.4-mini");
                    });
                *video_llm_model_codex = video_llm_model.clone();
            } else if video_llm_service == "AGY CLI" {
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("video_llm_agy_model")
                    .selected_text(if video_llm_model.is_empty() { "gemini-3.5-flash" } else { video_llm_model.as_str() })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(video_llm_model, "gemini-3.5-flash".to_string(), "gemini-3.5-flash");
                        ui.selectable_value(video_llm_model, "gemini-3.1-pro-preview".to_string(), "gemini-3.1-pro-preview");
                    });
                *video_llm_model_agy = video_llm_model.clone();
            } else if video_llm_service == "Pi CLI" {
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);
                let available_width = ui.available_width();
                ui.add(
                    egui::TextEdit::singleline(video_llm_model)
                        .hint_text("gemini-2.5-flash")
                        .desired_width(available_width),
                );
                *video_llm_model_pi = video_llm_model.clone();
            } else {
                // OpenRouter — дропдаун з пошуком
                ui.label(egui::RichText::new(translate(language, "model_label")).strong());
                ui.add_space(4.0);

                let is_loading = *openrouter_models_loading.lock().unwrap();
                let models_snapshot = openrouter_models.lock().unwrap().clone();

                if is_loading {
                    ui.label(egui::RichText::new(translate(language, "translation_models_loading")).weak().size(12.0));
                } else {
                    match models_snapshot {
                        None => {
                            *openrouter_models_loading.lock().unwrap() = true;
                            let models_arc = Arc::clone(openrouter_models);
                            let loading_arc = Arc::clone(openrouter_models_loading);
                            let ctx = ui.ctx().clone();
                            std::thread::spawn(move || {
                                let _permit = crate::api::openrouter::OpenRouterLimiter::get().acquire();
                                let agent = ureq::AgentBuilder::new()
                                    .timeout_connect(std::time::Duration::from_secs(10))
                                    .timeout(std::time::Duration::from_secs(15))
                                    .build();
                                let result = match agent
                                    .get("https://openrouter.ai/api/v1/models")
                                    .set("Accept", "application/json")
                                    .call()
                                {
                                    Ok(response) => match response.into_json::<crate::gui::pipeline::translation::ModelsResponse>() {
                                        Ok(data) => {
                                            let mut models = data.data;
                                            models.sort_by(|a, b| a.name.cmp(&b.name));
                                            Ok(models)
                                        }
                                        Err(e) => Err(format!("Помилка парсингу: {}", e)),
                                    },
                                    Err(e) => Err(format!("Помилка мережі: {}", e)),
                                };
                                *models_arc.lock().unwrap() = Some(result);
                                *loading_arc.lock().unwrap() = false;
                                ctx.request_repaint();
                            });
                            ui.label(egui::RichText::new(translate(language, "translation_models_loading")).weak().size(12.0));
                        }
                        Some(Ok(models)) => {
                            crate::gui::pipeline::translation::draw_model_selector(
                                ui,
                                language,
                                video_llm_model,
                                video_llm_model_search,
                                &models,
                            );
                            *video_llm_model_openrouter = video_llm_model.clone();
                        }
                        Some(Err(ref error)) => {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("❌ {}", error))
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .size(12.0),
                            ).wrap());
                            ui.add_space(4.0);
                            if ui.button(translate(language, "translation_models_retry")).clicked() {
                                *openrouter_models.lock().unwrap() = None;
                            }
                        }
                    }
                }
            }

            // Температура — тільки для OpenRouter
            if video_llm_service == "OpenRouter" {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(
                    format!("{}: {:.2}", translate(language, "translation_temperature_label"), *video_llm_temperature)
                ).strong());
                ui.add_space(4.0);
                let slider_width = ui.available_width();
                ui.scope(|ui| {
                    ui.style_mut().spacing.slider_width = slider_width;
                    ui.add(
                        egui::Slider::new(video_llm_temperature, 0.0..=2.0)
                            .step_by(0.01)
                            .show_value(false),
                    );
                });
            }
        }

        ui.add_space(8.0);

        // Перемикач режиму відеоряду: Генерація / Стоки
        let mut gen_mode = video_service.as_str() == "Googler";
        let mut stock_mode = video_service.as_str() == "Pexels" || video_service.as_str() == "Pixabay";

        ui.label(egui::RichText::new(translate(language, "video_service_mode_label")).strong());
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let was_gen = gen_mode;
            if ui.radio_value(&mut gen_mode, true, translate(language, "video_service_mode_generation")).changed() && !was_gen {
                *video_service = "Googler".to_string();
                stock_mode = false;
            }
            let was_stock = stock_mode;
            if ui.radio_value(&mut stock_mode, true, translate(language, "video_service_mode_stock")).changed() && !was_stock {
                *video_service = "Pexels".to_string();
                gen_mode = false;
            }
        });

        ui.add_space(6.0);

        // Вибір сервісу — залежно від режиму
        ui.label(egui::RichText::new(translate(language, "video_service_label")).strong());
        ui.add_space(4.0);

        if gen_mode {
            // Режим Генерація: Googler
            egui::ComboBox::from_id_salt("video_service_combo")
                .selected_text("Googler")
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(video_service, "Googler".to_string(), "Googler");
                });
        } else if stock_mode {
            // Режим Стоки: Pexels, Pixabay
            egui::ComboBox::from_id_salt("video_service_combo")
                .selected_text(if video_service.as_str() == "Pixabay" { "Pixabay Stock" } else { "Pexels Stock" })
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(video_service, "Pexels".to_string(), "Pexels Stock");
                    ui.selectable_value(video_service, "Pixabay".to_string(), "Pixabay Stock");
                });
        }

        if video_service.as_str() == "Pexels" {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(translate(language, "pexels_service_note"))
                    .color(ui.visuals().widgets.noninteractive.fg_stroke.color)
                    .size(11.0)
            );
        }

        if video_service.as_str() == "Pixabay" {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(translate(language, "pixabay_service_note"))
                    .color(ui.visuals().widgets.noninteractive.fg_stroke.color)
                    .size(11.0)
            );
        }

        if video_service.as_str() == "Googler" {
            ui.add_space(8.0);

            // Вибір типу медіа: зображення або відео
            ui.label(egui::RichText::new(translate(language, "video_media_type_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.radio_value(video_media_type, "image".to_string(), translate(language, "video_media_type_image"));
                ui.radio_value(video_media_type, "video".to_string(), translate(language, "video_media_type_video"));
            });

            ui.add_space(8.0);
            
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(translate(language, "video_upscale_label")).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    super::toggle_switch(ui, googler_video_upscale_enabled);
                });
            });

            if *googler_video_upscale_enabled {
                ui.add_space(6.0);
                ui.label(translate(language, "video_upscale_resolution_label"));
                ui.add_space(2.0);
                
                let selected_res_text = match googler_video_upscale_resolution.as_str() {
                    "2K" => "2K (2560x1440)",
                    "4K" => "4K (3840x2160)",
                    _ => "1080p (1920x1080)",
                };
                
                egui::ComboBox::from_id_salt("googler_video_upscale_resolution_combo")
                    .selected_text(selected_res_text)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(googler_video_upscale_resolution, "1080p".to_string(), "1080p (1920x1080)");
                        ui.selectable_value(googler_video_upscale_resolution, "2K".to_string(), "2K (2560x1440)");
                        ui.selectable_value(googler_video_upscale_resolution, "4K".to_string(), "4K (3840x2160)");
                    });

                ui.add_space(6.0);
                ui.label(translate(language, "video_upscale_quality_label"));
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.radio_value(googler_video_upscale_quality, "fast".to_string(), translate(language, "video_upscale_quality_fast"));
                    ui.radio_value(googler_video_upscale_quality, "balanced".to_string(), translate(language, "video_upscale_quality_balanced"));
                    ui.radio_value(googler_video_upscale_quality, "max".to_string(), translate(language, "video_upscale_quality_max"));
                });
            }

            ui.add_space(8.0);


            // Кнопка відкриття вікна пріоритетів
            let prio_id = ui.make_persistent_id("video_priorities_open");
            let mut prio_open: bool = ui.data_mut(|d| d.get_persisted(prio_id).unwrap_or(false));

            if ui.button(translate(language, "video_priorities_btn")).clicked() {
                prio_open = !prio_open;
                ui.data_mut(|d| d.insert_persisted(prio_id, prio_open));
            }

            if prio_open {
                let mut still_open = true;
                egui::Window::new(translate(language, "video_priorities_title"))
                    .open(&mut still_open)
                    .resizable(false)
                    .vscroll(true)
                    .collapsible(false)
                    .default_width(320.0)
                    .show(ui.ctx(), |ui| {
                        // Пріоритети зображень
                        ui.label(egui::RichText::new(translate(language, "video_priorities_image")).strong());
                        ui.add_space(4.0);
                        let mut swap_img: Option<(usize, usize)> = None;
                        for i in 0..googler_image_priority.len() {
                            let (name, credits) = image_provider_info(&googler_image_priority[i]);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("#{}", i + 1)).weak().monospace());
                                ui.add_space(4.0);
                                if arrow_button(ui, true, i > 0).clicked() {
                                    swap_img = Some((i - 1, i));
                                }
                                if arrow_button(ui, false, i < googler_image_priority.len() - 1).clicked() {
                                    swap_img = Some((i, i + 1));
                                }
                                ui.add_space(4.0);
                                ui.label(name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(credits).weak().size(11.0));
                                });
                            });
                        }
                        if let Some((a, b)) = swap_img {
                            googler_image_priority.swap(a, b);
                        }

                        ui.add_space(8.0);

                        // Пріоритети відео
                        ui.label(egui::RichText::new(translate(language, "video_priorities_video")).strong());
                        ui.add_space(4.0);
                        let mut swap_vid: Option<(usize, usize)> = None;
                        for i in 0..googler_video_priority.len() {
                            let provider_key = googler_video_priority[i].clone();
                            let (name, credits) = video_provider_info(&provider_key);
                            let is_disabled = googler_video_disabled.contains(&provider_key);
                            ui.horizontal(|ui| {
                                let mut enabled = !is_disabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    if enabled {
                                        googler_video_disabled.retain(|p| p != &provider_key);
                                    } else {
                                        googler_video_disabled.push(provider_key.clone());
                                    }
                                }
                                ui.label(egui::RichText::new(format!("#{}", i + 1)).weak().monospace());
                                ui.add_space(4.0);
                                if arrow_button(ui, true, i > 0).clicked() {
                                    swap_vid = Some((i - 1, i));
                                }
                                if arrow_button(ui, false, i < googler_video_priority.len() - 1).clicked() {
                                    swap_vid = Some((i, i + 1));
                                }
                                ui.add_space(4.0);
                                let label = egui::RichText::new(name);
                                let label = if is_disabled { label.weak() } else { label };
                                ui.label(label);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(credits).weak().size(11.0));
                                });
                            });
                        }
                        if let Some((a, b)) = swap_vid {
                            googler_video_priority.swap(a, b);
                        }

                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(translate(language, "video_priorities_retry_hint"))
                                .weak().size(11.0)
                        );
                    });
                if !still_open {
                    ui.data_mut(|d| d.insert_persisted(prio_id, false));
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Тригери медіа — накладення по ключовим фразам субтитрів
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(translate(language, "overlay_triggers_label")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                super::toggle_switch(ui, overlay_triggers_enabled);
            });
        });

        if *overlay_triggers_enabled {
            ui.add_space(6.0);

            let mut to_remove: Option<usize> = None;
            let mut to_duplicate: Option<usize> = None;

            for (idx, trigger) in overlay_triggers.iter_mut().enumerate() {
                ui.add_space(4.0);
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(translate(language, "overlay_triggers_phrase")).weak().size(11.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("×").color(egui::Color32::from_rgb(231, 76, 60))
                                    ).small()
                                ).clicked() {
                                    to_remove = Some(idx);
                                }
                                if duplicate_button(ui)
                                    .on_hover_text(translate(language, "overlay_triggers_duplicate"))
                                    .clicked()
                                {
                                    to_duplicate = Some(idx);
                                }
                            });
                        });
                        let btn_w = 68.0;
                        let spacing = ui.spacing().item_spacing.x;
                        let avail = ui.available_width();

                        ui.add_sized(
                            [avail, 18.0],
                            egui::TextEdit::singleline(&mut trigger.phrase)
                                .hint_text(translate(language, "overlay_triggers_phrase_hint")),
                        );

                        ui.add_space(4.0);

                        ui.label(egui::RichText::new(translate(language, "overlay_triggers_path")).weak().size(11.0));
                        ui.horizontal(|ui| {
                            let path_w = (avail - btn_w - spacing).max(40.0);
                            ui.add_sized(
                                [path_w, 18.0],
                                egui::TextEdit::singleline(&mut trigger.path)
                                    .hint_text(translate(language, "overlay_triggers_path_hint")),
                            );
                            if ui.add_sized([btn_w, 18.0], egui::Button::new(
                                translate(language, "overlay_triggers_select_file")
                            )).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Media", &["mp4", "mov", "avi", "mkv", "webm", "jpg", "jpeg", "png", "gif", "webp"])
                                    .pick_file()
                                {
                                    trigger.path = path.to_string_lossy().to_string();
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Grid вирівнює всі 4 колонки (мітка, значення, мітка, значення) рівномірно
                        egui::Grid::new(format!("trigger_xywh_{}", idx))
                            .num_columns(4)
                            .spacing([4.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("X:").weak().size(11.0));
                                ui.add(egui::DragValue::new(&mut trigger.x).speed(1.0).max_decimals(0));
                                ui.label(egui::RichText::new("Y:").weak().size(11.0));
                                ui.add(egui::DragValue::new(&mut trigger.y).speed(1.0).max_decimals(0));
                                ui.end_row();
                                ui.label(egui::RichText::new("W:").weak().size(11.0));
                                ui.add(egui::DragValue::new(&mut trigger.w).range(0..=9999).speed(1.0).max_decimals(0));
                                ui.label(egui::RichText::new("H:").weak().size(11.0));
                                ui.add(egui::DragValue::new(&mut trigger.h).range(0..=9999).speed(1.0).max_decimals(0));
                                ui.end_row();
                            });
                    });
            }

            if let Some(idx) = to_remove {
                overlay_triggers.remove(idx);
            }
            if let Some(idx) = to_duplicate {
                let cloned = overlay_triggers[idx].clone();
                overlay_triggers.insert(idx + 1, cloned);
            }

            ui.add_space(4.0);
            if ui.add_sized(
                [ui.available_width(), 22.0],
                egui::Button::new(translate(language, "overlay_triggers_add")),
            ).clicked() {
                overlay_triggers.push(crate::core::pipeline::montage::OverlayTrigger::default());
            }
        }

        ui.add_space(6.0);
    });
}
