use eframe::egui;
use std::sync::{Arc, Mutex};

const BASE_URL: &str = "https://googler.fast-gen.ai/api";

/// Баланс та ліміти акаунту Googler.
#[derive(Clone, Default)]
pub struct GooglerBalance {
    pub img_used: i64,
    pub img_limit: i64,
    pub video_used: i64,
    pub video_limit: i64,
    pub img_threads_active: i64,
    pub img_threads_allowed: i64,
    pub video_threads_active: i64,
    pub video_threads_allowed: i64,
}

#[derive(serde::Deserialize, Default)]
struct HourlyUsageStats {
    current_usage: i64,
}

#[derive(serde::Deserialize, Default)]
struct HourlyUsage {
    image_generation: Option<HourlyUsageStats>,
    video_generation: Option<HourlyUsageStats>,
}

#[derive(serde::Deserialize, Default)]
struct ActiveThreads {
    #[serde(default)]
    image_threads: i64,
    #[serde(default)]
    video_threads: i64,
}

#[derive(serde::Deserialize, Default)]
struct CurrentUsage {
    #[serde(default)]
    hourly_usage: HourlyUsage,
    #[serde(default)]
    active_threads: ActiveThreads,
}

#[derive(serde::Deserialize)]
struct AccountLimits {
    #[serde(default)]
    img_gen_per_hour_limit: serde_json::Value,
    #[serde(default)]
    video_gen_per_hour_limit: serde_json::Value,
    #[serde(default)]
    img_generation_threads_allowed: serde_json::Value,
    #[serde(default)]
    video_generation_threads_allowed: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct UsageResponse {
    account_limits: AccountLimits,
    current_usage: CurrentUsage,
}

/// Витягує числове значення з Value (може бути число або об'єкт).
fn extract_i64(val: &serde_json::Value) -> i64 {
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

fn parse_balance(data: UsageResponse) -> GooglerBalance {
    GooglerBalance {
        img_used: data.current_usage.hourly_usage.image_generation
            .map(|s| s.current_usage)
            .unwrap_or(0),
        img_limit: extract_i64(&data.account_limits.img_gen_per_hour_limit),
        video_used: data.current_usage.hourly_usage.video_generation
            .map(|s| s.current_usage)
            .unwrap_or(0),
        video_limit: extract_i64(&data.account_limits.video_gen_per_hour_limit),
        img_threads_active: data.current_usage.active_threads.image_threads,
        img_threads_allowed: extract_i64(&data.account_limits.img_generation_threads_allowed),
        video_threads_active: data.current_usage.active_threads.video_threads,
        video_threads_allowed: extract_i64(&data.account_limits.video_generation_threads_allowed),
    }
}

/// Фоново перевіряє ключ Googler і записує текстовий статус у `status_result`.
/// При успіху також оновлює `balance_result`.
pub fn check_key(
    key: String,
    status_result: Arc<Mutex<Option<String>>>,
    balance_result: Arc<Mutex<Option<GooglerBalance>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!("{}/v3/account/usage?api_key={}", BASE_URL, key);

        let (status_text, balance_opt) = match agent.get(&url).set("X-API-Key", &key).call() {
            Ok(response) => match response.into_json::<UsageResponse>() {
                Ok(data) => {
                    let bal = parse_balance(data);
                    let status = format!(
                        "✔ img: {}/{}/h • vid: {}/{}/h",
                        bal.img_used, bal.img_limit,
                        bal.video_used, bal.video_limit
                    );
                    (status, Some(bal))
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
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<GooglerBalance>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!("{}/v3/account/usage?api_key={}", BASE_URL, key);

        if let Ok(response) = agent.get(&url).set("X-API-Key", &key).call() {
            if let Ok(data) = response.into_json::<UsageResponse>() {
                *result.lock().unwrap() = Some(parse_balance(data));
                ctx.request_repaint();
            }
        }
    });
}
