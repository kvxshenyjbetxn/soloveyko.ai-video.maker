#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "256"]

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

pub const APP_VERSION: &str = "4.1.2";

fn main() -> eframe::Result {
    // Конфігуруємо параметри вікна нашого додатку
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])         // Розмір вікна за замовчуванням (1366х768)
            .with_min_inner_size([800.0, 600.0])       // Встановлюємо мінімальний розмір вікна для зручності
            .with_title(format!("Soloveyko.AI-Video.Maker.v{}", APP_VERSION)),   // Заголовок вікна програми
        ..Default::default()
    };

    // Запускаємо eframe
    eframe::run_native(
        "Soloveyko.AI-Video.Maker",
        options,
        Box::new(|cc| Ok(Box::new(VideoMakerApp::new(cc)))),
    )
}
