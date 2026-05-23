use eframe::egui;
use crate::localization::{Language, translate};

/// Кнопка зі стрілкою вгору або вниз, намальованою через Painter (незалежно від шрифту).
fn arrow_button(ui: &mut egui::Ui, up: bool, enabled: bool) -> egui::Response {
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
fn image_provider_info(key: &str) -> (&'static str, &'static str) {
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
fn video_provider_info(key: &str) -> (&'static str, &'static str) {
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
    text_split_mode: &mut String,
    text_split_char_limit: &mut usize,
    video_prompt: &mut String,
    googler_image_priority: &mut Vec<String>,
    googler_video_priority: &mut Vec<String>,
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

        // Промт для генерації зображень
        ui.label(egui::RichText::new(translate(language, "video_prompt_label")).strong());
        ui.add_space(4.0);

        let height_id = ui.make_persistent_id("video_prompt_height");
        let prompt_height: f32 = ui.data_mut(|d| d.get_persisted(height_id).unwrap_or(60.0));
        let available_width = ui.available_width();

        let te_resp = ui.add_sized(
            [available_width, prompt_height],
            egui::TextEdit::multiline(video_prompt)
                .hint_text(translate(language, "video_prompt_hint")),
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
