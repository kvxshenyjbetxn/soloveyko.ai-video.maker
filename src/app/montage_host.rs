use eframe::egui;

use super::VideoMakerApp;

impl VideoMakerApp {
    /// Обробляє runtime-логіку редактора монтажу та повʼязані дії з медіа.
    pub(super) fn handle_montage_runtime(
        &mut self,
        ctx: &egui::Context,
        regen_paths_snapshot: &std::collections::HashSet<std::path::PathBuf>,
    ) {
        // Перебудова таймлінії в редакторі після чату з агентом
        for job in &self.jobs {
            let requested = {
                let mut flag = job.timeline_rebuild_requested.lock().unwrap();
                if *flag {
                    *flag = false;
                    true
                } else {
                    false
                }
            };
            if requested {
                if Some(job.id) == self.montage_editor_open_job {
                    let save_path = std::path::Path::new(&job.settings.save_path);

                    // Знаходимо сегменти де агент змінив промти — перегенеруємо тільки їх
                    let changed = crate::core::pipeline::find_changed_prompts_for_rebuild(
                        save_path,
                        job.settings.video_style_enabled,
                        &job.settings.video_style_prompt,
                    );
                    if !changed.is_empty() {
                        let priority = if job.settings.video_media_type == "video" {
                            job.settings.googler_video_priority.clone()
                        } else {
                            job.settings.googler_image_priority.clone()
                        };
                        for (file_path, new_prompt) in changed {
                            crate::core::pipeline::regenerate_single_media(
                                file_path,
                                job.settings.video_media_type.clone(),
                                priority.clone(),
                                job.settings.googler_key.clone(),
                                Some(new_prompt),
                                job.id,
                                job.name.clone(),
                                ctx.clone(),
                                std::sync::Arc::new(std::sync::Mutex::new(None)),
                                std::sync::Arc::new(std::sync::Mutex::new(false)),
                                Some(std::sync::Arc::clone(&self.gallery_anim_loading)),
                                Some(std::sync::Arc::clone(&self.media_regen_results_queue)),
                                job.settings.googler_video_upscale_enabled,
                                job.settings.googler_video_upscale_resolution.clone(),
                                job.settings.googler_video_upscale_quality.clone(),
                            );
                        }
                    }

                    let preview_render = crate::gui::montage_editor::PreviewRenderSettings {
                        quality: crate::gui::montage_editor::PreviewQuality::from_storage(
                            &self.preview_quality,
                        ),
                        fps: self.preview_fps.max(15.0).min(60.0),
                    };
                    self.montage_editor_state =
                        Some(crate::gui::montage_editor::MontageEditorState::load(
                            save_path,
                            &job.name,
                            preview_render,
                        ));
                }
            }
        }
        // Блокуємо drag кліпів у превью коли поверх відкрито stock picker
        if let Some(ref mut editor) = self.montage_editor_state {
            editor.input_blocked = self.stock_picker_state.is_some();
        }

        // Редактор монтажу
        let preview_render = crate::gui::montage_editor::PreviewRenderSettings {
            quality: crate::gui::montage_editor::PreviewQuality::from_storage(
                &self.preview_quality,
            ),
            fps: self.preview_fps.max(15.0).min(60.0),
        };
        let montage_actions = crate::gui::montage_editor::draw_montage_editor_window(
            ctx,
            self.language,
            &mut self.montage_editor_open_job,
            &mut self.montage_editor_state,
            &self.jobs,
            &self.gallery_anim_loading,
            regen_paths_snapshot,
            preview_render,
        );

        // Оживлення зображень з редактора монтажу
        {
            let anim_loading = std::sync::Arc::clone(&self.gallery_anim_loading);
            let job_id = self.montage_editor_open_job.unwrap_or(0);
            let job_opt = self.jobs.iter().find(|j| j.id == job_id);
            let job_name = job_opt.map(|j| j.name.clone()).unwrap_or_default();
            let (upscale_enabled, upscale_resolution, upscale_quality) = if let Some(job) = job_opt
            {
                (
                    job.settings.googler_video_upscale_enabled,
                    job.settings.googler_video_upscale_resolution.clone(),
                    job.settings.googler_video_upscale_quality.clone(),
                )
            } else {
                (
                    self.googler_video_upscale_enabled,
                    self.googler_video_upscale_resolution.clone(),
                    self.googler_video_upscale_quality.clone(),
                )
            };
            for path in montage_actions.animate_paths {
                if anim_loading.lock().unwrap().contains(&path) {
                    continue;
                }
                crate::core::pipeline::animate_single_image(
                    path,
                    self.googler_video_priority.clone(),
                    self.googler_key.clone(),
                    job_id,
                    job_name.clone(),
                    ctx.clone(),
                    std::sync::Arc::clone(&anim_loading),
                    upscale_enabled,
                    upscale_resolution.clone(),
                    upscale_quality.clone(),
                );
            }
        }

        // Перегенерація медіа з редактора монтажу (аналогічно до галереї)
        if let Some((file, settings, is_custom, job_id, job_name)) = montage_actions.regen_action {
            if is_custom {
                self.media_regen_target = Some(file.clone());
                self.media_regen_batch_targets.clear();
                self.media_regen_media_type = settings.video_media_type.clone();
                self.media_regen_image_priority = settings.googler_image_priority.clone();
                self.media_regen_video_priority = settings.googler_video_priority.clone();
                self.media_regen_prompt = crate::core::pipeline::read_prompt_for_file(&file);
                self.media_regen_base_settings = Some(settings);
                self.media_regen_job_id = job_id;
                self.media_regen_job_name = job_name;
                self.media_regen_error = None;
                self.media_regen_window_open = true;
            } else {
                let priority = if settings.video_media_type == "video" {
                    settings.googler_video_priority.clone()
                } else {
                    settings.googler_image_priority.clone()
                };
                self.media_regen_target = Some(file.clone());
                self.media_regen_error = None;
                crate::core::pipeline::regenerate_single_media(
                    file,
                    settings.video_media_type.clone(),
                    priority,
                    settings.googler_key.clone(),
                    None,
                    job_id,
                    job_name,
                    ctx.clone(),
                    std::sync::Arc::clone(&self.media_regen_result),
                    std::sync::Arc::clone(&self.media_regen_loading),
                    Some(std::sync::Arc::clone(&self.media_regen_paths)),
                    Some(std::sync::Arc::clone(&self.media_regen_results_queue)),
                    settings.googler_video_upscale_enabled,
                    settings.googler_video_upscale_resolution.clone(),
                    settings.googler_video_upscale_quality.clone(),
                );
            }
        }

        if let Some((targets, settings, job_id, job_name)) = montage_actions.batch_regen_action {
            self.media_regen_target = None;
            self.media_regen_batch_targets = targets;
            self.media_regen_media_type = settings.video_media_type.clone();
            self.media_regen_image_priority = settings.googler_image_priority.clone();
            self.media_regen_video_priority = settings.googler_video_priority.clone();
            self.media_regen_prompt.clear();
            self.media_regen_base_settings = Some(settings);
            self.media_regen_job_id = job_id;
            self.media_regen_job_name = job_name;
            self.media_regen_error = None;
            self.media_regen_window_open = true;
        }

        if montage_actions.preview_hyperframes {
            if let Some(job_id) = self.montage_editor_open_job {
                if let Some(job) = self.jobs.iter().find(|j| j.id == job_id) {
                    let preview_dir = std::path::Path::new(&job.settings.save_path).join("preview-all");
                    if let Err(e) = crate::api::hyperframes::open_preview(
                        &preview_dir,
                        job.id,
                        &job.name,
                    ) {
                        crate::logger::log_job(job.id, &job.name, &e);
                    }
                } else if let Some(editor) = self.montage_editor_state.as_ref() {
                    let preview_dir = editor.save_path.join("preview-all");
                    if let Err(e) = crate::api::hyperframes::open_preview(
                        &preview_dir,
                        job_id,
                        &editor.job_name,
                    ) {
                        crate::logger::log_job(job_id, &editor.job_name, &e);
                    }
                }
            }
        }

        if montage_actions.render_hyperframes {
            if let Some(job_id) = self.montage_editor_open_job {
                if let Some(job) = self.jobs.iter().find(|j| j.id == job_id) {
                    crate::api::hyperframes::render_pending_segments_async(
                        std::path::Path::new(&job.settings.save_path).to_path_buf(),
                        job.id,
                        job.name.clone(),
                        ctx.clone(),
                        std::sync::Arc::clone(&job.timeline_rebuild_requested),
                    );
                } else if let Some(editor) = self.montage_editor_state.as_ref() {
                    crate::api::hyperframes::render_pending_segments_async(
                        editor.save_path.clone(),
                        job_id,
                        editor.job_name.clone(),
                        ctx.clone(),
                        std::sync::Arc::new(std::sync::Mutex::new(false)),
                    );
                }
            }
        }

        // Оновлюємо налаштування превʼю редактора, якщо користувач змінив їх у топбарі
        if let Some(new_render) = montage_actions.preview_render_changed {
            self.preview_quality = new_render.quality.storage_key().to_string();
            self.preview_fps = new_render.fps;
        }

        // Відкриваємо Stock Picker з редактора монтажу (клік на плейсхолдер)
        if let Some(seg_idx) = montage_actions.open_stock_picker {
            if let Some(job_id) = self.montage_editor_open_job {
                if let Some(job) = self.jobs.iter().find(|j| j.id == job_id) {
                    let picker_render = crate::gui::montage_editor::PreviewRenderSettings {
                        quality: crate::gui::montage_editor::PreviewQuality::from_storage(
                            &self.preview_quality,
                        ),
                        fps: self.preview_fps.max(15.0).min(60.0),
                    };
                    if let Some(mut state) = crate::gui::stock_picker::StockPickerState::new(
                        job.settings.save_path.clone(),
                        self.shared_stock_cache_enabled,
                        self.shared_stock_cache_dir.clone(),
                        job.settings.pexels_key.clone(),
                        job.settings.magnific_key.clone(),
                        job.settings.pixabay_key.clone(),
                        job.settings.video_service.clone(),
                        picker_render,
                    ) {
                        state.active_segment = seg_idx;
                        state.edit_keyword = state
                            .cache
                            .get(seg_idx)
                            .map(|s| s.keyword.clone())
                            .unwrap_or_default();
                        self.stock_picker_state = Some(state);
                    }
                }
            }
        }
    }
}
