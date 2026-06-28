use std::process::Stdio;
use std::sync::{Condvar, Mutex, OnceLock};

/// Створює Command для pi CLI.
/// На Windows pi — npm-пакет (.cmd + node.js), тому запускаємо node напряму зі скриптом.
/// Це надійніше ніж cmd /C, бо не залежить від PATH у shell-сесії.
fn pi_command() -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        // Читаємо pi.cmd → знаходимо node.exe та cli.js → запускаємо напряму
        if let Some((node_exe, script)) = crate::bundle::find_npm_node_script_windows("pi") {
            let mut cmd = std::process::Command::new(&node_exe);
            cmd.arg(&script);
            crate::bundle::set_no_window(&mut cmd);
            return cmd;
        }
        // Fallback: cmd /C pi.cmd (якщо парсинг не вдався)
        if let Some(cmd_path) = crate::bundle::find_npm_cmd_windows("pi") {
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", &cmd_path]);
            crate::bundle::set_no_window(&mut cmd);
            return cmd;
        }
        // Останній fallback через PATH
        crate::bundle::new_cli_command("pi")
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::bundle::new_direct_cli_command("pi")
    }
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

/// Нова Pi сесія. on_chunk викликається один раз з повним текстом відповіді.
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
    cmd.arg("--model")
        .arg(model)
        .arg("--session-id")
        .arg(session_id)
        .arg("-p")
        .arg(user_content)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let output = crate::api::process::output_tracked(&mut cmd, tracked_job_id).map_err(|e| {
        format!(
            "Failed to launch pi CLI: {}. Make sure pi is installed and in PATH.",
            e
        )
    })?;

    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        log("Pi CLI completed.");
        on_chunk(&response);
        Ok((response, session_id.to_string()))
    } else {
        let err = format!("Pi CLI error (exit {:?}): {}", output.status.code(), stderr);
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

    let mut cmd = pi_command();
    cmd.arg("--model")
        .arg(model)
        .arg("--session-id")
        .arg(session_id)
        .arg("-p")
        .arg(message)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let output = crate::api::process::output_tracked(&mut cmd, tracked_job_id)
        .map_err(|e| format!("Failed to launch pi CLI: {}", e))?;

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

    let mut cmd = pi_command();
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let tracked_job_id = job_info.as_ref().map(|(id, _)| *id);
    let output = crate::api::process::output_tracked(&mut cmd, tracked_job_id)
        .map_err(|e| format!("Failed to launch pi CLI: {}", e))?;

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
