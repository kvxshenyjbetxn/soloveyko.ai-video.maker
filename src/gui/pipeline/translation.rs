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
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

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

        ui.add_space(8.0);

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
