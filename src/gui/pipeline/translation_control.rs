use crate::localization::{translate, Language};
use eframe::egui;

/// Стан одного вікна контролю перекладу (зберігається per-job).
pub struct TranslationControlWindowState {
    pub text_input: String,
    pub regen_extended_open: bool,
    pub regen_service: String,
    pub regen_model: String,
    pub regen_model_openrouter: String,
    pub regen_model_claude: String,
    pub regen_model_gemini: String,
    pub regen_model_codex: String,
    pub regen_model_agy: String,
    pub regen_model_pi: String,
    pub regen_model_search: String,
    pub regen_prompt: String,
    pub regen_temperature: f32,
    pub regen_result:
        std::sync::Arc<std::sync::Mutex<Option<Result<(String, Option<f64>), String>>>>,
    pub regen_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    pub regen_error: Option<String>,
}

impl TranslationControlWindowState {
    pub fn new_with_text(text: String) -> Self {
        Self {
            text_input: text,
            regen_extended_open: false,
            regen_service: String::new(),
            regen_model: String::new(),
            regen_model_openrouter: String::new(),
            regen_model_claude: "sonnet".to_string(),
            regen_model_gemini: "gemini-2.5-flash".to_string(),
            regen_model_codex: "gpt-5.4-mini".to_string(),
            regen_model_agy: "gemini-3.5-flash".to_string(),
            regen_model_pi: "gemini-2.5-flash".to_string(),
            regen_model_search: String::new(),
            regen_prompt: String::new(),
            regen_temperature: 0.7,
            regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            regen_error: None,
        }
    }
}

/// Малює всі відкриті вікна контролю перекладу. Закриті вікна видаляються з мапи.
#[allow(clippy::too_many_arguments)]
pub fn draw_translation_control_windows(
    ctx: &egui::Context,
    language: Language,
    jobs: &[crate::queue::PipelineJob],
    open_job_controls: &mut std::collections::HashMap<u64, TranslationControlWindowState>,
    control_dismissed: &mut std::collections::HashSet<u64>,
    openrouter_models: &std::sync::Arc<
        std::sync::Mutex<
            Option<Result<Vec<crate::gui::pipeline::translation::OpenRouterModel>, String>>,
        >,
    >,
    openrouter_models_loading: &std::sync::Arc<std::sync::Mutex<bool>>,
) {
    let job_ids: Vec<u64> = open_job_controls.keys().cloned().collect();
    let mut to_remove = Vec::new();

    for job_id in job_ids {
        let Some(job_idx) = jobs.iter().position(|j| j.id == job_id) else {
            to_remove.push(job_id);
            continue;
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
        let montage_control_resume_arc =
            std::sync::Arc::clone(&jobs[job_idx].montage_control_resume);
        let agent_control_resume_arc = std::sync::Arc::clone(&jobs[job_idx].agent_control_resume);
        let capcut_mode_override_arc = std::sync::Arc::clone(&jobs[job_idx].capcut_mode_override);
        let agent_chat_arc = std::sync::Arc::clone(&jobs[job_idx].agent_chat);
        let agent_session_arc = std::sync::Arc::clone(&jobs[job_idx].agent_session);
        let job_settings = jobs[job_idx].settings.clone();

        let state = open_job_controls.get_mut(&job_id).unwrap();

        // Перевіряємо результат фонової перегенерації
        {
            let result = state.regen_result.lock().unwrap().take();
            if let Some(res) = result {
                match res {
                    Ok((text, cost)) => {
                        state.text_input = text;
                        if let Some(new_cost) = cost {
                            let mut existing = translation_cost_arc.lock().unwrap();
                            *existing = Some(existing.unwrap_or(0.0) + new_cost);
                        }
                        state.regen_error = None;
                    }
                    Err(e) => {
                        state.regen_error = Some(e);
                    }
                }
            }
        }

        let mut is_open = true;
        let mut should_continue = false;
        let mut control_closed = false;
        let mut trigger_simple_regen = false;
        let mut open_extended = false;

        egui::Window::new(format!(
            "{} — {}",
            translate(language, "control_window_title"),
            job_name
        ))
        .id(egui::Id::new(("translation_control", job_id)))
        .open(&mut is_open)
        .resizable(true)
        .default_size([500.0, 350.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(translate(language, "control_window_text"))
                        .strong()
                        .size(12.0),
                );
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut state.text_input)
                                .hint_text("Перекладений текст...")
                                .desired_width(f32::INFINITY)
                                .desired_rows(10),
                        );
                    });

                ui.add_space(4.0);

                let translated_char_count = state.text_input.chars().count();
                let translated_word_count = state.text_input.split_whitespace().count();
                let translated_token_count = crate::gui::editor::count_tokens(&state.text_input);
                let cost_snapshot = *translation_cost_arc.lock().unwrap();

                let text_color = ui.visuals().widgets.noninteractive.text_color();
                let accent_color = ui.visuals().selection.bg_fill;
                let bullet_color = text_color.linear_multiply(0.3);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(translate(language, "stats_chars"))
                            .size(12.0)
                            .color(text_color),
                    );
                    ui.label(
                        egui::RichText::new(format!(" {}", translated_char_count))
                            .size(12.0)
                            .strong()
                            .color(accent_color),
                    );
                    ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                    ui.label(
                        egui::RichText::new(translate(language, "stats_words"))
                            .size(12.0)
                            .color(text_color),
                    );
                    ui.label(
                        egui::RichText::new(format!(" {}", translated_word_count))
                            .size(12.0)
                            .strong()
                            .color(accent_color),
                    );
                    ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                    ui.label(
                        egui::RichText::new(translate(language, "stats_tokens"))
                            .size(12.0)
                            .color(text_color),
                    );
                    ui.label(
                        egui::RichText::new(format!(" {}", translated_token_count))
                            .size(12.0)
                            .strong()
                            .color(accent_color),
                    );
                    if let Some(cost) = cost_snapshot {
                        ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                        ui.label(
                            egui::RichText::new(translate(language, "control_window_cost"))
                                .size(12.0)
                                .color(text_color),
                        );
                        ui.label(
                            egui::RichText::new(format!(" ${:.5}", cost))
                                .size(12.0)
                                .strong()
                                .color(accent_color),
                        );
                    }
                });

                if let Some(ref err) = state.regen_error {
                    ui.add_space(4.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "{} {}",
                                translate(language, "control_regen_error"),
                                err
                            ))
                            .color(egui::Color32::from_rgb(231, 76, 60))
                            .size(11.0),
                        )
                        .wrap(),
                    );
                }

                ui.add_space(8.0);

                let is_regen_loading = *state.regen_loading.lock().unwrap();

                ui.horizontal(|ui| {
                    if ui
                        .button(translate(language, "job_name_cancel_btn"))
                        .clicked()
                    {
                        control_closed = true;
                    }

                    if ui
                        .add_enabled(
                            !is_regen_loading,
                            egui::Button::new(translate(language, "control_regen_btn")),
                        )
                        .clicked()
                    {
                        trigger_simple_regen = true;
                    }

                    if ui
                        .add_enabled(
                            !is_regen_loading,
                            egui::Button::new(translate(language, "control_regen_extended_btn")),
                        )
                        .clicked()
                    {
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
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new(translate(
                                    language,
                                    "control_window_continue_btn",
                                ))
                                .strong(),
                            ))
                            .clicked()
                        {
                            should_continue = true;
                        }
                    });
                });
            });
        });

        if control_closed {
            is_open = false;
        }

        // Проста перегенерація з оригінальними налаштуваннями задачі
        if trigger_simple_regen {
            let result_arc = std::sync::Arc::clone(&state.regen_result);
            let loading_arc = std::sync::Arc::clone(&state.regen_loading);
            let ctx_clone = ctx.clone();
            let text = job_settings.text.clone();
            let service = job_settings.translation_service.clone();
            let model = job_settings.translation_model.clone();
            let prompt = job_settings.translation_prompt.clone();
            let temperature = job_settings.translation_temperature;
            let key = job_settings.openrouter_key.clone();
            let job_info = Some((job_id, job_name.clone()));

            state.regen_error = None;
            *loading_arc.lock().unwrap() = true;
            let save_path_for_regen = job_settings.save_path.clone();
            std::thread::spawn(move || {
                let result = crate::core::llm::call_llm(
                    &service,
                    &key,
                    &model,
                    &prompt,
                    &text,
                    temperature,
                    job_info,
                    Some(save_path_for_regen.as_str()),
                    false,
                );
                *result_arc.lock().unwrap() = Some(result);
                *loading_arc.lock().unwrap() = false;
                ctx_clone.request_repaint();
            });
        }

        // Ініціалізуємо налаштування розширеної перегенерації при першому відкритті
        if open_extended && !state.regen_extended_open {
            state.regen_service = job_settings.translation_service.clone();
            state.regen_model = job_settings.translation_model.clone();
            state.regen_model_openrouter = if job_settings.translation_service == "OpenRouter" {
                job_settings.translation_model.clone()
            } else {
                state.regen_model_openrouter.clone()
            };
            state.regen_model_claude = if job_settings.translation_service == "Claude Code" {
                if job_settings.translation_model.is_empty() {
                    "sonnet".to_string()
                } else {
                    job_settings.translation_model.clone()
                }
            } else {
                state.regen_model_claude.clone()
            };
            state.regen_model_gemini = if job_settings.translation_service == "Gemini CLI" {
                if job_settings.translation_model.is_empty() {
                    "gemini-2.5-flash".to_string()
                } else {
                    job_settings.translation_model.clone()
                }
            } else {
                state.regen_model_gemini.clone()
            };
            state.regen_model_codex = if job_settings.translation_service == "Codex CLI" {
                if job_settings.translation_model.is_empty() {
                    "gpt-5.4-mini".to_string()
                } else {
                    job_settings.translation_model.clone()
                }
            } else {
                state.regen_model_codex.clone()
            };
            state.regen_model_agy = if job_settings.translation_service == "AGY CLI" {
                if job_settings.translation_model.is_empty() {
                    "gemini-3.5-flash".to_string()
                } else {
                    job_settings.translation_model.clone()
                }
            } else {
                state.regen_model_agy.clone()
            };
            state.regen_model_pi = if job_settings.translation_service == "Pi CLI" {
                if job_settings.translation_model.is_empty() {
                    "gemini-2.5-flash".to_string()
                } else {
                    job_settings.translation_model.clone()
                }
            } else {
                state.regen_model_pi.clone()
            };
            state.regen_prompt = job_settings.translation_prompt.clone();
            state.regen_temperature = job_settings.translation_temperature;
            state.regen_model_search.clear();
            state.regen_extended_open = true;
        }

        // Вікно розширеної перегенерації
        if state.regen_extended_open {
            let text_to_translate = job_settings.text.clone();
            let openrouter_key_ext = job_settings.openrouter_key.clone();
            let job_info_ext = Some((job_id, job_name.clone()));

            let mut ext_is_open = true;
            let mut trigger_ext_regen = false;

            egui::Window::new(translate(language, "control_regen_extended_title"))
                .id(egui::Id::new(("translation_control_ext", job_id)))
                .open(&mut ext_is_open)
                .resizable(true)
                .default_size([450.0, 500.0])
                .show(ctx, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(translate(language, "control_regen_settings_note"))
                                .weak()
                                .size(11.0),
                        )
                        .wrap(),
                    );
                    ui.add_space(6.0);
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.push_id(("control_regen_translation", job_id), |ui| {
                            crate::gui::pipeline::translation::draw_translation_section(
                                ui,
                                language,
                                &mut state.regen_prompt,
                                &mut state.regen_model,
                                &mut state.regen_model_search,
                                openrouter_models,
                                openrouter_models_loading,
                                &mut state.regen_temperature,
                                &mut state.regen_service,
                                &mut state.regen_model_openrouter,
                                &mut state.regen_model_claude,
                                &mut state.regen_model_gemini,
                                &mut state.regen_model_codex,
                                &mut state.regen_model_agy,
                                &mut state.regen_model_pi,
                            );
                        });
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let is_regen_loading = *state.regen_loading.lock().unwrap();

                    if is_regen_loading {
                        ui.label(
                            egui::RichText::new(translate(language, "control_regen_loading"))
                                .weak(),
                        );
                    } else if ui
                        .button(translate(language, "control_regen_run_btn"))
                        .clicked()
                    {
                        trigger_ext_regen = true;
                    }
                });

            if !ext_is_open {
                state.regen_extended_open = false;
            }

            if trigger_ext_regen {
                let result_arc = std::sync::Arc::clone(&state.regen_result);
                let loading_arc = std::sync::Arc::clone(&state.regen_loading);
                let ctx_clone = ctx.clone();
                let service = state.regen_service.clone();
                let model = state.regen_model.clone();
                let prompt = state.regen_prompt.clone();
                let temperature = state.regen_temperature;
                let save_path_ext = job_save_path.clone();

                state.regen_error = None;
                *loading_arc.lock().unwrap() = true;
                std::thread::spawn(move || {
                    let result = crate::core::llm::call_llm(
                        &service,
                        &openrouter_key_ext,
                        &model,
                        &prompt,
                        &text_to_translate,
                        temperature,
                        job_info_ext,
                        Some(save_path_ext.as_str()),
                        false,
                    );
                    *result_arc.lock().unwrap() = Some(result);
                    *loading_arc.lock().unwrap() = false;
                    ctx_clone.request_repaint();
                });
            }
        }

        if should_continue {
            // Оновлюємо перекладений текст у задачі та зберігаємо на диск
            *translated_text_arc.lock().unwrap() = Some(state.text_input.clone());
            let dir = std::path::Path::new(&job_save_path);
            let _ = std::fs::write(dir.join("text.txt"), state.text_input.as_str());

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
                montage_control_resume_arc,
                agent_control_resume_arc,
                agent_chat_arc,
                agent_session_arc,
                capcut_mode_override_arc,
                ctx_clone,
            );

            is_open = false;
        }

        if !is_open {
            if !should_continue {
                control_dismissed.insert(job_id);
            }
            to_remove.push(job_id);
        }
    }

    for id in to_remove {
        open_job_controls.remove(&id);
    }
}
