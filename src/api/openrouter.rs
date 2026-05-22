use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
struct CreditsData {
    total_credits: f64,
    total_usage: f64,
}

#[derive(serde::Deserialize)]
struct CreditsResponse {
    data: CreditsData,
}

/// Фоново завантажує баланс OpenRouter і записує результат в `result`.
pub fn fetch_balance(key: String, result: Arc<Mutex<Option<String>>>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let _permit = OpenRouterLimiter::get().acquire();

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let text = match agent
            .get("https://openrouter.ai/api/v1/credits")
            .set("Authorization", &format!("Bearer {}", key))
            .call()
        {
            Ok(resp) => match resp.into_json::<CreditsResponse>() {
                Ok(data) => {
                    let remaining = (data.data.total_credits - data.data.total_usage).max(0.0);
                    format!("${:.2}", remaining)
                }
                Err(_) => return,
            },
            Err(_) => return,
        };

        *result.lock().unwrap() = Some(text);
        ctx.request_repaint();
    });
}

use std::sync::{Condvar, OnceLock};

/// Лімітер одночасних запитів до OpenRouter (семафор)
pub struct OpenRouterLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl OpenRouterLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<OpenRouterLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| OpenRouterLimiter {
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
    pub fn acquire(&self) -> OpenRouterPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let max = *self.max_threads.lock().unwrap();
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        OpenRouterPermit { limiter: self }
    }

    /// Звільняє один потік та сповіщає інші очікуючі
    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }
}

/// Дозвіл на виконання запиту, який автоматично звільняється при виході з області видимості
pub struct OpenRouterPermit<'a> {
    limiter: &'a OpenRouterLimiter,
}

impl<'a> Drop for OpenRouterPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}
