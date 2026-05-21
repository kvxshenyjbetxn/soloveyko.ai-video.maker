mod app;
mod gui;
mod theme;

use app::VideoMakerApp;
use eframe::egui;

fn main() -> eframe::Result {
    // Конфігуруємо параметри вікна нашого додатку
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])         // Розмір вікна за замовчуванням (1366х768)
            .with_min_inner_size([800.0, 600.0])       // Встановлюємо мінімальний розмір вікна для зручності
            .with_title("Soloveyko.ai Video Maker"),   // Заголовок вікна програми
        ..Default::default()
    };

    // Запускаємо eframe
    eframe::run_native(
        "Soloveyko.ai Video Maker",
        options,
        Box::new(|cc| Ok(Box::new(VideoMakerApp::new(cc)))),
    )
}
