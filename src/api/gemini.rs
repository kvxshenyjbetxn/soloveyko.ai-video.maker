use std::process::Command;
use std::sync::{Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до Gemini CLI (семафор)
pub struct GeminiLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl GeminiLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<GeminiLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| GeminiLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    /// Встановлює максимальну кількість одночасних запитів
    pub fn set_max_threads(&self, max: usize) {
        let mut max_threads = self.max_threads.lock().unwrap();
        *max_threads = max;
        self.condvar.notify_all();
    }

    /// Отримує дозвіл на виконання запиту (блокує потік, якщо досягнуто ліміту)
    pub fn acquire(&self) -> GeminiPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        GeminiPermit { limiter: self }
    }

    /// Звільняє один потік та сповіщає інші очікуючі
    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }
}

/// Дозвіл на виконання запиту, який автоматично звільняється при виході з області видимості
pub struct GeminiPermit<'a> {
    limiter: &'a GeminiLimiter,
}

impl<'a> Drop for GeminiPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Викликає Gemini CLI для виконання перекладу сценарію або іншого тексту та повертає результат.
///
/// Використовує прапорці `--model`, `--prompt`, `--yolo` та `--skip-trust` для отримання результату від Gemini CLI.
pub fn call_gemini_cli(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
) -> Result<String, String> {
    let _permit = GeminiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Запуск Gemini CLI для перекладу. Модель: {}", model));

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(&["/C", "gemini"]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("gemini");

    // Запускаємо: gemini --model <model> --prompt "<prompt>" --yolo --skip-trust
    cmd.arg("--model")
        .arg(model)
        .arg("--prompt")
        .arg(user_content)
        .arg("--yolo")
        .arg("--skip-trust");

    // Записуємо інформацію про команду у лог
    let debug_command = format!(
        "gemini --model {} --prompt \"[текст промпту та сценарію]\" --yolo --skip-trust",
        model
    );
    log(&format!("Виконується: {}", debug_command));

    let output = cmd.output().map_err(|e| {
        let err_msg = format!("Не вдалося запустити gemini cli: {}. Перевірте, чи встановлено gemini CLI та чи додано його в PATH.", e);
        log(&err_msg);
        err_msg
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let cleaned = sanitize_gemini_output(&stdout);
        log("Gemini CLI успішно виконав переклад.");
        Ok(cleaned)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Gemini CLI помилка (код статусу: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        );
        log(&err_msg);
        Err(format!("Gemini CLI помилка: {}", stderr))
    }
}

/// Очищає вивід Gemini CLI від службових ANSI escape-кодів та системних повідомлень
fn sanitize_gemini_output(input: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    let mut chars = input.chars().peekable();
    
    // 1. Видалення ANSI escape-кодів без зовнішніх бібліотек
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            in_escape = true;
            if let Some(&'[') = chars.peek() {
                chars.next();
            }
            continue;
        }
        if in_escape {
            let code = c as u32;
            if (64..=126).contains(&code) {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }

    // 2. Фільтрація службових рядків ініціалізації CLI
    result.lines()
        .filter(|line| {
            let l = line.trim();
            !l.starts_with("🤖") &&
            !l.starts_with("Using ") &&
            !l.starts_with("Approved ") &&
            !l.starts_with("Trusting ") &&
            !l.starts_with("Authorized ") &&
            !l.starts_with("Active ") &&
            !l.starts_with("Working ") &&
            !l.starts_with("Session ")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
