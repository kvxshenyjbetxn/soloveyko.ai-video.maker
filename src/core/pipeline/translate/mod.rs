mod translate;

use std::sync::{Arc, Mutex};
use eframe::egui;

/// Виконує етап перекладу у фоновому потоці.
/// Підставляє `{{text}}` у промт, викликає OpenRouter, зберігає результат у `text.txt`.
pub fn run_translation(
    service: String,
    key: String,
    model: String,
    prompt: String,
    text: String,
    temperature: f32,
    save_path: String,
    status: Arc<Mutex<crate::queue::JobStatus>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        *status.lock().unwrap() = crate::queue::JobStatus::Running;
        ctx.request_repaint();

        // Підставляємо текст у промт: якщо є плейсхолдер {{text}} — замінюємо,
        // інакше — додаємо текст після промту
        let user_content = if prompt.contains("{{text}}") {
            prompt.replace("{{text}}", &text)
        } else if !prompt.is_empty() {
            format!("{}\n\n{}", prompt, text)
        } else {
            text
        };

        let result = if service == "Claude Code" {
            crate::api::claude::call_claude_code(&model, &user_content)
        } else {
            translate::call_openrouter(&key, &model, user_content, temperature)
        };

        match result {
            Ok(translated) => {
                let dir = std::path::Path::new(&save_path);
                if std::fs::create_dir_all(dir).is_ok() {
                    let _ = std::fs::write(dir.join("text.txt"), &translated);
                }
                *status.lock().unwrap() = crate::queue::JobStatus::Done;
            }
            Err(e) => {
                *status.lock().unwrap() = crate::queue::JobStatus::Failed(e);
            }
        }

        ctx.request_repaint();
    });
}
