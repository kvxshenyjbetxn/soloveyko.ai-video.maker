use eframe::egui;

use super::VideoMakerApp;
use crate::localization::translate;

impl VideoMakerApp {
    /// Малює повноекранний перегляд медіафайлу з галереї та обробляє перегенерацію з preview.
    pub(super) fn draw_gallery_preview_window(&mut self, ctx: &egui::Context) {
        // Повноекранний перегляд медіафайлу з галереї
        if let Some(path) = self.gallery_preview.clone() {
            let is_video = matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("mp4") | Some("webm") | Some("mov")
            );

            if is_video {
                // Якщо плеєр для іншого файлу — скидаємо
                if self.video_player.as_ref().map_or(false, |p| p.path != path) {
                    self.video_player = None;
                }

                // Якщо плеєра ще немає — створюємо і запускаємо streaming
                if self.video_player.is_none() {
                    let player =
                        crate::gui::gallery::video_player::VideoPlayer::new(path.clone(), 10.0);
                    crate::gui::gallery::video_player::start_fullscreen_extraction(
                        &player,
                        path.clone(),
                        ctx.clone(),
                    );
                    self.video_player = Some(player);
                }

                // Дренуємо нові кадри та відображаємо
                if let Some(ref mut player) = self.video_player {
                    player.drain_pending();
                    let keep_open =
                        crate::gui::gallery::video_player::draw_video_player(ctx, player);
                    if !keep_open {
                        self.gallery_preview = None;
                        self.video_player = None;
                    }
                }
            } else {
                // Зображення — існуючий перегляд
                let tex = self
                    .gallery_textures
                    .get(&path)
                    .and_then(|t| t.as_ref())
                    .cloned();
                if let Some(texture) = tex {
                    let regen_loading_this = self.media_regen_paths.lock().unwrap().contains(&path);
                    let (keep_open, regen_kind) =
                        crate::gui::gallery::draw_image_preview(ctx, &texture, regen_loading_this);
                    if !keep_open {
                        self.gallery_preview = None;
                    }
                    if let Some(is_custom) = regen_kind {
                        let job_info = self
                            .jobs
                            .iter()
                            .find(|j| {
                                let media_dir =
                                    std::path::Path::new(&j.settings.save_path).join("media");
                                path.starts_with(&media_dir)
                            })
                            .map(|j| (j.id, j.name.clone(), j.settings.clone()));

                        if let Some((job_id, job_name, settings)) = job_info {
                            if is_custom {
                                self.media_regen_target = Some(path.clone());
                                self.media_regen_media_type = settings.video_media_type.clone();
                                self.media_regen_image_priority =
                                    settings.googler_image_priority.clone();
                                self.media_regen_video_priority =
                                    settings.googler_video_priority.clone();
                                self.media_regen_prompt =
                                    crate::core::pipeline::read_prompt_for_file(&path);
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
                                self.media_regen_target = Some(path.clone());
                                self.media_regen_error = None;
                                self.gallery_textures.remove(&path);
                                crate::core::pipeline::regenerate_single_media(
                                    path,
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
                    }
                } else {
                    self.gallery_preview = None;
                }
            }
        }
    }
    /// Обробляє runtime-дії галереї після малювання центральної панелі.
    pub(super) fn handle_gallery_runtime(
        &mut self,
        ctx: &egui::Context,
        regen_action: Option<crate::gui::gallery::RegenAction>,
        prompt_view_request: Option<std::path::PathBuf>,
        hover_extract_request: Option<std::path::PathBuf>,
        thumb_requests: Vec<std::path::PathBuf>,
        image_load_requests: Vec<std::path::PathBuf>,
        animate_all: bool,
    ) {
        // Спливаюче вікно з логами обраної задачі
        // Обробка кнопок перегенерації з галереї
        if let Some((file, settings, is_custom, job_id, job_name)) = regen_action {
            if is_custom {
                self.media_regen_target = Some(file.clone());
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
                self.media_regen_error = None;
                self.gallery_textures.remove(&file);
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

        // Відкриття popup-вікна з промтом для обраного медіафайлу
        if let Some(file) = prompt_view_request {
            self.gallery_prompt_popup = Some(crate::core::pipeline::read_prompt_for_file(&file));
        }

        // Очищення текстур для видалених файлів (після анімації .jpg → .mp4)
        self.gallery_textures.retain(|path, _| path.exists());
        self.video_thumbnails.retain(|path, _| path.exists());
        self.video_hover_frames.retain(|path, _| path.exists());

        // Запуск hover-витягування кадрів, якщо галерея це запитала
        if let Some(path) = hover_extract_request {
            crate::gui::gallery::video_player::start_hover_extraction(
                path,
                ctx.clone(),
                std::sync::Arc::clone(&self.video_hover_loading),
                std::sync::Arc::clone(&self.video_hover_result),
            );
        }

        // Запуск thumbnail-витягування для нових відео
        for path in thumb_requests {
            if !self.video_thumbnails.contains_key(&path) {
                self.video_thumbnails.insert(path.clone(), None); // Резервуємо місце
                crate::gui::gallery::video_player::start_thumbnail_extraction(
                    path,
                    ctx.clone(),
                    std::sync::Arc::clone(&self.video_thumb_loading),
                    std::sync::Arc::clone(&self.video_thumb_result),
                );
            }
        }

        // Запуск асинхронного завантаження зображень галереї
        for path in image_load_requests {
            crate::gui::gallery::preview::start_image_loading(
                path,
                ctx.clone(),
                std::sync::Arc::clone(&self.gallery_image_loading),
                std::sync::Arc::clone(&self.gallery_image_result),
            );
        }

        // Дренування готових зображень у кеш текстур
        {
            let mut lock = self.gallery_image_result.lock().unwrap();
            if !lock.is_empty() {
                for (path, tex) in lock.drain(..) {
                    self.gallery_textures.insert(path, tex);
                }
            }
        }

        // Обробка результату hover-витягування
        let mut hover_results = Vec::new();
        {
            let mut lock = self.video_hover_result.lock().unwrap();
            if !lock.is_empty() {
                hover_results = std::mem::take(&mut *lock);
            }
        }
        for (path, frames) in hover_results {
            self.video_hover_frames.insert(path, frames);
        }

        // Обробка результату thumbnail-витягування
        let mut thumb_results = Vec::new();
        {
            let mut lock = self.video_thumb_result.lock().unwrap();
            if !lock.is_empty() {
                thumb_results = std::mem::take(&mut *lock);
            }
        }
        for (path, tex) in thumb_results {
            self.video_thumbnails.insert(path, tex);
        }

        // Обробка кнопки "Анімувати все"
        if animate_all {
            let anim_loading = std::sync::Arc::clone(&self.gallery_anim_loading);
            for job in &self.jobs {
                let media_dir = std::path::Path::new(&job.settings.save_path).join("media");
                if !media_dir.exists() {
                    continue;
                }
                if let Ok(entries) = std::fs::read_dir(&media_dir) {
                    let images: Vec<_> = entries
                        .filter_map(Result::ok)
                        .filter(|e| {
                            matches!(
                                e.path().extension().and_then(|x| x.to_str()),
                                Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
                            )
                        })
                        .map(|e| e.path())
                        .collect();
                    for img_path in images {
                        if anim_loading.lock().unwrap().contains(&img_path) {
                            continue;
                        }
                        self.gallery_textures.remove(&img_path);
                        crate::core::pipeline::animate_single_image(
                            img_path,
                            self.googler_video_priority.clone(),
                            self.googler_key.clone(),
                            job.id,
                            job.name.clone(),
                            ctx.clone(),
                            std::sync::Arc::clone(&anim_loading),
                            job.settings.googler_video_upscale_enabled,
                            job.settings.googler_video_upscale_resolution.clone(),
                            job.settings.googler_video_upscale_quality.clone(),
                        );
                    }
                }
            }
        }

        // Дренування черги результатів перегенерацій (підтримка паралельних)
        {
            let drained: Vec<_> = self
                .media_regen_results_queue
                .lock()
                .unwrap()
                .drain(..)
                .collect();
            for (path, outcome) in drained {
                match outcome {
                    Ok(()) => {
                        self.gallery_textures.remove(&path);
                        self.video_thumbnails.remove(&path);
                        self.video_hover_frames.remove(&path);
                        self.gallery_image_result
                            .lock()
                            .unwrap()
                            .retain(|(p, _)| p != &path);
                        self.gallery_image_loading.lock().unwrap().remove(&path);

                        if let Some(ref mut editor) = self.montage_editor_state {
                            if let Some(idx) = editor.media_pool.iter().position(|m| m.path == path)
                            {
                                let old_media = editor.media_pool[idx].clone();
                                let old_id = old_media.id.clone();
                                let was_selected = editor.selected_media_ids.remove(&old_id);
                                let was_dragged =
                                    editor.dragged_media_id.as_deref() == Some(old_id.as_str());

                                // Повністю прибираємо старі preview-кеші, бо файл перезаписано тим самим шляхом.
                                let _ = std::fs::remove_dir_all(&old_media.cache_dir);
                                let _ = std::fs::remove_dir_all(&old_media.sharp_cache_dir);
                                let _ = std::fs::remove_file(
                                    crate::gui::montage_editor::embedded_audio_cache_path(
                                        &path,
                                        &editor.save_path,
                                    ),
                                );
                                editor.frame_cache.clear_for_media_id(&old_id);
                                editor.pool_thumbnails.remove(&old_id);

                                let new_media = crate::gui::montage_editor::MediaItem::new(
                                    path.clone(),
                                    &editor.save_path,
                                    editor.preview_render,
                                );
                                let new_id = new_media.id.clone();
                                let new_name = new_media.name.clone();
                                let new_kind = new_media.kind.clone();
                                editor.media_pool[idx] = new_media;

                                // Кліпи мають перейти на новий media_id. Це також відсікає
                                // запізнілі async-текстури старого media_id після перегенерації.
                                let mut extract_embedded_audio = false;
                                for clip in &mut editor.clips {
                                    let same_media = clip.media_id == old_id
                                        || clip.path.as_deref() == Some(path.as_path());
                                    if same_media {
                                        clip.media_id = new_id.clone();
                                        clip.path = Some(path.clone());
                                        if clip.is_embedded_audio {
                                            clip.name = format!("A: {}", new_name);
                                            clip.kind = crate::gui::montage_editor::ClipKind::Audio;
                                            extract_embedded_audio = true;
                                        } else {
                                            clip.name = new_name.clone();
                                            clip.kind = new_kind.clone();
                                        }
                                    }
                                }
                                if extract_embedded_audio {
                                    crate::gui::montage_editor::extract_embedded_audio_async(
                                        path.clone(),
                                        editor.save_path.clone(),
                                    );
                                }

                                if was_selected {
                                    editor.selected_media_ids.insert(new_id.clone());
                                }
                                if was_dragged {
                                    editor.dragged_media_id = Some(new_id.clone());
                                }
                                editor.pool_thumbnails.remove(&new_id);
                                editor.active_audios.clear();
                                let _ = editor.save_to_timeline();
                            } else {
                                // Це новий файл для плейсхолдера — дамо редактору підхопити його
                                // через refresh_placeholder_clips на наступному кадрі.
                                editor.needs_stock_refresh = true;
                            }
                            if editor.pool_preview.as_deref() == Some(path.as_path()) {
                                // Виставляємо stale замість негайного None — дає GPU-бекенду
                                // кадр на звільнення старої текстури перед завантаженням нової
                                editor.pool_preview_texture = None;
                                editor.preview_stale_path = Some(path.clone());
                            }
                            ctx.request_repaint_after(std::time::Duration::from_millis(250));
                        }
                    }
                    Err(e) => {
                        self.media_regen_error = Some(e);
                    }
                }
            }
        }

        // Вікно кастомної перегенерації
        crate::gui::gallery::draw_media_regen_window(
            ctx,
            self.language,
            &mut self.media_regen_window_open,
            &self.media_regen_target,
            &mut self.media_regen_media_type,
            &mut self.media_regen_image_priority,
            &mut self.media_regen_video_priority,
            &mut self.media_regen_prompt,
            &self.media_regen_loading,
            &self.media_regen_base_settings,
            &mut self.media_regen_error,
            self.media_regen_job_id,
            &self.media_regen_job_name,
            &mut self.gallery_textures,
            &self.media_regen_result,
            &self.media_regen_paths,
            &self.media_regen_results_queue,
        );

        // Popup-вікно перегляду промту медіафайлу
        if let Some(ref prompt_text) = self.gallery_prompt_popup.clone() {
            let mut is_open = true;
            egui::Window::new(translate(self.language, "gallery_prompt_window_title"))
                .open(&mut is_open)
                .resizable(true)
                .default_width(420.0)
                .collapsible(false)
                .show(ctx, |ui| {
                    if prompt_text.is_empty() {
                        ui.label(
                            egui::RichText::new(translate(self.language, "gallery_prompt_empty"))
                                .weak(),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.label(prompt_text.as_str());
                            });
                        ui.add_space(8.0);
                        if ui
                            .button(translate(self.language, "gallery_prompt_copy_btn"))
                            .clicked()
                        {
                            ui.output_mut(|o| o.copied_text = prompt_text.clone());
                        }
                    }
                });
            if !is_open {
                self.gallery_prompt_popup = None;
            }
        }
    }
}
