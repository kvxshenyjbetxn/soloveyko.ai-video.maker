use eframe::egui;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

const BASE_URL: &str = "https://googler.fast-gen.ai/api";
const STORAGE_URL: &str = "https://storage.fast-gen.ai";

/// Баланс та ліміти акаунту Googler.
#[derive(Clone, Default)]
#[allow(dead_code)]
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
#[allow(dead_code)]
struct ActiveThreads {
    #[serde(default)]
    image_threads: i64,
    #[serde(default)]
    video_threads: i64,
    #[serde(default)]
    flow_ultra_threads: i64,
}

#[derive(serde::Deserialize, Default)]
struct CurrentUsage {
    #[serde(default)]
    hourly_usage: HourlyUsage,
    #[serde(default)]
    active_threads: ActiveThreads,
}

#[derive(serde::Deserialize, Default)]
#[allow(dead_code)]
struct AccountLimits {
    #[serde(default)]
    img_gen_per_hour_limit: i64,
    #[serde(default)]
    video_gen_per_hour_limit: i64,
    #[serde(default)]
    img_generation_threads_allowed: i64,
    #[serde(default)]
    video_generation_threads_allowed: i64,
    #[serde(default)]
    prompt_tokens_per_hour_limit: i64,
    #[serde(default)]
    flow_ultra_hour_limit: i64,
    #[serde(default)]
    flow_ultra_threads_allowed: i64,
}

#[derive(serde::Deserialize)]
struct UsageResponse {
    account_limits: AccountLimits,
    current_usage: CurrentUsage,
}

fn parse_balance(data: UsageResponse) -> GooglerBalance {
    GooglerBalance {
        img_used: data.current_usage.hourly_usage.image_generation
            .map(|s| s.current_usage)
            .unwrap_or(0),
        img_limit: data.account_limits.img_gen_per_hour_limit,
        video_used: data.current_usage.hourly_usage.video_generation
            .map(|s| s.current_usage)
            .unwrap_or(0),
        video_limit: data.account_limits.video_gen_per_hour_limit,
        img_threads_active: data.current_usage.active_threads.image_threads,
        img_threads_allowed: data.account_limits.img_generation_threads_allowed,
        video_threads_active: data.current_usage.active_threads.video_threads,
        video_threads_allowed: data.account_limits.video_generation_threads_allowed,
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

        let url = format!("{}/v5/usage", BASE_URL);

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

/// Відповідь після запуску операції генерації.
#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct OperationStarted {
    operation_id: String,
}

/// Статус асинхронної операції при опитуванні.
#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct OperationPollStatus {
    status: String,
    result: Option<Vec<String>>,
    error: Option<String>,
}

/// Опитує операцію до завершення (макс. ~5 хвилин). Повертає перший результат або помилку.
#[allow(dead_code)]
fn poll_operation(key: &str, operation_id: &str, agent: &ureq::Agent) -> Result<String, String> {
    let url = format!("{}/v4/operations/{}", BASE_URL, operation_id);
    let poll_interval = std::time::Duration::from_secs(3);

    for _ in 0..100 {
        std::thread::sleep(poll_interval);

        let response = agent
            .get(&url)
            .set("X-API-Key", key)
            .call()
            .map_err(|e| format!("Помилка опитування: {}", e))?;

        let status: OperationPollStatus = response
            .into_json()
            .map_err(|e| format!("Помилка парсингу статусу: {}", e))?;

        match status.status.as_str() {
            "success" => {
                return status
                    .result
                    .and_then(|r| r.into_iter().next())
                    .ok_or_else(|| "Порожній результат операції".to_string());
            }
            "error" => {
                return Err(status.error.unwrap_or_else(|| "Невідома помилка провайдера".to_string()));
            }
            _ => {} // pending / processing — продовжуємо опитування
        }
    }

    Err("Перевищено час очікування операції (5 хвилин)".to_string())
}

// ─── V5 API ──────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct V5GenerationStarted {
    id: String,
}

#[derive(serde::Deserialize)]
struct V5GenerationResult {
    download_path: Option<String>,
    data: Option<String>,
}

#[derive(serde::Deserialize)]
struct V5GenerationStatus {
    status: String,
    results: Option<Vec<V5GenerationResult>>,
    error: Option<String>,
}

/// Опитує v5 generation до завершення. Повертає data URI або завантажує з storage.
fn poll_v5_generation(key: &str, gen_id: &str, agent: &ureq::Agent) -> Result<String, String> {
    use std::io::Read;
    use base64::Engine;

    let url = format!("{}/v5/generations/{}", BASE_URL, gen_id);
    let poll_interval = std::time::Duration::from_secs(3);

    for _ in 0..100 {
        std::thread::sleep(poll_interval);

        let response = agent
            .get(&url)
            .set("X-API-Key", key)
            .call()
            .map_err(|e| format!("Помилка опитування v5: {}", e))?;

        let status: V5GenerationStatus = response
            .into_json()
            .map_err(|e| format!("Помилка парсингу статусу v5: {}", e))?;

        match status.status.as_str() {
            "succeeded" => {
                let result = status.results
                    .and_then(|r| r.into_iter().next())
                    .ok_or_else(|| "Порожній результат v5".to_string())?;

                if let Some(data) = result.data {
                    return Ok(data);
                }

                if let Some(path) = result.download_path {
                    let storage_url = format!("{}{}", STORAGE_URL, path);
                    let resp = agent
                        .get(&storage_url)
                        .call()
                        .map_err(|e| format!("Помилка завантаження медіа: {}", e))?;
                    let mime = resp.content_type().to_string();
                    let mut bytes = Vec::new();
                    resp.into_reader()
                        .read_to_end(&mut bytes)
                        .map_err(|e| format!("Помилка читання медіа: {}", e))?;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    return Ok(format!("data:{};base64,{}", mime, b64));
                }

                return Err("Результат v5 не містить даних".to_string());
            }
            "failed" => {
                return Err(status.error.unwrap_or_else(|| "Невідома помилка v5".to_string()));
            }
            _ => {} // queued / running — продовжуємо
        }
    }

    Err("Перевищено час очікування v5 (5 хвилин)".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────

/// Перевіряє, чи помилка є перевищенням ліміту одночасних запитів.
fn is_concurrency_exceeded(err: &str) -> bool {
    err.contains("rate_limit.concurrency_exceeded")
}

/// Спроба генерації зображення через конкретного провайдера.
fn try_generate_image(key: &str, prompt: &str, aspect_ratio: &str, provider: &str, agent: &ureq::Agent) -> Result<String, String> {
    let _permit = GooglerImageLimiter::get().acquire();
    let (url, body) = match provider {
        "flow_IMAGEN_3_5" => (
            format!("{}/v4/flow/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "model": "IMAGEN_3_5", "aspect_ratio": aspect_ratio}),
        ),
        "flow_GEM_PIX_2" => (
            format!("{}/v4/flow/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "model": "GEM_PIX_2", "aspect_ratio": aspect_ratio}),
        ),
        "flow_NARWHAL" => (
            format!("{}/v4/flow/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "model": "NARWHAL", "aspect_ratio": aspect_ratio}),
        ),
        "flower" => (
            format!("{}/v4/flower/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        "grok" => (
            format!("{}/v4/grok/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        "openai" => (
            format!("{}/v4/openai/image/generate", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        _ => return Err(format!("Невідомий провайдер зображень: {}", provider)),
    };

    let response = match agent
        .post(&url)
        .set("X-API-Key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(format!("HTTP {}: {}", code, body));
        }
        Err(e) => return Err(format!("Помилка запиту: {}", e)),
    };

    let op: OperationStarted = response
        .into_json()
        .map_err(|e| format!("Помилка парсингу відповіді: {}", e))?;

    poll_operation(key, &op.operation_id, agent)
}

/// Спроба генерації відео через конкретного провайдера.
fn try_generate_video(key: &str, prompt: &str, aspect_ratio: &str, provider: &str, agent: &ureq::Agent) -> Result<String, String> {
    let _permit = GooglerVideoLimiter::get().acquire();

    // v5 провайдери
    let v5_operation = match provider {
        "flow_omni_flash" => Some("flow_video_omni_flash_from_text_10s"),
        "flow_fast"        => Some("flow_video_from_text"),
        "flow_light"       => Some("flow_video_light_from_text"),
        "flow_quality"     => Some("flow_video_quality_from_text"),
        _ => None,
    };
    if let Some(operation) = v5_operation {
        let url = format!("{}/v5/generations", BASE_URL);
        let body = serde_json::json!({"operation": operation, "prompt": prompt, "aspect_ratio": aspect_ratio});
        let response = match agent.post(&url).set("X-API-Key", key).set("Content-Type", "application/json").send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                return Err(format!("HTTP {}: {}", code, body));
            }
            Err(e) => return Err(format!("Помилка запиту: {}", e)),
        };
        let started: V5GenerationStarted = response.into_json().map_err(|e| format!("Помилка парсингу v5: {}", e))?;
        return poll_v5_generation(key, &started.id, agent);
    }

    let (url, body) = match provider {
        "flow" => (
            format!("{}/v4/flow/video/from-text", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        "flower" => (
            format!("{}/v4/flower/video/from-text", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        "grok" => (
            format!("{}/v4/grok/video/from-text", BASE_URL),
            serde_json::json!({"prompt": prompt, "aspect_ratio": aspect_ratio}),
        ),
        _ => return Err(format!("Невідомий провайдер відео: {}", provider)),
    };

    let response = match agent
        .post(&url)
        .set("X-API-Key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(format!("HTTP {}: {}", code, body));
        }
        Err(e) => return Err(format!("Помилка запиту: {}", e)),
    };

    let op: OperationStarted = response
        .into_json()
        .map_err(|e| format!("Помилка парсингу відповіді: {}", e))?;

    poll_operation(key, &op.operation_id, agent)
}

/// Генерує зображення з перебором провайдерів за пріоритетом.
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
pub fn generate_image_with_priority(
    key: &str,
    prompt: &str,
    aspect_ratio: &str,
    priority: &[String],
) -> Result<String, String> {
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            match try_generate_image(key, prompt, aspect_ratio, provider, &agent) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if is_concurrency_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Зображення [{}] ліміт потоків, чекаю…",
                            provider
                        ));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Зображення [{}] спроба {}/{}: {}",
                            provider, failures, RETRIES + 1, e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                    }
                    std::thread::sleep(DELAY);
                }
            }
        }
    }

    Err("Всі провайдери зображень вичерпані".to_string())
}

/// Спроба анімації зображення (image-to-video) через конкретного провайдера.
fn try_animate_image(key: &str, image_data_uri: &str, prompt: &str, provider: &str, agent: &ureq::Agent) -> Result<String, String> {
    let _permit = GooglerVideoLimiter::get().acquire();
    let (url, body) = match provider {
        "flower" => (
            format!("{}/v4/flower/video/from-image", BASE_URL),
            serde_json::json!({"image": image_data_uri, "prompt": prompt, "aspect_ratio": "16:9"}),
        ),
        "flow" => (
            format!("{}/v4/flow/video/from-ingredients", BASE_URL),
            serde_json::json!({"prompt": prompt, "reference_images": [image_data_uri], "aspect_ratio": "16:9"}),
        ),
        "grok" => (
            format!("{}/v4/grok/video/from-image", BASE_URL),
            serde_json::json!({"image": image_data_uri, "prompt": prompt}),
        ),
        _ => return Err(format!("Провайдер {} не підтримує анімацію зображень", provider)),
    };

    let response = match agent
        .post(&url)
        .set("X-API-Key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(format!("HTTP {}: {}", code, body));
        }
        Err(e) => return Err(format!("Помилка запиту: {}", e)),
    };

    let op: OperationStarted = response
        .into_json()
        .map_err(|e| format!("Помилка парсингу відповіді: {}", e))?;

    poll_operation(key, &op.operation_id, agent)
}

/// Анімує зображення в відео з перебором провайдерів за пріоритетом (image-to-video).
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
pub fn animate_image_with_priority(
    key: &str,
    image_data_uri: &str,
    prompt: &str,
    priority: &[String],
) -> Result<String, String> {
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            match try_animate_image(key, image_data_uri, prompt, provider, &agent) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if is_concurrency_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Анімація [{}] ліміт потоків, чекаю…",
                            provider
                        ));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Анімація [{}] спроба {}/{}: {}",
                            provider, failures, RETRIES + 1, e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                    }
                    std::thread::sleep(DELAY);
                }
            }
        }
    }

    Err("Всі відео-провайдери вичерпані для анімації".to_string())
}

/// Генерує відео з перебором провайдерів за пріоритетом.
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
pub fn generate_video_with_priority(
    key: &str,
    prompt: &str,
    aspect_ratio: &str,
    priority: &[String],
) -> Result<String, String> {
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            match try_generate_video(key, prompt, aspect_ratio, provider, &agent) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if is_concurrency_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Відео [{}] ліміт потоків, чекаю…",
                            provider
                        ));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Відео [{}] спроба {}/{}: {}",
                            provider, failures, RETRIES + 1, e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                    }
                    std::thread::sleep(DELAY);
                }
            }
        }
    }

    Err("Всі провайдери відео вичерпані".to_string())
}

// ─── Локальні лімітери потоків ───────────────────────────────────────────────

/// Лімітер одночасних запитів генерації зображень Googler.
pub struct GooglerImageLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl GooglerImageLimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<GooglerImageLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| GooglerImageLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    pub fn set_max_threads(&self, max: usize) {
        *self.max_threads.lock().unwrap() = max;
        self.condvar.notify_all();
    }

    pub fn acquire(&self) -> GooglerImagePermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max { break; }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        GooglerImagePermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 { *active -= 1; }
        self.condvar.notify_one();
    }

    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

pub struct GooglerImagePermit<'a> {
    limiter: &'a GooglerImageLimiter,
}

impl Drop for GooglerImagePermit<'_> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Лімітер одночасних запитів генерації відео Googler.
pub struct GooglerVideoLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl GooglerVideoLimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<GooglerVideoLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| GooglerVideoLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    pub fn set_max_threads(&self, max: usize) {
        *self.max_threads.lock().unwrap() = max;
        self.condvar.notify_all();
    }

    pub fn acquire(&self) -> GooglerVideoPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max { break; }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        GooglerVideoPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 { *active -= 1; }
        self.condvar.notify_one();
    }

    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

pub struct GooglerVideoPermit<'a> {
    limiter: &'a GooglerVideoLimiter,
}

impl Drop for GooglerVideoPermit<'_> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Фоново завантажує інфо про ліміти Googler і записує результат у `result`.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<GooglerBalance>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!("{}/v5/usage", BASE_URL);

        if let Ok(response) = agent.get(&url).set("X-API-Key", &key).call() {
            if let Ok(data) = response.into_json::<UsageResponse>() {
                *result.lock().unwrap() = Some(parse_balance(data));
                ctx.request_repaint();
            }
        }
    });
}
