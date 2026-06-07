use std::io::{BufReader, Read};
use std::process::Stdio;
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

/// Читає stdout по chunks в реальному часі та викликає `on_chunk` для кожного.
pub fn call_claude_code_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = ClaudeLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Claude CLI agent session. Model: {}, session: {}", model, session_id));

    let mut cmd = crate::bundle::new_cli_command("claude");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(user_content)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--permission-mode").arg("bypassPermissions")
        .arg("--session-id").arg(session_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: claude --model {} -p \"[prompt]\" --allowedTools Bash,Write,Read --permission-mode bypassPermissions --session-id {}", model, session_id));

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to launch claude CLI: {}. Make sure claude CLI is installed and added to PATH.", e)
    })?;

    // Stderr читаємо в окремому потоці щоб не було deadlock
    let stderr_handle = {
        let stderr = child.stderr.take().unwrap();
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = BufReader::new(stderr).read_to_string(&mut s);
            s
        })
    };

    let mut stdout = child.stdout.take().unwrap();
    let mut full_output = String::new();
    let mut buf = [0u8; 512];

    loop {
        let n = stdout.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
        full_output.push_str(&chunk);
        on_chunk(&chunk);
    }

    let exit_status = child.wait().map_err(|e| format!("Wait error: {}", e))?;
    let stderr = stderr_handle.join().unwrap_or_default();

    if exit_status.success() {
        log("Claude CLI agent session completed successfully.");
        Ok((full_output.trim().to_string(), session_id.to_string()))
    } else {
        let err_msg = format!(
            "Claude CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            exit_status.code(), stderr.trim(), full_output.trim()
        );
        log(&err_msg);
        Err(format!("Claude CLI error: {}", stderr.trim()))
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

    let mut cmd = crate::bundle::new_cli_command("claude");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(message)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--permission-mode").arg("bypassPermissions")
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

    let mut cmd = crate::bundle::new_cli_command("claude");

    // Запускаємо: claude --model <model> -p "<prompt>" [--allowedTools Bash,Write,Read]
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content);

    if allow_tools {
        cmd.arg("--allowedTools").arg("Bash,Write,Read")
            .arg("--permission-mode").arg("bypassPermissions");
    }

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: claude --model {} -p \"[prompt and script text]\"", model));

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
