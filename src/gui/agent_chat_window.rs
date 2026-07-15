use crate::localization::{Language, translate};
use eframe::egui;

/// Стан одного вікна чату з агентом (зберігається per-job).
pub struct AgentChatWindowState {
    pub input: String,
    pub loading: std::sync::Arc<std::sync::Mutex<bool>>,
    pub result: std::sync::Arc<std::sync::Mutex<Option<Result<String, String>>>>,
    pub error: Option<String>,
}

impl AgentChatWindowState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            error: None,
        }
    }
}

#[derive(Clone, Copy)]
enum AgentChatKind {
    Timeline,
    Hyperframes,
}

/// Малює звичайні чати агента, який будує таймлайн.
pub fn draw_agent_chat_windows(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    open_agent_chats: &mut std::collections::HashMap<u64, AgentChatWindowState>,
) {
    draw_agent_chat_windows_impl(
        ctx,
        language,
        jobs,
        open_agent_chats,
        AgentChatKind::Timeline,
    );
}

/// Малює окремі чати для ізольованої генерації HyperFrames-кліпів.
pub fn draw_hyperframes_agent_chat_windows(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    open_agent_chats: &mut std::collections::HashMap<u64, AgentChatWindowState>,
) {
    draw_agent_chat_windows_impl(
        ctx,
        language,
        jobs,
        open_agent_chats,
        AgentChatKind::Hyperframes,
    );
}

fn draw_agent_chat_windows_impl(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    open_agent_chats: &mut std::collections::HashMap<u64, AgentChatWindowState>,
    kind: AgentChatKind,
) {
    use crate::queue::JobStatus;
    let job_ids: Vec<u64> = open_agent_chats.keys().cloned().collect();
    let mut to_remove = Vec::new();

    for job_id in job_ids {
        let Some(job_idx) = jobs.iter().position(|j| j.id == job_id) else {
            to_remove.push(job_id);
            continue;
        };

        let job_name = jobs[job_idx].name.clone();
        let job_status = jobs[job_idx].status.lock().unwrap().clone();
        let (agent_chat_arc, agent_session_arc, chat_file_name, title_key, allow_resume_pipeline) =
            match kind {
                AgentChatKind::Timeline => (
                    std::sync::Arc::clone(&jobs[job_idx].agent_chat),
                    std::sync::Arc::clone(&jobs[job_idx].agent_session),
                    "agent_chat.json",
                    "agent_chat_title",
                    true,
                ),
                AgentChatKind::Hyperframes => (
                    std::sync::Arc::clone(&jobs[job_idx].hyperframes_agent_chat),
                    std::sync::Arc::clone(&jobs[job_idx].hyperframes_agent_session),
                    "hyperframes_agent_chat.json",
                    "hyperframes_agent_chat_title",
                    false,
                ),
            };
        let timeline_rebuild_arc = std::sync::Arc::clone(&jobs[job_idx].timeline_rebuild_requested);
        let job_settings = jobs[job_idx].settings.clone();

        let agent_control_resume_arc = std::sync::Arc::clone(&jobs[job_idx].agent_control_resume);

        let state = open_agent_chats.get_mut(&job_id).unwrap();

        // Перевіряємо результат фонової відповіді агента
        {
            let result = state.result.lock().unwrap().take();
            if let Some(res) = result {
                match res {
                    Ok(response) => {
                        agent_chat_arc
                            .lock()
                            .unwrap()
                            .push(crate::queue::AgentChatMessage {
                                role: "agent".to_string(),
                                content: response,
                            });
                        let save_dir = std::path::Path::new(&job_settings.save_path);
                        save_agent_chat(save_dir, chat_file_name, &agent_chat_arc.lock().unwrap());
                        state.error = None;
                    }
                    Err(e) => {
                        state.error = Some(e);
                    }
                }
            }
        }

        let mut is_open = true;
        let mut trigger_send = false;
        let mut trigger_rebuild = false;
        let mut trigger_resume_pipeline = false;
        let is_awaiting_agent =
            allow_resume_pipeline && job_status == JobStatus::AwaitingAgentControl;

        let title = format!(
            "{} — #{} {}",
            translate(language, title_key),
            job_id + 1,
            job_name,
        );

        egui::Window::new(title)
            .id(egui::Id::new(("agent_chat", job_id, title_key)))
            .open(&mut is_open)
            .resizable(true)
            .min_size([380.0, 300.0])
            .default_size([640.0, 760.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let chat_snapshot = agent_chat_arc.lock().unwrap().clone();
                    let is_loading = *state.loading.lock().unwrap();
                    let pipeline_generating = job_status == crate::queue::JobStatus::Running
                        && chat_snapshot
                            .last()
                            .map(|m| m.role == "agent" && !m.content.contains("[STATS]"))
                            .unwrap_or(false);
                    let show_spinner = is_loading || pipeline_generating;

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
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(&msg.content).size(12.0),
                                                )
                                                .wrap(),
                                            );
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

                    if let Some(ref err) = state.error {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{} {}",
                                    translate(language, "agent_chat_error"),
                                    err
                                ))
                                .color(egui::Color32::from_rgb(231, 76, 60))
                                .size(11.0),
                            )
                            .wrap(),
                        );
                        ui.add_space(4.0);
                    }

                    // Кнопка «Продовжити пайплайн» — з'являється коли агент не створив segments.json
                    // і пайплайн чекає підтвердження від користувача
                    if is_awaiting_agent {
                        ui.add_space(4.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(39, 174, 96).linear_multiply(0.15))
                            .inner_margin(egui::Margin::same(8.0))
                            .rounding(egui::Rounding::same(6.0))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("⏸ ")
                                            .color(egui::Color32::from_rgb(39, 174, 96))
                                            .size(13.0)
                                            .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(translate(
                                            language,
                                            "agent_awaiting_hint",
                                        ))
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(39, 174, 96)),
                                    );
                                });
                                ui.add_space(4.0);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(translate(
                                                language,
                                                "agent_resume_pipeline_btn",
                                            ))
                                            .strong()
                                            .size(13.0),
                                        )
                                        .min_size(egui::vec2(ui.available_width(), 28.0)),
                                    )
                                    .clicked()
                                {
                                    trigger_resume_pipeline = true;
                                }
                            });
                        ui.add_space(4.0);
                    }

                    let has_session = agent_session_arc.lock().unwrap().is_some();

                    let input_response = ui.add(
                        egui::TextEdit::multiline(&mut state.input)
                            .hint_text(translate(language, "agent_chat_input_hint"))
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );
                    if input_response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    {
                        if !is_loading && has_session && !state.input.trim().is_empty() {
                            trigger_send = true;
                        }
                    }

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if show_spinner {
                            ui.add(egui::Spinner::new().size(16.0));
                            ui.add_space(6.0);
                        }

                        // Кнопка «Надіслати» — доступна коли є сесія і є текст
                        if ui
                            .add_enabled(
                                !is_loading && has_session && !state.input.trim().is_empty(),
                                egui::Button::new(translate(language, "agent_chat_send_btn")),
                            )
                            .clicked()
                        {
                            trigger_send = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Кнопка «Перебудувати таймлінію»
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(translate(
                                        language,
                                        "agent_chat_rebuild_btn",
                                    ))
                                    .strong(),
                                ))
                                .clicked()
                            {
                                trigger_rebuild = true;
                            }
                        });
                    });
                });
            });

        // Надсилаємо повідомлення агенту через --resume
        if trigger_send {
            let message = state.input.trim().to_string();
            if !message.is_empty() {
                let session_snapshot = agent_session_arc.lock().unwrap().clone();
                if let Some(session) = session_snapshot {
                    agent_chat_arc
                        .lock()
                        .unwrap()
                        .push(crate::queue::AgentChatMessage {
                            role: "user".to_string(),
                            content: message.clone(),
                        });
                    let save_dir = std::path::Path::new(&job_settings.save_path);
                    save_agent_chat(save_dir, chat_file_name, &agent_chat_arc.lock().unwrap());
                    state.input.clear();
                    state.error = None;

                    let result_arc = std::sync::Arc::clone(&state.result);
                    let loading_arc = std::sync::Arc::clone(&state.loading);
                    let ctx_clone = ctx.clone();
                    let save_path = job_settings.save_path.clone();
                    *loading_arc.lock().unwrap() = true;

                    std::thread::spawn(move || {
                        let result = crate::core::pipeline::call_agent_resume(
                            &session.service,
                            &session.model,
                            &message,
                            &session.session_id,
                            Some((job_id, String::new())),
                            Some(&save_path),
                        );
                        *result_arc.lock().unwrap() = Some(result);
                        *loading_arc.lock().unwrap() = false;
                        ctx_clone.request_repaint();
                    });
                }
            }
        }

        if trigger_rebuild {
            *timeline_rebuild_arc.lock().unwrap() = true;
        }

        if trigger_resume_pipeline {
            let (lock, cvar) = &*agent_control_resume_arc;
            *lock.lock().unwrap() = true;
            cvar.notify_one();
        }

        if !is_open {
            to_remove.push(job_id);
        }
    }

    for id in to_remove {
        open_agent_chats.remove(&id);
    }
}

/// Рендерить вміст повідомлення агента рядок за рядком.
fn render_message_content(ui: &mut egui::Ui, content: &str) {
    let lines: Vec<&str> = content.split('\n').collect();

    let has_final_stats = lines.iter().any(|l| l.starts_with("[STATS]"));
    let last_live_idx = if has_final_stats {
        None
    } else {
        lines.iter().rposition(|l| l.starts_with("[LIVE_STATS]"))
    };

    for (i, line) in lines.iter().enumerate() {
        if let Some(data) = line.strip_prefix("[STATS]") {
            render_stats_line(ui, data);
        } else if line.starts_with("[LIVE_STATS]") {
            if Some(i) == last_live_idx {
                if let Some(data) = line.strip_prefix("[LIVE_STATS]") {
                    render_live_stats_line(ui, data);
                }
            }
        } else if let Some(text) = line.strip_prefix("[->] ") {
            render_result_line(ui, text, false);
        } else if let Some(text) = line.strip_prefix("[!!] ") {
            render_result_line(ui, text, true);
        } else if let Some(text) = line.strip_prefix("[THINK]") {
            render_think_line(ui, text);
        } else if line.is_empty() {
            ui.add_space(2.0);
        } else {
            ui.add(egui::Label::new(egui::RichText::new(*line).size(12.0)).wrap());
        }
    }
}

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

fn render_stats_line(ui: &mut egui::Ui, data: &str) {
    let mut iter = data.split('|');
    let duration_s: f64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let in_tok: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let out_tok: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let cost: f64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let turns: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let sep_color = ui.visuals().weak_text_color().linear_multiply(0.35);

    ui.add_space(2.0);
    draw_h_line(ui, sep_color);
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("{:.1}s", duration_s))
                .size(11.0)
                .weak(),
        );
        draw_dot_sep(ui, sep_color);

        let green = egui::Color32::from_rgb(46, 180, 100);
        draw_icon_arrow_up(ui, green);
        ui.label(
            egui::RichText::new(format!("{}", in_tok))
                .size(11.0)
                .color(green),
        );
        ui.add_space(4.0);

        let blue = egui::Color32::from_rgb(52, 152, 219);
        draw_icon_arrow_down(ui, blue);
        ui.label(
            egui::RichText::new(format!("{} tok", out_tok))
                .size(11.0)
                .color(blue),
        );

        draw_dot_sep(ui, sep_color);
        ui.label(
            egui::RichText::new(format!("${:.4}", cost))
                .size(11.0)
                .weak(),
        );
        draw_dot_sep(ui, sep_color);
        ui.label(
            egui::RichText::new(format!("{} turns", turns))
                .size(11.0)
                .weak(),
        );
    });

    ui.add_space(4.0);
    draw_h_line(ui, sep_color);
    ui.add_space(2.0);
}

fn render_live_stats_line(ui: &mut egui::Ui, data: &str) {
    let mut iter = data.split('|');
    let in_tok: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let out_tok: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let dim = ui.visuals().weak_text_color().linear_multiply(0.5);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let green = egui::Color32::from_rgb(46, 180, 100).linear_multiply(0.7);
        draw_icon_arrow_up(ui, green);
        ui.label(
            egui::RichText::new(format!("{}", in_tok))
                .size(10.0)
                .color(green),
        );
        ui.add_space(4.0);
        let blue = egui::Color32::from_rgb(52, 152, 219).linear_multiply(0.7);
        draw_icon_arrow_down(ui, blue);
        ui.label(
            egui::RichText::new(format!("{} tok", out_tok))
                .size(10.0)
                .color(blue),
        );
        ui.add_space(4.0);
        ui.label(egui::RichText::new("...").size(10.0).color(dim));
    });
}

fn render_think_line(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, 14.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let color = ui.visuals().weak_text_color().linear_multiply(0.4);
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);
        }
        ui.add_space(5.0);
        let color = ui.visuals().weak_text_color().linear_multiply(0.7);
        ui.add(
            egui::Label::new(egui::RichText::new(text).size(11.0).color(color).italics()).wrap(),
        );
    });
}

fn draw_h_line(ui: &mut egui::Ui, color: egui::Color32) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(1.0, color),
        );
    }
}

fn draw_dot_sep(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().circle_filled(rect.center(), 1.5, color);
    }
}

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

fn draw_icon_x(painter: &egui::Painter, rect: egui::Rect) {
    let color = egui::Color32::from_rgb(220, 80, 60);
    let stroke = egui::Stroke::new(1.5, color);
    let d = 2.5;
    let c = rect.center();
    painter.line_segment(
        [egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)],
        stroke,
    );
}

fn save_agent_chat(
    save_dir: &std::path::Path,
    file_name: &str,
    chat: &[crate::queue::AgentChatMessage],
) {
    let messages: Vec<serde_json::Value> = chat
        .iter()
        .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
        .collect();
    let json = serde_json::to_string_pretty(&messages).unwrap_or_default();
    let _ = std::fs::write(save_dir.join(file_name), json);
}
