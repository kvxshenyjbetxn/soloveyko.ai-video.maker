use std::io::{BufReader, Read};
use std::process::Stdio;
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

    /// Повертає кількість активних потоків
    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
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

/// Чекає завершення процесу, парсить JSON та викликає `on_chunk` один раз з результатом.
/// Gemini з --output-format json видає весь output тільки в кінці, тому справжній streaming неможливий.
pub fn call_gemini_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = GeminiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Gemini CLI agent session. Model: {}, session: {}", model, session_id));

    let mut cmd = crate::bundle::new_cli_command("gemini");
    cmd.arg("--model").arg(model)
        .arg("--output-format").arg("json")
        .arg("--prompt").arg(user_content)
        .arg("--yolo")
        .arg("--skip-trust")
        .arg("--session-id").arg(session_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: gemini --model {} --prompt \"[prompt]\" --yolo --session-id {}", model, session_id));

    let mut child = cmd.spawn().map_err(|e| format!("Failed to launch gemini CLI: {}", e))?;

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
        full_output.push_str(&String::from_utf8_lossy(&buf[..n]));
    }

    let exit_status = child.wait().map_err(|e| format!("Wait error: {}", e))?;
    let stderr = stderr_handle.join().unwrap_or_default();

    if exit_status.success() {
        let response = parse_gemini_json_response(&full_output)
            .ok_or_else(|| "Gemini CLI: failed to parse JSON response".to_string())?;
        log("Gemini CLI agent session completed successfully.");
        on_chunk(&response);
        Ok((response, session_id.to_string()))
    } else {
        let err_msg = format!(
            "Gemini CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            exit_status.code(), stderr.trim(), full_output.trim()
        );
        log(&err_msg);
        Err(format!("Gemini CLI error: {}", stderr.trim()))
    }
}

/// Продовжує існуючу сесію Gemini CLI (--resume) та повертає відповідь.
pub fn call_gemini_resume(
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = GeminiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Resuming Gemini CLI session: {}", session_id));

    let mut cmd = crate::bundle::new_cli_command("gemini");
    cmd.arg("--model").arg(model)
        .arg("--output-format").arg("json")
        .arg("--prompt").arg(message)
        .arg("--yolo")
        .arg("--skip-trust")
        .arg("--resume").arg(session_id);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        format!("Failed to launch gemini CLI: {}", e)
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response = parse_gemini_json_response(&stdout)
            .ok_or_else(|| "Gemini CLI: failed to parse JSON response".to_string())?;
        log("Gemini CLI resume completed.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Gemini CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(), stderr, stdout
        );
        log(&err_msg);
        Err(format!("Gemini CLI error: {}", stderr))
    }
}

/// Викликає Gemini CLI для виконання перекладу сценарію або іншого тексту та повертає результат.
///
/// `allow_tools` — для сумісності з `call_claude_code`; Gemini завжди має `--yolo`, тому параметр
/// не змінює поведінку, але дозволяє уніфікований виклик через `call_llm`.
pub fn call_gemini_cli(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    _allow_tools: bool,
) -> Result<String, String> {
    let _permit = GeminiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Gemini CLI translation. Model: {}", model));

    let mut cmd = crate::bundle::new_cli_command("gemini");

    // Запускаємо: gemini --model <model> --output-format json --prompt "<prompt>" --yolo --skip-trust
    // JSON-формат гарантує чисту відповідь без технічного сміття у полі "response"
    cmd.arg("--model")
        .arg(model)
        .arg("--output-format")
        .arg("json")
        .arg("--prompt")
        .arg(user_content)
        .arg("--yolo")
        .arg("--skip-trust");

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!(
        "Running: gemini --model {} --output-format json --prompt \"[prompt]\" --yolo --skip-trust",
        model
    ));

    let output = cmd.output().map_err(|e| {
        let err_msg = format!("Failed to launch gemini CLI: {}. Make sure gemini CLI is installed and added to PATH.", e);
        log(&err_msg);
        err_msg
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response = parse_gemini_json_response(&stdout)
            .ok_or_else(|| "Gemini CLI: failed to parse JSON response".to_string())?;
        log("Gemini CLI translation completed successfully.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Gemini CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        );
        log(&err_msg);
        Err(format!("Gemini CLI error: {}", stderr))
    }
}

/// Витягує відповідь моделі з JSON-виводу Gemini CLI.
/// Структура відповіді: `{"session_id": "...", "response": "текст відповіді", "stats": {...}}`
fn parse_gemini_json_response(output: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(output.trim()).ok()?;
    let text = json.get("response")?.as_str()?;
    Some(text.trim().to_string())
}
