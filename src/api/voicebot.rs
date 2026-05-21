use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct BalanceResponse {
    pub balance_text: String,
}

/// Фоново завантажує баланс VoiceBot і записує результат в `result`.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let text = match agent
            .get("https://voiceapi.csv666.ru/balance")
            .set("X-API-Key", &key)
            .set("Accept", "application/json")
            .call()
        {
            Ok(resp) => match resp.into_json::<BalanceResponse>() {
                Ok(data) => data.balance_text,
                Err(_) => return,
            },
            Err(_) => return,
        };

        *result.lock().unwrap() = Some(text);
        ctx.request_repaint();
    });
}
