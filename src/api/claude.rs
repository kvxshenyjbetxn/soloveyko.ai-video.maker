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

/// Читає stdout по рядках (NDJSON), парсить JSON-події та викликає `on_chunk` з відформатованим текстом.
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

    // ВАЖЛИВО: НЕ замінювати на new_cli_command("claude")!
    // cmd /C ламає передачу аргументів з великим/складним промтом на Windows —
    // агент не отримує --dangerously-skip-permissions і питає дозволи замість запису файлу.
    // Claude і Gemini CLI запускаються напряму, без cmd /C. Codex — окремо, не чіпати.
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(user_content)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--dangerously-skip-permissions")
        .arg("--output-format").arg("stream-json")
        .arg("--verbose")
        .arg("--session-id").arg(session_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    log(&format!("Running: claude --model {} -p \"[prompt]\" --allowedTools Bash,Write,Read --dangerously-skip-permissions --output-format stream-json --session-id {}", model, session_id));

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
    let mut line_buf = String::new();
    let mut final_result = String::new();
    let mut raw_output = String::new();
    let mut acc_in: u64 = 0;
    let mut acc_out: u64 = 0;
    let mut buf = [0u8; 512];

    loop {
        let n = stdout.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
        raw_output.push_str(&chunk);
        line_buf.push_str(&chunk);

        // Обробляємо повні рядки NDJSON
        while let Some(pos) = line_buf.find('\n') {
            let line = line_buf[..pos].to_string();
            line_buf = line_buf[pos + 1..].to_string();
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(text) = format_claude_json_event(trimmed, &mut final_result, &mut acc_in, &mut acc_out) {
                on_chunk(&text);
            }
        }
    }

    // Обробляємо залишок буфера (якщо останній рядок без '\n')
    let remaining = line_buf.trim().to_string();
    if !remaining.is_empty() {
        if let Some(text) = format_claude_json_event(&remaining, &mut final_result, &mut acc_in, &mut acc_out) {
            on_chunk(&text);
        }
    }

    let exit_status = child.wait().map_err(|e| format!("Wait error: {}", e))?;
    let stderr = stderr_handle.join().unwrap_or_default();

    if exit_status.success() {
        log("Claude CLI agent session completed successfully.");
        Ok((final_result.trim().to_string(), session_id.to_string()))
    } else {
        let err_msg = format!(
            "Claude CLI error (exit code: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            exit_status.code(), stderr.trim(), raw_output.trim()
        );
        log(&err_msg);
        Err(format!("Claude CLI error: {}", stderr.trim()))
    }
}

/// Парсить одну NDJSON-подію від Claude CLI --output-format stream-json та повертає людиночитаний текст.
///
/// Структура подій stream-json:
/// - "assistant": message.content[] з блоками "thinking", "text", "tool_use"; message.usage з токенами
/// - "user":      message.content[] з блоками "tool_result"
/// - "result":    фінальна подія з полем "result" та загальною статистикою
///
/// acc_in/acc_out — накопичені токени сесії (для [LIVE_STATS]).
fn format_claude_json_event(
    line: &str,
    final_result: &mut String,
    acc_in: &mut u64,
    acc_out: &mut u64,
) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = v.get("type")?.as_str()?;

    match event_type {
        "assistant" => {
            let content = v.get("message")?.get("content")?.as_array()?;
            let mut parts = Vec::new();
            for item in content {
                match item.get("type").and_then(|t| t.as_str()) {
                    Some("thinking") => {
                        // Блоки роздумів — тільки для моделей з extended thinking
                        if let Some(text) = item.get("thinking").and_then(|t| t.as_str()) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                for tline in trimmed.lines() {
                                    parts.push(format!("[THINK]{}", tline));
                                }
                            }
                        }
                    }
                    Some("text") => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                parts.push(trimmed.to_string());
                            }
                        }
                    }
                    Some("tool_use") => {
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Tool");
                        let input = item.get("input").unwrap_or(&serde_json::Value::Null);
                        let detail = match name {
                            "Bash" => input.get("command").and_then(|c| c.as_str()).map(|c| format!("$ {}", c)),
                            "Read" => input.get("file_path").or_else(|| input.get("path")).and_then(|p| p.as_str()).map(|p| p.to_string()),
                            "Write" => input.get("file_path").or_else(|| input.get("path")).and_then(|p| p.as_str()).map(|p| p.to_string()),
                            _ => Some(input.to_string()),
                        }.unwrap_or_default();
                        parts.push(format!("[{}] {}", name, detail));
                    }
                    _ => {}
                }
            }
            // Накопичуємо токени та додаємо live-рядок статистики після кожного ходу
            if let Some(usage) = v.get("message").and_then(|m| m.get("usage")) {
                let in_tok  = usage.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                let out_tok = usage.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
                *acc_in  += in_tok;
                *acc_out += out_tok;
                parts.push(format!("[LIVE_STATS]{}|{}", acc_in, acc_out));
            }
            if parts.is_empty() { None } else { Some(format!("{}\n", parts.join("\n"))) }
        }
        "user" => {
            // Tool results повертаються як "user" повідомлення
            let content = v.get("message")?.get("content")?.as_array()?;
            let mut results = Vec::new();
            for item in content {
                if item.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                    let is_error = item.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
                    // [->] і [!!] — ASCII-маркери, рендерер малює іконки вручну
                    let prefix = if is_error { "[!!] " } else { "[->] " };
                    // content може бути рядком або масивом блоків
                    if let Some(text) = item.get("content").and_then(|c| c.as_str()) {
                        let first_line = text.trim().lines().next().unwrap_or("").trim();
                        if !first_line.is_empty() {
                            let preview = if first_line.len() > 150 { format!("{}...", &first_line[..150]) } else { first_line.to_string() };
                            results.push(format!("{}{}", prefix, preview));
                        }
                    } else if let Some(blocks) = item.get("content").and_then(|c| c.as_array()) {
                        for block in blocks {
                            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    let first_line = text.trim().lines().next().unwrap_or("").trim();
                                    if !first_line.is_empty() {
                                        let preview = if first_line.len() > 150 { format!("{}...", &first_line[..150]) } else { first_line.to_string() };
                                        results.push(format!("{}{}", prefix, preview));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if results.is_empty() { None } else { Some(format!("{}\n", results.join("\n"))) }
        }
        "result" => {
            if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                *final_result = result.to_string();
            }
            // Структурований маркер [STATS] — рендерер малює статистику вручну через Painter
            let duration_s   = v.get("duration_ms").and_then(|d| d.as_f64()).map(|ms| ms / 1000.0);
            let cost         = v.get("total_cost_usd").and_then(|c| c.as_f64());
            let input_tokens = v.get("usage").and_then(|u| u.get("input_tokens")).and_then(|t| t.as_u64());
            let output_tokens= v.get("usage").and_then(|u| u.get("output_tokens")).and_then(|t| t.as_u64());
            let num_turns    = v.get("num_turns").and_then(|t| t.as_u64());

            let has_data = duration_s.is_some() || input_tokens.is_some() || cost.is_some();
            if !has_data { return None; }
            Some(format!("\n[STATS]{:.1}|{}|{}|{:.6}|{}\n",
                duration_s.unwrap_or(0.0),
                input_tokens.unwrap_or(0),
                output_tokens.unwrap_or(0),
                cost.unwrap_or(0.0),
                num_turns.unwrap_or(0),
            ))
        }
        _ => None,
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

    // ВАЖЛИВО: НЕ замінювати на new_cli_command. Див. коментар у call_claude_code_new_session_streaming.
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(message)
        .arg("--allowedTools").arg("Bash,Write,Read")
        .arg("--dangerously-skip-permissions")
        .arg("--output-format").arg("stream-json")
        .arg("--verbose")
        .arg("--resume").arg(session_id);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        format!("Failed to launch claude CLI: {}", e)
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response = parse_claude_json_response(&stdout)
            .unwrap_or_else(|| stdout.trim().to_string());
        log("Claude CLI resume completed.");
        Ok(response)
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

/// Витягує фінальну відповідь з NDJSON-виводу Claude CLI --output-format json.
/// Шукає подію типу "result" та повертає її поле "result".
fn parse_claude_json_response(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if v.get("type").and_then(|t| t.as_str()) == Some("result") {
                if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                    return Some(result.trim().to_string());
                }
            }
        }
    }
    None
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

    // ВАЖЛИВО: НЕ замінювати на new_cli_command. Див. коментар у call_claude_code_new_session_streaming.
    let mut cmd = std::process::Command::new("claude");

    // Запускаємо: claude --model <model> -p "<prompt>" [--allowedTools Bash,Write,Read]
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content);

    if allow_tools {
        cmd.arg("--allowedTools").arg("Bash,Write,Read")
            .arg("--dangerously-skip-permissions");
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
