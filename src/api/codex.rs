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

/// Парсить одну NDJSON-подію від Codex CLI --json та повертає людиночитаний текст.
///
/// Підтримувані типи подій:
/// - "item.completed" / "item.started" type "agent_message"     → текст відповіді
/// - "item.completed" type "file_change" (status completed)     → [->] назва_файлу (kind)
/// - "item.started"   type "command_execution"                  → [Bash] $ команда (одразу при старті)
/// - "item.completed" type "command_execution" (є вивід)        → [->] перший рядок виводу
/// - "turn.completed"                                           → [STATS] з токенами
fn format_codex_json_event(
    line: &str,
    final_result: &mut String,
    acc_in: &mut u64,
    acc_out: &mut u64,
) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = v.get("type")?.as_str()?;

    match event_type {
        "item.started" => {
            let item = v.get("item")?;
            match item.get("type").and_then(|t| t.as_str())? {
                "command_execution" => {
                    // Показуємо команду одразу при старті — до завершення виконання
                    let cmd = item.get("command").and_then(|c| c.as_str()).unwrap_or("");
                    if cmd.is_empty() {
                        return None;
                    }
                    // Відрізаємо шлях до оболонки ("pwsh.exe" -Command 'xxx' → xxx)
                    let display = extract_shell_command(cmd);
                    Some(format!("[Bash] $ {}\n", display))
                }
                _ => None,
            }
        }
        "item.completed" => {
            let item = v.get("item")?;
            match item.get("type").and_then(|t| t.as_str())? {
                "agent_message" => {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            *final_result = trimmed.to_string();
                            return Some(format!("{}\n", trimmed));
                        }
                    }
                    None
                }
                "file_change" => {
                    if item.get("status").and_then(|s| s.as_str()) != Some("completed") {
                        return None;
                    }
                    let changes = item.get("changes")?.as_array()?;
                    let mut parts = Vec::new();
                    for change in changes {
                        let path = change.get("path").and_then(|p| p.as_str()).unwrap_or("");
                        let kind = change
                            .get("kind")
                            .and_then(|k| k.as_str())
                            .unwrap_or("modify");
                        let filename = std::path::Path::new(path)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or(path);
                        parts.push(format!("[->] {} ({})", filename, kind));
                    }
                    if parts.is_empty() {
                        None
                    } else {
                        Some(format!("{}\n", parts.join("\n")))
                    }
                }
                "command_execution" => {
                    // Показуємо перший рядок виводу як результат команди
                    let output = item
                        .get("aggregated_output")
                        .or_else(|| item.get("output"))
                        .and_then(|o| o.as_str())
                        .unwrap_or("");
                    let first = output.trim().lines().next().unwrap_or("").trim();
                    if first.is_empty() {
                        return None;
                    }
                    let preview = if first.len() > 150 {
                        format!("{}...", &first[..150])
                    } else {
                        first.to_string()
                    };
                    Some(format!("[->] {}\n", preview))
                }
                _ => None,
            }
        }
        "turn.completed" => {
            if let Some(usage) = v.get("usage") {
                let in_tok = usage
                    .get("input_tokens")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0);
                let out_tok = usage
                    .get("output_tokens")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0);
                *acc_in += in_tok;
                *acc_out += out_tok;
                Some(format!(
                    "\n[STATS]{:.1}|{}|{}|{:.6}|{}\n",
                    0.0_f64, acc_in, acc_out, 0.0_f64, 1u64
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Відрізає префікс оболонки з команди Codex.
/// Windows: "pwsh.exe" -Command 'Get-Content file' → "Get-Content file"
/// macOS:   /bin/bash -c 'cat file'                → "cat file"
/// Якщо формат не розпізнано — повертає оригінал.
fn extract_shell_command(cmd: &str) -> &str {
    // PowerShell (Windows): -Command 'xxx' або -Command "xxx"
    for prefix in ["-Command '", "-Command \""] {
        if let Some(pos) = cmd.find(prefix) {
            let start = pos + prefix.len();
            let quote = prefix.chars().last().unwrap();
            let end = cmd[start..]
                .rfind(quote)
                .map(|p| start + p)
                .unwrap_or(cmd.len());
            if end > start {
                return &cmd[start..end];
            }
        }
    }
    // bash/zsh (macOS/Linux): -c 'xxx' або -c "xxx"
    for prefix in [" -c '", " -c \""] {
        if let Some(pos) = cmd.find(prefix) {
            let start = pos + prefix.len();
            let quote = prefix.chars().last().unwrap();
            let end = cmd[start..]
                .rfind(quote)
                .map(|p| start + p)
                .unwrap_or(cmd.len());
            if end > start {
                return &cmd[start..end];
            }
        }
    }
    cmd
}

/// Читає stdout по рядках (NDJSON --json), парсить події та викликає `on_chunk` з відформатованим текстом.
/// thread_id береться з події thread.started і повертається як actual_session_id.
pub fn call_codex_new_session_streaming(
    model: &str,
    user_content: &str,
    _session_id: &str,
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

    log(&format!(
        "Starting Codex CLI agent session. Model: {}",
        model
    ));

    let mut cmd = crate::bundle::new_cli_command("codex");
    cmd.arg("exec")
        .arg("--model")
        .arg(model)
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("--json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!(
        "Running: codex exec --model {} --json --dangerously-bypass-approvals-and-sandbox -",
        model
    ));

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let mut child = crate::api::process::spawn_tracked(&mut cmd, tracked_job_id).map_err(|e| {
        format!(
            "Failed to launch codex CLI: {}. Make sure codex CLI is installed and added to PATH.",
            e
        )
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(user_content.as_bytes());
    }

    let stderr_handle = {
        let stderr = child.stderr.take().unwrap();
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = BufReader::new(stderr).read_to_string(&mut s);
            s
        })
    };

    let mut stdout = child.stdout.take().unwrap();
    let mut line_buf = String::new();
    let mut final_result = String::new();
    let mut actual_session_id = String::new();
    let mut acc_in: u64 = 0;
    let mut acc_out: u64 = 0;
    let mut buf = [0u8; 512];

    loop {
        let n = stdout
            .read(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
        line_buf.push_str(&chunk);

        while let Some(pos) = line_buf.find('\n') {
            let line = line_buf[..pos].to_string();
            line_buf = line_buf[pos + 1..].to_string();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Витягуємо thread_id з першої події
            if actual_session_id.is_empty() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if v.get("type").and_then(|t| t.as_str()) == Some("thread.started") {
                        if let Some(id) = v.get("thread_id").and_then(|id| id.as_str()) {
                            actual_session_id = id.to_string();
                            log(&format!("Codex thread_id: {}", actual_session_id));
                        }
                    }
                }
            }

            if let Some(text) =
                format_codex_json_event(trimmed, &mut final_result, &mut acc_in, &mut acc_out)
            {
                on_chunk(&text);
            }
        }
    }

    // Обробляємо залишок буфера
    let remaining = line_buf.trim().to_string();
    if !remaining.is_empty() {
        if let Some(text) =
            format_codex_json_event(&remaining, &mut final_result, &mut acc_in, &mut acc_out)
        {
            on_chunk(&text);
        }
    }

    let exit_status = child.wait().map_err(|e| format!("Wait error: {}", e))?;
    let stderr = stderr_handle.join().unwrap_or_default();

    if exit_status.success() {
        log("Codex CLI agent session completed successfully.");
        Ok((final_result.trim().to_string(), actual_session_id))
    } else {
        let err_msg = format!(
            "Codex CLI error (exit code: {:?}).\n--- STDERR ---\n{}",
            exit_status.code(),
            stderr.trim()
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
        .arg("--model")
        .arg(model)
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("--json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let mut child = crate::api::process::spawn_tracked(&mut cmd, tracked_job_id)
        .map_err(|e| format!("Failed to launch codex CLI resume: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(message.as_bytes());
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for codex CLI resume: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        log("Codex CLI resume completed.");
        let response =
            parse_codex_json_response(&stdout).unwrap_or_else(|| stdout.trim().to_string());
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log(&format!(
            "Codex CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        ));
        Err(format!("Codex CLI error: {}", stderr))
    }
}

/// Витягує фінальну відповідь агента з NDJSON-виводу Codex CLI --json.
/// Шукає останній item.completed type agent_message.
fn parse_codex_json_response(output: &str) -> Option<String> {
    let mut last_text = None;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if v.get("type").and_then(|t| t.as_str()) == Some("item.completed") {
                if let Some(item) = v.get("item") {
                    if item.get("type").and_then(|t| t.as_str()) == Some("agent_message") {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            let trimmed_text = text.trim();
                            if !trimmed_text.is_empty() {
                                last_text = Some(trimmed_text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    last_text
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
    cmd.arg("exec").arg("--model").arg(model);

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

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let mut child = crate::api::process::spawn_tracked(&mut cmd, tracked_job_id).map_err(|e| {
        let err_msg = format!(
            "Failed to launch codex CLI: {}. Make sure codex CLI is installed and added to PATH.",
            e
        );
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
