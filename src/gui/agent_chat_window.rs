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
        .default_size([520.0, 500.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Область прокрутки з повідомленнями чату
                let chat_snapshot = agent_chat_arc.lock().unwrap().clone();

                egui::ScrollArea::vertical()
                    .max_height(340.0)
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
                                    ui.add(egui::Label::new(
                                        egui::RichText::new(&msg.content).size(12.0)
                                    ).wrap());
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

                let is_loading = *agent_chat_loading.lock().unwrap();
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
                    // Enter без Shift — відправляємо (якщо є що відправляти)
                    if !is_loading && has_session && !agent_chat_input.trim().is_empty() {
                        trigger_send = true;
                    }
                }

                ui.add_space(4.0);

                // Кнопки
                ui.horizontal(|ui| {
                    if is_loading {
                        ui.label(
                            egui::RichText::new(translate(language, "agent_chat_loading"))
                                .weak()
                                .size(11.0),
                        );
                    } else {
                        if ui.add_enabled(
                            has_session && !agent_chat_input.trim().is_empty(),
                            egui::Button::new(translate(language, "agent_chat_send_btn")),
                        ).clicked() {
                            trigger_send = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new(translate(language, "agent_chat_continue_btn")).strong()
                            )
                        ).clicked() {
                            trigger_continue = true;
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

fn save_agent_chat(save_dir: &std::path::Path, chat: &[crate::queue::AgentChatMessage]) {
    let messages: Vec<serde_json::Value> = chat.iter().map(|m| {
        serde_json::json!({ "role": m.role, "content": m.content })
    }).collect();
    let json = serde_json::to_string_pretty(&messages).unwrap_or_default();
    let _ = std::fs::write(save_dir.join("agent_chat.json"), json);
}
