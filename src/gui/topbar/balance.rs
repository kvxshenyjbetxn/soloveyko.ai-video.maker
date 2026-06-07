use eframe::egui;
use crate::localization::{Language, translate};
use super::thread_load_color;

/// Вікно балансів сервісів (OpenRouter, VoiceBot, Googler).
pub fn draw_balance_window(
    ctx: &egui::Context,
    open: &mut bool,
    language: Language,
    openrouter_key: &str,
    openrouter_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    voicebot_key: &str,
    voicebot_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    googler_key: &str,
    googler_balance: &std::sync::Arc<std::sync::Mutex<Option<crate::api::googler::GooglerBalance>>>,
) {
    use std::sync::Arc;

    egui::Window::new(translate(language, "balance_window_title"))
        .open(open)
        .resizable(false)
        .vscroll(true)
        .collapsible(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            // --- OpenRouter ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OpenRouter").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !openrouter_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::openrouter::fetch_balance(
                                openrouter_key.to_string(),
                                Arc::clone(openrouter_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = openrouter_balance.try_lock() {
                    match guard.as_ref() {
                        Some(text) => { ui.label(text.as_str()); }
                        None if openrouter_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // --- VoiceBot ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("VoiceBot").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !voicebot_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::voicebot::fetch_balance(
                                voicebot_key.to_string(),
                                Arc::clone(voicebot_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = voicebot_balance.try_lock() {
                    match guard.as_ref() {
                        Some(text) => { ui.label(text.as_str()); }
                        None if voicebot_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // --- Googler ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Googler").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !googler_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::googler::fetch_balance(
                                googler_key.to_string(),
                                Arc::clone(googler_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = googler_balance.try_lock() {
                    match guard.as_ref() {
                        Some(bal) => {
                            egui::Grid::new("googler_balance_grid")
                                .num_columns(2)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(translate(language, "balance_img_per_hour"));
                                    ui.label(format!("{} / {}", bal.img_used, bal.img_limit));
                                    ui.end_row();
                                    ui.label(translate(language, "balance_video_per_hour"));
                                    ui.label(format!("{} / {}", bal.video_used, bal.video_limit));
                                    ui.end_row();
                                });
                        }
                        None if googler_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                        }
                    }
                }
            });

            ui.add_space(6.0);

            // Кнопка "Оновити всі"
            ui.vertical_centered(|ui| {
                if ui.button(translate(language, "balance_refresh_all")).clicked() {
                    let ctx2 = ui.ctx().clone();
                    if !openrouter_key.is_empty() {
                        crate::api::openrouter::fetch_balance(
                            openrouter_key.to_string(),
                            Arc::clone(openrouter_balance),
                            ctx2.clone(),
                        );
                    }
                    if !voicebot_key.is_empty() {
                        crate::api::voicebot::fetch_balance(
                            voicebot_key.to_string(),
                            Arc::clone(voicebot_balance),
                            ctx2.clone(),
                        );
                    }
                    if !googler_key.is_empty() {
                        crate::api::googler::fetch_balance(
                            googler_key.to_string(),
                            Arc::clone(googler_balance),
                            ctx2,
                        );
                    }
                }
            });
        });
}

/// Вікно потоків сервісів (ліміти і активні потоки).
pub fn draw_threads_window(
    ctx: &egui::Context,
    open: &mut bool,
    language: Language,
    openrouter_max_threads: &mut usize,
    claude_max_threads: &mut usize,
    gemini_max_threads: &mut usize,
    codex_max_threads: &mut usize,
    voicebot_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    edge_tts_max_threads: &mut usize,
    googler_image_max_threads: &mut usize,
    googler_video_max_threads: &mut usize,
    ffmpeg_max_threads: &mut usize,
) {
    egui::Window::new(translate(language, "threads_window_title"))
        .open(open)
        .resizable(false)
        .vscroll(true)
        .collapsible(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            let active_label = |ui: &mut egui::Ui, active: usize, max: usize| {
                let color = thread_load_color(active, max, ui.visuals().weak_text_color());
                ui.label(egui::RichText::new(format!("{} / {}", active, max)).color(color));
            };

            // --- OpenRouter ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("OpenRouter").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::openrouter::OpenRouterLimiter::get().active_count(), *openrouter_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_openrouter_threads"));
                    let mut val = *openrouter_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=25)).changed() {
                        *openrouter_max_threads = val;
                        crate::api::openrouter::OpenRouterLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Claude Code ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("Claude Code").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::claude::ClaudeLimiter::get().active_count(), *claude_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_claude_threads"));
                    let mut val = *claude_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=25)).changed() {
                        *claude_max_threads = val;
                        crate::api::claude::ClaudeLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Gemini CLI ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("Gemini CLI").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::gemini::GeminiLimiter::get().active_count(), *gemini_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_gemini_threads"));
                    let mut val = *gemini_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=25)).changed() {
                        *gemini_max_threads = val;
                        crate::api::gemini::GeminiLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Codex CLI ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("Codex CLI").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::codex::CodexLimiter::get().active_count(), *codex_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_codex_threads"));
                    let mut val = *codex_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=25)).changed() {
                        *codex_max_threads = val;
                        crate::api::codex::CodexLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- VoiceBot ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("VoiceBot").strong());
                ui.separator();
                ui.add_space(4.0);
                let _ = voicebot_balance;
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    let active = crate::api::voicebot::VoiceBotLimiter::get().active_count();
                    active_label(ui, active, 5);
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(translate(language, "balance_voicebot_limit")).weak());
            });

            ui.add_space(4.0);

            // --- AssemblyAI ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("AssemblyAI").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    let active = crate::api::assemblyai::AssemblyAILimiter::get().active_count();
                    active_label(ui, active, 5);
                });
                ui.add_space(4.0);
                ui.label(egui::RichText::new(translate(language, "balance_assemblyai_limit")).weak());
            });

            ui.add_space(4.0);

            // --- Edge TTS ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("Edge TTS").strong());
                ui.separator();
                ui.label(egui::RichText::new(translate(language, "balance_edge_tts_status")).weak());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::edgetts::EdgeTTSLimiter::get().active_count(), *edge_tts_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_edge_tts_threads"));
                    let mut val = *edge_tts_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=25)).changed() {
                        *edge_tts_max_threads = val;
                        crate::api::edgetts::EdgeTTSLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Googler ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("Googler").strong());
                ui.separator();
                ui.add_space(4.0);
                let img_active = crate::api::googler::GooglerImageLimiter::get().active_count();
                let vid_active = crate::api::googler::GooglerVideoLimiter::get().active_count();
                egui::Grid::new("googler_threads_grid")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(translate(language, "balance_img_threads"));
                        ui.horizontal(|ui| {
                            let color = thread_load_color(img_active, *googler_image_max_threads, ui.visuals().weak_text_color());
                            ui.label(egui::RichText::new(format!("{} /", img_active)).color(color));
                            let mut val = *googler_image_max_threads;
                            if ui.add(egui::Slider::new(&mut val, 5..=25)).changed() {
                                *googler_image_max_threads = val;
                                crate::api::googler::GooglerImageLimiter::get().set_max_threads(val);
                            }
                        });
                        ui.end_row();
                        ui.label(translate(language, "balance_video_threads"));
                        ui.horizontal(|ui| {
                            let color = thread_load_color(vid_active, *googler_video_max_threads, ui.visuals().weak_text_color());
                            ui.label(egui::RichText::new(format!("{} /", vid_active)).color(color));
                            let mut val = *googler_video_max_threads;
                            if ui.add(egui::Slider::new(&mut val, 5..=25)).changed() {
                                *googler_video_max_threads = val;
                                crate::api::googler::GooglerVideoLimiter::get().set_max_threads(val);
                            }
                        });
                        ui.end_row();
                    });
            });

            ui.add_space(4.0);

            // --- FFmpeg ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(egui::RichText::new("FFmpeg").strong());
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "balance_active_threads"));
                    active_label(ui, crate::api::ffmpeg::FfmpegLimiter::get().active_count(), *ffmpeg_max_threads);
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_ffmpeg_threads"));
                    let mut val = *ffmpeg_max_threads;
                    if ui.add(egui::Slider::new(&mut val, 1..=8)).changed() {
                        *ffmpeg_max_threads = val;
                        crate::api::ffmpeg::FfmpegLimiter::get().set_max_threads(val);
                    }
                });
            });
        });
}
