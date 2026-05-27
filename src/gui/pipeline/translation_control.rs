use eframe::egui;
use crate::localization::{Language, translate};

/// Вікно контролю перекладу — перегляд/редагування тексту та запуск пайплайну далі.
#[allow(clippy::too_many_arguments)]
pub fn draw_translation_control_window(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    selected_job_control: &mut Option<u64>,
    control_text_input: &mut String,
    control_regen_extended_open: &mut bool,
    control_regen_service: &mut String,
    control_regen_model: &mut String,
    control_regen_model_openrouter: &mut String,
    control_regen_model_claude: &mut String,
    control_regen_model_gemini: &mut String,
    control_regen_model_search: &mut String,
    control_regen_prompt: &mut String,
    control_regen_temperature: &mut f32,
    control_regen_result: &std::sync::Arc<std::sync::Mutex<Option<Result<(String, Option<f64>), String>>>>,
    control_regen_loading: &std::sync::Arc<std::sync::Mutex<bool>>,
    control_regen_error: &mut Option<String>,
    control_dismissed: &mut std::collections::HashSet<u64>,
    openrouter_models: &std::sync::Arc<std::sync::Mutex<Option<Result<Vec<crate::gui::pipeline::translation::OpenRouterModel>, String>>>>,
    openrouter_models_loading: &std::sync::Arc<std::sync::Mutex<bool>>,
) {
    let job_id = match *selected_job_control {
        Some(id) => id,
        None => return,
    };

    let mut is_open = true;
    let mut should_continue = false;
    let mut is_confirmed = false;

    let Some(job_idx) = jobs.iter().position(|j| j.id == job_id) else {
        *selected_job_control = None;
        return;
    };

    let job_name = jobs[job_idx].name.clone();
    let job_save_path = jobs[job_idx].settings.save_path.clone();
    let translated_text_arc = std::sync::Arc::clone(&jobs[job_idx].translated_text);
    let translation_cost_arc = std::sync::Arc::clone(&jobs[job_idx].total_cost);
    let audio_duration_arc = std::sync::Arc::clone(&jobs[job_idx].audio_duration);
    let status_arc = std::sync::Arc::clone(&jobs[job_idx].status);
    let translation_stage_arc = std::sync::Arc::clone(&jobs[job_idx].translation_stage);
    let voiceover_stage_arc = std::sync::Arc::clone(&jobs[job_idx].voiceover_stage);
    let video_stage_arc = std::sync::Arc::clone(&jobs[job_idx].video_stage);
    let subtitles_stage_arc = std::sync::Arc::clone(&jobs[job_idx].subtitles_stage);
    let prompts_progress_arc = std::sync::Arc::clone(&jobs[job_idx].prompts_progress);
    let media_progress_arc = std::sync::Arc::clone(&jobs[job_idx].media_progress);
    let montage_stage_arc = std::sync::Arc::clone(&jobs[job_idx].montage_stage);
    let montage_progress_arc = std::sync::Arc::clone(&jobs[job_idx].montage_progress);
    let montage_file_size_arc = std::sync::Arc::clone(&jobs[job_idx].montage_file_size);
    let media_control_resume_arc = std::sync::Arc::clone(&jobs[job_idx].media_control_resume);
    let job_settings = jobs[job_idx].settings.clone();

    // Перевіряємо результат фонової перегенерації
    {
        let result = control_regen_result.lock().unwrap().take();
        if let Some(res) = result {
            match res {
                Ok((text, cost)) => {
                    *control_text_input = text;
                    if let Some(new_cost) = cost {
                        let mut existing = translation_cost_arc.lock().unwrap();
                        *existing = Some(existing.unwrap_or(0.0) + new_cost);
                    }
                    *control_regen_error = None;
                }
                Err(e) => {
                    *control_regen_error = Some(e);
                }
            }
        }
    }

    let mut control_closed = false;
    let mut trigger_simple_regen = false;
    let mut open_extended = false;

    egui::Window::new(format!("{} — {}", translate(language, "control_window_title"), job_name))
        .open(&mut is_open)
        .resizable(true)
        .default_size([500.0, 350.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(translate(language, "control_window_text")).strong().size(12.0));
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(control_text_input)
                                .hint_text("Перекладений текст...")
                                .desired_width(f32::INFINITY)
                                .desired_rows(10),
                        );
                    });

                ui.add_space(4.0);

                let translated_char_count = control_text_input.chars().count();
                let translated_word_count = control_text_input.split_whitespace().count();
                let translated_token_count = crate::gui::editor::count_tokens(control_text_input);
                let cost_snapshot = *translation_cost_arc.lock().unwrap();

                let text_color = ui.visuals().widgets.noninteractive.text_color();
                let accent_color = ui.visuals().selection.bg_fill;
                let bullet_color = text_color.linear_multiply(0.3);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(translate(language, "stats_chars")).size(12.0).color(text_color));
                    ui.label(egui::RichText::new(format!(" {}", translated_char_count)).size(12.0).strong().color(accent_color));
                    ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                    ui.label(egui::RichText::new(translate(language, "stats_words")).size(12.0).color(text_color));
                    ui.label(egui::RichText::new(format!(" {}", translated_word_count)).size(12.0).strong().color(accent_color));
                    ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                    ui.label(egui::RichText::new(translate(language, "stats_tokens")).size(12.0).color(text_color));
                    ui.label(egui::RichText::new(format!(" {}", translated_token_count)).size(12.0).strong().color(accent_color));
                    if let Some(cost) = cost_snapshot {
                        ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                        ui.label(egui::RichText::new(translate(language, "control_window_cost")).size(12.0).color(text_color));
                        ui.label(egui::RichText::new(format!(" ${:.5}", cost)).size(12.0).strong().color(accent_color));
                    }
                });

                if let Some(ref err) = *control_regen_error {
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("{} {}", translate(language, "control_regen_error"), err))
                            .color(egui::Color32::from_rgb(231, 76, 60))
                            .size(11.0),
                    ).wrap());
                }

                ui.add_space(8.0);

                let is_regen_loading = *control_regen_loading.lock().unwrap();

                ui.horizontal(|ui| {
                    if ui.button(translate(language, "job_name_cancel_btn")).clicked() {
                        control_closed = true;
                    }

                    if ui.add_enabled(
                        !is_regen_loading,
                        egui::Button::new(translate(language, "control_regen_btn")),
                    ).clicked() {
                        trigger_simple_regen = true;
                    }

                    if ui.add_enabled(
                        !is_regen_loading,
                        egui::Button::new(translate(language, "control_regen_extended_btn")),
                    ).clicked() {
                        open_extended = true;
                    }

                    if is_regen_loading {
                        ui.label(
                            egui::RichText::new(translate(language, "control_regen_loading"))
                                .weak()
                                .size(11.0),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new(translate(language, "control_window_continue_btn")).strong()
                            )
                        ).clicked() {
                            should_continue = true;
                        }
                    });
                });
            });
        });

    if control_closed {
        is_open = false;
        is_confirmed = true;
    }

    // Запускаємо просту перегенерацію з оригінальними налаштуваннями задачі
    if trigger_simple_regen {
        let result_arc = std::sync::Arc::clone(control_regen_result);
        let loading_arc = std::sync::Arc::clone(control_regen_loading);
        let ctx_clone = ctx.clone();
        let text = job_settings.text.clone();
        let service = job_settings.translation_service.clone();
        let model = job_settings.translation_model.clone();
        let prompt = job_settings.translation_prompt.clone();
        let temperature = job_settings.translation_temperature;
        let key = job_settings.openrouter_key.clone();
        let job_info = Some((job_id, job_name.clone()));

        *control_regen_error = None;
        *loading_arc.lock().unwrap() = true;
        std::thread::spawn(move || {
            let result = crate::core::llm::call_llm(
                &service, &key, &model, &prompt, &text, temperature, job_info,
            );
            *result_arc.lock().unwrap() = Some(result);
            *loading_arc.lock().unwrap() = false;
            ctx_clone.request_repaint();
        });
    }

    // Ініціалізуємо налаштування розширеної перегенерації при першому відкритті
    if open_extended && !*control_regen_extended_open {
        *control_regen_service = job_settings.translation_service.clone();
        *control_regen_model = job_settings.translation_model.clone();
        *control_regen_model_openrouter = if job_settings.translation_service == "OpenRouter" {
            job_settings.translation_model.clone()
        } else {
            control_regen_model_openrouter.clone()
        };
        *control_regen_model_claude = if job_settings.translation_service == "Claude Code" {
            if job_settings.translation_model.is_empty() { "sonnet".to_string() } else { job_settings.translation_model.clone() }
        } else {
            control_regen_model_claude.clone()
        };
        *control_regen_model_gemini = if job_settings.translation_service == "Gemini CLI" {
            if job_settings.translation_model.is_empty() { "gemini-2.5-flash".to_string() } else { job_settings.translation_model.clone() }
        } else {
            control_regen_model_gemini.clone()
        };
        *control_regen_prompt = job_settings.translation_prompt.clone();
        *control_regen_temperature = job_settings.translation_temperature;
        control_regen_model_search.clear();
        *control_regen_extended_open = true;
    }

    // Вікно розширеної перегенерації з одноразовими налаштуваннями
    if *control_regen_extended_open {
        let text_to_translate = job_settings.text.clone();
        let openrouter_key_ext = job_settings.openrouter_key.clone();
        let job_info_ext = Some((job_id, job_name.clone()));

        let mut ext_is_open = true;
        let mut trigger_ext_regen = false;

        egui::Window::new(translate(language, "control_regen_extended_title"))
            .open(&mut ext_is_open)
            .resizable(true)
            .default_size([450.0, 500.0])
            .show(ctx, |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(translate(language, "control_regen_settings_note"))
                        .weak()
                        .size(11.0),
                ).wrap());
                ui.add_space(6.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.push_id("control_regen_translation", |ui| {
                        crate::gui::pipeline::translation::draw_translation_section(
                            ui,
                            language,
                            control_regen_prompt,
                            control_regen_model,
                            control_regen_model_search,
                            openrouter_models,
                            openrouter_models_loading,
                            control_regen_temperature,
                            control_regen_service,
                            control_regen_model_openrouter,
                            control_regen_model_claude,
                            control_regen_model_gemini,
                        );
                    });
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                let is_regen_loading = *control_regen_loading.lock().unwrap();

                if is_regen_loading {
                    ui.label(
                        egui::RichText::new(translate(language, "control_regen_loading"))
                            .weak(),
                    );
                } else if ui.button(translate(language, "control_regen_run_btn")).clicked() {
                    trigger_ext_regen = true;
                }
            });

        if !ext_is_open {
            *control_regen_extended_open = false;
        }

        if trigger_ext_regen {
            let result_arc = std::sync::Arc::clone(control_regen_result);
            let loading_arc = std::sync::Arc::clone(control_regen_loading);
            let ctx_clone = ctx.clone();
            let service = control_regen_service.clone();
            let model = control_regen_model.clone();
            let prompt = control_regen_prompt.clone();
            let temperature = *control_regen_temperature;

            *control_regen_error = None;
            *loading_arc.lock().unwrap() = true;
            std::thread::spawn(move || {
                let result = crate::core::llm::call_llm(
                    &service, &openrouter_key_ext, &model, &prompt, &text_to_translate, temperature, job_info_ext,
                );
                *result_arc.lock().unwrap() = Some(result);
                *loading_arc.lock().unwrap() = false;
                ctx_clone.request_repaint();
            });
        }
    }

    if should_continue {
        // Оновлюємо перекладений текст у задачі та зберігаємо на диск
        *translated_text_arc.lock().unwrap() = Some(control_text_input.clone());
        let dir = std::path::Path::new(&job_save_path);
        let _ = std::fs::write(dir.join("text.txt"), control_text_input.as_str());

        // Змінюємо статус і запускаємо пайплайн знову
        *status_arc.lock().unwrap() = crate::queue::JobStatus::Running;

        let ctx_clone = ctx.clone();
        crate::core::pipeline::run_pipeline(
            job_id,
            job_name,
            job_settings,
            status_arc,
            translation_stage_arc,
            voiceover_stage_arc,
            video_stage_arc,
            subtitles_stage_arc,
            montage_stage_arc,
            translated_text_arc,
            translation_cost_arc,
            audio_duration_arc,
            prompts_progress_arc,
            media_progress_arc,
            montage_progress_arc,
            montage_file_size_arc,
            media_control_resume_arc,
            ctx_clone,
        );

        is_open = false;
    }

    if !is_open {
        if !is_confirmed {
            control_dismissed.insert(job_id);
        }
        *selected_job_control = None;
        control_text_input.clear();
        *control_regen_extended_open = false;
        *control_regen_error = None;
    }
}
