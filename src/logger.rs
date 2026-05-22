use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn get_logs_mutex() -> &'static Mutex<Vec<String>> {
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

/// Додає повідомлення до глобального логу.
pub fn log(msg: &str) {
    let formatted = format!("[{}] {}", get_timestamp(), msg);
    
    // Виводимо також у стандартний вивід для консолі розробника
    println!("{}", formatted);
    
    if let Ok(mut logs) = get_logs_mutex().lock() {
        logs.push(formatted);
        // Обмежуємо кількість записів в логу до 1000
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
}

/// Отримує список усіх логів.
pub fn get_logs() -> Vec<String> {
    if let Ok(logs) = get_logs_mutex().lock() {
        logs.clone()
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
