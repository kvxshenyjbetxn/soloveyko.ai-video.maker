use eframe::egui;
use crate::localization::{Language, translate};
use crate::gui::settings::storage::{TaskHistoryEntry, PipelineTemplate};

fn format_ts(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    if let chrono::LocalResult::Single(dt) = Local.timestamp_opt(ts, 0) {
        dt.format("%d.%m %H:%M").to_string()
    } else {
        String::new()
    }
}

/// Малює 5 крапок-індикаторів увімкнених етапів через painter — без тексту.
fn stage_dots(ui: &mut egui::Ui, language: Language, settings: &PipelineTemplate) {
    let stages = [
        (settings.pipeline_translation_enabled, translate(language, "task_history_stage_t")),
        (settings.pipeline_voiceover_enabled,   translate(language, "task_history_stage_v")),
        (settings.pipeline_video_enabled,       translate(language, "task_history_stage_vid")),
        (settings.pipeline_subtitles_enabled,   translate(language, "task_history_stage_s")),
        (settings.pipeline_editing_enabled,     translate(language, "task_history_stage_m")),
    ];

    let r = 3.5_f32;
    let gap = 5.0_f32;
    let total_w = stages.len() as f32 * (r * 2.0) + (stages.len() - 1) as f32 * gap;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(total_w, r * 2.0), egui::Sense::hover());

    let painter = ui.painter();
    let mut cx = rect.min.x + r;
    let cy = rect.center().y;

    for (enabled, _label) in &stages {
        let color = if *enabled {
            egui::Color32::from_rgb(46, 204, 113)
        } else {
            egui::Color32::from_rgba_unmultiplied(120, 120, 120, 60)
        };
        painter.circle_filled(egui::pos2(cx, cy), r, color);
        cx += r * 2.0 + gap;
    }

    // Tooltip: перелік увімкнених етапів
    let active: Vec<&str> = stages.iter()
        .filter(|(en, _)| *en)
        .map(|(_, lbl)| *lbl)
        .collect();
    if !active.is_empty() {
        resp.on_hover_text(active.join(", "));
    }
}

/// Малює ліву панель з историчними задачами.
///
/// Повертає `Some((PipelineTemplate, text))` — налаштування і текст для відновлення.
pub fn draw_task_history_panel(
    ui: &mut egui::Ui,
    language: Language,
    entries: &[TaskHistoryEntry],
    _delete_idx: &mut Option<usize>,
) -> Option<(PipelineTemplate, String)> {
    let mut apply: Option<(PipelineTemplate, String)> = None;

    let panel_width = ui.available_width();
    ui.set_max_width(panel_width);
    ui.set_min_width(panel_width);

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

            for (orig_idx, entry) in entries.iter().enumerate().rev() {
                let mem_id = egui::Id::new("hist_hover").with(orig_idx);
                let inner_width = panel_width - 12.0;

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

                let frame_resp = egui::Frame::none()
                    .fill(hover_fill)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 4.0, bottom: 4.0 })
                    .show(ui, |ui| {
                        ui.set_max_width(inner_width);

                        // Рядок 1: назва
                        ui.add_sized(
                            [inner_width, 16.0],
                            egui::Label::new(
                                egui::RichText::new(&entry.name).size(12.0).strong(),
                            )
                            .truncate(),
                        );

                        // Рядок 2: дата + крапки етапів в одному рядку
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format_ts(entry.created_at))
                                    .size(10.0)
                                    .weak(),
                            ));
                            ui.add_space(4.0);
                            stage_dots(ui, language, &entry.settings);
                        });
                    });

                let card_rect = frame_resp.response.rect;
                ui.ctx().data_mut(|d| d.insert_temp(mem_id, card_rect));

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
