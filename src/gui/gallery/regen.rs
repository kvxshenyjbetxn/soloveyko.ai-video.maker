use crate::localization::{Language, translate};
use eframe::egui;

/// Вікно кастомної перегенерації медіафайлу з галереї.
pub fn draw_media_regen_window(
    ctx: &egui::Context,
    language: Language,
    media_regen_window_open: &mut bool,
    media_regen_target: &Option<std::path::PathBuf>,
    media_regen_batch_targets: &mut Vec<std::path::PathBuf>,
    media_regen_media_type: &mut String,
    media_regen_image_priority: &mut Vec<String>,
    media_regen_video_priority: &mut Vec<String>,
    media_regen_prompt: &mut String,
    media_regen_loading: &std::sync::Arc<std::sync::Mutex<bool>>,
    media_regen_base_settings: &Option<crate::queue::JobSettings>,
    media_regen_error: &mut Option<String>,
    media_regen_job_id: u64,
    media_regen_job_name: &str,
    gallery_textures: &mut std::collections::HashMap<
        std::path::PathBuf,
        Option<egui::TextureHandle>,
    >,
    media_regen_result: &std::sync::Arc<std::sync::Mutex<Option<Result<(), String>>>>,
    media_regen_paths: &std::sync::Arc<
        std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>,
    >,
    media_regen_results_queue: &std::sync::Arc<
        std::sync::Mutex<Vec<(std::path::PathBuf, Result<(), String>)>>,
    >,
) {
    if !*media_regen_window_open {
        return;
    }

    let is_batch_mode = !media_regen_batch_targets.is_empty();
    let file_name = media_regen_target
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let title = if is_batch_mode {
        format!(
            "{}: {}",
            translate(language, "gallery_regen_batch_window_title"),
            media_regen_batch_targets.len()
        )
    } else {
        format!(
            "{}: {}",
            translate(language, "gallery_regen_window_title"),
            file_name
        )
    };
    let mut is_open = true;
    let mut should_close = false;

    egui::Window::new(title)
        .open(&mut is_open)
        .resizable(true)
        .vscroll(true)
        .default_width(380.0)
        .collapsible(false)
        .show(ctx, |ui| {
            use crate::gui::pipeline::video::{
                arrow_button, image_provider_info, video_provider_info,
            };

            ui.label(
                egui::RichText::new(translate(language, "gallery_regen_media_type_label")).strong(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.radio_value(
                    media_regen_media_type,
                    "image".to_string(),
                    translate(language, "video_media_type_image"),
                );
                ui.radio_value(
                    media_regen_media_type,
                    "video".to_string(),
                    translate(language, "video_media_type_video"),
                );
            });

            ui.add_space(8.0);

            if *media_regen_media_type == "video" {
                ui.label(
                    egui::RichText::new(translate(language, "gallery_regen_priority_video_label"))
                        .strong(),
                );
                ui.add_space(4.0);
                let mut swap: Option<(usize, usize)> = None;
                for i in 0..media_regen_video_priority.len() {
                    let (name, credits) = video_provider_info(&media_regen_video_priority[i]);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("#{}", i + 1))
                                .weak()
                                .monospace(),
                        );
                        ui.add_space(4.0);
                        if arrow_button(ui, true, i > 0).clicked() {
                            swap = Some((i - 1, i));
                        }
                        if arrow_button(ui, false, i < media_regen_video_priority.len() - 1)
                            .clicked()
                        {
                            swap = Some((i, i + 1));
                        }
                        ui.add_space(4.0);
                        ui.label(name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(credits).weak().size(11.0));
                        });
                    });
                }
                if let Some((a, b)) = swap {
                    media_regen_video_priority.swap(a, b);
                }
            } else {
                ui.label(
                    egui::RichText::new(translate(language, "gallery_regen_priority_image_label"))
                        .strong(),
                );
                ui.add_space(4.0);
                let mut swap: Option<(usize, usize)> = None;
                for i in 0..media_regen_image_priority.len() {
                    let (name, credits) = image_provider_info(&media_regen_image_priority[i]);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("#{}", i + 1))
                                .weak()
                                .monospace(),
                        );
                        ui.add_space(4.0);
                        if arrow_button(ui, true, i > 0).clicked() {
                            swap = Some((i - 1, i));
                        }
                        if arrow_button(ui, false, i < media_regen_image_priority.len() - 1)
                            .clicked()
                        {
                            swap = Some((i, i + 1));
                        }
                        ui.add_space(4.0);
                        ui.label(name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(credits).weak().size(11.0));
                        });
                    });
                }
                if let Some((a, b)) = swap {
                    media_regen_image_priority.swap(a, b);
                }
            }

            ui.add_space(8.0);
            if is_batch_mode {
                ui.label(
                    egui::RichText::new(translate(language, "gallery_regen_batch_hint")).weak(),
                );
            } else {
                ui.label(
                    egui::RichText::new(translate(language, "gallery_regen_prompt_label")).strong(),
                );
                ui.add_space(4.0);
                ui.add_sized(
                    [ui.available_width(), 80.0],
                    egui::TextEdit::multiline(media_regen_prompt),
                );
            }

            ui.add_space(8.0);

            let is_loading = if is_batch_mode {
                let loading_paths = media_regen_paths.lock().unwrap();
                media_regen_batch_targets.iter().any(|path| {
                    loading_paths.contains(path)
                        || loading_paths.contains(&path.with_extension("mp4"))
                })
            } else {
                media_regen_target
                    .as_ref()
                    .map(|path| {
                        let loading_paths = media_regen_paths.lock().unwrap();
                        loading_paths.contains(path)
                            || loading_paths.contains(&path.with_extension("mp4"))
                    })
                    .unwrap_or(false)
            };
            ui.add_enabled_ui(!is_loading, |ui| {
                ui.horizontal(|ui| {
                    let start_label = if is_batch_mode {
                        translate(language, "gallery_regen_batch_start_btn")
                    } else {
                        translate(language, "gallery_regen_start_btn")
                    };
                    if ui.button(start_label).clicked() {
                        if let Some(base) = media_regen_base_settings.as_ref() {
                            let priority = if *media_regen_media_type == "video" {
                                media_regen_video_priority.clone()
                            } else {
                                media_regen_image_priority.clone()
                            };
                            *media_regen_error = None;

                            if is_batch_mode {
                                let files = media_regen_batch_targets.clone();
                                for file in &files {
                                    gallery_textures.remove(file);
                                }
                                crate::core::pipeline::regenerate_multiple_media(
                                    files,
                                    media_regen_media_type.clone(),
                                    priority,
                                    base.googler_key.clone(),
                                    media_regen_job_id,
                                    media_regen_job_name.to_string(),
                                    ctx.clone(),
                                    Some(std::sync::Arc::clone(media_regen_paths)),
                                    Some(std::sync::Arc::clone(media_regen_results_queue)),
                                    base.googler_video_upscale_enabled,
                                    base.googler_video_upscale_resolution.clone(),
                                    base.googler_video_upscale_quality.clone(),
                                );
                                should_close = true;
                            } else if let Some(file) = media_regen_target.clone() {
                                let custom_prompt = if media_regen_prompt.trim().is_empty() {
                                    None
                                } else {
                                    Some(media_regen_prompt.clone())
                                };
                                gallery_textures.remove(&file);
                                crate::core::pipeline::regenerate_single_media(
                                    file,
                                    media_regen_media_type.clone(),
                                    priority,
                                    base.googler_key.clone(),
                                    custom_prompt,
                                    media_regen_job_id,
                                    media_regen_job_name.to_string(),
                                    ctx.clone(),
                                    std::sync::Arc::clone(media_regen_result),
                                    std::sync::Arc::clone(media_regen_loading),
                                    Some(std::sync::Arc::clone(media_regen_paths)),
                                    Some(std::sync::Arc::clone(media_regen_results_queue)),
                                    base.googler_video_upscale_enabled,
                                    base.googler_video_upscale_resolution.clone(),
                                    base.googler_video_upscale_quality.clone(),
                                );
                                should_close = true;
                            }
                        }
                    }
                    if is_loading {
                        ui.spinner();
                        ui.label(translate(language, "gallery_regen_loading"));
                    }
                });
            });

            if let Some(ref err) = *media_regen_error {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), err);
            }
        });

    if !is_open || should_close {
        media_regen_batch_targets.clear();
    }
    *media_regen_window_open = is_open && !should_close;
}
