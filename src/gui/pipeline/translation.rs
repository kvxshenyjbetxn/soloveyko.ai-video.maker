use crate::localization::{translate, Language};
use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

/// Малює секцію "Переклад" на панелі пайплайну.
pub fn draw_translation_section(
    ui: &mut egui::Ui,
    language: Language,
    translation_prompt: &mut String,
    translation_model: &mut String,
    translation_model_search: &mut String,
    openrouter_models: &Arc<Mutex<Option<Result<Vec<OpenRouterModel>, String>>>>,
    openrouter_models_loading: &Arc<Mutex<bool>>,
    translation_temperature: &mut f32,
    translation_service: &mut String,
    translation_model_openrouter: &mut String,
    translation_model_claude: &mut String,
    translation_model_gemini: &mut String,
    translation_model_codex: &mut String,
    translation_model_agy: &mut String,
    translation_model_pi: &mut String,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Зберігаємо початковий сервіс для виявлення перемикання
        let previous_service = translation_service.clone();

        // Вибір сервісу перекладу
        ui.label(egui::RichText::new(translate(language, "translation_service_label")).strong());
        ui.add_space(4.0);

        let mut service_changed = false;
        egui::ComboBox::from_id_salt("translation_service_combo")
            .selected_text(if translation_service == "Claude Code" {
                translate(language, "translation_service_claude_code")
            } else if translation_service == "Gemini CLI" {
                translate(language, "translation_service_gemini_cli")
            } else if translation_service == "Codex CLI" {
                translate(language, "translation_service_codex_cli")
            } else if translation_service == "AGY CLI" {
                translate(language, "translation_service_agy_cli")
            } else if translation_service == "Pi CLI" {
                translate(language, "translation_service_pi_cli")
            } else {
                translate(language, "translation_service_openrouter")
            })
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        translation_service,
                        "OpenRouter".to_string(),
                        translate(language, "translation_service_openrouter"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
                if ui
                    .selectable_value(
                        translation_service,
                        "Claude Code".to_string(),
                        translate(language, "translation_service_claude_code"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
                if ui
                    .selectable_value(
                        translation_service,
                        "Gemini CLI".to_string(),
                        translate(language, "translation_service_gemini_cli"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
                if ui
                    .selectable_value(
                        translation_service,
                        "Codex CLI".to_string(),
                        translate(language, "translation_service_codex_cli"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
                if ui
                    .selectable_value(
                        translation_service,
                        "AGY CLI".to_string(),
                        translate(language, "translation_service_agy_cli"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
                if ui
                    .selectable_value(
                        translation_service,
                        "Pi CLI".to_string(),
                        translate(language, "translation_service_pi_cli"),
                    )
                    .clicked()
                {
                    service_changed = true;
                }
            });

        if translation_service != &previous_service {
            // Зберігаємо поточну модель у слот попереднього сервісу
            if previous_service == "OpenRouter" {
                *translation_model_openrouter = translation_model.clone();
            } else if previous_service == "Claude Code" {
                *translation_model_claude = translation_model.clone();
            } else if previous_service == "Gemini CLI" {
                *translation_model_gemini = translation_model.clone();
            } else if previous_service == "Codex CLI" {
                *translation_model_codex = translation_model.clone();
            } else if previous_service == "AGY CLI" {
                *translation_model_agy = translation_model.clone();
            } else if previous_service == "Pi CLI" {
                *translation_model_pi = translation_model.clone();
            }

            // Завантажуємо збережену модель для нового сервісу
            if translation_service == "OpenRouter" {
                *translation_model = translation_model_openrouter.clone();
            } else if translation_service == "Claude Code" {
                *translation_model = if translation_model_claude.is_empty() {
                    "sonnet".to_string()
                } else {
                    translation_model_claude.clone()
                };
            } else if translation_service == "Gemini CLI" {
                *translation_model = if translation_model_gemini.is_empty() {
                    "gemini-2.5-flash".to_string()
                } else {
                    translation_model_gemini.clone()
                };
            } else if translation_service == "Codex CLI" {
                *translation_model = if translation_model_codex.is_empty() {
                    "gpt-5.4-mini".to_string()
                } else {
                    translation_model_codex.clone()
                };
            } else if translation_service == "AGY CLI" {
                *translation_model = if translation_model_agy.is_empty() {
                    "default".to_string()
                } else {
                    translation_model_agy.clone()
                };
            } else if translation_service == "Pi CLI" {
                *translation_model = if translation_model_pi.is_empty() {
                    "gemini-2.5-flash".to_string()
                } else {
                    translation_model_pi.clone()
                };
            }
            service_changed = true;
        }

        if service_changed && translation_service == "Claude Code" {
            // Перевіряємо, чи модель валідна для Claude Code
            if translation_model != "sonnet"
                && translation_model != "opus"
                && translation_model != "haiku"
            {
                *translation_model = "sonnet".to_string();
            }
        }
        if service_changed && translation_service == "Gemini CLI" {
            // Перевіряємо, чи модель валідна для Gemini CLI
            if translation_model != "gemini-2.5-flash"
                && translation_model != "gemini-2.5-pro"
                && translation_model != "gemini-3-flash-preview"
                && translation_model != "gemini-3.1-pro-preview"
                && translation_model != "gemini-2.5-flash-lite"
            {
                *translation_model = "gemini-2.5-flash".to_string();
            }
        }
        if service_changed && translation_service == "Codex CLI" {
            // Перевіряємо, чи модель валідна для Codex CLI
            if translation_model != "gpt-5.5" && translation_model != "gpt-5.4-mini" {
                *translation_model = "gpt-5.4-mini".to_string();
            }
        }
        if service_changed && translation_service == "AGY CLI" {
            if translation_model.is_empty() {
                *translation_model = "default".to_string();
            }
        }
        if service_changed && translation_service == "Pi CLI" {
            if translation_model.is_empty() {
                *translation_model = "gemini-2.5-flash".to_string();
            }
        }

        ui.add_space(8.0);

        // Поле промту для моделі перекладу
        let expand_id = ui.make_persistent_id("translation_prompt_expand");
        let mut expand_open: bool = ui.data_mut(|d| d.get_persisted(expand_id).unwrap_or(false));

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(translate(language, "translation_prompt_label")).strong());
            if ui
                .small_button("⛶")
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
            .id_salt("translation_prompt_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(translation_prompt)
                        .desired_width(available_width)
                        .hint_text(translate(language, "translation_prompt_hint")),
                )
            })
            .inner;

        ui.add_space(4.0);

        // Кнопка швидкої вставки плейсхолдера {{text}} за поточним положенням курсора
        if ui
            .button(translate(language, "translation_insert_placeholder"))
            .clicked()
        {
            let text_edit_id = te_resp.id;
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                let to_insert = "{{text}}";
                if let Some(cursor_range) = state.cursor.char_range() {
                    let cursor_idx = cursor_range.primary.index;
                    let byte_idx = translation_prompt
                        .char_indices()
                        .map(|(b_idx, _)| b_idx)
                        .nth(cursor_idx)
                        .unwrap_or(translation_prompt.len());
                    translation_prompt.insert_str(byte_idx, to_insert);
                    let new_char_idx = cursor_idx + to_insert.chars().count();
                    let new_cursor = egui::text::CCursor::new(new_char_idx);
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                    state.store(ui.ctx(), text_edit_id);
                } else {
                    translation_prompt.push_str(to_insert);
                }
            } else {
                translation_prompt.push_str("{{text}}");
            }
            te_resp.request_focus();
        }

        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(translate(language, "translation_placeholder_hint"))
                .weak()
                .size(11.0),
        );

        // Розгорнуте вікно редагування промту
        if expand_open {
            let mut still_open = true;
            egui::Window::new(translate(language, "translation_prompt_label"))
                .open(&mut still_open)
                .resizable(true)
                .collapsible(false)
                .constrain(true)
                .default_size([600.0, 400.0])
                .show(ui.ctx(), |ui| {
                    let win_te_resp = egui::ScrollArea::vertical()
                        .max_height(ui.ctx().screen_rect().height() * 0.7)
                        .id_salt("win_translation_prompt_scroll")
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(translation_prompt)
                                    .desired_width(f32::INFINITY)
                                    .hint_text(translate(language, "translation_prompt_hint")),
                            )
                        })
                        .inner;
                    let win_te_id = win_te_resp.id;
                    ui.add_space(4.0);
                    if ui
                        .button(translate(language, "translation_insert_placeholder"))
                        .clicked()
                    {
                        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), win_te_id) {
                            let to_insert = "{{text}}";
                            if let Some(cursor_range) = state.cursor.char_range() {
                                let cursor_idx = cursor_range.primary.index;
                                let byte_idx = translation_prompt
                                    .char_indices()
                                    .map(|(b_idx, _)| b_idx)
                                    .nth(cursor_idx)
                                    .unwrap_or(translation_prompt.len());
                                translation_prompt.insert_str(byte_idx, to_insert);
                                let new_char_idx = cursor_idx + to_insert.chars().count();
                                let new_cursor = egui::text::CCursor::new(new_char_idx);
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(
                                        new_cursor,
                                    )));
                                state.store(ui.ctx(), win_te_id);
                            } else {
                                translation_prompt.push_str(to_insert);
                            }
                        } else {
                            translation_prompt.push_str("{{text}}");
                        }
                        ui.ctx().memory_mut(|m| m.request_focus(win_te_id));
                    }
                });
            if !still_open {
                ui.data_mut(|d| d.insert_persisted(expand_id, false));
            }
        }

        ui.add_space(8.0);

        if translation_service == "Claude Code" {
            // Вибір моделі Anthropic для Claude Code
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("claude_code_model")
                .selected_text(if translation_model.is_empty() {
                    "sonnet"
                } else {
                    translation_model.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(translation_model, "sonnet".to_string(), "sonnet");
                    ui.selectable_value(translation_model, "opus".to_string(), "opus");
                    ui.selectable_value(translation_model, "haiku".to_string(), "haiku");
                });
        } else if translation_service == "Gemini CLI" {
            // Вибір моделі Google для Gemini CLI
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("gemini_cli_model")
                .selected_text(if translation_model.is_empty() {
                    "gemini-2.5-flash"
                } else {
                    translation_model.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        translation_model,
                        "gemini-2.5-flash".to_string(),
                        "gemini-2.5-flash",
                    );
                    ui.selectable_value(
                        translation_model,
                        "gemini-2.5-pro".to_string(),
                        "gemini-2.5-pro",
                    );
                    ui.selectable_value(
                        translation_model,
                        "gemini-3-flash-preview".to_string(),
                        "gemini-3-flash-preview",
                    );
                    ui.selectable_value(
                        translation_model,
                        "gemini-3.1-pro-preview".to_string(),
                        "gemini-3.1-pro-preview",
                    );
                    ui.selectable_value(
                        translation_model,
                        "gemini-2.5-flash-lite".to_string(),
                        "gemini-2.5-flash-lite",
                    );
                });
        } else if translation_service == "Codex CLI" {
            // Вибір моделі для Codex CLI
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("codex_cli_model")
                .selected_text(if translation_model.is_empty() {
                    "gpt-5.4-mini"
                } else {
                    translation_model.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(translation_model, "gpt-5.5".to_string(), "gpt-5.5");
                    ui.selectable_value(
                        translation_model,
                        "gpt-5.4-mini".to_string(),
                        "gpt-5.4-mini",
                    );
                });
        } else if translation_service == "AGY CLI" {
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("translation_agy_model")
                .selected_text(if translation_model.is_empty() {
                    "gemini-3.5-flash"
                } else {
                    translation_model.as_str()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        translation_model,
                        "gemini-3.5-flash".to_string(),
                        "gemini-3.5-flash",
                    );
                    ui.selectable_value(
                        translation_model,
                        "gemini-3.1-pro-preview".to_string(),
                        "gemini-3.1-pro-preview",
                    );
                });
        } else if translation_service == "Pi CLI" {
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);
            // Pi підтримує будь-який провайдер/модель у форматі "provider/model" або просто "model"
            let available_width = ui.available_width();
            ui.add(
                egui::TextEdit::singleline(translation_model)
                    .hint_text("gemini-2.5-flash")
                    .desired_width(available_width),
            );
        } else {
            // Вибір моделі OpenRouter
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);

            let is_loading = *openrouter_models_loading.lock().unwrap();
            let models_snapshot = openrouter_models.lock().unwrap().clone();

            if is_loading {
                ui.label(
                    egui::RichText::new(translate(language, "translation_models_loading"))
                        .weak()
                        .size(12.0),
                );
            } else {
                match models_snapshot {
                    None => {
                        // Запускаємо завантаження моделей у фоновому потоці
                        *openrouter_models_loading.lock().unwrap() = true;
                        let models_arc = Arc::clone(openrouter_models);
                        let loading_arc = Arc::clone(openrouter_models_loading);
                        let ctx = ui.ctx().clone();

                        std::thread::spawn(move || {
                            let _permit =
                                crate::api::openrouter::OpenRouterLimiter::get().acquire();

                            let agent = ureq::AgentBuilder::new()
                                .timeout_connect(std::time::Duration::from_secs(10))
                                .timeout(std::time::Duration::from_secs(15))
                                .build();

                            let result = match agent
                                .get("https://openrouter.ai/api/v1/models")
                                .set("Accept", "application/json")
                                .call()
                            {
                                Ok(response) => match response.into_json::<ModelsResponse>() {
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

                        ui.label(
                            egui::RichText::new(translate(language, "translation_models_loading"))
                                .weak()
                                .size(12.0),
                        );
                    }
                    Some(Ok(models)) => {
                        draw_model_selector(
                            ui,
                            language,
                            translation_model,
                            translation_model_search,
                            &models,
                        );
                    }
                    Some(Err(ref error)) => {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("❌ {}", error))
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .size(12.0),
                            )
                            .wrap(),
                        );
                        ui.add_space(4.0);
                        if ui
                            .button(translate(language, "translation_models_retry"))
                            .clicked()
                        {
                            *openrouter_models.lock().unwrap() = None;
                        }
                    }
                }
            }
        }

        if translation_service != "Claude Code"
            && translation_service != "Gemini CLI"
            && translation_service != "Codex CLI"
            && translation_service != "AGY CLI"
            && translation_service != "Pi CLI"
        {
            ui.add_space(8.0);

            // Повзунок температури моделі
            ui.label(
                egui::RichText::new(format!(
                    "{}: {:.2}",
                    translate(language, "translation_temperature_label"),
                    *translation_temperature
                ))
                .strong(),
            );
            ui.add_space(4.0);
            let slider_width = ui.available_width();
            ui.scope(|ui| {
                ui.style_mut().spacing.slider_width = slider_width;
                ui.add(
                    egui::Slider::new(translation_temperature, 0.0..=2.0)
                        .step_by(0.01)
                        .show_value(false),
                );
            });
        }

        // Синхронізуємо активну модель з відповідним слотом
        if translation_service == "OpenRouter" {
            *translation_model_openrouter = translation_model.clone();
        } else if translation_service == "Claude Code" {
            *translation_model_claude = translation_model.clone();
        } else if translation_service == "Gemini CLI" {
            *translation_model_gemini = translation_model.clone();
        } else if translation_service == "Codex CLI" {
            *translation_model_codex = translation_model.clone();
        } else if translation_service == "AGY CLI" {
            *translation_model_agy = translation_model.clone();
        } else if translation_service == "Pi CLI" {
            *translation_model_pi = translation_model.clone();
        }

        ui.add_space(6.0);
    });
}

/// Відображає кнопку-дропдаун з пошуком для вибору моделі OpenRouter.
pub fn draw_model_selector(
    ui: &mut egui::Ui,
    language: Language,
    translation_model: &mut String,
    translation_model_search: &mut String,
    models: &[OpenRouterModel],
) {
    let selected_name = models
        .iter()
        .find(|m| m.id == *translation_model)
        .map(|m| m.name.as_str())
        .unwrap_or(translate(language, "translation_model_hint"));

    let popup_id = ui.make_persistent_id("translation_model_popup");
    let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));

    let btn = ui.add_sized(
        [ui.available_width(), 20.0],
        egui::Button::new(selected_name),
    );

    if btn.clicked() {
        ui.memory_mut(|mem| {
            if is_open {
                mem.close_popup();
            } else {
                mem.open_popup(popup_id);
            }
        });
        if !is_open {
            translation_model_search.clear();
        }
    }

    egui::popup_below_widget(
        ui,
        popup_id,
        &btn,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(200.0);
            ui.set_max_width(450.0);

            ui.add(
                egui::TextEdit::singleline(translation_model_search)
                    .hint_text(translate(language, "translation_model_search"))
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(4.0);

            let search_lower = translation_model_search.to_lowercase();
            let filtered: Vec<&OpenRouterModel> = models
                .iter()
                .filter(|m| {
                    search_lower.is_empty()
                        || m.name.to_lowercase().contains(&search_lower)
                        || m.id.to_lowercase().contains(&search_lower)
                })
                .collect();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for model in filtered {
                        let selected = model.id == *translation_model;
                        if ui.selectable_label(selected, &model.name).clicked() {
                            *translation_model = model.id.clone();
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                });
        },
    );
}
