use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Структура, яка представляє один запис логу.
#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub job_id: Option<u64>,
    pub job_name: Option<String>,
    pub message: String,
}

static LOGS: OnceLock<Mutex<Vec<LogEntry>>> = OnceLock::new();

fn get_logs_mutex() -> &'static Mutex<Vec<LogEntry>> {
    LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn get_timestamp() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        // Отримуємо години, хвилини та секунди (у системному часі UTC)
        let secs = duration.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        "00:00:00".to_string()
    }
}

/// Додає повідомлення до глобального логу без прив'язки до задачі.
pub fn log(msg: &str) {
    log_with_job(None, None, msg);
}

/// Додає повідомлення до логу із прив'язкою до конкретної задачі.
pub fn log_job(job_id: u64, job_name: &str, msg: &str) {
    log_with_job(Some(job_id), Some(job_name.to_string()), msg);
}

fn log_with_job(job_id: Option<u64>, job_name: Option<String>, msg: &str) {
    let timestamp = get_timestamp();
    
    // Форматуємо для виводу в консоль
    let formatted = if let (Some(id), Some(name)) = (job_id, &job_name) {
        format!("[{}] [Задача #{}: {}] {}", timestamp, id + 1, name, msg)
    } else {
        format!("[{}] {}", timestamp, msg)
    };
    
    println!("{}", formatted);
    
    let entry = LogEntry {
        timestamp,
        job_id,
        job_name,
        message: msg.to_string(),
    };
    
    if let Ok(mut logs) = get_logs_mutex().lock() {
        logs.push(entry);
        // Обмежуємо кількість записів в логу до 1000
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
}

/// Отримує відформатований список усіх логів для відображення в інтерфейсі.
pub fn get_logs() -> Vec<String> {
    if let Ok(logs) = get_logs_mutex().lock() {
        logs.iter().map(|entry| {
            if let (Some(id), Some(name)) = (entry.job_id, &entry.job_name) {
                format!("[{}] [Задача #{}: {}] {}", entry.timestamp, id + 1, name, entry.message)
            } else {
                format!("[{}] {}", entry.timestamp, entry.message)
            }
        }).collect()
    } else {
        Vec::new()
    }
}

/// Отримує логи тільки для конкретної задачі.
pub fn get_job_logs(job_id: u64) -> Vec<String> {
    if let Ok(logs) = get_logs_mutex().lock() {
        logs.iter()
            .filter(|entry| entry.job_id == Some(job_id))
            .map(|entry| format!("[{}] {}", entry.timestamp, entry.message))
            .collect()
    } else {
        Vec::new()
    }
}

/// Очищає список логів.
pub fn clear_logs() {
    if let Ok(mut logs) = get_logs_mutex().lock() {
        logs.clear();
    }
}
