use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
struct BalanceResponse {
    balance_text: String,
}

/// Малює секцію "АПІ" на панелі пайплайну з підтримкою OpenRouter та Voice Bot.
pub fn draw_api_section(
    ui: &mut egui::Ui,
    language: Language,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
    voicebot_key: &mut String,
    voicebot_status: &mut Option<String>,
    voicebot_test_result: &Arc<Mutex<Option<String>>>,
) {
    // Опитуємо результат фонового тесту Voice Bot і переносимо у voicebot_status
    if let Ok(mut guard) = voicebot_test_result.try_lock() {
        if let Some(result) = guard.take() {
            *voicebot_status = Some(result);
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
                    .desired_width((available_width - 90.0).max(100.0))
            );

            if response.changed() {
                *openrouter_status = None;
            }

            let check_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn"))
            );

            if check_btn.clicked() {
                let trimmed = openrouter_key.trim();
                if trimmed.is_empty() {
                    *openrouter_status = Some(translate(language, "api_status_empty").to_string());
                } else if trimmed.starts_with("sk-or-") && trimmed.len() >= 15 {
                    *openrouter_status = Some(translate(language, "api_status_success").to_string());
                } else {
                    *openrouter_status = Some(translate(language, "api_status_invalid").to_string());
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
                    egui::RichText::new(status.as_str()).color(text_color).size(12.0)
                ).wrap()
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
                    .desired_width((available_width - 90.0).max(100.0))
            );

            if vb_response.changed() {
                *voicebot_status = None;
            }

            let test_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn"))
            );

            if test_btn.clicked() {
                let trimmed = voicebot_key.trim().to_string();
                if trimmed.is_empty() {
                    *voicebot_status = Some(translate(language, "api_status_empty").to_string());
                } else {
                    *voicebot_status = Some(translate(language, "voicebot_status_checking").to_string());

                    let result_arc = Arc::clone(voicebot_test_result);
                    let ctx = ui.ctx().clone();

                    std::thread::spawn(move || {
                        let agent = ureq::AgentBuilder::new()
                            .timeout_connect(std::time::Duration::from_secs(10))
                            .timeout(std::time::Duration::from_secs(15))
                            .build();

                        let status_text = match agent
                            .get("https://voiceapi.csv666.ru/balance")
                            .set("X-API-Key", &trimmed)
                            .set("Accept", "application/json")
                            .call()
                        {
                            Ok(response) => match response.into_json::<BalanceResponse>() {
                                Ok(data) => format!("✔ Баланс: {}", data.balance_text),
                                Err(_) => "✔ Ключ валідний".to_string(),
                            },
                            Err(ureq::Error::Status(401, _)) => "❌ Невірний ключ".to_string(),
                            Err(ureq::Error::Status(code, _)) if code >= 500 => {
                                format!("⚠ Сервер тимчасово недоступний ({})", code)
                            }
                            Err(ureq::Error::Status(code, _)) => {
                                format!("❌ Помилка ({})", code)
                            }
                            Err(_) => "❌ Помилка мережі. Перевірте з'єднання.".to_string(),
                        };

                        *result_arc.lock().unwrap() = Some(status_text);
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
                    egui::RichText::new(status.as_str()).color(text_color).size(12.0)
                ).wrap()
            );
        }

        ui.add_space(6.0);
    });
}
