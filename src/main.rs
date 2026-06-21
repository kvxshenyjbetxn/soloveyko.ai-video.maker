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

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn load_icon() -> egui::IconData {
    let bytes = include_bytes!("../video.maker.png");
    let img = image::load_from_memory(bytes)
        .expect("Не вдалося завантажити іконку")
        .to_rgba8();
    let (width, height) = img.dimensions();
    egui::IconData { rgba: img.into_raw(), width, height }
}

fn renderer_backend() -> eframe::Renderer {
    match std::env::var("SOLOVEYKO_RENDERER") {
        Ok(value) if value.eq_ignore_ascii_case("wgpu") => eframe::Renderer::Wgpu,
        _ => eframe::Renderer::Glow,
    }
}

fn main() -> eframe::Result {
    // Конфігуруємо параметри вікна нашого додатку
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 768.0])         // Розмір вікна за замовчуванням (1366х768)
            .with_min_inner_size([800.0, 600.0])       // Встановлюємо мінімальний розмір вікна для зручності
            .with_title(format!("Soloveyko.AI-Video.Maker.v{}", APP_VERSION))   // Заголовок вікна програми
            .with_icon(std::sync::Arc::new(load_icon())),
        // За замовчуванням лишаємо стабільний glow-бекенд.
        // Експериментальний wgpu можна увімкнути через SOLOVEYKO_RENDERER=wgpu.
        renderer: renderer_backend(),
        ..Default::default()
    };

    // Запускаємо eframe
    eframe::run_native(
        "Soloveyko.AI-Video.Maker",
        options,
        Box::new(|cc| Ok(Box::new(VideoMakerApp::new(cc)))),
    )
}
