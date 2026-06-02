use eframe::egui;
use crate::localization::{Language, translate};
use crate::gui::settings::storage::{TaskHistoryEntry, PipelineTemplate};

/// Форматує Unix timestamp у рядок "DD.MM HH:MM".
fn format_ts(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    if let chrono::LocalResult::Single(dt) = Local.timestamp_opt(ts, 0) {
        dt.format("%d.%m %H:%M").to_string()
    } else {
        String::new()
    }
}

/// Малює маленький чіп-бейдж з міткою етапу.
fn stage_chip(ui: &mut egui::Ui, label: &str, enabled: bool) {
    let color = if enabled {
        egui::Color32::from_rgb(46, 204, 113)
    } else {
        ui.visuals().widgets.inactive.fg_stroke.color
    };
    let bg = if enabled {
        egui::Color32::from_rgba_unmultiplied(46, 204, 113, 35)
    } else {
        egui::Color32::TRANSPARENT
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(9.0),
        color,
    );
    let size = galley.size() + egui::vec2(6.0, 3.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), bg);
        ui.painter().galley(rect.min + egui::vec2(3.0, 1.5), galley, color);
    }
}

/// Малює ліву панель з историчними задачами.
///
/// Повертає `Some((PipelineTemplate, text))` — налаштування і текст для відновлення.
pub fn draw_task_history_panel(
    ui: &mut egui::Ui,
    language: Language,
    entries: &[TaskHistoryEntry],
    delete_idx: &mut Option<usize>,
) -> Option<(PipelineTemplate, String)> {
    let mut apply: Option<(PipelineTemplate, String)> = None;

    let panel_width = ui.available_width();
    ui.set_max_width(panel_width);
    ui.set_min_width(panel_width);

    // Заголовок
    egui::Frame::none()
        .inner_margin(egui::Margin { left: 8.0, right: 8.0, top: 8.0, bottom: 4.0 })
        .show(ui, |ui| {
            ui.set_max_width(panel_width - 16.0);
            ui.add(egui::Label::new(
                egui::RichText::new(translate(language, "task_history_title"))
                    .size(16.0)
                    .strong(),
            ));
        });

    ui.separator();

    if entries.is_empty() {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(8.0, 8.0))
            .show(ui, |ui| {
                ui.set_max_width(panel_width - 16.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(translate(language, "task_history_empty"))
                            .weak()
                            .size(11.0),
                    )
                    .wrap(),
                );
            });
        return None;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.set_max_width(panel_width);

            // Новіші зверху — enumerate().rev() дає orig_idx = реальний індекс у векторі
            for (orig_idx, entry) in entries.iter().enumerate().rev() {
                // ID для Memory hover (унікальний за позицією у масиві)
                let mem_id = egui::Id::new("hist_hover").with(orig_idx);

                let inner_width = panel_width - 12.0; // 6px з кожного боку

                // Hover через rect з попереднього фрейму — fill малюється ДО вмісту
                let prev_rect = ui.ctx().data(|d| d.get_temp::<egui::Rect>(mem_id));
                let pointer = ui.ctx().pointer_hover_pos();
                let is_hovered = match (prev_rect, pointer) {
                    (Some(r), Some(p)) => r.contains(p),
                    _ => false,
                };
                let hover_fill = if is_hovered {
                    ui.visuals().widgets.hovered.weak_bg_fill
                } else {
                    egui::Color32::TRANSPARENT
                };

                // Frame з hover-фоном — малюється ДО вмісту, тому текст видно поверх
                let frame_resp = egui::Frame::none()
                    .fill(hover_fill)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 4.0, bottom: 4.0 })
                    .show(ui, |ui| {
                        ui.set_max_width(inner_width);

                        // Назва + кнопка видалення
                        ui.horizontal(|ui| {
                            let del_btn_w = 18.0;
                            let spacing = ui.spacing().item_spacing.x;
                            let name_w = (inner_width - del_btn_w - spacing).max(20.0);

                            ui.add_sized(
                                [name_w, 16.0],
                                egui::Label::new(
                                    egui::RichText::new(&entry.name).size(12.0).strong(),
                                )
                                .truncate(),
                            );

                            let del_resp = ui.add_sized(
                                [del_btn_w, 16.0],
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(9.0)
                                        .color(ui.visuals().widgets.inactive.fg_stroke.color),
                                )
                                .frame(false),
                            );
                            if del_resp.clicked() {
                                *delete_idx = Some(orig_idx);
                            }
                            del_resp.on_hover_text(translate(language, "task_history_delete_tooltip"));
                        });

                        // Дата + назва шаблону
                        ui.horizontal_wrapped(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format_ts(entry.created_at))
                                    .size(10.0)
                                    .weak(),
                            ));
                            if let Some(ref tmpl) = entry.template_name {
                                ui.add_space(3.0);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(format!("📄 {}", tmpl))
                                            .size(10.0)
                                            .weak(),
                                    )
                                    .truncate(),
                                );
                            }
                        });

                        // Чіпи увімкнених етапів
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            stage_chip(ui, translate(language, "task_history_stage_t"),   entry.settings.pipeline_translation_enabled);
                            stage_chip(ui, translate(language, "task_history_stage_v"),   entry.settings.pipeline_voiceover_enabled);
                            stage_chip(ui, translate(language, "task_history_stage_vid"), entry.settings.pipeline_video_enabled);
                            stage_chip(ui, translate(language, "task_history_stage_s"),   entry.settings.pipeline_subtitles_enabled);
                            stage_chip(ui, translate(language, "task_history_stage_m"),   entry.settings.pipeline_editing_enabled);
                        });
                    });

                // Зберігаємо rect для hover-detect в наступному фреймі
                let card_rect = frame_resp.response.rect;
                ui.ctx().data_mut(|d| d.insert_temp(mem_id, card_rect));

                // Клік по картці — через Response::interact, не реєструє новий widget ID
                let card_resp = frame_resp.response.interact(egui::Sense::click());
                if is_hovered {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if card_resp.clicked() {
                    apply = Some((entry.settings.clone(), entry.text.clone()));
                }

                ui.separator();
            }
        });

    apply
}
