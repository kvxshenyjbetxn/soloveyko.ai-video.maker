use std::sync::{Condvar, Mutex, OnceLock};

// ─── Глобальний реєстр PID активних FFmpeg-процесів ─────────────────────────

/// Зберігає PID усіх активних ffmpeg-процесів. На виході з програми вбиває їх усі,
/// щоб уникнути зависання фонових процесів після закриття вікна.
pub struct ChildTracker {
    pids: Mutex<Vec<u32>>,
}

impl ChildTracker {
    pub fn get() -> &'static Self {
        static TRACKER: OnceLock<ChildTracker> = OnceLock::new();
        TRACKER.get_or_init(|| ChildTracker {
            pids: Mutex::new(Vec::new()),
        })
    }

    pub fn add(&self, pid: u32) {
        self.pids.lock().unwrap().push(pid);
    }

    pub fn remove(&self, pid: u32) {
        self.pids.lock().unwrap().retain(|&p| p != pid);
    }

    /// Примусово завершує всі зареєстровані ffmpeg-процеси.
    pub fn kill_all(&self) {
        let pids: Vec<u32> = std::mem::take(&mut *self.pids.lock().unwrap());
        for pid in pids {
            kill_by_pid(pid);
        }
    }
}

/// Запускає команду, реєструє PID, чекає завершення, знімає реєстрацію.
/// Замінник для `cmd.status()` з автоматичним відстеженням процесу.
pub fn run_tracked(cmd: &mut std::process::Command) -> std::io::Result<std::process::ExitStatus> {
    let mut child = cmd.spawn()?;
    let pid = child.id();
    ChildTracker::get().add(pid);
    let status = child.wait();
    ChildTracker::get().remove(pid);
    status
}

#[cfg(windows)]
fn kill_by_pid(pid: u32) {
    use std::ffi::c_void;
    unsafe extern "system" {
        fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> *mut c_void;
        fn TerminateProcess(h_process: *mut c_void, u_exit_code: u32) -> i32;
        fn CloseHandle(h_object: *mut c_void) -> i32;
    }
    const PROCESS_TERMINATE: u32 = 0x0001;
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
        }
    }
}

#[cfg(not(windows))]
fn kill_by_pid(pid: u32) {
    let _ = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status();
}

/// Лімітер одночасних процесів FFmpeg (семафор)
pub struct FfmpegLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl FfmpegLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<FfmpegLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| FfmpegLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(2),
        })
    }

    /// Встановлює максимальну кількість одночасних процесів
    pub fn set_max_threads(&self, max: usize) {
        let mut max_threads = self.max_threads.lock().unwrap();
        *max_threads = max;
        self.condvar.notify_all();
    }

    /// Отримує дозвіл на запуск (блокує потік, якщо досягнуто ліміту)
    pub fn acquire(&self) -> FfmpegPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        FfmpegPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }

    /// Повертає кількість активних процесів FFmpeg
    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

/// Дозвіл на запуск FFmpeg, який автоматично звільняється при виході з області видимості
pub struct FfmpegPermit<'a> {
    limiter: &'a FfmpegLimiter,
}

impl<'a> Drop for FfmpegPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}
