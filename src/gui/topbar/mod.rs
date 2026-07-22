use crate::localization::{Language, translate};
use eframe::egui;

pub mod balance;
pub use balance::{draw_balance_window, draw_threads_window};

/// Повертає колір для відображення навантаження потоків за відсотком зайнятості.
/// 0 → default, 1–40% → зелений, 41–75% → жовтий, 76–100% → червоний.
pub fn thread_load_color(active: usize, max: usize, default: egui::Color32) -> egui::Color32 {
    if active == 0 || max == 0 {
        return default;
    }
    let pct = active as f32 / max as f32;
    if pct <= 0.40 {
        egui::Color32::from_rgb(80, 200, 100)
    } else if pct <= 0.75 {
        egui::Color32::from_rgb(255, 200, 0)
    } else {
        egui::Color32::from_rgb(220, 70, 70)
    }
}

/// Малює клікабельний чіп з довільним кольором тексту. Основа для balance та thread чіпів.
pub fn draw_chip(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> egui::Response {
    let font_id = egui::FontId::new(13.0, egui::FontFamily::Proportional);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_id, text_color));
    let padding = egui::vec2(8.0, 4.0);
    let desired_size = galley.rect.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() {
            ui.visuals().widgets.hovered.weak_bg_fill
        } else {
            ui.visuals().faint_bg_color
        };
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(4.0), fill);
        ui.painter().galley(rect.min + padding, galley, text_color);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Малює компактний чіп з балансом. При наведенні підсвічується і змінює курсор.
pub fn draw_balance_chip(ui: &mut egui::Ui, prefix: &str, value: &str) -> egui::Response {
    draw_chip(
        ui,
        &format!("{}: {}", prefix, value),
        ui.visuals().text_color(),
    )
}

/// Малює верхню навігаційну панель з вкладками та балансами.
pub fn draw_navigation_bar(
    ctx: &egui::Context,
    active_tab: &mut crate::app::Tab,
    jobs: &[crate::queue::PipelineJob],
    language: Language,
    openrouter_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    lumean_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    googler_balance: &std::sync::Arc<std::sync::Mutex<Option<crate::api::googler::GooglerBalance>>>,
    balance_window_open: &mut bool,
) {
    use crate::app::Tab;
    egui::TopBottomPanel::top("navigation_bar")
        .min_height(40.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(
                    active_tab,
                    Tab::Main,
                    egui::RichText::new(translate(language, "tab_main")).size(14.0),
                );

                let has_media = jobs.iter().any(|j| {
                    std::path::Path::new(&j.settings.save_path)
                        .join("media")
                        .exists()
                });
                if has_media {
                    ui.selectable_value(
                        active_tab,
                        Tab::Gallery,
                        egui::RichText::new(translate(language, "tab_gallery")).size(14.0),
                    );
                } else if *active_tab == Tab::Gallery {
                    *active_tab = Tab::Main;
                }

                ui.selectable_value(
                    active_tab,
                    Tab::Settings,
                    egui::RichText::new(translate(language, "tab_settings")).size(14.0),
                );
                ui.selectable_value(
                    active_tab,
                    Tab::Logs,
                    egui::RichText::new(translate(language, "tab_logs")).size(14.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if ui
                        .add(egui::Button::new(egui::RichText::new("⚙").size(16.0)).frame(false))
                        .clicked()
                    {
                        *balance_window_open = true;
                    }
                    ui.add_space(4.0);
                    if let Ok(guard) = openrouter_balance.try_lock() {
                        if let Some(text) = guard.as_ref() {
                            if draw_balance_chip(ui, "OpenRouter", text).clicked() {
                                *balance_window_open = true;
                            }
                            ui.separator();
                        }
                    }
                    if let Ok(guard) = lumean_balance.try_lock() {
                        if let Some(text) = guard.as_ref() {
                            let display = text.split_whitespace().next().unwrap_or(text.as_str());
                            if draw_balance_chip(ui, "Lumean", display).clicked() {
                                *balance_window_open = true;
                            }
                            ui.separator();
                        }
                    }
                    if let Ok(guard) = googler_balance.try_lock() {
                        if let Some(bal) = guard.as_ref() {
                            let text = format!(
                                "img: {}/{} vid: {}/{}",
                                bal.img_used, bal.img_limit, bal.video_used, bal.video_limit,
                            );
                            if draw_balance_chip(ui, "Googler", &text).clicked() {
                                *balance_window_open = true;
                            }
                        }
                    }
                });
            });
        });
}

/// Малює нижній рядок статусу потоків (реєструвати ДО SidePanel).
pub fn draw_status_bar(
    ctx: &egui::Context,
    openrouter_max_threads: usize,
    claude_max_threads: usize,
    gemini_max_threads: usize,
    codex_max_threads: usize,
    agy_max_threads: usize,
    pi_max_threads: usize,
    edge_tts_max_threads: usize,
    ffmpeg_max_threads: usize,
    googler_image_max_threads: usize,
    googler_video_max_threads: usize,
    threads_window_open: &mut bool,
) {
    egui::TopBottomPanel::bottom("status_bar")
        .min_height(40.0)
        .show(ctx, |ui| {
            let mut open_threads = false;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let normal = ui.visuals().text_color();

                let thread_chip =
                    |ui: &mut egui::Ui, name: &str, active: usize, max: usize| -> bool {
                        let color = thread_load_color(active, max, normal);
                        draw_chip(ui, &format!("{}: {}/{}", name, active, max), color).clicked()
                    };

                if ui
                    .add(egui::Button::new(egui::RichText::new("⚙").size(16.0)).frame(false))
                    .clicked()
                {
                    open_threads = true;
                }
                ui.add_space(4.0);

                if thread_chip(
                    ui,
                    "Googler vid",
                    crate::api::googler::GooglerVideoLimiter::get().active_count(),
                    googler_video_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "Googler img",
                    crate::api::googler::GooglerImageLimiter::get().active_count(),
                    googler_image_max_threads,
                ) {
                    open_threads = true;
                }
                ui.separator();
                if thread_chip(
                    ui,
                    "FFmpeg",
                    crate::api::ffmpeg::FfmpegLimiter::get().active_count(),
                    ffmpeg_max_threads,
                ) {
                    open_threads = true;
                }
                ui.separator();
                if thread_chip(
                    ui,
                    "AssemblyAI",
                    crate::api::assemblyai::AssemblyAILimiter::get().active_count(),
                    5,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "EdgeTTS",
                    crate::api::edgetts::EdgeTTSLimiter::get().active_count(),
                    edge_tts_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "Lumean",
                    crate::api::lumean::LumeanLimiter::get().active_count(),
                    5,
                ) {
                    open_threads = true;
                }
                ui.separator();
                if thread_chip(
                    ui,
                    "Gemini",
                    crate::api::gemini::GeminiLimiter::get().active_count(),
                    gemini_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "Claude",
                    crate::api::claude::ClaudeLimiter::get().active_count(),
                    claude_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "Codex",
                    crate::api::codex::CodexLimiter::get().active_count(),
                    codex_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "AGY",
                    crate::api::agy::AgyLimiter::get().active_count(),
                    agy_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "Pi",
                    crate::api::pi::PiLimiter::get().active_count(),
                    pi_max_threads,
                ) {
                    open_threads = true;
                }
                if thread_chip(
                    ui,
                    "OR",
                    crate::api::openrouter::OpenRouterLimiter::get().active_count(),
                    openrouter_max_threads,
                ) {
                    open_threads = true;
                }
            });
            if open_threads {
                *threads_window_open = true;
            }
        });
}
