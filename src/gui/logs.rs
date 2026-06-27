use crate::localization::{Language, translate};
use eframe::egui;

/// Повноекранне вікно логів конкретної задачі. Повертає false якщо треба закрити.
pub fn draw_job_logs_window(
    ctx: &egui::Context,
    language: Language,
    job_id: u64,
    job_name: &str,
    auto_scroll_logs: &mut bool,
    copied_toast: &mut Option<(String, std::time::Instant)>,
) -> bool {
    let mut is_open = true;
    let mut copied_toast_data = None;

    egui::Window::new(format!(
        "{} #{}: {}",
        translate(language, "job_logs_title"),
        job_id + 1,
        job_name
    ))
    .open(&mut is_open)
    .resizable(true)
    .default_size([550.0, 350.0])
    .show(ctx, |ui| {
        let job_logs = crate::logger::get_job_logs(job_id);
        if job_logs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.colored_label(
                    ui.visuals().weak_text_color(),
                    translate(language, "job_logs_empty"),
                );
                ui.add_space(40.0);
            });
        } else {
            ui.horizontal(|ui| {
                ui.checkbox(auto_scroll_logs, translate(language, "logs_autoscroll"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let copy_all_btn = egui::Button::new(translate(language, "job_logs_copy_all"))
                        .frame(true)
                        .rounding(4.0);
                    if ui.add(copy_all_btn).clicked() {
                        let all_job_logs = job_logs.join("\n");
                        ui.ctx().copy_text(all_job_logs.clone());
                        copied_toast_data = Some((all_job_logs, std::time::Instant::now()));
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let terminal_bg = if ui.visuals().dark_mode {
                egui::Color32::from_rgb(15, 15, 15)
            } else {
                egui::Color32::from_rgb(30, 30, 30)
            };

            egui::Frame::none()
                .fill(terminal_bg)
                .rounding(6.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(*auto_scroll_logs)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                for log_line in job_logs {
                                    let (time_part, msg_part) = if log_line.starts_with('[')
                                        && log_line.chars().nth(9) == Some(']')
                                    {
                                        (&log_line[0..10], &log_line[10..])
                                    } else {
                                        ("", log_line.as_str())
                                    };

                                    let is_error = msg_part.contains("помилка")
                                        || msg_part.contains("failed")
                                        || msg_part.contains("STDERR")
                                        || msg_part.contains("Error")
                                        || msg_part.contains("Err");
                                    let is_success = msg_part.contains("успішно")
                                        || msg_part.contains("success")
                                        || msg_part.contains("Ok");
                                    let is_command = msg_part.contains("Виконується:")
                                        || msg_part.contains("Запуск")
                                        || msg_part.contains("Running");

                                    let text_color = if is_error {
                                        egui::Color32::from_rgb(239, 83, 80)
                                    } else if is_success {
                                        egui::Color32::from_rgb(102, 187, 106)
                                    } else if is_command {
                                        egui::Color32::from_rgb(129, 212, 250)
                                    } else {
                                        egui::Color32::from_rgb(220, 220, 220)
                                    };

                                    let mut job = egui::text::LayoutJob::default();
                                    if !time_part.is_empty() {
                                        job.append(
                                            time_part,
                                            0.0,
                                            egui::TextFormat {
                                                font_id: egui::FontId::monospace(11.0),
                                                color: egui::Color32::from_gray(110),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    job.append(
                                        msg_part,
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::monospace(11.0),
                                            color: text_color,
                                            ..Default::default()
                                        },
                                    );

                                    let label_resp = ui.add(
                                        egui::Label::new(job).wrap().sense(egui::Sense::click()),
                                    );
                                    if label_resp.clicked() {
                                        ui.ctx().copy_text(log_line.clone());
                                        copied_toast_data =
                                            Some((log_line.clone(), std::time::Instant::now()));
                                    }
                                    label_resp
                                        .on_hover_text(translate(language, "logs_click_to_copy"));
                                    ui.add_space(3.0);
                                }
                            });
                        });
                });
        }
    });

    if let Some(toast) = copied_toast_data {
        *copied_toast = Some(toast);
    }

    is_open
}

/// Малює вкладку системних логів роботи додатку.
pub fn draw_logs_tab(
    ui: &mut egui::Ui,
    language: Language,
    auto_scroll_logs: &mut bool,
    copied_toast: &mut Option<(String, std::time::Instant)>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        // Кнопки керування логом у верхній панелі
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Кнопка очищення логів
                let clear_btn = egui::Button::new(
                    egui::RichText::new(translate(language, "logs_clear"))
                        .color(egui::Color32::from_rgb(239, 83, 80)),
                )
                .frame(true)
                .rounding(4.0);

                if ui.add(clear_btn).clicked() {
                    crate::logger::clear_logs();
                }

                ui.add_space(8.0);

                // Кнопка копіювання логів
                let copy_btn = egui::Button::new(translate(language, "logs_copy"))
                    .frame(true)
                    .rounding(4.0);

                if ui.add(copy_btn).clicked() {
                    let all_logs = crate::logger::get_logs().join("\n");
                    ui.ctx().copy_text(all_logs.clone());
                    *copied_toast = Some((all_logs, std::time::Instant::now()));
                }

                ui.add_space(12.0);

                // Чекбокс автопрокрутки
                ui.checkbox(auto_scroll_logs, translate(language, "logs_autoscroll"));
            });
        });

        ui.add_space(4.0);

        let logs = crate::logger::get_logs();

        if logs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(translate(language, "logs_empty"))
                        .weak()
                        .size(14.0),
                );
            });
        } else {
            let terminal_bg = if ui.visuals().dark_mode {
                egui::Color32::from_rgb(15, 15, 15)
            } else {
                egui::Color32::from_rgb(30, 30, 30)
            };

            egui::Frame::none()
                .fill(terminal_bg)
                .rounding(6.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .stick_to_bottom(*auto_scroll_logs)
                        .show(ui, |ui| {
                            for log_line in logs {
                                // Парсимо часову мітку та повідомлення
                                let (time_part, msg_part) = if log_line.starts_with('[')
                                    && log_line.chars().nth(9) == Some(']')
                                {
                                    (&log_line[0..10], &log_line[10..])
                                } else {
                                    ("", log_line.as_str())
                                };

                                // Визначаємо колір тексту залежно від типу події
                                let is_error = msg_part.contains("помилка")
                                    || msg_part.contains("failed")
                                    || msg_part.contains("STDERR")
                                    || msg_part.contains("Error")
                                    || msg_part.contains("Err");

                                let is_success = msg_part.contains("успішно")
                                    || msg_part.contains("success")
                                    || msg_part.contains("Ok");

                                let is_command = msg_part.contains("Виконується:")
                                    || msg_part.contains("Запуск")
                                    || msg_part.contains("Running");

                                let text_color = if is_error {
                                    egui::Color32::from_rgb(239, 83, 80)
                                } else if is_success {
                                    egui::Color32::from_rgb(102, 187, 106)
                                } else if is_command {
                                    egui::Color32::from_rgb(129, 212, 250)
                                } else {
                                    egui::Color32::from_rgb(220, 220, 220)
                                };

                                let mut job = egui::text::LayoutJob::default();

                                if !time_part.is_empty() {
                                    job.append(
                                        time_part,
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::monospace(11.0),
                                            color: egui::Color32::from_gray(110),
                                            ..Default::default()
                                        },
                                    );
                                }

                                job.append(
                                    msg_part,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::monospace(11.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );

                                let label_resp = ui
                                    .add(egui::Label::new(job).wrap().sense(egui::Sense::click()));

                                if label_resp.clicked() {
                                    ui.ctx().copy_text(log_line.clone());
                                    *copied_toast =
                                        Some((log_line.clone(), std::time::Instant::now()));
                                }

                                label_resp.on_hover_text(translate(language, "logs_click_to_copy"));

                                ui.add_space(3.0);
                            }
                        });
                });
        }
    });
}
