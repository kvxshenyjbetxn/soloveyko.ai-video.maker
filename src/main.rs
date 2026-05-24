#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod core;
mod bundle;
mod gui;
mod queue;
mod theme;
mod localization;
mod logger;

use app::VideoMakerApp;
use eframe::egui;

fn main() -> eframe::Result {
    // Конфігуруємо параметри вікна нашого додатку
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])         // Розмір вікна за замовчуванням (1366х768)
            .with_min_inner_size([800.0, 600.0])       // Встановлюємо мінімальний розмір вікна для зручності
            .with_title("Soloveyko.AI-Video.Maker.v1.0.0"),   // Заголовок вікна програми
        ..Default::default()
    };

    // Запускаємо eframe
    eframe::run_native(
        "Soloveyko.AI-Video.Maker",
        options,
        Box::new(|cc| Ok(Box::new(VideoMakerApp::new(cc)))),
    )
}
