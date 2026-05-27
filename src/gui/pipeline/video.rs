use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

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
        "flow"   => ("Flow (VEO)",       "1 кр."),
        "flower" => ("Flower (Veo 3.1)", "1 кр."),
        "grok"   => ("Grok",             "1 кр."),
        _        => ("Unknown",          ""),
    }
}

/// Малює секцію "Відеоряд" на панелі пайплайну.
pub fn draw_video_section(
    ui: &mut egui::Ui,
    language: Language,
    video_service: &mut String,
    video_media_type: &mut String,
    text_split_mode: &mut String,
    text_split_char_limit: &mut usize,
    video_prompt: &mut String,
    googler_image_priority: &mut Vec<String>,
    googler_video_priority: &mut Vec<String>,
    video_llm_service: &mut String,
    video_llm_model: &mut String,
    video_llm_model_openrouter: &mut String,
    video_llm_model_claude: &mut String,
    video_llm_model_gemini: &mut String,
    video_llm_temperature: &mut f32,
    video_llm_model_search: &mut String,
    openrouter_models: &Arc<Mutex<Option<Result<Vec<crate::gui::pipeline::translation::OpenRouterModel>, String>>>>,
    openrouter_models_loading: &Arc<Mutex<bool>>,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

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

        // Вибір ЛЛМ-сервісу для генерації промтів
        ui.label(egui::RichText::new(translate(language, "video_llm_service_label")).strong());
        ui.add_space(4.0);

        let previous_llm_service = video_llm_service.clone();
        egui::ComboBox::from_id_salt("video_llm_service_combo")
            .selected_text(
                if video_llm_service == "Claude Code" {
                    translate(language, "translation_service_claude_code")
                } else if video_llm_service == "Gemini CLI" {
                    translate(language, "translation_service_gemini_cli")
                } else if video_llm_service == "OpenRouter" {
                    translate(language, "translation_service_openrouter")
                } else {
                    translate(language, "video_llm_service_none")
                }
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(video_llm_service, "None".to_string(), translate(language, "video_llm_service_none"));
                ui.selectable_value(video_llm_service, "OpenRouter".to_string(), translate(language, "translation_service_openrouter"));
                ui.selectable_value(video_llm_service, "Claude Code".to_string(), translate(language, "translation_service_claude_code"));
                ui.selectable_value(video_llm_service, "Gemini CLI".to_string(), translate(language, "translation_service_gemini_cli"));
            });

        // При зміні сервісу — відновлюємо відповідну збережену модель
        if *video_llm_service != previous_llm_service {
            if previous_llm_service == "OpenRouter" {
                *video_llm_model_openrouter = video_llm_model.clone();
            } else if previous_llm_service == "Claude Code" {
                *video_llm_model_claude = video_llm_model.clone();
            } else if previous_llm_service == "Gemini CLI" {
                *video_llm_model_gemini = video_llm_model.clone();
            }
            if video_llm_service == "OpenRouter" {
                *video_llm_model = video_llm_model_openrouter.clone();
            } else if video_llm_service == "Claude Code" {
                *video_llm_model = if video_llm_model_claude.is_empty() { "sonnet".to_string() } else { video_llm_model_claude.clone() };
            } else if video_llm_service == "Gemini CLI" {
                *video_llm_model = if video_llm_model_gemini.is_empty() { "gemini-2.5-flash".to_string() } else { video_llm_model_gemini.clone() };
            }
        }

        ui.add_space(8.0);

        // Промт для генерації зображень
        let expand_id = ui.make_persistent_id("video_prompt_expand");
        let mut expand_open: bool = ui.data_mut(|d| d.get_persisted(expand_id).unwrap_or(false));

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

        // Кнопка вставки плейсхолдера {{text}} за поточним положенням курсора
        if ui.button(translate(language, "video_insert_placeholder")).clicked() {
            let text_edit_id = te_resp.id;
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                let to_insert = "{{text}}";
                if let Some(cursor_range) = state.cursor.char_range() {
                    let cursor_idx = cursor_range.primary.index;
                    let byte_idx = video_prompt
                        .char_indices()
                        .map(|(b_idx, _)| b_idx)
                        .nth(cursor_idx)
                        .unwrap_or(video_prompt.len());
                    video_prompt.insert_str(byte_idx, to_insert);
                    let new_char_idx = cursor_idx + to_insert.chars().count();
                    let new_cursor = egui::text::CCursor::new(new_char_idx);
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                    state.store(ui.ctx(), text_edit_id);
                } else {
                    video_prompt.push_str(to_insert);
                }
            } else {
                video_prompt.push_str("{{text}}");
            }
            te_resp.request_focus();
        }

        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(translate(language, "video_placeholder_hint"))
                .weak()
                .size(11.0)
        );

        // Розгорнуте вікно редагування промту
        if expand_open {
            let mut still_open = true;
            egui::Window::new(translate(language, "video_prompt_label"))
                .open(&mut still_open)
                .resizable(true)
                .collapsible(false)
                .default_size([600.0, 400.0])
                .show(ui.ctx(), |ui| {
                    let te_height = (ui.available_height() - 36.0).max(100.0);
                    let win_te_resp = ui.add_sized(
                        [ui.available_width(), te_height],
                        egui::TextEdit::multiline(video_prompt)
                            .hint_text(translate(language, "video_prompt_hint")),
                    );
                    let win_te_id = win_te_resp.id;
                    ui.add_space(4.0);
                    if ui.button(translate(language, "video_insert_placeholder")).clicked() {
                        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), win_te_id) {
                            let to_insert = "{{text}}";
                            if let Some(cursor_range) = state.cursor.char_range() {
                                let cursor_idx = cursor_range.primary.index;
                                let byte_idx = video_prompt
                                    .char_indices()
                                    .map(|(b_idx, _)| b_idx)
                                    .nth(cursor_idx)
                                    .unwrap_or(video_prompt.len());
                                video_prompt.insert_str(byte_idx, to_insert);
                                let new_char_idx = cursor_idx + to_insert.chars().count();
                                let new_cursor = egui::text::CCursor::new(new_char_idx);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                                state.store(ui.ctx(), win_te_id);
                            } else {
                                video_prompt.push_str(to_insert);
                            }
                        } else {
                            video_prompt.push_str("{{text}}");
                        }
                        ui.ctx().memory_mut(|m| m.request_focus(win_te_id));
                    }
                });
            if !still_open {
                ui.data_mut(|d| d.insert_persisted(expand_id, false));
            }
        }

        // Вибір моделі та температури якщо обрано ЛЛМ-сервіс
        if video_llm_service != "None" {
            ui.add_space(8.0);

            if video_llm_service == "Claude Code" {
                ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
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
                ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
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
            } else {
                // OpenRouter — дропдаун з пошуком
                ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
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

        // Вибір сервісу — нижче, бо є технічними деталями
        ui.label(egui::RichText::new(translate(language, "video_service_label")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("video_service_combo")
            .selected_text(video_service.as_str())
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(video_service, "Googler".to_string(), "Googler");
            });

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
                            let (name, credits) = video_provider_info(&googler_video_priority[i]);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("#{}", i + 1)).weak().monospace());
                                ui.add_space(4.0);
                                if arrow_button(ui, true, i > 0).clicked() {
                                    swap_vid = Some((i - 1, i));
                                }
                                if arrow_button(ui, false, i < googler_video_priority.len() - 1).clicked() {
                                    swap_vid = Some((i, i + 1));
                                }
                                ui.add_space(4.0);
                                ui.label(name);
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

        ui.add_space(6.0);
    });
}
