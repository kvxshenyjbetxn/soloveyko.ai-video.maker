use std::process::Stdio;
use std::sync::{Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до AGY CLI (семафор)
pub struct AgyLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl AgyLimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<AgyLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| AgyLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    pub fn set_max_threads(&self, max: usize) {
        *self.max_threads.lock().unwrap() = max;
        self.condvar.notify_all();
    }

    pub fn acquire(&self) -> AgyPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        AgyPermit { limiter: self }
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

pub struct AgyPermit<'a> {
    limiter: &'a AgyLimiter,
}

impl<'a> Drop for AgyPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Нова AGY сесія. on_chunk викликається один раз з повним текстом відповіді.
pub fn call_agy_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = AgyLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!(
        "Starting AGY CLI. Model: {}, session: {}",
        model, session_id
    ));

    // ВАЖЛИВО: НЕ замінювати на new_cli_command — cmd /C ламає великі аргументи на Windows.
    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content)
        .arg("--dangerously-skip-permissions")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        format!(
            "Failed to launch agy CLI: {}. Make sure agy is installed and in PATH.",
            e
        )
    })?;

    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        log("AGY CLI completed.");
        on_chunk(&response);
        Ok((response, session_id.to_string()))
    } else {
        let err = format!(
            "AGY CLI error (exit {:?}): {}",
            output.status.code(),
            stderr
        );
        log(&err);
        Err(err)
    }
}

/// Продовжує останню AGY сесію (--continue).
pub fn call_agy_resume(
    model: &str,
    message: &str,
    _session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let _permit = AgyLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log("Resuming AGY CLI session (--continue).");

    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(message)
        .arg("--continue")
        .arg("--dangerously-skip-permissions")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to launch agy CLI: {}", e))?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("AGY CLI resume completed.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "AGY CLI error (exit {:?}): {}",
            output.status.code(),
            stderr
        ))
    }
}

/// Простий виклик AGY CLI (переклад тощо).
pub fn call_agy_cli(
    model: &str,
    user_content: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    _allow_tools: bool,
) -> Result<String, String> {
    let _permit = AgyLimiter::get().acquire();
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("AGY CLI call. Model: {}", model));

    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content)
        .arg("--dangerously-skip-permissions")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to launch agy CLI: {}", e))?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        log("AGY CLI call completed.");
        Ok(response)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "AGY CLI error (exit {:?}): {}",
            output.status.code(),
            stderr
        ))
    }
}
