use std::sync::{Condvar, Mutex, OnceLock};

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
