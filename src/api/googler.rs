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
        img_used: data
            .current_usage
            .hourly_usage
            .image_generation
            .map(|s| s.current_usage)
            .unwrap_or(0),
        img_limit: data.account_limits.img_gen_per_hour_limit,
        video_used: data
            .current_usage
            .hourly_usage
            .video_generation
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

        let url = format!("{}/v6/usage", BASE_URL);

        let (status_text, balance_opt) = match agent.get(&url).set("X-API-Key", &key).call() {
            Ok(response) => match response.into_json::<UsageResponse>() {
                Ok(data) => {
                    let bal = parse_balance(data);
                    let status = format!(
                        "✔ img: {}/{}/h • vid: {}/{}/h",
                        bal.img_used, bal.img_limit, bal.video_used, bal.video_limit
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

/// Відповідь після запуску генерації v6.
#[derive(serde::Deserialize)]
struct V6GenerationStarted {
    id: String,
}

/// Окремий результат генерації v6.
#[derive(serde::Deserialize)]
struct V6GenerationResult {
    #[serde(default, alias = "download_path")]
    download_url: Option<String>,
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

/// Статус генерації v6 при опитуванні.
#[derive(serde::Deserialize)]
struct V6GenerationStatus {
    status: String,
    #[serde(default)]
    results: Option<Vec<V6GenerationResult>>,
    #[serde(default)]
    error: Option<String>,
}

/// Повертає повний URL завантаження для нового v6 `download_url`
/// і для старого сумісного `download_path`.
fn resolve_download_url(download_url: &str) -> String {
    if download_url.starts_with("http://") || download_url.starts_with("https://") {
        download_url.to_string()
    } else {
        format!("{}{}", STORAGE_URL, download_url)
    }
}

/// Завантажує файл результату та повертає його як data URI.
fn fetch_result_data_uri(download_url: &str, agent: &ureq::Agent) -> Result<String, String> {
    use base64::Engine;
    use std::io::Read;

    let response = agent
        .get(&resolve_download_url(download_url))
        .call()
        .map_err(|e| format!("Помилка завантаження медіа: {}", e))?;

    let mime = response.content_type().to_string();
    let mime = if mime.is_empty() {
        "application/octet-stream".to_string()
    } else {
        mime
    };

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Помилка читання медіа: {}", e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// Опитує generation v6 до завершення. Повертає data URI, текст або помилку.
fn poll_v6_generation(
    key: &str,
    generation_id: &str,
    agent: &ureq::Agent,
    job_id: Option<u64>,
) -> Result<String, String> {
    let url = format!("{}/v6/generations/{}", BASE_URL, generation_id);
    let poll_interval = std::time::Duration::from_secs(3);

    for _ in 0..100 {
        std::thread::sleep(poll_interval);

        // Перевіряємо, чи задача не скасована
        if let Some(id) = job_id {
            if crate::queue::is_job_cancelled(id) {
                return Err(crate::queue::cancelled_error());
            }
        }

        let response = agent
            .get(&url)
            .set("X-API-Key", key)
            .call()
            .map_err(|e| format!("Помилка опитування v6: {}", e))?;

        let status: V6GenerationStatus = response
            .into_json()
            .map_err(|e| format!("Помилка парсингу статусу v6: {}", e))?;

        match status.status.as_str() {
            "succeeded" => {
                let result = status
                    .results
                    .and_then(|results| results.into_iter().next())
                    .ok_or_else(|| "Порожній результат v6".to_string())?;

                if let Some(data) = result.data {
                    return Ok(data);
                }
                if let Some(text) = result.text {
                    return Ok(text);
                }
                if let Some(download_url) = result.download_url {
                    return fetch_result_data_uri(&download_url, agent);
                }

                return Err("Результат v6 не містить даних".to_string());
            }
            "failed" => {
                return Err(status
                    .error
                    .unwrap_or_else(|| "Невідома помилка v6".to_string()));
            }
            _ => {}
        }
    }

    Err("Перевищено час очікування v6 (5 хвилин)".to_string())
}

/// Запускає generation v6 і повертає id створеної генерації.
fn start_v6_generation(
    key: &str,
    body: serde_json::Value,
    agent: &ureq::Agent,
) -> Result<String, String> {
    let url = format!("{}/v6/generations", BASE_URL);

    let response = match agent
        .post(&url)
        .set("X-API-Key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
    {
        Ok(response) => response,
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            return Err(format!("HTTP {}: {}", code, body));
        }
        Err(error) => return Err(format!("Помилка запиту: {}", error)),
    };

    let started: V6GenerationStarted = response
        .into_json()
        .map_err(|e| format!("Помилка парсингу відповіді v6: {}", e))?;

    Ok(started.id)
}

/// Перевіряє, чи помилка є перевищенням ліміту одночасних запитів.
fn is_concurrency_exceeded(err: &str) -> bool {
    err.contains("rate_limit.concurrency_exceeded")
}

/// Перевіряє, чи помилка є перевищенням годинного ліміту.
fn is_hourly_limit_exceeded(err: &str) -> bool {
    err.contains("rate_limit.hourly_exceeded")
}

/// Формує запит генерації зображення для канонічних операцій v6.
fn image_generation_body(
    provider: &str,
    prompt: &str,
    aspect_ratio: &str,
) -> Result<serde_json::Value, String> {
    let body = match provider {
        // Старі Flow-ключі мігруємо на підтримувані v6 моделі.
        "flow_IMAGEN_3_5" | "flow_GEM_PIX_2" | "flow_nano_banana_pro" => {
            serde_json::json!({
                "operation": "nano_banana_pro_image_generate",
                "prompt": prompt,
                "aspect_ratio": aspect_ratio,
            })
        }
        "flow_NARWHAL" | "flow_nano_banana_2" => serde_json::json!({
            "operation": "nano_banana_2_image_generate",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "flower" => serde_json::json!({
            "operation": "flower_image_generate",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "grok" => serde_json::json!({
            "operation": "grok_image_generate",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
            "quality": "speed",
        }),
        "openai" => serde_json::json!({
            "operation": "openai_image_generate",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        _ => return Err(format!("Невідомий провайдер зображень: {}", provider)),
    };

    Ok(body)
}

/// Формує запит text-to-video для канонічних операцій v6.
fn video_generation_body(
    provider: &str,
    prompt: &str,
    aspect_ratio: &str,
) -> Result<serde_json::Value, String> {
    let body = match provider {
        "flow" | "flow_fast" => serde_json::json!({
            "operation": "flow_video_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "flower" => serde_json::json!({
            "operation": "flower_video_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "grok" => serde_json::json!({
            "operation": "grok_video_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
            "resolution": "480p",
        }),
        "flow_omni_flash" => serde_json::json!({
            "operation": "flow_video_omni_flash_from_text_10s",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "flow_light" => serde_json::json!({
            "operation": "flow_video_light_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "flow_ultra_light" => serde_json::json!({
            "operation": "flow_video_ultra_light_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        "flow_quality" => serde_json::json!({
            "operation": "flow_video_quality_from_text",
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }),
        _ => return Err(format!("Невідомий провайдер відео: {}", provider)),
    };

    Ok(body)
}

/// Формує запит image-to-video для канонічних операцій v6.
fn animation_generation_body(
    provider: &str,
    image_data_uri: &str,
    prompt: &str,
) -> Result<serde_json::Value, String> {
    let body = match provider {
        "flow" | "flow_fast" => serde_json::json!({
            "operation": "flow_video_from_ingredients",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "inputs": [image_data_uri],
        }),
        "flower" => serde_json::json!({
            "operation": "flower_video_from_image",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "inputs": [image_data_uri],
        }),
        "grok" => serde_json::json!({
            "operation": "grok_video_from_image",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "resolution": "480p",
            "inputs": [image_data_uri],
        }),
        "flow_omni_flash" => serde_json::json!({
            "operation": "flow_video_omni_flash_from_ingredients_10s",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "inputs": [image_data_uri],
        }),
        "flow_light" => serde_json::json!({
            "operation": "flow_video_light_from_ingredients",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "inputs": [image_data_uri],
        }),
        "flow_ultra_light" => serde_json::json!({
            "operation": "flow_video_ultra_light_from_ingredients",
            "prompt": prompt,
            "aspect_ratio": "16:9",
            "inputs": [image_data_uri],
        }),
        _ => {
            return Err(format!(
                "Провайдер {} не підтримує анімацію зображень",
                provider
            ));
        }
    };

    Ok(body)
}

/// Генерує зображення з перебором провайдерів за пріоритетом.
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
/// `on_started` викликається лише тоді, коли запит справді отримав слот у лімітері
/// і прямо зараз відправляється на генерацію.
/// Повертає `(provider_name, data_uri)` — щоб caller знав який провайдер переміг.
pub fn generate_image_with_priority<F>(
    key: &str,
    prompt: &str,
    aspect_ratio: &str,
    priority: &[String],
    mut on_started: F,
    job_id: Option<u64>,
) -> Result<(String, String), String>
where
    F: FnMut(&str),
{
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            // Перевіряємо скасування перед кожною спробою
            if let Some(id) = job_id {
                if crate::queue::is_job_cancelled(id) {
                    return Err(crate::queue::cancelled_error());
                }
            }

            let _permit = GooglerImageLimiter::get().acquire();
            let result = image_generation_body(provider, prompt, aspect_ratio).and_then(|body| {
                let generation_id = start_v6_generation(key, body, &agent)?;
                on_started(provider);
                poll_v6_generation(key, &generation_id, &agent, job_id)
            });

            match result {
                Ok(result) => return Ok((provider.clone(), result)),
                Err(e) => {
                    // Якщо це помилка скасування - повертаємо її одразу
                    if crate::queue::is_cancelled_error(&e) {
                        return Err(e);
                    }

                    if is_concurrency_exceeded(&e) {
                        std::thread::sleep(DELAY);
                    } else if is_hourly_limit_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Зображення [{}] годинний ліміт, чекаю 5 хв…",
                            provider
                        ));
                        std::thread::sleep(std::time::Duration::from_secs(300));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Зображення [{}] спроба {}/{}: {}",
                            provider,
                            failures,
                            RETRIES + 1,
                            e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                        std::thread::sleep(DELAY);
                    }
                }
            }
        }
    }

    Err("Всі провайдери зображень вичерпані".to_string())
}

/// Анімує зображення в відео з перебором провайдерів за пріоритетом (image-to-video).
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
/// `on_started` викликається лише тоді, коли запит справді отримав слот у лімітері
/// і прямо зараз починає анімацію, а не просто стоїть у черзі.
/// Повертає `(provider_name, data_uri)` — щоб caller знав який провайдер переміг.
pub fn animate_image_with_priority<F>(
    key: &str,
    image_data_uri: &str,
    prompt: &str,
    priority: &[String],
    mut on_started: F,
    job_id: Option<u64>,
) -> Result<(String, String), String>
where
    F: FnMut(&str),
{
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            // Перевіряємо скасування перед кожною спробою
            if let Some(id) = job_id {
                if crate::queue::is_job_cancelled(id) {
                    return Err(crate::queue::cancelled_error());
                }
            }

            let _permit = GooglerVideoLimiter::get().acquire();
            on_started(provider);

            let result =
                animation_generation_body(provider, image_data_uri, prompt).and_then(|body| {
                    let generation_id = start_v6_generation(key, body, &agent)?;
                    on_started(provider);
                    poll_v6_generation(key, &generation_id, &agent, job_id)
                });

            match result {
                Ok(result) => return Ok((provider.clone(), result)),
                Err(e) => {
                    // Якщо це помилка скасування - повертаємо її одразу
                    if crate::queue::is_cancelled_error(&e) {
                        return Err(e);
                    }

                    if is_concurrency_exceeded(&e) {
                        std::thread::sleep(DELAY);
                    } else if is_hourly_limit_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Анімація [{}] годинний ліміт, чекаю 5 хв…",
                            provider
                        ));
                        std::thread::sleep(std::time::Duration::from_secs(300));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Анімація [{}] спроба {}/{}: {}",
                            provider,
                            failures,
                            RETRIES + 1,
                            e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                        std::thread::sleep(DELAY);
                    }
                }
            }
        }
    }

    Err("Всі відео-провайдери вичерпані для анімації".to_string())
}

/// Повертає читабельну назву моделі генерації зображень Googler для логів.
pub fn image_provider_model_name(key: &str) -> &'static str {
    match key {
        "flow_IMAGEN_3_5" | "flow_GEM_PIX_2" | "flow_nano_banana_pro" => "Nano Banana Pro (Flow)",
        "flow_NARWHAL" | "flow_nano_banana_2" => "Nano Banana 2 (Flow)",
        "flower" => "Flower Image",
        "grok" => "Grok Image",
        "openai" => "OpenAI Image",
        _ => "Unknown",
    }
}

/// Повертає читабельну назву відеомоделі Googler для логів.
pub fn video_provider_model_name(key: &str) -> &'static str {
    match key {
        "flow" | "flow_fast" => "Flow Video Fast",
        "flower" => "Flower Video",
        "grok" => "Grok Video",
        "flow_omni_flash" => "Flow Video Omni Flash",
        "flow_light" => "Flow Video Light",
        "flow_ultra_light" => "Flow Video Ultra Light",
        "flow_quality" => "Flow Video Quality",
        _ => "Unknown",
    }
}

/// Генерує відео з перебором провайдерів за пріоритетом.
/// Для кожного провайдера: 3 спроби з паузою 5с між ними.
/// Повертає `(provider_name, data_uri)` — щоб caller знав який провайдер переміг.
/// `on_started` викликається лише тоді, коли запит справді отримав слот у лімітері
/// і прямо зараз відправляється на генерацію.
pub fn generate_video_with_priority_logged<F>(
    key: &str,
    prompt: &str,
    aspect_ratio: &str,
    priority: &[String],
    mut on_started: F,
    job_id: Option<u64>,
) -> Result<(String, String), String>
where
    F: FnMut(&str),
{
    const RETRIES: u32 = 2;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(300))
        .build();

    for provider in priority {
        let mut failures = 0u32;
        loop {
            // Перевіряємо скасування перед кожною спробою
            if let Some(id) = job_id {
                if crate::queue::is_job_cancelled(id) {
                    return Err(crate::queue::cancelled_error());
                }
            }

            let _permit = GooglerVideoLimiter::get().acquire();
            let result = video_generation_body(provider, prompt, aspect_ratio).and_then(|body| {
                let generation_id = start_v6_generation(key, body, &agent)?;
                on_started(provider);
                poll_v6_generation(key, &generation_id, &agent, job_id)
            });

            match result {
                Ok(result) => return Ok((provider.clone(), result)),
                Err(e) => {
                    // Якщо це помилка скасування - повертаємо її одразу
                    if crate::queue::is_cancelled_error(&e) {
                        return Err(e);
                    }

                    if is_concurrency_exceeded(&e) {
                        std::thread::sleep(DELAY);
                    } else if is_hourly_limit_exceeded(&e) {
                        crate::logger::log(&format!(
                            " Відео [{}] годинний ліміт, чекаю 5 хв…",
                            provider
                        ));
                        std::thread::sleep(std::time::Duration::from_secs(300));
                    } else {
                        failures += 1;
                        crate::logger::log(&format!(
                            " Відео [{}] спроба {}/{}: {}",
                            provider,
                            failures,
                            RETRIES + 1,
                            e
                        ));
                        if failures > RETRIES {
                            break;
                        }
                        std::thread::sleep(DELAY);
                    }
                }
            }
        }
    }

    Err("Всі відео-провайдери вичерпані".to_string())
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
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        GooglerImagePermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
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
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        GooglerVideoPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
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

        let url = format!("{}/v6/usage", BASE_URL);

        if let Ok(response) = agent.get(&url).set("X-API-Key", &key).call() {
            if let Ok(data) = response.into_json::<UsageResponse>() {
                *result.lock().unwrap() = Some(parse_balance(data));
                ctx.request_repaint();
            }
        }
    });
}
