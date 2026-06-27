use crate::api;
use crate::localization::{translate, Language};
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Малює секцію "АПІ" на панелі пайплайну з підтримкою OpenRouter, Voice Bot, Googler та AssemblyAI.
pub fn draw_api_section(
    ui: &mut egui::Ui,
    language: Language,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    openrouter_balance: &Arc<Mutex<Option<String>>>,
    voicebot_key: &mut String,
    voicebot_status: &mut Option<String>,
    voicebot_test_result: &Arc<Mutex<Option<String>>>,
    voicebot_balance: &Arc<Mutex<Option<String>>>,
    googler_key: &mut String,
    googler_status: &mut Option<String>,
    googler_test_result: &Arc<Mutex<Option<String>>>,
    googler_balance: &Arc<Mutex<Option<crate::api::googler::GooglerBalance>>>,
    assemblyai_key: &mut String,
    assemblyai_status: &mut Option<String>,
    assemblyai_test_result: &Arc<Mutex<Option<String>>>,
    pexels_key: &mut String,
    pexels_status: &mut Option<String>,
    pexels_test_result: &Arc<Mutex<Option<String>>>,
    pixabay_key: &mut String,
    pixabay_status: &mut Option<String>,
    pixabay_test_result: &Arc<Mutex<Option<String>>>,
) {
    // Опитуємо результат фонового тесту Voice Bot і переносимо у voicebot_status
    if let Ok(mut guard) = voicebot_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *voicebot_status = Some(result);
        }
    }

    // Опитуємо результат фонового тесту Googler і переносимо у googler_status
    if let Ok(mut guard) = googler_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *googler_status = Some(result);
        }
    }

    // Опитуємо результат фонового тесту AssemblyAI і переносимо у assemblyai_status
    if let Ok(mut guard) = assemblyai_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *assemblyai_status = Some(result);
        }
    }

    // Опитуємо результат фонового тесту Pexels і переносимо у pexels_status
    if let Ok(mut guard) = pexels_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *pexels_status = Some(result);
        }
    }

    // Опитуємо результат фонового тесту Pixabay і переносимо у pixabay_status
    if let Ok(mut guard) = pixabay_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *pixabay_status = Some(result);
        }
    }

    ui.vertical(|ui| {
        ui.add_space(4.0);

        // --- OpenRouter ---
        ui.label(egui::RichText::new("OpenRouter").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(openrouter_key)
                    .password(true)
                    .hint_text("sk-or-...")
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if response.changed() {
                *openrouter_status = None;
                if let Ok(mut bal) = openrouter_balance.try_lock() {
                    *bal = None;
                }
            }

            let check_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if check_btn.clicked() {
                let trimmed = openrouter_key.trim();
                if trimmed.is_empty() {
                    *openrouter_status = Some(translate(language, "api_status_empty").to_string());
                } else if trimmed.starts_with("sk-or-") && trimmed.len() >= 15 {
                    *openrouter_status =
                        Some(translate(language, "api_status_success").to_string());
                    api::openrouter::fetch_balance(
                        trimmed.to_string(),
                        Arc::clone(openrouter_balance),
                        ui.ctx().clone(),
                    );
                } else {
                    *openrouter_status =
                        Some(translate(language, "api_status_invalid").to_string());
                }
            }
        });

        if let Some(status) = openrouter_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let text_color = if is_success {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // --- Voice Bot ---
        ui.label(egui::RichText::new("Voice Bot").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let vb_response = ui.add(
                egui::TextEdit::singleline(voicebot_key)
                    .password(true)
                    .hint_text(translate(language, "voicebot_key_hint"))
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if vb_response.changed() {
                *voicebot_status = None;
                if let Ok(mut bal) = voicebot_balance.try_lock() {
                    *bal = None;
                }
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if test_btn.clicked() {
                let trimmed = voicebot_key.trim().to_string();
                if trimmed.is_empty() {
                    *voicebot_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *voicebot_status =
                        Some(translate(language, "voicebot_status_checking").to_string());

                    let result_arc = Arc::clone(voicebot_test_result);
                    let balance_arc = Arc::clone(voicebot_balance);
                    let ctx = ui.ctx().clone();

                    std::thread::spawn(move || {
                        let agent = ureq::AgentBuilder::new()
                            .timeout_connect(std::time::Duration::from_secs(10))
                            .timeout(std::time::Duration::from_secs(15))
                            .build();

                        let (status_text, balance_opt) = match agent
                            .get("https://voiceapi.csv666.ru/balance")
                            .set("X-API-Key", &trimmed)
                            .set("Accept", "application/json")
                            .call()
                        {
                            Ok(response) => {
                                match response.into_json::<api::voicebot::BalanceResponse>() {
                                    Ok(data) => (
                                        format!("✔ Баланс: {}", data.balance_text),
                                        Some(data.balance_text),
                                    ),
                                    Err(_) => ("✔ Ключ валідний".to_string(), None),
                                }
                            }
                            Err(ureq::Error::Status(401, _)) => {
                                ("❌ Невірний ключ".to_string(), None)
                            }
                            Err(ureq::Error::Status(code, _)) if code >= 500 => {
                                (format!("⚠ Сервер тимчасово недоступний ({})", code), None)
                            }
                            Err(ureq::Error::Status(code, _)) => {
                                (format!("❌ Помилка ({})", code), None)
                            }
                            Err(_) => ("❌ Помилка мережі. Перевірте з'єднання.".to_string(), None),
                        };

                        *result_arc.lock().unwrap() = Some(status_text);
                        if let Some(bal) = balance_opt {
                            *balance_arc.lock().unwrap() = Some(bal);
                        }
                        ctx.request_repaint();
                    });
                }
            }
        });

        if let Some(status) = voicebot_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let is_checking = status.starts_with('⏳');
            let text_color = if is_success || is_checking {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // --- Googler ---
        ui.label(egui::RichText::new("Googler").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let gr_response = ui.add(
                egui::TextEdit::singleline(googler_key)
                    .password(true)
                    .hint_text(translate(language, "googler_key_hint"))
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if gr_response.changed() {
                *googler_status = None;
                if let Ok(mut bal) = googler_balance.try_lock() {
                    *bal = None;
                }
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if test_btn.clicked() {
                let trimmed = googler_key.trim().to_string();
                if trimmed.is_empty() {
                    *googler_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *googler_status =
                        Some(translate(language, "googler_status_checking").to_string());

                    api::googler::check_key(
                        trimmed,
                        Arc::clone(googler_test_result),
                        Arc::clone(googler_balance),
                        ui.ctx().clone(),
                    );
                }
            }
        });

        if let Some(status) = googler_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let is_checking = status.starts_with('⏳');
            let text_color = if is_success || is_checking {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // --- AssemblyAI ---
        ui.label(egui::RichText::new("AssemblyAI").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let ai_response = ui.add(
                egui::TextEdit::singleline(assemblyai_key)
                    .password(true)
                    .hint_text(translate(language, "assemblyai_key_hint"))
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if ai_response.changed() {
                *assemblyai_status = None;
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if test_btn.clicked() {
                let trimmed = assemblyai_key.trim().to_string();
                if trimmed.is_empty() {
                    *assemblyai_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *assemblyai_status = Some("⏳ Перевірка...".to_string());

                    let result_arc = Arc::clone(assemblyai_test_result);
                    let ctx = ui.ctx().clone();

                    std::thread::spawn(move || {
                        let status = crate::api::assemblyai::check_key(&trimmed);
                        *result_arc.lock().unwrap() = Some(status);
                        ctx.request_repaint();
                    });
                }
            }
        });

        if let Some(status) = assemblyai_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let is_checking = status.starts_with('⏳');
            let text_color = if is_success || is_checking {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // --- Pexels ---
        ui.label(egui::RichText::new("Pexels Stock").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let px_response = ui.add(
                egui::TextEdit::singleline(pexels_key)
                    .password(true)
                    .hint_text(translate(language, "pexels_key_hint"))
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if px_response.changed() {
                *pexels_status = None;
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if test_btn.clicked() {
                let trimmed = pexels_key.trim().to_string();
                if trimmed.is_empty() {
                    *pexels_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *pexels_status =
                        Some(translate(language, "pexels_status_checking").to_string());

                    let result_arc = Arc::clone(pexels_test_result);
                    let ctx = ui.ctx().clone();

                    std::thread::spawn(move || {
                        let status = crate::api::stock::pexels::check_key(&trimmed);
                        *result_arc.lock().unwrap() = Some(status);
                        ctx.request_repaint();
                    });
                }
            }
        });

        if let Some(status) = pexels_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let is_checking = status.starts_with('⏳');
            let text_color = if is_success || is_checking {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // --- Pixabay ---
        ui.label(egui::RichText::new("Pixabay Stock").strong());
        ui.add_space(4.0);

        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let pbx_response = ui.add(
                egui::TextEdit::singleline(pixabay_key)
                    .password(true)
                    .hint_text(translate(language, "pixabay_key_hint"))
                    .desired_width((available_width - 90.0).max(100.0)),
            );

            if pbx_response.changed() {
                *pixabay_status = None;
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn")),
            );

            if test_btn.clicked() {
                let trimmed = pixabay_key.trim().to_string();
                if trimmed.is_empty() {
                    *pixabay_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *pixabay_status =
                        Some(translate(language, "pexels_status_checking").to_string());

                    let result_arc = Arc::clone(pixabay_test_result);
                    let ctx = ui.ctx().clone();

                    std::thread::spawn(move || {
                        let status = crate::api::stock::pixabay::check_key(&trimmed);
                        *result_arc.lock().unwrap() = Some(status);
                        ctx.request_repaint();
                    });
                }
            }
        });

        if let Some(status) = pixabay_status {
            ui.add_space(4.0);
            let is_success = status.starts_with('✔');
            let is_checking = status.starts_with('⏳');
            let text_color = if is_success || is_checking {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0),
                )
                .wrap(),
            );
        }

        ui.add_space(6.0);
    });
}
