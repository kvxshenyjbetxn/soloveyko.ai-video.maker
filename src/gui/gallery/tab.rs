use super::{
    RegenAction,
    icons::{draw_eye_icon, draw_menu_icon, draw_play_triangle, draw_refresh_icon},
};
use crate::localization::translate;
use eframe::egui;

/// Малює вкладку галереї медіафайлів із деревом задач.
pub fn draw_gallery_tab(
    ui: &mut egui::Ui,
    language: crate::localization::Language,
    jobs: &[crate::queue::PipelineJob],
    gallery_textures: &mut std::collections::HashMap<
        std::path::PathBuf,
        Option<egui::TextureHandle>,
    >,
    gallery_preview: &mut Option<std::path::PathBuf>,
    regen_paths: &std::collections::HashSet<std::path::PathBuf>,
    regen_action: &mut Option<RegenAction>,
    anim_loading: &std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    animate_all: &mut bool,
    video_hover_frames: &std::collections::HashMap<std::path::PathBuf, Vec<egui::TextureHandle>>,
    video_hover_state: &mut std::collections::HashMap<
        std::path::PathBuf,
        (usize, std::time::Instant),
    >,
    video_hover_loading: &std::collections::HashSet<std::path::PathBuf>,
    hover_extract_request: &mut Option<std::path::PathBuf>,
    video_thumbnails: &std::collections::HashMap<std::path::PathBuf, Option<egui::TextureHandle>>,
    video_thumb_loading: &std::collections::HashSet<std::path::PathBuf>,
    thumb_requests: &mut Vec<std::path::PathBuf>,
    prompt_view_request: &mut Option<std::path::PathBuf>,
    image_load_requests: &mut Vec<std::path::PathBuf>,
    image_loading: &std::collections::HashSet<std::path::PathBuf>,
) -> bool {
    let awaiting: Vec<_> = jobs
        .iter()
        .filter(|j| *j.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMediaControl)
        .collect();

    let mut switch_to_main = false;

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new(translate(language, "gallery_animate_all_btn")).strong())
            .clicked()
        {
            *animate_all = true;
        }
        if !awaiting.is_empty() {
            if ui
                .button(egui::RichText::new(translate(language, "gallery_continue_btn")).strong())
                .clicked()
            {
                for job in awaiting {
                    let (lock, cvar) = &*job.media_control_resume;
                    *lock.lock().unwrap() = true;
                    cvar.notify_one();
                }
                switch_to_main = true;
            }
        }
    });
    ui.add_space(4.0);

    // Збираємо дані задач та їхніх медіафайлів заздалегідь, щоб уникнути borrow конфліктів
    let mut job_media: Vec<(
        u64,
        String,
        Vec<std::path::PathBuf>,
        bool,
        crate::queue::JobSettings,
    )> = Vec::new();
    for job in jobs {
        let media_dir = std::path::Path::new(&job.settings.save_path).join("media");
        if !media_dir.exists() {
            continue;
        }

        let mut files: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&media_dir) {
            let mut sorted: Vec<_> = entries
                .filter_map(Result::ok)
                .filter(|e| {
                    let p = e.path();
                    matches!(
                        p.extension().and_then(|x| x.to_str()),
                        Some("jpg")
                            | Some("jpeg")
                            | Some("png")
                            | Some("webp")
                            | Some("mp4")
                            | Some("webm")
                            | Some("mov")
                    )
                })
                .collect();
            sorted.sort_by_key(|e| e.path());
            files = sorted.into_iter().map(|e| e.path()).collect();
        }

        if files.is_empty() {
            continue;
        }

        let is_awaiting =
            *job.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMediaControl;
        job_media.push((
            job.id,
            job.name.clone(),
            files,
            is_awaiting,
            job.settings.clone(),
        ));
    }

    if job_media.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(translate(language, "gallery_empty"))
                    .weak()
                    .size(14.0),
            );
        });
        return false;
    }

    let anim_set = anim_loading.lock().unwrap().clone();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (job_id, job_name, files, is_awaiting, job_settings) in &job_media {
                let header_id = ui.make_persistent_id(format!("gallery_job_{}", job_id));
                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    header_id,
                    true,
                );

                let header = ui.horizontal(|ui| {
                    state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                    let label = ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "#{} {} ({})",
                                job_id + 1,
                                job_name,
                                files.len()
                            ))
                            .strong(),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if *is_awaiting {
                        ui.label(
                            egui::RichText::new(translate(language, "gallery_awaiting_label"))
                                .color(egui::Color32::from_rgb(230, 126, 34))
                                .size(12.0),
                        );
                    }
                    label
                });

                if header.inner.clicked() {
                    state.toggle(ui);
                }
                state.store(ui.ctx());

                state.show_body_indented(&header.response, ui, |ui| {
                    let thumb_size = 120.0;
                    let spacing = 8.0;
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                    ui.horizontal_wrapped(|ui| {
                        for (idx, file_path) in files.iter().enumerate() {
                            let is_video = matches!(
                                file_path.extension().and_then(|e| e.to_str()),
                                Some("mp4") | Some("webm") | Some("mov")
                            );

                            // Обчислюємо раніше, щоб не запускати завантаження під час регенерації
                            let is_animating = anim_set.contains(file_path);
                            let this_regen = regen_paths.contains(file_path);

                            let img_resp = if is_video {
                                let display = egui::vec2(thumb_size * (16.0 / 9.0), thumb_size);
                                let (resp, painter) =
                                    ui.allocate_painter(display, egui::Sense::click());

                                let is_hovered = resp.hovered();
                                let uv = egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                );

                                if !this_regen
                                    && !video_thumbnails.contains_key(file_path)
                                    && !video_thumb_loading.contains(file_path)
                                {
                                    thumb_requests.push(file_path.clone());
                                }

                                if is_hovered {
                                    if let Some(frames) = video_hover_frames.get(file_path) {
                                        if !frames.is_empty() {
                                            let state = video_hover_state
                                                .entry(file_path.clone())
                                                .or_insert((0, std::time::Instant::now()));
                                            let frame_dur = std::time::Duration::from_millis(250);
                                            if state.1.elapsed() >= frame_dur {
                                                state.0 = (state.0 + 1) % frames.len();
                                                state.1 = std::time::Instant::now();
                                            }
                                            ui.ctx().request_repaint_after(
                                                frame_dur.saturating_sub(state.1.elapsed()),
                                            );
                                            painter.image(
                                                frames[state.0].id(),
                                                resp.rect,
                                                uv,
                                                egui::Color32::WHITE,
                                            );
                                        } else {
                                            painter.rect_filled(
                                                resp.rect,
                                                4.0,
                                                egui::Color32::from_gray(25),
                                            );
                                        }
                                    } else {
                                        if !video_hover_loading.contains(file_path)
                                            && hover_extract_request.is_none()
                                        {
                                            *hover_extract_request = Some(file_path.clone());
                                        }
                                        if let Some(Some(thumb)) = video_thumbnails.get(file_path) {
                                            painter.image(
                                                thumb.id(),
                                                resp.rect,
                                                uv,
                                                egui::Color32::WHITE,
                                            );
                                        } else {
                                            painter.rect_filled(
                                                resp.rect,
                                                4.0,
                                                egui::Color32::from_gray(25),
                                            );
                                        }
                                    }
                                } else {
                                    if let Some(Some(thumb)) = video_thumbnails.get(file_path) {
                                        painter.image(
                                            thumb.id(),
                                            resp.rect,
                                            uv,
                                            egui::Color32::WHITE,
                                        );
                                    } else {
                                        painter.rect_filled(
                                            resp.rect,
                                            4.0,
                                            egui::Color32::from_gray(25),
                                        );
                                    }
                                }

                                draw_play_triangle(&painter, resp.rect.center(), 13.0);

                                resp
                            } else {
                                // Не завантажуємо під час активної регенерації — уникаємо гонки
                                if !this_regen && !gallery_textures.contains_key(file_path) {
                                    if !image_loading.contains(file_path) {
                                        image_load_requests.push(file_path.clone());
                                        // Резервуємо місце щоб уникнути повторних запитів
                                        gallery_textures.insert(file_path.clone(), None);
                                    }
                                }
                                if let Some(Some(tex)) = gallery_textures.get(file_path) {
                                    let img_size = tex.size_vec2();
                                    let aspect = if img_size.y > 0.0 {
                                        img_size.x / img_size.y
                                    } else {
                                        1.0
                                    };
                                    let display = egui::vec2(thumb_size * aspect, thumb_size);
                                    ui.add(
                                        egui::Image::from_texture(tex)
                                            .fit_to_exact_size(display)
                                            .sense(egui::Sense::click()),
                                    )
                                } else {
                                    ui.add_sized(
                                        [thumb_size, thumb_size],
                                        egui::Label::new(
                                            egui::RichText::new(format!("#{}", idx + 1)).weak(),
                                        ),
                                    )
                                }
                            };

                            if is_animating || this_regen {
                                ui.painter().rect_filled(
                                    img_resp.rect,
                                    0.0,
                                    egui::Color32::from_black_alpha(130),
                                );
                                ui.put(img_resp.rect, egui::Spinner::new());
                            } else if !is_video {
                                // Оверлей іконок (тільки для зображень): ока | refresh | menu
                                let bw = 22.0;
                                let gap = 3.0;
                                let pad = 4.0;
                                let custom_rect = egui::Rect::from_min_size(
                                    egui::pos2(
                                        img_resp.rect.right() - bw - pad,
                                        img_resp.rect.bottom() - bw - pad,
                                    ),
                                    egui::vec2(bw, bw),
                                );
                                let same_rect = egui::Rect::from_min_size(
                                    egui::pos2(
                                        img_resp.rect.right() - bw * 2.0 - gap - pad,
                                        img_resp.rect.bottom() - bw - pad,
                                    ),
                                    egui::vec2(bw, bw),
                                );
                                let prompt_rect = egui::Rect::from_min_size(
                                    egui::pos2(
                                        img_resp.rect.right() - bw * 3.0 - gap * 2.0 - pad,
                                        img_resp.rect.bottom() - bw - pad,
                                    ),
                                    egui::vec2(bw, bw),
                                );

                                let same_resp = ui.interact(
                                    same_rect,
                                    egui::Id::new(("gs", *job_id, idx)),
                                    egui::Sense::click(),
                                );
                                let custom_resp = ui.interact(
                                    custom_rect,
                                    egui::Id::new(("gc", *job_id, idx)),
                                    egui::Sense::click(),
                                );
                                let prompt_resp = ui.interact(
                                    prompt_rect,
                                    egui::Id::new(("gp", *job_id, idx)),
                                    egui::Sense::click(),
                                );

                                let bg_n = egui::Color32::from_black_alpha(150);
                                let bg_h = egui::Color32::from_black_alpha(220);
                                let painter = ui.painter();
                                painter.rect_filled(
                                    same_rect,
                                    4.0,
                                    if same_resp.hovered() { bg_h } else { bg_n },
                                );
                                painter.rect_filled(
                                    custom_rect,
                                    4.0,
                                    if custom_resp.hovered() { bg_h } else { bg_n },
                                );
                                painter.rect_filled(
                                    prompt_rect,
                                    4.0,
                                    if prompt_resp.hovered() { bg_h } else { bg_n },
                                );

                                let col = egui::Color32::WHITE;
                                draw_refresh_icon(
                                    painter,
                                    same_rect.center(),
                                    5.5,
                                    egui::Stroke::new(1.5, col),
                                );
                                draw_menu_icon(
                                    painter,
                                    custom_rect.center(),
                                    5.0,
                                    egui::Stroke::new(1.5, col),
                                );
                                draw_eye_icon(
                                    painter,
                                    prompt_rect.center(),
                                    5.0,
                                    egui::Stroke::new(1.2, col),
                                );

                                if same_resp
                                    .on_hover_text(translate(
                                        language,
                                        "gallery_regen_same_tooltip",
                                    ))
                                    .clicked()
                                {
                                    *regen_action = Some((
                                        file_path.clone(),
                                        job_settings.clone(),
                                        false,
                                        *job_id,
                                        job_name.clone(),
                                    ));
                                } else if custom_resp
                                    .on_hover_text(translate(
                                        language,
                                        "gallery_regen_custom_tooltip",
                                    ))
                                    .clicked()
                                {
                                    *regen_action = Some((
                                        file_path.clone(),
                                        job_settings.clone(),
                                        true,
                                        *job_id,
                                        job_name.clone(),
                                    ));
                                } else if prompt_resp
                                    .on_hover_text(translate(language, "gallery_prompt_tooltip"))
                                    .clicked()
                                {
                                    *prompt_view_request = Some(file_path.clone());
                                } else if img_resp.clicked() {
                                    *gallery_preview = Some(file_path.clone());
                                }
                            } else if img_resp.clicked() {
                                *gallery_preview = Some(file_path.clone());
                            }
                        }
                    });

                    ui.add_space(8.0);
                });

                ui.add_space(4.0);
            }
        });

    switch_to_main
}
