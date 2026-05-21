use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
struct CreditsData {
    total_credits: f64,
    total_usage: f64,
}

#[derive(serde::Deserialize)]
struct CreditsResponse {
    data: CreditsData,
}

/// Фоново завантажує баланс OpenRouter і записує результат в `result`.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let text = match agent
            .get("https://openrouter.ai/api/v1/credits")
            .set("Authorization", &format!("Bearer {}", key))
            .call()
        {
            Ok(resp) => match resp.into_json::<CreditsResponse>() {
                Ok(data) => {
                    let remaining = (data.data.total_credits - data.data.total_usage).max(0.0);
                    format!("${:.2}", remaining)
                }
                Err(_) => return,
            },
            Err(_) => return,
        };

        *result.lock().unwrap() = Some(text);
        ctx.request_repaint();
    });
}
