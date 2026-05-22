use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
struct ModelsResponse {
    data: Vec<OpenRouterModel>,
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
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Вибір сервісу перекладу
        ui.label(egui::RichText::new(translate(language, "translation_service_label")).strong());
        ui.add_space(4.0);

        let mut service_changed = false;
        egui::ComboBox::from_id_salt("translation_service_combo")
            .selected_text(
                if translation_service == "Claude Code" {
                    translate(language, "translation_service_claude_code")
                } else {
                    translate(language, "translation_service_openrouter")
                }
            )
            .show_ui(ui, |ui| {
                if ui.selectable_value(translation_service, "OpenRouter".to_string(), translate(language, "translation_service_openrouter")).clicked() {
                    service_changed = true;
                }
                if ui.selectable_value(translation_service, "Claude Code".to_string(), translate(language, "translation_service_claude_code")).clicked() {
                    service_changed = true;
                }
            });

        if service_changed && translation_service == "Claude Code" {
            // Перевіряємо, чи модель валідна для Claude Code
            if translation_model != "sonnet" && translation_model != "opus" && translation_model != "haiku" {
                *translation_model = "sonnet".to_string();
            }
        }

        ui.add_space(8.0);

        // Поле промту для моделі перекладу
        ui.label(egui::RichText::new(translate(language, "translation_prompt_label")).strong());
        ui.add_space(4.0);

        let height_id = ui.make_persistent_id("translation_prompt_height");
        let prompt_height: f32 = ui.data_mut(|d| d.get_persisted(height_id).unwrap_or(60.0));
        let available_width = ui.available_width();

        let te_resp = ui.add_sized(
            [available_width, prompt_height],
            egui::TextEdit::multiline(translation_prompt)
                .hint_text(translate(language, "translation_prompt_hint")),
        );

        // Ручка зміни розміру
        let handle_rect = egui::Rect::from_min_size(
            egui::pos2(te_resp.rect.left(), te_resp.rect.bottom()),
            egui::vec2(te_resp.rect.width(), 8.0),
        );
        let handle_resp = ui.allocate_rect(handle_rect, egui::Sense::drag());

        if handle_resp.hovered() || handle_resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if handle_resp.dragged() {
            let new_height = (prompt_height + handle_resp.drag_delta().y).max(40.0);
            ui.data_mut(|d| d.insert_persisted(height_id, new_height));
        }

        if ui.is_rect_visible(handle_rect) {
            let color = if handle_resp.dragged() || handle_resp.hovered() {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke.color
            };
            let center = handle_rect.center();
            for i in -1_i32..=1 {
                ui.painter().circle_filled(
                    egui::pos2(center.x + i as f32 * 8.0, center.y),
                    2.0,
                    color,
                );
            }
        }

        ui.add_space(4.0);

        // Кнопка швидкої вставки плейсхолдера {{text}} за поточним положенням курсора
        if ui.button(translate(language, "translation_insert_placeholder")).clicked() {
            let text_edit_id = te_resp.id;
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                let to_insert = "{{text}}";
                if let Some(cursor_range) = state.cursor.char_range() {
                    let cursor_idx = cursor_range.primary.index;
                    
                    // Перетворюємо char індекс у byte індекс для безпечної роботи з UTF-8 рядком
                    let byte_idx = translation_prompt
                        .char_indices()
                        .map(|(b_idx, _)| b_idx)
                        .nth(cursor_idx)
                        .unwrap_or(translation_prompt.len());
                    
                    translation_prompt.insert_str(byte_idx, to_insert);
                    
                    // Встановлюємо курсор одразу після вставленого плейсхолдера
                    let new_char_idx = cursor_idx + to_insert.chars().count();
                    let new_cursor = egui::text::CCursor::new(new_char_idx);
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(new_cursor)));
                    state.store(ui.ctx(), text_edit_id);
                } else {
                    // Якщо поле не було у фокусі, додаємо в кінець
                    translation_prompt.push_str(to_insert);
                }
            } else {
                // Якщо стан ще не ініціалізовано
                translation_prompt.push_str("{{text}}");
            }
            te_resp.request_focus();
        }

        ui.add_space(8.0);

        if translation_service == "Claude Code" {
            // Вибір моделі Anthropic для Claude Code
            ui.label(egui::RichText::new(translate(language, "translation_model_label")).strong());
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("claude_code_model")
                .selected_text(if translation_model.is_empty() { "sonnet" } else { translation_model.as_str() })
                .show_ui(ui, |ui| {
                    ui.selectable_value(translation_model, "sonnet".to_string(), "sonnet");
                    ui.selectable_value(translation_model, "opus".to_string(), "opus");
                    ui.selectable_value(translation_model, "haiku".to_string(), "haiku");
                });
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

        if translation_service != "Claude Code" {
            ui.add_space(8.0);

            // Повзунок температури моделі
            ui.label(egui::RichText::new(
                format!("{}: {:.2}", translate(language, "translation_temperature_label"), *translation_temperature)
            ).strong());
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

        ui.add_space(6.0);
    });
}

/// Відображає кнопку-дропдаун з пошуком для вибору моделі OpenRouter.
fn draw_model_selector(
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
