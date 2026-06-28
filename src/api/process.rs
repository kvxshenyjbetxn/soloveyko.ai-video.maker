use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command, ExitStatus, Output};
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
struct ProcessRegistry {
    all_pids: Vec<u32>,
    by_job: HashMap<u64, Vec<u32>>,
}

/// Глобальний трекер дочірніх процесів програми.
/// Дає змогу вбити всі процеси конкретної задачі або всі процеси при виході.
pub struct ProcessTracker {
    registry: Mutex<ProcessRegistry>,
}

impl ProcessTracker {
    pub fn get() -> &'static Self {
        static TRACKER: OnceLock<ProcessTracker> = OnceLock::new();
        TRACKER.get_or_init(|| ProcessTracker {
            registry: Mutex::new(ProcessRegistry::default()),
        })
    }

    pub fn add(&self, pid: u32, job_id: Option<u64>) {
        let mut registry = self.registry.lock().unwrap();
        if !registry.all_pids.contains(&pid) {
            registry.all_pids.push(pid);
        }
        if let Some(job_id) = job_id {
            let job_pids = registry.by_job.entry(job_id).or_default();
            if !job_pids.contains(&pid) {
                job_pids.push(pid);
            }
        }
    }

    pub fn remove(&self, pid: u32) {
        let mut registry = self.registry.lock().unwrap();
        registry.all_pids.retain(|tracked| *tracked != pid);
        registry.by_job.retain(|_, pids| {
            pids.retain(|tracked| *tracked != pid);
            !pids.is_empty()
        });
    }

    pub fn kill_job(&self, job_id: u64) {
        let pids = {
            let mut registry = self.registry.lock().unwrap();
            let pids = registry.by_job.remove(&job_id).unwrap_or_default();
            registry
                .all_pids
                .retain(|tracked| !pids.contains(tracked));
            pids
        };

        for pid in pids {
            kill_process_tree(pid);
        }
    }

    pub fn kill_all(&self) {
        let pids = {
            let mut registry = self.registry.lock().unwrap();
            registry.by_job.clear();
            std::mem::take(&mut registry.all_pids)
        };

        for pid in pids {
            kill_process_tree(pid);
        }
    }
}

/// Обгортка над Child, яка автоматично знімає PID з трекера після wait().
pub struct TrackedChild {
    child: Child,
    pid: u32,
}

impl TrackedChild {
    pub fn wait(mut self) -> std::io::Result<ExitStatus> {
        let result = self.child.wait();
        ProcessTracker::get().remove(self.pid);
        result
    }

    pub fn wait_with_output(self) -> std::io::Result<Output> {
        let result = self.child.wait_with_output();
        ProcessTracker::get().remove(self.pid);
        result
    }
}

impl Deref for TrackedChild {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl DerefMut for TrackedChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

/// Запускає дочірній процес у власній process-group та реєструє його в трекері.
pub fn spawn_tracked(cmd: &mut Command, job_id: Option<u64>) -> std::io::Result<TrackedChild> {
    prepare_command_for_tracking(cmd);
    let child = cmd.spawn()?;
    let pid = child.id();
    ProcessTracker::get().add(pid, job_id);
    Ok(TrackedChild { child, pid })
}

/// Аналог Command::output(), але з реєстрацією процесу по задачі.
pub fn output_tracked(cmd: &mut Command, job_id: Option<u64>) -> std::io::Result<Output> {
    spawn_tracked(cmd, job_id)?.wait_with_output()
}

/// Аналог Command::status(), але з реєстрацією процесу по задачі.
pub fn status_tracked(cmd: &mut Command, job_id: Option<u64>) -> std::io::Result<ExitStatus> {
    spawn_tracked(cmd, job_id)?.wait()
}

/// Примусово завершує всі дочірні процеси конкретної задачі.
pub fn kill_job_processes(job_id: u64) {
    ProcessTracker::get().kill_job(job_id);
}

/// Примусово завершує всі дочірні процеси програми.
pub fn kill_all_processes() {
    ProcessTracker::get().kill_all();
}

#[cfg(windows)]
fn prepare_command_for_tracking(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;

    // Всі tracked-процеси у нас фонові: вони не повинні відкривати консоль,
    // але мають жити у власній process-group для коректного taskkill /T.
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
}

#[cfg(unix)]
fn prepare_command_for_tracking(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;

    cmd.process_group(0);
}

#[cfg(not(any(unix, windows)))]
fn prepare_command_for_tracking(_cmd: &mut Command) {}

#[cfg(windows)]
fn kill_process_tree(pid: u32) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = Command::new("taskkill");
    cmd.creation_flags(CREATE_NO_WINDOW)
        .args(["/PID", &pid.to_string(), "/T", "/F"]);
    let _ = cmd.status();
}

#[cfg(unix)]
fn kill_process_tree(pid: u32) {
    use std::thread;
    use std::time::Duration;

    let group = format!("-{}", pid);
    let _ = Command::new("kill").args(["-TERM", "--", &group]).status();
    thread::sleep(Duration::from_millis(300));
    let _ = Command::new("kill").args(["-KILL", "--", &group]).status();
    let _ = Command::new("kill").args(["-KILL", &pid.to_string()]).status();
}

#[cfg(not(any(unix, windows)))]
fn kill_process_tree(pid: u32) {
    let _ = pid;
}
