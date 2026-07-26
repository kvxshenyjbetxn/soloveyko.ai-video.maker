use eframe::egui;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

const BASE_URL: &str = "https://api.lumean.app/api/public";
const MAX_CONCURRENT_REQUESTS: usize = 5;

#[derive(Default)]
struct ProxyConfig {
    enabled: bool,
    url: String,
}

static PROXY_CONFIG: OnceLock<Mutex<ProxyConfig>> = OnceLock::new();

/// Оновлює проксі лише для запитів до Lumean.
pub fn configure_proxy(enabled: bool, url: &str) {
    let mut config = PROXY_CONFIG
        .get_or_init(|| Mutex::new(ProxyConfig::default()))
        .lock()
        .unwrap();
    config.enabled = enabled;
    config.url = url.trim().to_string();
}

fn proxy_url() -> Option<String> {
    let config = PROXY_CONFIG
        .get_or_init(|| Mutex::new(ProxyConfig::default()))
        .lock()
        .unwrap();
    config.enabled.then(|| normalize_proxy_url(&config.url))
}

/// Перетворює формат `хост:порт:логін:пароль` у формат, потрібний HTTP-клієнту.
fn normalize_proxy_url(proxy: &str) -> String {
    if proxy.contains("://") {
        return proxy.to_string();
    }

    let mut parts = proxy.splitn(4, ':');
    let (Some(host), Some(port), Some(login), Some(password)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return proxy.to_string();
    };

    if host.is_empty() || port.is_empty() || login.is_empty() || password.is_empty() {
        return proxy.to_string();
    }

    format!("http://{login}:{password}@{host}:{port}")
}

#[cfg(test)]
mod tests {
    use super::normalize_proxy_url;

    #[test]
    fn normalizes_host_port_login_password_proxy() {
        assert_eq!(
            normalize_proxy_url("s-37320.sp1.ovh:11001:VWewNcKM_0:970sGXvxL8bn"),
            "http://VWewNcKM_0:970sGXvxL8bn@s-37320.sp1.ovh:11001"
        );
    }
}

/// Обмежує кількість одночасних запитів до Lumean.
pub struct LumeanLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
}

impl LumeanLimiter {
    /// Повертає глобальний лімітер Lumean.
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<LumeanLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| LumeanLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
        })
    }

    /// Очікує вільне місце для запиту.
    pub fn acquire(&self) -> LumeanPermit<'_> {
        let mut active = self.active.lock().unwrap();
        while *active >= MAX_CONCURRENT_REQUESTS {
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        LumeanPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        *active = active.saturating_sub(1);
        self.condvar.notify_one();
    }

    /// Повертає кількість активних запитів.
    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

/// Дозвіл на один запит Lumean.
pub struct LumeanPermit<'a> {
    limiter: &'a LumeanLimiter,
}

impl Drop for LumeanPermit<'_> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

#[derive(serde::Deserialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(serde::Deserialize)]
struct UsageRecord {
    service_name: String,
    remaining: u64,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct LumeanTemplate {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
struct OrderCreateResponse {
    id: String,
}

#[derive(serde::Deserialize)]
struct OrderResponse {
    status: String,
    #[serde(default)]
    result: Option<OrderResult>,
}

#[derive(serde::Deserialize)]
struct OrderResult {
    #[serde(default)]
    files: Vec<String>,
}

#[derive(serde::Deserialize)]
struct StorageUrlResponse {
    url: String,
}

/// Фоново завантажує доступний залишок токенів Lumean.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        if let Ok(balance) = fetch_usage(&key) {
            *result.lock().unwrap() = Some(balance);
            ctx.request_repaint();
        }
    });
}

/// Перевіряє ключ через endpoint usage та повертає залишок токенів у UI.
pub fn check_key(
    key: String,
    result: Arc<Mutex<Option<String>>>,
    balance: Arc<Mutex<Option<String>>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let (status, usage) = match fetch_usage(&key) {
            Ok(usage) => (format!("✔ Залишок токенів: {usage}"), Some(usage)),
            Err(ureq::Error::Status(401, _)) => ("❌ Невірний ключ Lumean".to_string(), None),
            Err(ureq::Error::Status(403, _)) => (
                "❌ Для ключа потрібен дозвіл billing.read".to_string(),
                None,
            ),
            Err(ureq::Error::Status(code, _)) if code >= 500 => {
                (format!("⚠ Сервер тимчасово недоступний ({code})"), None)
            }
            Err(ureq::Error::Status(code, _)) => (format!("❌ Помилка Lumean ({code})"), None),
            Err(_) => ("❌ Помилка мережі. Перевірте з'єднання.".to_string(), None),
        };

        *result.lock().unwrap() = Some(status);
        if let Some(usage) = usage {
            *balance.lock().unwrap() = Some(usage);
        }
        ctx.request_repaint();
    });
}

/// Завантажує доступні шаблони Lumean.
pub fn fetch_templates(key: &str) -> Result<Vec<LumeanTemplate>, String> {
    request_agent()
        .map_err(format_request_error)?
        .get(&format!("{BASE_URL}/templates"))
        .set("X-API-KEY", key)
        .set("Accept", "application/json")
        .call()
        .map_err(format_request_error)?
        .into_json::<ApiResponse<Vec<LumeanTemplate>>>()
        .map(|response| response.data)
        .map_err(|error| format!("Помилка обробки шаблонів Lumean: {error}"))
}

/// Створює TTS-замовлення через обраний шаблон і повертає UUID замовлення.
pub fn create_tts_order(key: &str, text: &str, template_id: &str) -> Result<String, String> {
    let body = ureq::serde_json::json!({
        "template_id": template_id,
        "input_text": text,
    });

    request_agent()
        .map_err(format_request_error)?
        .post(&format!("{BASE_URL}/orders"))
        .set("X-API-KEY", key)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(format_request_error)?
        .into_json::<ApiResponse<OrderCreateResponse>>()
        .map(|response| response.data.id)
        .map_err(|error| format!("Помилка обробки відповіді Lumean: {error}"))
}

/// Повертає статус замовлення і шлях до готового аудіофайлу.
pub fn get_order_status(key: &str, order_id: &str) -> Result<(String, Option<String>), String> {
    let response = request_agent()
        .map_err(format_request_error)?
        .get(&format!("{BASE_URL}/orders/{order_id}"))
        .set("X-API-KEY", key)
        .set("Accept", "application/json")
        .call()
        .map_err(format_request_error)?
        .into_json::<ApiResponse<OrderResponse>>()
        .map_err(|error| format!("Помилка обробки статусу Lumean: {error}"))?;

    let audio_path = response
        .data
        .result
        .and_then(|result| result.files.into_iter().next());
    Ok((response.data.status, audio_path))
}

/// Завантажує аудіофайл за шляхом із `order.result.files`.
pub fn download_result(key: &str, storage_path: &str, save_dir: &str) -> Result<String, String> {
    use std::io::Read;

    let signed_url = request_agent()
        .map_err(format_request_error)?
        .post(&format!("{BASE_URL}/storage/url"))
        .set("X-API-KEY", key)
        .set("Content-Type", "application/json")
        .send_json(ureq::serde_json::json!({ "path": storage_path }))
        .map_err(format_request_error)?
        .into_json::<ApiResponse<StorageUrlResponse>>()
        .map(|response| response.data.url)
        .map_err(|error| format!("Помилка обробки посилання Lumean: {error}"))?;

    let mut bytes = Vec::new();
    request_agent()
        .map_err(format_request_error)?
        .get(&signed_url)
        .call()
        .map_err(format_request_error)?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Не вдалося завантажити аудіофайл: {error}"))?;

    let extension = std::path::Path::new(storage_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .unwrap_or("mp3");
    let filename = format!("voice.{extension}");
    std::fs::write(std::path::Path::new(save_dir).join(&filename), bytes)
        .map_err(|error| format!("Не вдалося зберегти аудіофайл: {error}"))?;

    Ok(filename)
}

fn fetch_usage(key: &str) -> Result<String, ureq::Error> {
    let usage = request_agent()?
        .get(&format!("{BASE_URL}/usage"))
        .set("X-API-KEY", key)
        .set("Accept", "application/json")
        .call()?
        .into_json::<ApiResponse<Vec<UsageRecord>>>()
        .map_err(ureq::Error::from)?;

    Ok(usage
        .data
        .iter()
        .map(|record| format!("{}: {}", record.service_name, record.remaining))
        .collect::<Vec<_>>()
        .join(" · "))
}

fn request_agent() -> Result<ureq::Agent, ureq::Error> {
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30));

    if let Some(url) = proxy_url() {
        builder = builder.proxy(ureq::Proxy::new(url)?);
    }

    Ok(builder.build())
}

fn format_request_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(401, _) => "Невірний ключ Lumean".to_string(),
        ureq::Error::Status(402, _) => "Недостатньо доступних токенів Lumean".to_string(),
        ureq::Error::Status(403, _) => "Ключ Lumean не має потрібного дозволу".to_string(),
        ureq::Error::Status(429, _) => "Перевищено ліміт запитів або токенів Lumean".to_string(),
        ureq::Error::Status(code, _) => format!("Помилка Lumean ({code})"),
        error => format!("Помилка мережі Lumean: {error}"),
    }
}
