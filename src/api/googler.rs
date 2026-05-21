use eframe::egui;
use std::sync::{Arc, Mutex};

const BASE_URL: &str = "https://googler.fast-gen.ai/api";

#[derive(serde::Deserialize)]
struct AccountLimits {
    img_gen_per_hour_limit: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct UsageResponse {
    account_limits: AccountLimits,
}

/// Витягує числове значення ліміту з Value (може бути число або об'єкт).
fn extract_limit(val: &serde_json::Value) -> i64 {
    match val {
        serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
        serde_json::Value::Object(map) => {
            for key in &["used", "count", "current_usage"] {
                if let Some(v) = map.get(*key).and_then(|v| v.as_i64()) {
                    return v;
                }
            }
            0
        }
        _ => 0,
    }
}

/// Фоново перевіряє ключ Googler і записує текстовий статус у `status_result`.
/// При успіху також оновлює `balance_result`.
pub fn check_key(
    key: String,
    status_result: Arc<Mutex<Option<String>>>,
    balance_result: Arc<Mutex<Option<String>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!("{}/v3/account/usage?api_key={}", BASE_URL, key);

        let (status_text, balance_opt) = match agent
            .get(&url)
            .set("X-API-Key", &key)
            .call()
        {
            Ok(response) => match response.into_json::<UsageResponse>() {
                Ok(data) => {
                    let limit = extract_limit(&data.account_limits.img_gen_per_hour_limit);
                    (
                        format!("✔ {} img/h", limit),
                        Some(format!("{} img/h", limit)),
                    )
                }
                Err(_) => ("✔ Ключ валідний".to_string(), None),
            },
            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                ("❌ Невірний ключ".to_string(), None)
            }
            Err(ureq::Error::Status(code, _)) if code >= 500 => {
                (format!("⚠ Сервер тимчасово недоступний ({})", code), None)
            }
            Err(ureq::Error::Status(code, _)) => (format!("❌ Помилка ({})", code), None),
            Err(_) => ("❌ Помилка мережі. Перевірте з'єднання.".to_string(), None),
        };

        *status_result.lock().unwrap() = Some(status_text);
        if let Some(bal) = balance_opt {
            *balance_result.lock().unwrap() = Some(bal);
        }
        ctx.request_repaint();
    });
}

/// Фоново завантажує інфо про ліміти Googler і записує результат у `result`.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!("{}/v3/account/usage?api_key={}", BASE_URL, key);

        if let Ok(response) = agent
            .get(&url)
            .set("X-API-Key", &key)
            .call()
        {
            if let Ok(data) = response.into_json::<UsageResponse>() {
                let limit = extract_limit(&data.account_limits.img_gen_per_hour_limit);
                *result.lock().unwrap() = Some(format!("{} img/h", limit));
                ctx.request_repaint();
            }
        }
    });
}
