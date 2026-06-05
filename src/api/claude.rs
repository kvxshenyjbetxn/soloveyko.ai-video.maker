use std::process::Command;
use std::sync::{Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до Claude Code (семафор)
pub struct ClaudeLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl ClaudeLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<ClaudeLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| ClaudeLimiter {
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
    pub fn acquire(&self) -> ClaudePermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        ClaudePermit { limiter: self }
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

/// Дозвіл на виконання запиту, який автоматично звільняється при виході з області видимості
pub struct ClaudePermit<'a> {
    limiter: &'a ClaudeLimiter,
}

impl<'a> Drop for ClaudePermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Викликає Claude CLI з новою сесією (--session-id) та інструментами для запису файлів.
/// Повертає текст відповіді. Використовується для першого запуску агента при контролі агента.
pub fn call_claude_code_new_session(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = ClaudeLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Claude CLI agent session. Model: {}, session: {}", model, session_id));

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(&["/C", "claude"]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("claude");

    cmd.arg("--model").arg(model)
        .arg("-p").arg(user_content)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--session-id").arg(session_id);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: claude --model {} -p \"[prompt]\" --allowedTools Bash,Write,Read --session-id {}", model, session_id));

    let output = cmd.output().map_err(|e| {
        format!("Failed to launch claude CLI: {}. Make sure claude CLI is installed and added to PATH.", e)
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("Claude CLI agent session completed successfully.");
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Claude CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(), stderr, stdout
        );
        log(&err_msg);
        Err(format!("Claude CLI error: {}", stderr))
    }
}

/// Продовжує існуючу сесію Claude CLI (--resume) та повертає відповідь.
/// Використовується для чату з агентом після паузи "Контроль агента".
pub fn call_claude_code_resume(
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = ClaudeLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Resuming Claude CLI session: {}", session_id));

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(&["/C", "claude"]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("claude");

    cmd.arg("--model").arg(model)
        .arg("-p").arg(message)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--resume").arg(session_id);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        format!("Failed to launch claude CLI: {}", e)
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("Claude CLI resume completed.");
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Claude CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(), stderr, stdout
        );
        log(&err_msg);
        Err(format!("Claude CLI error: {}", stderr))
    }
}

/// Викликає Claude CLI для виконання перекладу сценарію або іншого тексту та повертає результат.
///
/// При `allow_tools = true` додає `--allowedTools Bash,Write,Read` — потрібно для агентного режиму,
/// де Claude має записувати файли на диск.
pub fn call_claude_code(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    allow_tools: bool,
) -> Result<String, String> {
    let _permit = ClaudeLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Claude CLI translation. Model: {}", model));

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(&["/C", "claude"]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("claude");

    // Запускаємо: claude --model <model> -p "<prompt>" [--allowedTools Bash,Write,Read]
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content);

    if allow_tools {
        cmd.arg("--allowedTools").arg("Bash,Write,Read");
    }

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Записуємо інформацію про команду у лог
    let debug_command = format!(
        "claude --model {} -p \"[prompt and script text]\"",
        model
    );
    log(&format!("Running: {}", debug_command));

    let output = cmd.output().map_err(|e| {
        let err_msg = format!("Failed to launch claude CLI: {}. Make sure claude CLI is installed and added to PATH.", e);
        log(&err_msg);
        err_msg
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("Claude CLI translation completed successfully.");
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Claude CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        );
        log(&err_msg);
        Err(format!("Claude CLI error: {}", stderr))
    }
}
