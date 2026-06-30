use eframe::egui;

use super::{Tab, VideoMakerApp};

impl VideoMakerApp {
    /// Збирає поточний стан усіх налаштувань пайплайну в знімок PipelineTemplate.
    fn current_pipeline_template(&self) -> crate::gui::settings::storage::PipelineTemplate {
        crate::gui::settings::storage::PipelineTemplate {
            openrouter_key: self.openrouter_key.clone(),
            assemblyai_key: self.assemblyai_key.clone(),
            pexels_key: self.pexels_key.clone(),
            pixabay_key: self.pixabay_key.clone(),
            voiceover_provider: self.voiceover_provider.clone(),
            voiceover_template_uuid: self.voiceover_template_uuid.clone(),
            pipeline_translation_enabled: self.pipeline_translation_enabled,
            pipeline_translation_control_enabled: self.pipeline_translation_control_enabled,
            pipeline_control_auto_open: self.pipeline_control_auto_open,
            pipeline_media_control_enabled: self.pipeline_media_control_enabled,
            pipeline_montage_control_enabled: self.pipeline_montage_control_enabled,
            pipeline_voiceover_enabled: self.pipeline_voiceover_enabled,
            pipeline_video_enabled: self.pipeline_video_enabled,
            pipeline_subtitles_enabled: self.pipeline_subtitles_enabled,
            pipeline_editing_enabled: self.pipeline_editing_enabled,
            translation_prompt: self.translation_prompt.clone(),
            translation_model: self.translation_model.clone(),
            translation_model_openrouter: self.translation_model_openrouter.clone(),
            translation_model_claude: self.translation_model_claude.clone(),
            translation_model_gemini: self.translation_model_gemini.clone(),
            translation_model_codex: self.translation_model_codex.clone(),
            translation_model_agy: self.translation_model_agy.clone(),
            translation_model_pi: self.translation_model_pi.clone(),
            video_service: self.video_service.clone(),
            text_split_mode: self.text_split_mode.clone(),
            text_split_mode_openrouter: self.text_split_mode_openrouter.clone(),
            text_split_char_limit: self.text_split_char_limit,
            translation_temperature: self.translation_temperature,
            translation_service: self.translation_service.clone(),
            edge_tts_voice: self.edge_tts_voice.clone(),
            edge_tts_rate: self.edge_tts_rate.clone(),
            edge_tts_pitch: self.edge_tts_pitch.clone(),
            edge_tts_volume: self.edge_tts_volume.clone(),
            googler_image_max_threads: self.googler_image_max_threads,
            googler_video_max_threads: self.googler_video_max_threads,
            voiceover_convert_to_wav: self.voiceover_convert_to_wav,
            video_prompt: self.video_prompt.clone(),
            video_context_enabled: self.video_context_enabled,
            video_context_mode: self.video_context_mode.clone(),
            video_context_chars: self.video_context_chars,
            video_agent_mode: self.video_agent_mode.clone(),
            googler_video_upscale_enabled: self.googler_video_upscale_enabled,
            googler_video_upscale_resolution: self.googler_video_upscale_resolution.clone(),
            googler_video_upscale_quality: self.googler_video_upscale_quality.clone(),
            video_agent_prompt: self.video_agent_prompt.clone(),
            video_style_enabled: self.video_style_enabled,
            video_style_prompt: self.video_style_prompt.clone(),
            googler_image_priority: self.googler_image_priority.clone(),
            googler_video_priority: self.googler_video_priority.clone(),
            googler_video_disabled: self.googler_video_disabled.clone(),
            video_media_type: self.video_media_type.clone(),
            subtitles_service: self.subtitles_service.clone(),
            whisper_language: self.whisper_language.clone(),
            whisper_model: self.whisper_model.clone(),
            whisper_max_line_width: self.whisper_max_line_width,
            subtitle_font_size: self.subtitle_font_size,
            subtitle_color: self.subtitle_color,
            subtitle_margin_v: self.subtitle_margin_v,
            subtitle_karaoke: self.subtitle_karaoke,
            subtitle_karaoke_mode: self.subtitle_karaoke_mode,
            subtitle_karaoke_highlight_color: self.subtitle_karaoke_highlight_color,
            subtitle_karaoke_outline_color: self.subtitle_karaoke_outline_color,
            subtitle_karaoke_bold: self.subtitle_karaoke_bold,
            subtitle_karaoke_scale: self.subtitle_karaoke_scale,
            subtitle_font: self.subtitle_font.clone(),
            capcut_enabled: self.capcut_enabled,
            capcut_draft_path: self.capcut_draft_path.clone(),
            montage_service: self.montage_service.clone(),
            montage_fps: self.montage_fps,
            montage_preset: self.montage_preset.clone(),
            montage_bitrate: self.montage_bitrate,
            montage_transition: self.montage_transition.clone(),
            montage_transition_duration: self.montage_transition_duration,
            montage_image_zoom_enabled: self.montage_image_zoom_enabled,
            montage_image_zoom_intensity: self.montage_image_zoom_intensity,
            montage_image_zoom_mode: self.montage_image_zoom_mode.clone(),
            montage_image_zoom_scale: self.montage_image_zoom_scale,
            montage_image_shake_enabled: self.montage_image_shake_enabled,
            montage_image_shake_intensity: self.montage_image_shake_intensity,
            video_llm_service: self.video_llm_service.clone(),
            video_llm_model: self.video_llm_model.clone(),
            video_llm_model_openrouter: self.video_llm_model_openrouter.clone(),
            video_llm_model_claude: self.video_llm_model_claude.clone(),
            video_llm_model_gemini: self.video_llm_model_gemini.clone(),
            video_llm_model_codex: self.video_llm_model_codex.clone(),
            video_llm_model_agy: self.video_llm_model_agy.clone(),
            video_llm_model_pi: self.video_llm_model_pi.clone(),
            video_llm_temperature: self.video_llm_temperature,
            overlay_triggers_enabled: self.overlay_triggers_enabled,
            overlay_triggers: self.overlay_triggers.clone(),
        }
    }

    /// Додає задачу з историї безпосередньо в чергу, не змінюючи налаштування панелі.
    pub(super) fn enqueue_from_history(
        &mut self,
        entry: &crate::gui::settings::storage::TaskHistoryEntry,
    ) {
        use crate::localization::translate;

        let save_path = if cfg!(target_os = "macos") {
            &self.save_path_macos
        } else {
            &self.save_path_windows
        };

        if save_path.trim().is_empty() {
            self.queue_error =
                Some(translate(self.language, "queue_error_no_save_path").to_string());
            return;
        }

        let base = save_path.trim_end_matches('/').trim_end_matches('\\');
        let actual_path = format!("{}/{}", base, entry.name);

        if let Err(e) = std::fs::create_dir_all(&actual_path) {
            self.queue_error = Some(format!(
                "{}: {}",
                translate(self.language, "queue_error_create_dir"),
                e
            ));
            return;
        }

        // Якщо задача мала шаблон — беремо з нього основні налаштування.
        // Але control-прапорці завжди беремо з поточної панелі,
        // щоб відновлення поважало актуальний вибір користувача.
        let t = entry
            .template_name
            .as_deref()
            .and_then(|name| crate::gui::settings::storage::load_template(name))
            .unwrap_or_else(|| self.current_pipeline_template());

        // Для задач з history поважаємо початково увімкнені етапи самої задачі.
        // Це важливо для відновлення старих задач, навіть якщо шаблон або поточна панель
        // уже були змінені після першого запуску.
        let has_stage_snapshot = entry.stage_translation
            || entry.stage_voiceover
            || entry.stage_video
            || entry.stage_subtitles
            || entry.stage_editing;
        let translation_enabled = if has_stage_snapshot {
            entry.stage_translation
        } else {
            t.pipeline_translation_enabled
        };
        let voiceover_enabled = if has_stage_snapshot {
            entry.stage_voiceover
        } else {
            t.pipeline_voiceover_enabled
        };
        let video_enabled = if has_stage_snapshot {
            entry.stage_video
        } else {
            t.pipeline_video_enabled
        };
        let subtitles_enabled = if has_stage_snapshot {
            entry.stage_subtitles
        } else {
            t.pipeline_subtitles_enabled
        };
        let montage_enabled = if has_stage_snapshot {
            entry.stage_editing
        } else {
            t.pipeline_editing_enabled
        };

        let settings = crate::queue::JobSettings {
            text: entry.text.clone(),
            save_path: actual_path,
            translation_enabled,
            translation_control_enabled: self.pipeline_translation_control_enabled,
            translation_prompt: t.translation_prompt.clone(),
            translation_model: t.translation_model.clone(),
            translation_temperature: t.translation_temperature,
            translation_service: t.translation_service.clone(),
            openrouter_key: t.openrouter_key.clone(),
            voiceover_enabled,
            voicebot_key: self.voicebot_key.clone(),
            voiceover_template_uuid: t.voiceover_template_uuid.clone(),
            voiceover_provider: t.voiceover_provider.clone(),
            edge_tts_voice: t.edge_tts_voice.clone(),
            edge_tts_rate: t.edge_tts_rate.clone(),
            edge_tts_pitch: t.edge_tts_pitch.clone(),
            edge_tts_volume: t.edge_tts_volume.clone(),
            voiceover_convert_to_wav: t.voiceover_convert_to_wav,
            video_enabled,
            video_service: t.video_service.clone(),
            video_media_type: t.video_media_type.clone(),
            video_prompt: t.video_prompt.clone(),
            video_context_enabled: t.video_context_enabled,
            video_context_mode: t.video_context_mode.clone(),
            video_context_chars: t.video_context_chars,
            video_agent_mode: t.video_agent_mode.clone(),
            video_agent_prompt: t.video_agent_prompt.clone(),
            video_style_enabled: t.video_style_enabled,
            video_style_prompt: t.video_style_prompt.clone(),
            video_llm_service: t.video_llm_service.clone(),
            video_llm_model: t.video_llm_model.clone(),
            video_llm_temperature: t.video_llm_temperature,
            text_split_mode: t.text_split_mode.clone(),
            text_split_char_limit: t.text_split_char_limit,
            googler_key: self.googler_key.clone(),
            googler_image_priority: t.googler_image_priority.clone(),
            googler_video_priority: t
                .googler_video_priority
                .iter()
                .filter(|p| !t.googler_video_disabled.contains(p))
                .cloned()
                .collect(),
            googler_image_max_threads: t.googler_image_max_threads,
            googler_video_upscale_enabled: t.googler_video_upscale_enabled,
            googler_video_upscale_resolution: t.googler_video_upscale_resolution.clone(),
            googler_video_upscale_quality: t.googler_video_upscale_quality.clone(),
            assemblyai_key: t.assemblyai_key.clone(),
            pexels_key: t.pexels_key.clone(),
            pixabay_key: t.pixabay_key.clone(),
            subtitles_enabled,
            subtitles_service: t.subtitles_service.clone(),
            whisper_language: t.whisper_language.clone(),
            whisper_model: t.whisper_model.clone(),
            whisper_max_line_width: t.whisper_max_line_width,
            subtitle_font_size: t.subtitle_font_size,
            subtitle_color: t.subtitle_color,
            subtitle_margin_v: t.subtitle_margin_v,
            subtitle_karaoke: t.subtitle_karaoke,
            subtitle_karaoke_mode: t.subtitle_karaoke_mode,
            subtitle_karaoke_highlight_color: t.subtitle_karaoke_highlight_color,
            subtitle_karaoke_outline_color: t.subtitle_karaoke_outline_color,
            subtitle_karaoke_bold: t.subtitle_karaoke_bold,
            subtitle_karaoke_scale: t.subtitle_karaoke_scale,
            subtitle_font: t.subtitle_font.clone(),
            montage_enabled,
            montage_service: t.montage_service.clone(),
            capcut_enabled: t.capcut_enabled,
            capcut_draft_path: t.capcut_draft_path.clone(),
            montage_fps: t.montage_fps,
            montage_preset: t.montage_preset.clone(),
            montage_bitrate: t.montage_bitrate,
            montage_transition: t.montage_transition.clone(),
            montage_transition_duration: t.montage_transition_duration,
            montage_image_zoom_enabled: t.montage_image_zoom_enabled,
            montage_image_zoom_intensity: t.montage_image_zoom_intensity,
            montage_image_zoom_mode: t.montage_image_zoom_mode.clone(),
            montage_image_zoom_scale: t.montage_image_zoom_scale,
            montage_image_shake_enabled: t.montage_image_shake_enabled,
            montage_image_shake_intensity: t.montage_image_shake_intensity,
            media_control_enabled: self.pipeline_media_control_enabled,
            montage_control_enabled: self.pipeline_montage_control_enabled,
            overlay_triggers_enabled: t.overlay_triggers_enabled,
            overlay_triggers: t.overlay_triggers.clone(),
            resume_from_stage: None,
            skip_agent_on_resume: false,
            skip_existing_media: false,
        };

        let found = crate::gui::pipeline::resume::FoundFiles::scan(
            std::path::Path::new(&settings.save_path),
            &entry.name,
        );

        if found.has_any() {
            self.resume_dialog_open = true;
            self.resume_pending = Some(crate::gui::pipeline::resume::ResumePendingData::new(
                entry.name.clone(),
                found,
                settings,
            ));
        } else {
            let id = self.job_counter;
            self.job_counter += 1;
            self.jobs.push(crate::queue::PipelineJob::new(
                id,
                entry.name.clone(),
                settings,
            ));
        }
    }
    /// Малює бічні панелі головної вкладки: історію задач і налаштування пайплайну.
    pub(super) fn draw_main_side_panels(&mut self, ctx: &egui::Context) {
        // Відображаємо ліву панель историії ТІЛЬКИ на вкладці "Основна"
        if self.active_tab == Tab::Main {
            let mut delete_history_idx: Option<usize> = None;
            let side_frame_left = egui::Frame::side_top_panel(ctx.style().as_ref())
                .inner_margin(egui::Margin::same(0.0));
            egui::SidePanel::left("task_history_panel")
                .frame(side_frame_left)
                .exact_width(160.0)
                .resizable(false)
                .show(ctx, |ui| {
                    let applied = crate::gui::task_history::draw_task_history_panel(
                        ui,
                        self.language,
                        &self.task_history,
                        &mut delete_history_idx,
                    );
                    if let Some(entry) = applied {
                        self.text_input = entry.text.clone();
                        self.enqueue_from_history(&entry);
                    }
                });
            if let Some(idx) = delete_history_idx {
                crate::gui::settings::storage::remove_from_task_history(
                    &mut self.task_history,
                    idx,
                );
            }
        }

        // Відображаємо бічну панель пайплайну ТІЛЬКИ на вкладці "Основна"
        if self.active_tab == Tab::Main {
            let jobs_len_before = self.jobs.len();
            let prev_translation_service = self.translation_service.clone();

            // default_width передається лише як початкове значення при першому запуску.
            // egui::Memory зберігає ширину між кадрами сам — нічого читати назад не потрібно.
            let side_frame = egui::Frame::side_top_panel(ctx.style().as_ref())
                .inner_margin(egui::Margin::same(0.0));
            egui::SidePanel::right("pipeline_panel")
                .frame(side_frame)
                .default_width(self.pipeline_width)
                .width_range(100.0..=750.0)
                .resizable(true)
                .show(ctx, |ui| {
                    crate::gui::pipeline::draw_pipeline_panel(
                        ui,
                        self.language,
                        &mut self.openrouter_key,
                        &mut self.openrouter_status,
                        &self.openrouter_balance,
                        &mut self.voicebot_key,
                        &mut self.voicebot_status,
                        &self.voicebot_test_result,
                        &self.voicebot_balance,
                        &mut self.googler_key,
                        &mut self.googler_status,
                        &self.googler_test_result,
                        &self.googler_balance,
                        &mut self.assemblyai_key,
                        &mut self.assemblyai_status,
                        &self.assemblyai_test_result,
                        &mut self.pexels_key,
                        &mut self.pexels_status,
                        &self.pexels_test_result,
                        &mut self.pixabay_key,
                        &mut self.pixabay_status,
                        &self.pixabay_test_result,
                        &mut self.voiceover_provider,
                        &mut self.voiceover_template_uuid,
                        &self.voicebot_templates,
                        &self.voicebot_loading,
                        &mut self.edge_tts_voice,
                        &mut self.edge_tts_rate,
                        &mut self.edge_tts_pitch,
                        &mut self.edge_tts_volume,
                        &self.edge_tts_voices,
                        &self.edge_tts_loading_voices,
                        &mut self.edge_tts_show_all_languages,
                        &mut self.template_name_input,
                        &mut self.saved_templates,
                        &mut self.template_status,
                        &mut self.pipeline_translation_enabled,
                        &mut self.pipeline_translation_control_enabled,
                        &mut self.pipeline_control_auto_open,
                        &mut self.pipeline_media_control_enabled,
                        &mut self.pipeline_montage_control_enabled,
                        &mut self.pipeline_voiceover_enabled,
                        &mut self.pipeline_video_enabled,
                        &mut self.pipeline_subtitles_enabled,
                        &mut self.pipeline_editing_enabled,
                        &mut self.translation_prompt,
                        &mut self.translation_model,
                        &mut self.translation_model_openrouter,
                        &mut self.translation_model_claude,
                        &mut self.translation_model_gemini,
                        &mut self.translation_model_codex,
                        &mut self.translation_model_agy,
                        &mut self.translation_model_pi,
                        &mut self.translation_model_search,
                        &self.openrouter_models,
                        &self.openrouter_models_loading,
                        &mut self.video_service,
                        &mut self.video_media_type,
                        &mut self.text_split_mode,
                        &mut self.text_split_mode_openrouter,
                        &mut self.text_split_char_limit,
                        &mut self.video_prompt,
                        &mut self.video_context_enabled,
                        &mut self.video_context_mode,
                        &mut self.video_context_chars,
                        &mut self.video_agent_mode,
                        &mut self.video_agent_prompt,
                        &mut self.video_style_enabled,
                        &mut self.video_style_prompt,
                        &mut self.video_llm_service,
                        &mut self.video_llm_model,
                        &mut self.video_llm_model_openrouter,
                        &mut self.video_llm_model_claude,
                        &mut self.video_llm_model_gemini,
                        &mut self.video_llm_model_codex,
                        &mut self.video_llm_model_agy,
                        &mut self.video_llm_model_pi,
                        &mut self.video_llm_temperature,
                        &mut self.video_llm_model_search,
                        &mut self.translation_temperature,
                        &mut self.translation_service,
                        &mut self.save_path_macos,
                        &mut self.save_path_windows,
                        &mut self.googler_image_max_threads,
                        &mut self.googler_video_max_threads,
                        &mut self.voiceover_convert_to_wav,
                        &mut self.googler_image_priority,
                        &mut self.googler_video_priority,
                        &mut self.googler_video_disabled,
                        &mut self.subtitles_service,
                        &mut self.whisper_language,
                        &mut self.whisper_model,
                        &mut self.whisper_max_line_width,
                        &self.whisper_model_download,
                        &mut self.subtitle_font_size,
                        &mut self.subtitle_color,
                        &mut self.subtitle_margin_v,
                        &mut self.subtitle_karaoke,
                        &mut self.subtitle_karaoke_mode,
                        &mut self.subtitle_karaoke_highlight_color,
                        &mut self.subtitle_karaoke_outline_color,
                        &mut self.subtitle_karaoke_bold,
                        &mut self.subtitle_karaoke_scale,
                        &mut self.subtitle_font,
                        &self.available_subtitle_fonts,
                        &mut self.capcut_enabled,
                        &mut self.capcut_draft_path,
                        &mut self.montage_service,
                        &mut self.montage_fps,
                        &mut self.montage_preset,
                        &mut self.montage_bitrate,
                        &mut self.montage_transition,
                        &mut self.montage_transition_duration,
                        &mut self.montage_image_zoom_enabled,
                        &mut self.montage_image_zoom_intensity,
                        &mut self.montage_image_zoom_mode,
                        &mut self.montage_image_zoom_scale,
                        &mut self.montage_image_shake_enabled,
                        &mut self.montage_image_shake_intensity,
                        &mut self.overlay_triggers_enabled,
                        &mut self.overlay_triggers,
                        &mut self.googler_video_upscale_enabled,
                        &mut self.googler_video_upscale_resolution,
                        &mut self.googler_video_upscale_quality,
                        &self.text_input,
                        &mut self.jobs,
                        &mut self.job_counter,
                        &mut self.queue_error,
                        &mut self.job_name_dialog_open,
                        &mut self.job_name_input,
                        &mut self.resume_dialog_open,
                        &mut self.resume_pending,
                    );
                });

            // Діалог відновлення задачі (знайдені наявні файли)
            crate::gui::pipeline::resume::draw_resume_dialog(
                ctx,
                self.language,
                &mut self.resume_dialog_open,
                &mut self.resume_pending,
                &mut self.jobs,
                &mut self.job_counter,
            );

            // Якщо нова задача була додана в чергу — записуємо в history
            if self.jobs.len() > jobs_len_before {
                if let Some(last_job) = self.jobs.last() {
                    let template_name = if !self.template_name_input.is_empty()
                        && self.saved_templates.contains(&self.template_name_input)
                    {
                        Some(self.template_name_input.clone())
                    } else {
                        None
                    };
                    let entry = crate::gui::settings::storage::TaskHistoryEntry {
                        id: last_job.id,
                        name: last_job.name.clone(),
                        created_at: chrono::Utc::now().timestamp(),
                        template_name,
                        text: self.text_input.clone(),
                        stage_translation: self.pipeline_translation_enabled,
                        stage_voiceover: self.pipeline_voiceover_enabled,
                        stage_video: self.pipeline_video_enabled,
                        stage_subtitles: self.pipeline_subtitles_enabled,
                        stage_editing: self.pipeline_editing_enabled,
                    };
                    crate::gui::settings::storage::append_to_task_history(
                        &mut self.task_history,
                        entry,
                    );
                }
            }

            if self.translation_service != prev_translation_service {
                if self.translation_service == "Gemini CLI"
                    || self.translation_service == "Claude Code"
                {
                    self.tool_checks.start(ctx.clone());
                    self.pending_tool_check = Some(self.translation_service.clone());
                } else {
                    self.pending_tool_check = None;
                }
            }
        }
    }
}
