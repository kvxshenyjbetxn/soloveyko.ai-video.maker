use eframe::egui;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до VoiceBot (семафор)
pub struct VoiceBotLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl VoiceBotLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<VoiceBotLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| VoiceBotLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5), // Фіксовано 5 потоків за замовчуванням
        })
    }

    /// Отримує дозвіл на виконання запиту (блокує потік, якщо досягнуто ліміту)
    pub fn acquire(&self) -> VoiceBotPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        VoiceBotPermit { limiter: self }
    }

    /// Звільняє один потік та сповіщає інші очікуючі
    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }

    /// Повертає кількість активних потоків
    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

/// Дозвіл на виконання запиту VoiceBot
pub struct VoiceBotPermit<'a> {
    limiter: &'a VoiceBotLimiter,
}

impl<'a> Drop for VoiceBotPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

#[derive(serde::Deserialize)]
pub struct BalanceResponse {
    pub balance_text: String,
}

#[derive(serde::Deserialize)]
struct TaskCreateResponse {
    task_id: u64,
}

#[derive(serde::Deserialize)]
struct TaskStatusResponse {
    status: String,
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

/// Створює нову TTS-задачу на сервері та повертає її ID.
pub fn create_tts_task(key: &str, text: &str, template_uuid: Option<&str>) -> Result<u64, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let body = if let Some(uuid) = template_uuid {
        ureq::serde_json::json!({ "text": text, "template_uuid": uuid })
    } else {
        ureq::serde_json::json!({ "text": text })
    };

    let resp = agent
        .post("https://voiceapi.csv666.ru/tasks")
        .set("X-API-Key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| match e {
            ureq::Error::Status(401, _) => "Invalid VoiceBot API key (X-API-Key)".to_string(),
            ureq::Error::Status(402, _) => "Insufficient VoiceBot balance".to_string(),
            ureq::Error::Status(429, _) => "Active TTS task limit exceeded".to_string(),
            other => format!("Request error: {}", other),
        })?;

    resp.into_json::<TaskCreateResponse>()
        .map(|r| r.task_id)
        .map_err(|e| format!("Response parsing error: {}", e))
}

/// Повертає поточний статус TTS-задачі.
pub fn get_task_status(key: &str, task_id: u64) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build();

    let resp = agent
        .get(&format!(
            "https://voiceapi.csv666.ru/tasks/{}/status",
            task_id
        ))
        .set("X-API-Key", key)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Failed to get task status: {}", e))?;

    resp.into_json::<TaskStatusResponse>()
        .map(|r| r.status)
        .map_err(|e| format!("Status parsing error: {}", e))
}

/// Завантажує результат TTS-задачі та зберігає у вказану папку як voice.mp3 або voice.zip.
/// Повертає назву збереженого файлу.
pub fn download_task_result(key: &str, task_id: u64, save_dir: &str) -> Result<String, String> {
    use std::io::Read;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .build();

    let resp = agent
        .get(&format!(
            "https://voiceapi.csv666.ru/tasks/{}/result",
            task_id
        ))
        .set("X-API-Key", key)
        .call()
        .map_err(|e| format!("Failed to download result: {}", e))?;

    let content_type = resp.content_type().to_string();
    let ext = if content_type.contains("zip") {
        "zip"
    } else {
        "mp3"
    };
    let filename = format!("voice.{}", ext);

    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read response data: {}", e))?;

    let path = std::path::Path::new(save_dir).join(&filename);
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(filename)
}
