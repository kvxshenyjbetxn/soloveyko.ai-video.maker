use eframe::egui;
use crate::localization::{Language, translate};

/// Вікно чату з агентом — відображає историю, поле для введення та кнопки керування.
#[allow(clippy::too_many_arguments)]
pub fn draw_agent_chat_window(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    selected_agent_chat: &mut Option<u64>,
    agent_chat_input: &mut String,
    agent_chat_loading: &std::sync::Arc<std::sync::Mutex<bool>>,
    agent_chat_error: &mut Option<String>,
    agent_chat_result: &std::sync::Arc<std::sync::Mutex<Option<Result<String, String>>>>,
) {
    let job_id = match *selected_agent_chat {
        Some(id) => id,
        None => return,
    };

    let Some(job_idx) = jobs.iter().position(|j| j.id == job_id) else {
        *selected_agent_chat = None;
        return;
    };

    let job_name = jobs[job_idx].name.clone();
    let job_status = jobs[job_idx].status.lock().unwrap().clone();
    let agent_chat_arc = std::sync::Arc::clone(&jobs[job_idx].agent_chat);
    let agent_session_arc = std::sync::Arc::clone(&jobs[job_idx].agent_session);
    let agent_control_resume_arc = std::sync::Arc::clone(&jobs[job_idx].agent_control_resume);
    let job_settings = jobs[job_idx].settings.clone();
    let job_id_clone = job_id;

    // Перевіряємо результат фонової відповіді агента
    {
        let result = agent_chat_result.lock().unwrap().take();
        if let Some(res) = result {
            match res {
                Ok(response) => {
                    agent_chat_arc.lock().unwrap().push(crate::queue::AgentChatMessage {
                        role: "agent".to_string(),
                        content: response,
                    });
                    let save_dir = std::path::Path::new(&job_settings.save_path);
                    save_agent_chat(save_dir, &agent_chat_arc.lock().unwrap());
                    *agent_chat_error = None;
                }
                Err(e) => {
                    *agent_chat_error = Some(e);
                }
            }
        }
    }

    let mut is_open = true;
    let mut trigger_send = false;
    let mut trigger_continue = false;

    let title = format!("{} — #{} {}",
        translate(language, "agent_chat_title"),
        job_id + 1,
        job_name,
    );

    egui::Window::new(title)
        .open(&mut is_open)
        .resizable(true)
        .min_size([380.0, 300.0])
        .default_size([640.0, 760.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Область прокрутки з повідомленнями чату
                let chat_snapshot = agent_chat_arc.lock().unwrap().clone();

                let is_loading = *agent_chat_loading.lock().unwrap();
                // Спінер для всіх агентів — показується внизу біля кнопки надіслати
                let pipeline_generating = job_status == crate::queue::JobStatus::Running
                    && chat_snapshot.last().map(|m| m.role == "agent").unwrap_or(false);
                let show_spinner = is_loading || pipeline_generating;

                // ScrollArea займає весь простір вікна за винятком нижньої панелі (~120px)
                let scroll_height = (ui.available_height() - 120.0).max(80.0);
                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        for msg in &chat_snapshot {
                            let is_user = msg.role == "user";
                            let bg_color = if is_user {
                                ui.visuals().selection.bg_fill.linear_multiply(0.25)
                            } else {
                                ui.visuals().widgets.noninteractive.bg_fill
                            };
                            let role_label = if is_user { "You" } else { "Agent" };
                            let role_color = if is_user {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::from_rgb(46, 204, 113)
                            };

                            egui::Frame::none()
                                .fill(bg_color)
                                .inner_margin(egui::Margin::same(8.0))
                                .rounding(egui::Rounding::same(6.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(role_label)
                                            .strong()
                                            .size(11.0)
                                            .color(role_color),
                                    );
                                    ui.add_space(2.0);
                                    if is_user {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&msg.content).size(12.0)
                                        ).wrap());
                                    } else {
                                        render_message_content(ui, &msg.content);
                                    }
                                });
                            ui.add_space(4.0);
                        }
                        if chat_snapshot.is_empty() {
                            ui.label(
                                egui::RichText::new("— история чату порожня —")
                                    .weak()
                                    .size(11.0)
                                    .italics(),
                            );
                        }
                    });

                ui.separator();
                ui.add_space(4.0);

                // Відображаємо помилку якщо є
                if let Some(ref err) = *agent_chat_error {
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("{} {}", translate(language, "agent_chat_error"), err))
                            .color(egui::Color32::from_rgb(231, 76, 60))
                            .size(11.0),
                    ).wrap());
                    ui.add_space(4.0);
                }

                let has_session = agent_session_arc.lock().unwrap().is_some();

                // Поле введення повідомлення (Enter = надіслати, Shift+Enter = новий рядок)
                let input_response = ui.add(
                    egui::TextEdit::multiline(agent_chat_input)
                        .hint_text(translate(language, "agent_chat_input_hint"))
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                );
                if input_response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                {
                    if !is_loading && has_session && !agent_chat_input.trim().is_empty() {
                        trigger_send = true;
                    }
                }

                ui.add_space(4.0);

                // Кнопки + спінер в одному рядку
                ui.horizontal(|ui| {
                    // Спінер зліва від кнопки надіслати — для всіх агентів
                    if show_spinner {
                        ui.add(egui::Spinner::new().size(16.0));
                        ui.add_space(6.0);
                    }

                    if ui.add_enabled(
                        !is_loading && has_session && !agent_chat_input.trim().is_empty(),
                        egui::Button::new(translate(language, "agent_chat_send_btn")),
                    ).clicked() {
                        trigger_send = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if job_settings.agent_control_enabled {
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new(translate(language, "agent_chat_continue_btn")).strong()
                                )
                            ).clicked() {
                                trigger_continue = true;
                            }
                        }
                    });
                });
            });
        });

    // Надсилаємо повідомлення агенту через --resume
    if trigger_send {
        let message = agent_chat_input.trim().to_string();
        if !message.is_empty() {
            let session_snapshot = agent_session_arc.lock().unwrap().clone();
            if let Some(session) = session_snapshot {
                agent_chat_arc.lock().unwrap().push(crate::queue::AgentChatMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                });
                let save_dir = std::path::Path::new(&job_settings.save_path);
                save_agent_chat(save_dir, &agent_chat_arc.lock().unwrap());
                agent_chat_input.clear();
                *agent_chat_error = None;

                let result_arc = std::sync::Arc::clone(agent_chat_result);
                let loading_arc = std::sync::Arc::clone(agent_chat_loading);
                let ctx_clone = ctx.clone();
                let save_path = job_settings.save_path.clone();
                *loading_arc.lock().unwrap() = true;

                std::thread::spawn(move || {
                    let result = crate::core::pipeline::call_agent_resume(
                        &session.service,
                        &session.model,
                        &message,
                        &session.session_id,
                        Some((job_id_clone, String::new())),
                        Some(&save_path),
                    );
                    *result_arc.lock().unwrap() = Some(result);
                    *loading_arc.lock().unwrap() = false;
                    ctx_clone.request_repaint();
                });
            }
        }
    }

    // Продовжуємо пайплайн
    if trigger_continue {
        let (lock, cvar) = &*agent_control_resume_arc;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
        is_open = false;
    }

    if !is_open {
        *selected_agent_chat = None;
        *agent_chat_error = None;
    }
}

/// Рендерить вміст повідомлення агента рядок за рядком.
/// Розпізнає маркери [->], [!!], [STATS] і малює для них кастомні іконки через Painter.
fn render_message_content(ui: &mut egui::Ui, content: &str) {
    for line in content.split('\n') {
        if let Some(data) = line.strip_prefix("[STATS]") {
            render_stats_line(ui, data);
        } else if let Some(text) = line.strip_prefix("[->] ") {
            render_result_line(ui, text, false);
        } else if let Some(text) = line.strip_prefix("[!!] ") {
            render_result_line(ui, text, true);
        } else if line.is_empty() {
            // порожній рядок — невелике відступання
            ui.add_space(2.0);
        } else {
            ui.add(egui::Label::new(egui::RichText::new(line).size(12.0)).wrap());
        }
    }
}

/// Рядок результату інструменту: [->] або [!!] зі намальованою іконкою.
fn render_result_line(ui: &mut egui::Ui, text: &str, is_error: bool) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 14.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            if is_error {
                draw_icon_x(ui.painter(), rect);
            } else {
                draw_icon_l_arrow(ui.painter(), rect, ui.visuals().weak_text_color());
            }
        }
        ui.add_space(3.0);
        let color = if is_error {
            egui::Color32::from_rgb(220, 80, 60)
        } else {
            ui.visuals().weak_text_color()
        };
        ui.add(egui::Label::new(egui::RichText::new(text).size(11.0).color(color)).wrap());
    });
}

/// Рядок статистики: намальовані роздільники, стрілки токенів, крапки між секціями.
fn render_stats_line(ui: &mut egui::Ui, data: &str) {
    let mut iter = data.split('|');
    let duration_s: f64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let in_tok: u64     = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let out_tok: u64    = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let cost: f64       = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let turns: u64      = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let sep_color = ui.visuals().weak_text_color().linear_multiply(0.35);

    ui.add_space(2.0);
    draw_h_line(ui, sep_color);
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("{:.1}s", duration_s)).size(11.0).weak());
        draw_dot_sep(ui, sep_color);

        let green = egui::Color32::from_rgb(46, 180, 100);
        draw_icon_arrow_up(ui, green);
        ui.label(egui::RichText::new(format!("{}", in_tok)).size(11.0).color(green));
        ui.add_space(4.0);

        let blue = egui::Color32::from_rgb(52, 152, 219);
        draw_icon_arrow_down(ui, blue);
        ui.label(egui::RichText::new(format!("{} tok", out_tok)).size(11.0).color(blue));

        draw_dot_sep(ui, sep_color);
        ui.label(egui::RichText::new(format!("${:.4}", cost)).size(11.0).weak());
        draw_dot_sep(ui, sep_color);
        ui.label(egui::RichText::new(format!("{} turns", turns)).size(11.0).weak());
    });

    ui.add_space(4.0);
    draw_h_line(ui, sep_color);
    ui.add_space(2.0);
}

// --- Базові функції малювання ---

/// Горизонтальна лінія на всю ширину (заміна ───).
fn draw_h_line(ui: &mut egui::Ui, color: egui::Color32) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().line_segment([rect.left_center(), rect.right_center()], egui::Stroke::new(1.0, color));
    }
}

/// Крапка-роздільник між елементами (заміна ·).
fn draw_dot_sep(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().circle_filled(rect.center(), 1.5, color);
    }
}

/// Стрілка вгору для вхідних токенів (заміна ↑).
fn draw_icon_arrow_up(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 14.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let stroke = egui::Stroke::new(1.5, color);
        let p = ui.painter();
        let mx = rect.center().x;
        let tip = egui::pos2(mx, rect.top() + 3.0);
        let bot = egui::pos2(mx, rect.bottom() - 3.0);
        p.line_segment([tip, bot], stroke);
        p.line_segment([tip, egui::pos2(mx - 3.0, tip.y + 4.5)], stroke);
        p.line_segment([tip, egui::pos2(mx + 3.0, tip.y + 4.5)], stroke);
    }
}

/// Стрілка вниз для вихідних токенів (заміна ↓).
fn draw_icon_arrow_down(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 14.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let stroke = egui::Stroke::new(1.5, color);
        let p = ui.painter();
        let mx = rect.center().x;
        let top = egui::pos2(mx, rect.top() + 3.0);
        let tip = egui::pos2(mx, rect.bottom() - 3.0);
        p.line_segment([top, tip], stroke);
        p.line_segment([tip, egui::pos2(mx - 3.0, tip.y - 4.5)], stroke);
        p.line_segment([tip, egui::pos2(mx + 3.0, tip.y - 4.5)], stroke);
    }
}

/// L-подібна стрілка вправо (заміна ↳) для результату інструменту.
fn draw_icon_l_arrow(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    let x = rect.left() + 3.0;
    let my = rect.center().y;
    let rx = rect.right() - 2.0;
    painter.line_segment([egui::pos2(x, rect.top() + 2.0), egui::pos2(x, my)], stroke);
    painter.line_segment([egui::pos2(x, my), egui::pos2(rx, my)], stroke);
    painter.line_segment([egui::pos2(rx - 3.5, my - 2.5), egui::pos2(rx, my)], stroke);
    painter.line_segment([egui::pos2(rx - 3.5, my + 2.5), egui::pos2(rx, my)], stroke);
}

/// X-хрест червоний (заміна ✗) для помилок інструменту.
fn draw_icon_x(painter: &egui::Painter, rect: egui::Rect) {
    let color = egui::Color32::from_rgb(220, 80, 60);
    let stroke = egui::Stroke::new(1.5, color);
    let d = 2.5;
    let c = rect.center();
    painter.line_segment([egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)], stroke);
    painter.line_segment([egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)], stroke);
}

fn save_agent_chat(save_dir: &std::path::Path, chat: &[crate::queue::AgentChatMessage]) {
    let messages: Vec<serde_json::Value> = chat.iter().map(|m| {
        serde_json::json!({ "role": m.role, "content": m.content })
    }).collect();
    let json = serde_json::to_string_pretty(&messages).unwrap_or_default();
    let _ = std::fs::write(save_dir.join("agent_chat.json"), json);
}
