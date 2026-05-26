use eframe::egui;
use crate::localization::translate;
use crate::app::Tab;

fn format_file_size(bytes: u64) -> String {
    format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
}

fn stage_color(stage: &crate::queue::StageStatus, ui: &egui::Ui) -> egui::Color32 {
    match stage {
        crate::queue::StageStatus::Pending => ui.visuals().weak_text_color(),
        crate::queue::StageStatus::Running => egui::Color32::from_rgb(255, 200, 0),
        crate::queue::StageStatus::Done    => egui::Color32::from_rgb(46, 204, 113),
        crate::queue::StageStatus::Failed  => egui::Color32::from_rgb(231, 76, 60),
    }
}

/// Малює нижню панель черги задач пайплайну.
pub fn draw_queue_panel(
    ui: &mut egui::Ui,
    language: crate::localization::Language,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
    selected_job_logs: &mut Option<(u64, String)>,
    selected_job_control: &mut Option<u64>,
    control_text_input: &mut String,
    whisper_model_download: &std::sync::Arc<std::sync::Mutex<crate::gui::welcome::BinaryDownload>>,
    active_tab: &mut Tab,
    retry_request: &mut Option<(u64, crate::queue::RetryStage)>,
) {
    ui.add_space(4.0);

    // Загальна вартість всіх OpenRouter запитів у черзі
    let total_cost: f64 = jobs.iter()
        .filter_map(|j| *j.translation_cost.lock().unwrap())
        .sum();

    // Верхній рядок керування
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(translate(language, "queue_panel_title")).strong().size(13.0));
        ui.label(egui::RichText::new(format!("({})", jobs.len())).weak().size(11.0));

        if total_cost > 0.0 {
            ui.label(
                egui::RichText::new(format!("${:.5}", total_cost))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(46, 204, 113)),
            );
        }

        let has_pending = jobs.iter().any(|j| {
            *j.status.lock().unwrap() == crate::queue::JobStatus::Pending
        });

        // Перевіряємо, чи всі потрібні моделі завантажені для задач у черзі
        let model_download_state = whisper_model_download.lock().unwrap().clone();
        let whisper_blocked: Option<String> = jobs.iter()
            .filter(|j| *j.status.lock().unwrap() == crate::queue::JobStatus::Pending)
            .find(|j| {
                j.settings.subtitles_enabled
                    && j.settings.subtitles_service == "Whisper"
                    && !crate::bundle::whisper_model_exists(&j.settings.whisper_model)
            })
            .map(|j| {
                let is_downloading = matches!(model_download_state, crate::gui::welcome::BinaryDownload::Downloading(_));
                if is_downloading {
                    format!("⏳ Модель Whisper '{}' ще завантажується...", j.settings.whisper_model)
                } else {
                    format!("⚠ Модель Whisper '{}' не завантажена. Завантажте її в секції Субтитри.", j.settings.whisper_model)
                }
            });

        let can_run = has_pending && whisper_blocked.is_none();

        let has_active = jobs.iter().any(|j| {
            let s = j.status.lock().unwrap().clone();
            matches!(
                s,
                crate::queue::JobStatus::Running
                    | crate::queue::JobStatus::AwaitingControl
                    | crate::queue::JobStatus::AwaitingMediaControl
            )
        });
        let can_clear = !jobs.is_empty() && !has_active;

        let mut clicked = false;
        let mut clear_clicked = false;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let run_btn = ui.add_enabled(
                can_run,
                egui::Button::new(egui::RichText::new(translate(language, "queue_run_btn")).strong()),
            );
            if run_btn.clicked() {
                clicked = true;
            }
            if let Some(ref msg) = whisper_blocked {
                run_btn.on_disabled_hover_text(msg);
            }

            let clear_btn = ui.add_enabled(
                can_clear,
                egui::Button::new(egui::RichText::new(translate(language, "queue_clear_btn"))),
            );
            if clear_btn.clicked() {
                clear_clicked = true;
            }
            if has_active {
                clear_btn.on_disabled_hover_text(translate(language, "queue_clear_disabled_hint"));
            }

            // Малюємо загальний прогресбар черги всередині right_to_left макету,
            // щоб він зайняв весь доступний простір по центру.
            if !jobs.is_empty() {
                ui.add_space(8.0);

                let total_jobs = jobs.len();
                let overall_progress = if total_jobs > 0 {
                    let sum: f32 = jobs.iter().map(|j| {
                        let status = j.status.lock().unwrap().clone();
                        match status {
                            crate::queue::JobStatus::Done => 1.0,
                            crate::queue::JobStatus::Running
                            | crate::queue::JobStatus::AwaitingControl
                            | crate::queue::JobStatus::AwaitingMediaControl => {
                                let (prog, _, _) = j.calculate_progress();
                                prog
                            }
                            _ => 0.0,
                        }
                    }).sum();
                    sum / total_jobs as f32
                } else {
                    0.0
                };

                let is_running = jobs.iter().any(|j| {
                    let s = j.status.lock().unwrap().clone();
                    s == crate::queue::JobStatus::Running || s == crate::queue::JobStatus::AwaitingMediaControl
                });

                let pct_label = egui::RichText::new(format!("{:.0}%", overall_progress * 100.0))
                    .size(11.0)
                    .weak();
                ui.label(pct_label);
                ui.add_space(4.0);

                let bar_width = ui.available_width() - 8.0;
                if bar_width > 30.0 {
                    let bar = egui::ProgressBar::new(overall_progress)
                        .animate(is_running)
                        .desired_height(6.0);
                    ui.add_sized([bar_width, 6.0], bar);
                }
            }
        });

        if clear_clicked {
            jobs.clear();
            *job_counter = 0;
        }

        if clicked {
            let ctx = ui.ctx().clone();
            for job in jobs.iter() {
                if *job.status.lock().unwrap() != crate::queue::JobStatus::Pending {
                    continue;
                }
                crate::core::pipeline::run_pipeline(
                    job.id,
                    job.name.clone(),
                    job.settings.clone(),
                    std::sync::Arc::clone(&job.status),
                    std::sync::Arc::clone(&job.translation_stage),
                    std::sync::Arc::clone(&job.voiceover_stage),
                    std::sync::Arc::clone(&job.video_stage),
                    std::sync::Arc::clone(&job.subtitles_stage),
                    std::sync::Arc::clone(&job.montage_stage),
                    std::sync::Arc::clone(&job.translated_text),
                    std::sync::Arc::clone(&job.translation_cost),
                    std::sync::Arc::clone(&job.audio_duration),
                    std::sync::Arc::clone(&job.prompts_progress),
                    std::sync::Arc::clone(&job.media_progress),
                    std::sync::Arc::clone(&job.montage_progress),
                    std::sync::Arc::clone(&job.montage_file_size),
                    std::sync::Arc::clone(&job.media_control_resume),
                    ctx.clone(),
                );
            }
        }
    });

    ui.add_space(10.0);

    // Список задач з горизонтальною прокруткою
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for job in jobs.iter() {
                    let status = job.status.lock().unwrap().clone();
                    let translation_stage = job.translation_stage.lock().unwrap().clone();
                    let voiceover_stage = job.voiceover_stage.lock().unwrap().clone();
                    let video_stage = job.video_stage.lock().unwrap().clone();
                    let subtitles_stage = job.subtitles_stage.lock().unwrap().clone();
                    let montage_stage = job.montage_stage.lock().unwrap().clone();
                    let montage_pct = *job.montage_progress.lock().unwrap();

                    let (status_text, status_color): (String, egui::Color32) = match &status {
                        crate::queue::JobStatus::Pending => (
                            translate(language, "queue_status_pending").to_string(),
                            ui.visuals().weak_text_color(),
                        ),
                        crate::queue::JobStatus::Running => {
                            let (prog, _, _) = job.calculate_progress();
                            (
                                format!("{} ({:.0}%)", translate(language, "queue_status_running"), prog * 100.0),
                                egui::Color32::from_rgb(255, 200, 0),
                            )
                        }
                        crate::queue::JobStatus::AwaitingControl => {
                            let (prog, _, _) = job.calculate_progress();
                            (
                                format!("{} ({:.0}%)", translate(language, "queue_status_awaiting_control"), prog * 100.0),
                                egui::Color32::from_rgb(155, 89, 182),
                            )
                        }
                        crate::queue::JobStatus::AwaitingMediaControl => {
                            let (prog, _, _) = job.calculate_progress();
                            (
                                format!("{} ({:.0}%)", translate(language, "queue_status_awaiting_media"), prog * 100.0),
                                egui::Color32::from_rgb(230, 126, 34),
                            )
                        }
                        crate::queue::JobStatus::Done => (
                            translate(language, "queue_status_done").to_string(),
                            egui::Color32::from_rgb(46, 204, 113),
                        ),
                        crate::queue::JobStatus::Failed(_) => (
                            translate(language, "queue_status_failed").to_string(),
                            egui::Color32::from_rgb(231, 76, 60),
                        ),
                    };

                    // Визначаємо чи задача може бути повторена (не виконується зараз)
                    let can_retry = !matches!(
                        status,
                        crate::queue::JobStatus::Running
                            | crate::queue::JobStatus::AwaitingControl
                            | crate::queue::JobStatus::AwaitingMediaControl
                    );

                    let group_frame = egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 6.0, bottom: 6.0 });
                    // Closure повертає (card_clicked, retry_clicked).
                    let response = group_frame.show(ui, |ui| -> (bool, bool) {
                        ui.set_width(210.0);
                        let mut card_clicked = false;
                        let mut retry_clicked = false;

                        ui.vertical(|ui| {
                            ui.add_space(3.0);

                            // Назва задачі — клікабельна для відкриття логу; ↺ — retry всієї задачі
                            ui.horizontal(|ui| {
                                let title = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(format!("#{} {}", job.id + 1, &job.name))
                                            .strong().size(15.0)
                                    ).sense(egui::Sense::click())
                                );
                                if title.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if title.clicked() {
                                    card_clicked = true;
                                }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let btn = ui.add_enabled(
                                        can_retry,
                                        egui::Button::new(egui::RichText::new("↺").size(11.0)).small(),
                                    );
                                    if btn.on_hover_text(translate(language, "job_retry_tooltip")).clicked() {
                                        *retry_request = Some((job.id, crate::queue::RetryStage::Translation));
                                        retry_clicked = true;
                                    }
                                });
                            });

                            ui.add_space(3.0);

                            // Активні етапи — кожен з нового рядка з кольором та кнопкою retry
                            let orig_text = &job.settings.text;
                            let orig_chars = orig_text.chars().count();
                            let orig_tokens = crate::gui::editor::count_tokens(orig_text);

                            if job.settings.translation_enabled {
                                let translated_opt = job.translated_text.lock().unwrap();
                                let cost_opt = job.translation_cost.lock().unwrap();

                                let cost_str = if let Some(cost) = *cost_opt {
                                    format!(", ${:.5}", cost)
                                } else {
                                    String::new()
                                };

                                let translation_label = if let Some(ref trans_text) = *translated_opt {
                                    let trans_chars = trans_text.chars().count();
                                    let trans_tokens = crate::gui::editor::count_tokens(trans_text);
                                    format!(
                                        "{} ({} {}, {} {}{})",
                                        translate(language, "translation"),
                                        trans_tokens, translate(language, "stats_tokens_short"),
                                        trans_chars, translate(language, "stats_chars_short"),
                                        cost_str
                                    )
                                } else {
                                    format!(
                                        "{} ({} {}, {} {}{})",
                                        translate(language, "translation"),
                                        orig_tokens, translate(language, "stats_tokens_short"),
                                        orig_chars, translate(language, "stats_chars_short"),
                                        cost_str
                                    )
                                };
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(translation_label)
                                        .color(stage_color(&translation_stage, ui)).size(12.5));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = ui.add_enabled(
                                            can_retry,
                                            egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                        );
                                        if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                            *retry_request = Some((job.id, crate::queue::RetryStage::Translation));
                                            retry_clicked = true;
                                        }
                                    });
                                });
                            } else if job.settings.voiceover_enabled {
                                let original_label = format!(
                                    "{} ({} {}, {} {})",
                                    translate(language, "voiceover_text_source_original"),
                                    orig_tokens, translate(language, "stats_tokens_short"),
                                    orig_chars, translate(language, "stats_chars_short")
                                );
                                ui.label(
                                    egui::RichText::new(original_label)
                                        .color(egui::Color32::from_rgb(46, 204, 113))
                                        .size(12.5),
                                );
                            }

                            if job.settings.voiceover_enabled {
                                let voice_label = if voiceover_stage == crate::queue::StageStatus::Done {
                                    let dur_opt = job.audio_duration.lock().unwrap();
                                    if let Some(secs) = *dur_opt {
                                        let total_s = secs as u64;
                                        let h = total_s / 3600;
                                        let m = (total_s % 3600) / 60;
                                        let s = total_s % 60;
                                        let dur_str = if h > 0 {
                                            format!("{}{}{}{}{}{}",
                                                h, translate(language, "time_hours_short"),
                                                m, translate(language, "time_mins_short"),
                                                s, translate(language, "time_secs_short"))
                                        } else if m > 0 {
                                            format!("{}{}{}{}",
                                                m, translate(language, "time_mins_short"),
                                                s, translate(language, "time_secs_short"))
                                        } else {
                                            format!("{}{}", s, translate(language, "time_secs_short"))
                                        };
                                        format!("{} ({})", translate(language, "voiceover"), dur_str)
                                    } else {
                                        translate(language, "voiceover").to_string()
                                    }
                                } else if job.settings.translation_enabled {
                                    let translated_opt = job.translated_text.lock().unwrap();
                                    if translated_opt.is_some() {
                                        translate(language, "voiceover").to_string()
                                    } else {
                                        format!("{} ({})", translate(language, "voiceover"),
                                            translate(language, "queue_waiting_translation"))
                                    }
                                } else {
                                    translate(language, "voiceover").to_string()
                                };

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(voice_label)
                                        .color(stage_color(&voiceover_stage, ui)).size(12.5));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = ui.add_enabled(
                                            can_retry,
                                            egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                        );
                                        if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                            *retry_request = Some((job.id, crate::queue::RetryStage::Voiceover));
                                            retry_clicked = true;
                                        }
                                    });
                                });
                            }

                            if job.settings.video_enabled {
                                match &video_stage {
                                    crate::queue::StageStatus::Running | crate::queue::StageStatus::Done => {
                                        let prompts_opt = *job.prompts_progress.lock().unwrap();
                                        let media_opt   = *job.media_progress.lock().unwrap();
                                        let color = stage_color(&video_stage, ui);
                                        ui.horizontal(|ui| {
                                            let prompts_str = match prompts_opt {
                                                Some((done, total)) => format!("{} ({}/{})", translate(language, "video_prompts"), done, total),
                                                None => translate(language, "video_prompts").to_string(),
                                            };
                                            ui.label(egui::RichText::new(prompts_str).color(color).size(12.5));
                                            let media_str = match media_opt {
                                                Some((done, total)) => format!("{} ({}/{})", translate(language, "video_media"), done, total),
                                                None => translate(language, "video_media").to_string(),
                                            };
                                            ui.label(egui::RichText::new(media_str).color(color).size(12.5));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let btn = ui.add_enabled(
                                                    can_retry,
                                                    egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                                );
                                                if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                                    *retry_request = Some((job.id, crate::queue::RetryStage::Video));
                                                    retry_clicked = true;
                                                }
                                            });
                                        });
                                    }
                                    _ => {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(translate(language, "video"))
                                                .color(stage_color(&video_stage, ui)).size(12.5));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let btn = ui.add_enabled(
                                                    can_retry,
                                                    egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                                );
                                                if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                                    *retry_request = Some((job.id, crate::queue::RetryStage::Video));
                                                    retry_clicked = true;
                                                }
                                            });
                                        });
                                    }
                                }
                            }

                            if job.settings.voiceover_enabled {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(translate(language, "subtitles"))
                                        .color(stage_color(&subtitles_stage, ui)).size(12.5));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = ui.add_enabled(
                                            can_retry,
                                            egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                        );
                                        if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                            *retry_request = Some((job.id, crate::queue::RetryStage::Subtitles));
                                            retry_clicked = true;
                                        }
                                    });
                                });
                            }

                            if job.settings.montage_enabled {
                                let montage_label = match &montage_stage {
                                    crate::queue::StageStatus::Running => {
                                        match montage_pct {
                                            Some(pct) => format!("{} ({:.0}%)", translate(language, "editing"), pct * 100.0),
                                            None => translate(language, "editing").to_string(),
                                        }
                                    }
                                    crate::queue::StageStatus::Done => {
                                        let size_str = job.montage_file_size.lock().unwrap()
                                            .map(format_file_size)
                                            .unwrap_or_default();
                                        if size_str.is_empty() {
                                            format!("{} (100%)", translate(language, "editing"))
                                        } else {
                                            format!("{} (100%  {})", translate(language, "editing"), size_str)
                                        }
                                    }
                                    _ => translate(language, "editing").to_string(),
                                };
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(montage_label)
                                        .color(stage_color(&montage_stage, ui)).size(12.5));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn = ui.add_enabled(
                                            can_retry,
                                            egui::Button::new(egui::RichText::new("↺").size(10.0)).small(),
                                        );
                                        if btn.on_hover_text(translate(language, "stage_retry_tooltip")).clicked() {
                                            *retry_request = Some((job.id, crate::queue::RetryStage::Montage));
                                            retry_clicked = true;
                                        }
                                    });
                                });
                            }

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(status_text)
                                        .color(status_color)
                                        .size(13.0),
                                );

                                if let crate::queue::JobStatus::Failed(err) = &status {
                                    ui.label(
                                        egui::RichText::new("⚠ помилка")
                                            .color(egui::Color32::from_rgb(231, 76, 60))
                                            .size(12.0),
                                    ).on_hover_text(err);
                                }
                            });

                            ui.add_space(3.0);

                            // Індивідуальний прогрес бар картки задачі
                            let (prog, _, _) = job.calculate_progress();
                            let is_job_running = status == crate::queue::JobStatus::Running
                                || status == crate::queue::JobStatus::AwaitingMediaControl;

                            ui.horizontal(|ui| {
                                let pct_text = format!("{:.0}%", prog * 100.0);
                                let pct_galley = ui.painter().layout_no_wrap(
                                    pct_text.clone(),
                                    egui::FontId::proportional(11.0),
                                    ui.visuals().weak_text_color(),
                                );
                                let pct_width = pct_galley.size().x + 4.0;
                                let bar_width = (ui.available_width() - pct_width - ui.spacing().item_spacing.x).max(20.0);

                                let bar = egui::ProgressBar::new(prog)
                                    .animate(is_job_running)
                                    .desired_height(6.0);
                                ui.add_sized([bar_width, 6.0], bar);

                                ui.label(
                                    egui::RichText::new(pct_text)
                                        .size(11.0)
                                        .weak(),
                                );
                            });
                        });

                        (card_clicked, retry_clicked)
                    });

                    let (card_clicked, any_retry_clicked) = response.inner;

                    if card_clicked && !any_retry_clicked {
                        if status == crate::queue::JobStatus::AwaitingControl {
                            *selected_job_control = Some(job.id);
                            if let Some(text) = job.translated_text.lock().unwrap().as_ref() {
                                *control_text_input = text.clone();
                            } else {
                                *control_text_input = String::new();
                            }
                        } else if status == crate::queue::JobStatus::AwaitingMediaControl {
                            *active_tab = Tab::Gallery;
                        } else {
                            *selected_job_logs = Some((job.id, job.name.clone()));
                        }
                    }

                    ui.add_space(4.0);
                }
            });
        });
}
