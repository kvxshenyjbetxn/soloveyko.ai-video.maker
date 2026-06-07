use std::io::{BufReader, Read};
use std::process::Stdio;
use std::sync::{Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до Codex CLI (семафор)
pub struct CodexLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl CodexLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<CodexLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| CodexLimiter {
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
    pub fn acquire(&self) -> CodexPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        CodexPermit { limiter: self }
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
pub struct CodexPermit<'a> {
    limiter: &'a CodexLimiter,
}

impl<'a> Drop for CodexPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Очищає вивід Codex від технічних заголовків та підписів.
pub fn extract_codex_response(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut response_lines = Vec::new();
    let mut collecting = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "codex" {
            collecting = true;
            response_lines.clear();
            continue;
        }
        if trimmed == "tokens used" {
            break;
        }
        if collecting {
            response_lines.push(line);
        }
    }

    if response_lines.is_empty() {
        output.trim().to_string()
    } else {
        response_lines.join("\n").trim().to_string()
    }
}

/// Знаходить session id у виводі Codex.
pub fn find_session_id(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("session id:") {
            if let Some(id) = trimmed.split(':').nth(1) {
                return Some(id.trim().to_string());
            }
        }
    }
    None
}

/// Читає stdout по chunks в реальному часі та викликає `on_chunk` для кожного.
pub fn call_codex_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = CodexLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Codex CLI agent session. Model: {}", model));

    let mut cmd = crate::bundle::new_cli_command("codex");
    cmd.arg("exec")
        .arg("--model").arg(model)
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("-") // Читаємо промпт зі stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: codex exec --model {} --dangerously-bypass-approvals-and-sandbox -", model));

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to launch codex CLI: {}. Make sure codex CLI is installed and added to PATH.", e)
    })?;

    // Записуємо промпт у stdin та закриваємо його для передачі EOF
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(user_content.as_bytes());
    }

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
    let mut sent_len = 0;
    let mut actual_session_id = session_id.to_string();

    loop {
        let n = stdout.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
        full_output.push_str(&chunk);

        // Шукаємо session id, якщо ще не знайшли
        if actual_session_id == session_id {
            if let Some(id) = find_session_id(&full_output) {
                actual_session_id = id;
                log(&format!("Detected real Codex session ID: {}", actual_session_id));
            }
        }

        // Обчислюємо корисний текст для стрімінгу
        let mut start_pos = None;
        if full_output.starts_with("codex\n") {
            start_pos = Some(6);
        } else if full_output.starts_with("codex\r\n") {
            start_pos = Some(7);
        } else if let Some(pos) = full_output.find("\ncodex\n") {
            start_pos = Some(pos + 7);
        } else if let Some(pos) = full_output.find("\ncodex\r\n") {
            start_pos = Some(pos + 8);
        } else if let Some(pos) = full_output.find("\r\ncodex\r\n") {
            start_pos = Some(pos + 9);
        }

        if let Some(sp) = start_pos {
            let sub = &full_output[sp..];
            let mut end_pos = None;
            if let Some(pos) = sub.find("\ntokens used\n") {
                end_pos = Some(pos);
            } else if let Some(pos) = sub.find("\ntokens used\r\n") {
                end_pos = Some(pos);
            } else if let Some(pos) = sub.find("\r\ntokens used\r\n") {
                end_pos = Some(pos);
            }

            let useful_text = match end_pos {
                Some(ep) => &sub[..ep],
                None => sub,
            };

            if useful_text.len() > sent_len {
                let to_send = &useful_text[sent_len..];
                on_chunk(to_send);
                sent_len = useful_text.len();
            }
        }
    }

    let exit_status = child.wait().map_err(|e| format!("Wait error: {}", e))?;
    let stderr = stderr_handle.join().unwrap_or_default();

    if exit_status.success() {
        log("Codex CLI agent session completed successfully.");
        let final_response = extract_codex_response(&full_output);
        Ok((final_response, actual_session_id))
    } else {
        let err_msg = format!(
            "Codex CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            exit_status.code(), stderr.trim(), full_output.trim()
        );
        log(&err_msg);
        Err(format!("Codex CLI error: {}", stderr.trim()))
    }
}

/// Продовжує існуючу сесію Codex CLI (exec resume) та повертає відповідь.
/// Використовується для чату з агентом після паузи "Контроль агента".
pub fn call_codex_resume(
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = CodexLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Resuming Codex CLI session: {}", session_id));

    let mut cmd = crate::bundle::new_cli_command("codex");
    cmd.arg("exec")
        .arg("resume")
        .arg(session_id)
        .arg("--model").arg(model)
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("-") // Читаємо повідомлення зі stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to launch codex CLI resume: {}", e)
    })?;

    // Записуємо повідомлення у stdin та закриваємо його для передачі EOF
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(message.as_bytes());
    }

    let output = child.wait_with_output().map_err(|e| {
        format!("Failed to wait for codex CLI resume: {}", e)
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        log("Codex CLI resume completed.");
        let clean_response = extract_codex_response(&stdout);
        Ok(clean_response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Codex CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(), stderr, stdout
        );
        log(&err_msg);
        Err(format!("Codex CLI error: {}", stderr))
    }
}

/// Викликає Codex CLI для виконання перекладу сценарію або іншого тексту та повертає результат.
pub fn call_codex(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    allow_tools: bool,
) -> Result<String, String> {
    let _permit = CodexLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting Codex CLI translation. Model: {}", model));

    let mut cmd = crate::bundle::new_cli_command("codex");

    // Запускаємо: codex exec --model <model> - [--dangerously-bypass-approvals-and-sandbox]
    cmd.arg("exec")
        .arg("--model")
        .arg(model);

    if allow_tools {
        cmd.arg("--dangerously-bypass-approvals-and-sandbox");
    }

    cmd.arg("-") // Читаємо текст перекладу зі stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: codex exec --model {} -", model));

    let mut child = cmd.spawn().map_err(|e| {
        let err_msg = format!("Failed to launch codex CLI: {}. Make sure codex CLI is installed and added to PATH.", e);
        log(&err_msg);
        err_msg
    })?;

    // Записуємо текст у stdin та закриваємо його для передачі EOF
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(user_content.as_bytes());
    }

    let output = child.wait_with_output().map_err(|e| {
        let err_msg = format!("Failed to wait for codex CLI: {}", e);
        log(&err_msg);
        err_msg
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        log("Codex CLI translation completed successfully.");
        let clean_response = extract_codex_response(&stdout);
        Ok(clean_response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Codex CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        );
        log(&err_msg);
        Err(format!("Codex CLI error: {}", stderr))
    }
}

