use std::io::{BufReader, Read, Write};
use std::process::Stdio;
use std::sync::{Condvar, Mutex, OnceLock};

/// Зберігає промт у txt-файл і повертає аргумент `@file` для Pi CLI.
/// Це обходить ліміт довжини командного рядка на Windows.
fn write_pi_prompt_file(
    content: &str,
    working_dir: Option<&str>,
    file_tag: &str,
) -> Result<(std::path::PathBuf, String), String> {
    let safe_tag: String = file_tag
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let file_name = format!(".pi-prompt-{}.txt", safe_tag);

    if let Some(dir) = working_dir {
        let dir_path = std::path::Path::new(dir);
        if dir_path.is_dir() {
            let path = dir_path.join(&file_name);
            std::fs::write(&path, content)
                .map_err(|e| format!("Failed to write Pi prompt file {}: {}", path.display(), e))?;
            return Ok((path, format!("@{}", file_name)));
        }
    }

    let path = std::env::temp_dir().join(&file_name);
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write Pi prompt file {}: {}", path.display(), e))?;
    Ok((path.clone(), format!("@{}", path.to_string_lossy())))
}

fn remove_pi_prompt_file(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

/// Створює Command для pi CLI.
/// На Windows пріоритетно запускаємо саме pi.cmd, як це робить користувач у терміналі.
/// Це зменшує розбіжності між ручним запуском і запуском з програми.
fn pi_command() -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        // 1. Пріоритет: явний pi.cmd з npm-директорії.
        // Це найближче до звичайного ручного запуску `pi` у консолі.
        if let Some(cmd_path) = crate::bundle::find_npm_cmd_windows("pi") {
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", &cmd_path]);
            crate::bundle::set_no_window(&mut cmd);
            return cmd;
        }

        // 2. Fallback: прямий запуск через node + cli.js.
        if let Some((node_exe, script)) = crate::bundle::find_npm_node_script_windows("pi") {
            let mut cmd = std::process::Command::new(&node_exe);
            cmd.arg(&script);
            crate::bundle::set_no_window(&mut cmd);
            return cmd;
        }

        // 3. Останній fallback через PATH.
        crate::bundle::new_cli_command("pi")
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::bundle::new_direct_cli_command("pi")
    }
}

/// Створює Command для перевірки наявності Pi CLI у welcome-вікні.
/// Використовує той самий launch path, що і основні виклики агента.
pub fn version_command() -> std::process::Command {
    pi_command()
}

/// Лімітер одночасних запитів до Pi CLI (семафор)
pub struct PiLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl PiLimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<PiLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| PiLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    pub fn set_max_threads(&self, max: usize) {
        *self.max_threads.lock().unwrap() = max;
        self.condvar.notify_all();
    }

    pub fn acquire(&self) -> PiPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        PiPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }

    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

pub struct PiPermit<'a> {
    limiter: &'a PiLimiter,
}

impl<'a> Drop for PiPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Додає живий вивід Pi до логу відповідної задачі.
fn log_pi_progress(job_info: &Option<(u64, String)>, output: &str) {
    let output = output.trim();
    if output.is_empty() {
        return;
    }

    if let Some((id, name)) = job_info {
        crate::logger::log_job(*id, name, output);
    } else {
        crate::logger::log(output);
    }
}

/// Перетворює JSONL-подію Pi RPC на текст для чату та журналу.
fn format_pi_rpc_event(event: &str, response: &mut String) -> Option<String> {
    let event = serde_json::from_str::<serde_json::Value>(event).ok()?;
    let event_type = event.get("type")?.as_str()?;

    match event_type {
        "agent_start" => Some("\n▶ Pi: агент розпочав роботу\n".to_string()),
        "agent_settled" => Some("\n✓ Pi: агент завершив роботу\n".to_string()),
        "message_update" => {
            let update = event.get("assistantMessageEvent")?;
            match update.get("type")?.as_str()? {
                "text_delta" => {
                    let text = update.get("delta")?.as_str()?;
                    response.push_str(text);
                    Some(text.to_string())
                }
                "thinking_delta" => update
                    .get("delta")
                    .and_then(serde_json::Value::as_str)
                    .map(|text| format!("💭 {}", text)),
                "toolcall_end" => {
                    let tool_call = update.get("toolCall")?;
                    let name = tool_call
                        .get("name")
                        .or_else(|| tool_call.get("toolName"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("інструмент");
                    let args = tool_call
                        .get("arguments")
                        .or_else(|| tool_call.get("args"))
                        .map(serde_json::Value::to_string)
                        .unwrap_or_default();
                    Some(format!("\n🔧 Pi планує: {} {}\n", name, args))
                }
                _ => None,
            }
        }
        "tool_execution_start" => {
            let name = event
                .get("toolName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("інструмент");
            let args = event
                .get("args")
                .map(serde_json::Value::to_string)
                .unwrap_or_default();
            Some(format!("\n🔧 Pi виконує: {} {}\n", name, args))
        }
        "tool_execution_end" => {
            let name = event
                .get("toolName")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("інструмент");
            let status = if event
                .get("isError")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                "помилка"
            } else {
                "готово"
            };
            Some(format!("✓ Pi: {} — {}\n", name, status))
        }
        "extension_error" => event
            .get("error")
            .map(|error| format!("✗ Pi extension error: {}\n", error)),
        _ => None,
    }
}

/// Нова Pi сесія з живим прогресом через JSONL RPC-протокол.
pub fn call_pi_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = PiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!(
        "Starting Pi CLI. Model: {}, session: {}",
        model, session_id
    ));

    let mut cmd = pi_command();
    cmd.arg("--mode")
        .arg("rpc")
        .arg("--model")
        .arg(model)
        .arg("--tools")
        .arg("read,edit,write")
        .arg("--session-id")
        .arg(session_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let mut child = crate::api::process::spawn_tracked(&mut cmd, tracked_job_id).map_err(|e| {
        format!(
            "Failed to launch pi CLI: {}. Make sure pi is installed and in PATH.",
            e
        )
    })?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to capture Pi CLI input".to_string())?;
    let command = serde_json::json!({
        "id": session_id,
        "type": "prompt",
        "message": user_content,
    });
    writeln!(stdin, "{}", command)
        .map_err(|e| format!("Failed to send prompt to Pi CLI: {}", e))?;

    let stderr_job_info = job_info.clone();
    let stderr_handle = child.stderr.take().map(|stderr| {
        std::thread::spawn(move || {
            let mut output = String::new();
            let _ = BufReader::new(stderr).read_to_string(&mut output);
            log_pi_progress(&stderr_job_info, &output);
            output
        })
    });

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture Pi CLI output".to_string())?;
    let mut response = String::new();
    let mut line_buffer = String::new();
    let mut buffer = [0_u8; 512];
    let mut settled = false;

    while !settled {
        let bytes_read = stdout
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read Pi CLI output: {}", e))?;
        if bytes_read == 0 {
            break;
        }

        line_buffer.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
        while let Some(newline) = line_buffer.find('\n') {
            let line = line_buffer[..newline].trim_end_matches('\r').to_string();
            line_buffer.drain(..=newline);
            if line.is_empty() {
                continue;
            }

            if let Some(progress) = format_pi_rpc_event(&line, &mut response) {
                log_pi_progress(&job_info, &progress);
                on_chunk(&progress);
            }
            settled = serde_json::from_str::<serde_json::Value>(&line)
                .ok()
                .and_then(|event| {
                    event
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .as_deref()
                == Some("agent_settled");
            if settled {
                break;
            }
        }
    }

    drop(stdin);

    // RPC-процес очікує нових команд після завершення агента, тому завершуємо його самі.
    if settled {
        let _ = child.kill();
    }
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for pi CLI: {}", e))?;
    let stderr = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    if settled {
        log("Pi CLI completed.");
        Ok((response.trim().to_string(), session_id.to_string()))
    } else {
        let err = format!("Pi CLI error (exit {:?}): {}", status.code(), stderr.trim());
        log(&err);
        Err(err)
    }
}

/// Продовжує останню Pi сесію (--resume).
pub fn call_pi_resume(
    model: &str,
    message: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = PiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Resuming Pi CLI session: {}", session_id));

    let (prompt_path, prompt_arg) = write_pi_prompt_file(message, working_dir, session_id)?;

    let mut cmd = pi_command();
    cmd.arg("--model")
        .arg(model)
        .arg("--tools")
        .arg("read,edit,write")
        .arg("--session-id")
        .arg(session_id)
        .arg("-p")
        .arg(&prompt_arg)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let output_result = crate::api::process::output_tracked(&mut cmd, tracked_job_id);
    remove_pi_prompt_file(&prompt_path);
    let output = output_result.map_err(|e| format!("Failed to launch pi CLI: {}", e))?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("Pi CLI resume completed.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "Pi CLI error (exit {:?}): {}",
            output.status.code(),
            stderr
        ))
    }
}

/// Простий виклик Pi CLI (переклад тощо).
pub fn call_pi_cli(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    _allow_tools: bool,
) -> Result<String, String> {
    let _permit = PiLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Pi CLI call. Model: {}", model));

    let (prompt_path, prompt_arg) = write_pi_prompt_file(user_content, working_dir, "oneshot")?;

    let mut cmd = pi_command();
    cmd.arg("--model")
        .arg(model)
        .arg("--tools")
        .arg("read,edit,write")
        .arg("-p")
        .arg(&prompt_arg)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let output_result = crate::api::process::output_tracked(&mut cmd, tracked_job_id);
    remove_pi_prompt_file(&prompt_path);
    let output = output_result.map_err(|e| format!("Failed to launch pi CLI: {}", e))?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("Pi CLI call completed.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "Pi CLI error (exit {:?}): {}",
            output.status.code(),
            stderr
        ))
    }
}
