use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{Align2, Color32, Pos2, Rect, ScrollArea, Sense, Stroke, Vec2};
use crate::localization::{Language, translate};
use super::state::MontageEditorState;
use super::types::{ClipKind, DragMode, ClipDragState, EditorClip, OpacityDragState, TrackKind};
use super::utils::uuid_str;

// ─── Таймлінія ───────────────────────────────────────────────────────────────

pub(super) fn draw_timeline(
    ui: &mut egui::Ui,
    language: Language,
    editor: &mut MontageEditorState,
    anim_loading: &Arc<Mutex<HashSet<PathBuf>>>,
    regen_paths: &HashSet<PathBuf>,
) {
    let track_h = 40.0;
    let ruler_h = 22.0;
    let label_w = 110.0;
    let total_dur = editor.total_dur();
    let zoom = editor.timeline_zoom;

    // Drag-handle для зміни висоти панелі таймлайну
    let handle_h = 4.0;
    let (handle_rect, handle_resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), handle_h),
        Sense::drag(),
    );
    let handle_color = if handle_resp.hovered() || handle_resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        Color32::from_rgb(9, 123, 244)
    } else {
        Color32::from_rgb(45, 45, 52)
    };
    ui.painter_at(handle_rect).rect_filled(handle_rect, 0.0, handle_color);
    // Перетягування вгору (delta.y < 0) збільшує висоту, вниз — зменшує
    if handle_resp.dragged() {
        editor.timeline_height = (editor.timeline_height - handle_resp.drag_delta().y)
            .clamp(80.0, 500.0);
    }

    // ─── Кнопки додавання доріжок (як у kadr) ────────────────────────────
    ui.horizontal(|ui| {
        let spacing = 6.0;
        // Кнопка "+ Відео"
        if ui.add_sized([70.0, 20.0], egui::Button::new(translate(language, "montage_editor_add_video_track"))).clicked() {
            // Зсуваємо всі кліпи вниз, щоб нова доріжка з'явилась зверху
            for clip in &mut editor.clips {
                clip.track_idx += 1;
            }
            editor.num_tracks += 1;
            editor.track_kinds.insert(0, TrackKind::Video);
            // Голосова доріжка теж зсувається вниз
            editor.voiceover_track_idx += 1;
        }
        ui.add_space(spacing);
        // Кнопка "+ Аудіо"
        if ui.add_sized([70.0, 20.0], egui::Button::new(translate(language, "montage_editor_add_audio_track"))).clicked() {
            // Аудіо-доріжки додаються знизу (після відео)
            editor.num_tracks += 1;
            editor.track_kinds.push(TrackKind::Audio);
        }
        ui.weak(format!("{} {}", editor.num_tracks, translate(language, "montage_editor_tracks_count")));
    });

    let avail_h = ui.available_height();
    let has_vo = editor.audio_path.is_some();
    let vo_pos = editor.voiceover_track_idx.min(editor.num_tracks);
    let total_rows = editor.num_tracks + if has_vo { 1 } else { 0 };
    let total_tracks_h = ruler_h + (track_h + 2.0) * total_rows as f32;
    let timeline_w = (total_dur + 10.0) * zoom;

    // Візуальна позиція звичайної доріжки з урахуванням голосової
    let track_visual = |ti: usize| -> usize {
        if has_vo && ti >= vo_pos { ti + 1 } else { ti }
    };
    // Y-координата візуальної позиції
    let vis_y = |rect: Rect, vis: usize| -> f32 {
        rect.top() + ruler_h + (track_h + 2.0) * vis as f32
    };

    ui.horizontal(|ui| {
        // Резервуємо місце для sticky-колонки лейблів
        let (labels_rect, _) = ui.allocate_exact_size(
            Vec2::new(label_w, avail_h.min(total_tracks_h)),
            Sense::hover(),
        );

        let output = ScrollArea::both()
            .id_salt("editor_timeline_scroll")
            .max_height(avail_h)
            .auto_shrink([false, true])
            .show(ui, |ui| {

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(timeline_w, total_tracks_h), Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 0.0, Color32::from_rgb(14, 14, 17));

            let ruler_rect = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + ruler_h));
            painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(20, 20, 24));

            let secs_visible = (timeline_w / zoom) as i32 + 2;
            for sec in 0..=secs_visible {
                let x = rect.left() + sec as f32 * zoom;
                let major = sec % 5 == 0;
                let tick_h = if major { 12.0 } else { 6.0 };
                painter.line_segment(
                    [Pos2::new(x, rect.top() + ruler_h - tick_h), Pos2::new(x, rect.top() + ruler_h)],
                    Stroke::new(1.0, Color32::from_rgb(60, 60, 70)),
                );
                if major {
                    let m = (sec / 60) as u32; let s = (sec % 60) as u32;
                    painter.text(
                        Pos2::new(x, rect.top() + 4.0), Align2::CENTER_TOP,
                        format!("{:02}:{:02}", m, s),
                        egui::FontId::proportional(9.0), Color32::from_rgb(110, 110, 120),
                    );
                }
            }

            // Фони доріжок (з інтерлівінгом голосової)
            for vis in 0..total_rows {
                let track_y = vis_y(rect, vis);
                let track_row = Rect::from_min_size(Pos2::new(rect.left(), track_y), Vec2::new(rect.width(), track_h));
                let is_vo_row = has_vo && vis == vo_pos;
                let track_bg = if !is_vo_row && editor.drop_target_track == Some(if vis < vo_pos { vis } else { vis - 1 }) {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 20)
                } else if is_vo_row {
                    Color32::from_rgb(14, 22, 16) // зелений фон для голосової
                } else {
                    let ti = if vis < vo_pos { vis } else { vis - 1 };
                    let kind = editor.track_kinds.get(ti).copied().unwrap_or(TrackKind::Video);
                    match kind {
                        TrackKind::Video => Color32::from_rgb(16, 16, 20),
                        TrackKind::Audio => Color32::from_rgb(14, 22, 16),
                    }
                };
                painter.rect_filled(track_row, 0.0, track_bg);
                painter.line_segment(
                    [Pos2::new(rect.left(), track_y + track_h), Pos2::new(rect.right(), track_y + track_h)],
                    Stroke::new(1.0, Color32::from_rgb(28, 28, 33)),
                );
            }

            let mouse_pos = ui.input(|i| i.pointer.hover_pos());

            // Ghost-preview при drag з медіа-пулу
            if let Some(ref drag_id) = editor.dragged_media_id.clone() {
                if let Some(pos) = mouse_pos {
                    if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                        let click_t = ((pos.x - rect.left()) / zoom).max(0.0);
                        let rel_y = pos.y - (rect.top() + ruler_h);
                        let vis_idx = ((rel_y / (track_h + 2.0)) as usize).min(total_rows - 1);
                        // Конвертуємо візуальну позицію в track_idx, пропускаючи голосову
                        let t_idx = if has_vo && vis_idx == vo_pos {
                            // Не можна кидати на голосову доріжку — беремо найближчу
                            if vis_idx > 0 { vis_idx - 1 } else { 0 }
                        } else {
                            if vis_idx < vo_pos { vis_idx } else { vis_idx - 1 }
                        };
                        editor.drop_target_track = Some(t_idx);

                        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
                            let preview_x = rect.left() + click_t * zoom;
                            let preview_w = (media.duration_secs * zoom).max(4.0);
                            let preview_vis = track_visual(t_idx);
                            let preview_y = vis_y(rect, preview_vis) + 2.0;
                            let preview_rect = Rect::from_min_size(
                                Pos2::new(preview_x, preview_y),
                                Vec2::new(preview_w, track_h - 4.0),
                            );
                            painter.rect_filled(preview_rect, 4.0, Color32::from_rgba_unmultiplied(9, 123, 244, 45));
                            painter.rect_stroke(preview_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(9, 123, 244)));
                            painter.text(
                                Pos2::new(preview_rect.left() + 6.0, preview_rect.top() + 4.0),
                                Align2::LEFT_TOP,
                                format!("{:.1}s → V{}", click_t, t_idx + 1),
                                egui::FontId::proportional(10.0), Color32::WHITE,
                            );
                        }
                        ui.ctx().request_repaint();
                    } else {
                        editor.drop_target_track = None;
                    }
                }
            }

            // Drop з медіа-пулу на таймлінію
            if ui.input(|i| !i.pointer.any_down()) {
                if let Some(drag_id) = editor.dragged_media_id.take() {
                    if let Some(pos) = mouse_pos {
                        if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                            let t_idx = {
                                let rel_y = pos.y - (rect.top() + ruler_h);
                                let vis_idx = ((rel_y / (track_h + 2.0)) as usize).min(total_rows - 1);
                                if has_vo && vis_idx == vo_pos {
                                    if vis_idx > 0 { vis_idx - 1 } else { 0 }
                                } else {
                                    if vis_idx < vo_pos { vis_idx } else { vis_idx - 1 }
                                }
                            };
                            let start = ((pos.x - rect.left()) / zoom).max(0.0);
                            if let Some(media) = editor.media_pool.iter().find(|m| m.id == drag_id) {
                                let kind = media.kind.clone();
                                let name = media.name.clone();
                                let path = Some(media.path.clone());
                                let duration = media.duration_secs;
                                let media_id = media.id.clone();
                                let new_id = uuid_str();
                                editor.selected_clip_id = Some(new_id.clone());
                                editor.clips.push(EditorClip {
                                    id: new_id,
                                    media_id,
                                    path,
                                    name,
                                    start_secs: start,
                                    duration,
                                    track_idx: t_idx,
                                    kind,
                                    scale: 1.0,
                                    pos_x: 0.0,
                                    pos_y: 0.0,
                                    zoom_enabled: false,
                                    shake_enabled: false,
                                    is_placeholder: false,
                                    trim_start: 0.0,
                                    stock_seg_idx: None,
                                    overlap_transition: "fade".to_string(),
                                    opacity: 1.0,
                                });
                            }
                        }
                    }
                    editor.drop_target_track = None;
                }
            }

            // Обробка перетягування кліпів (move / trim)
            update_clip_drag(ui.ctx(), editor, rect, ruler_h, track_h, zoom, has_vo, vo_pos, total_rows);

            // Обробка drag смужки прозорості
            update_opacity_drag(ui, editor);

            // Кліпи
            let clips_snapshot: Vec<EditorClip> = editor.clips.clone();
            for clip in &clips_snapshot {
                let visual_idx = track_visual(clip.track_idx);
                let track_y = vis_y(rect, visual_idx);
                let cx = rect.left() + clip.start_secs * zoom;
                let cw = (clip.duration * zoom).max(4.0);
                let clip_rect = Rect::from_min_size(
                    Pos2::new(cx, track_y + 2.0),
                    Vec2::new(cw, track_h - 4.0),
                );

                // ─── Плейсхолдер (media ще не обрано) ───────────────────────
                if clip.is_placeholder {
                    let seg_idx = clip.media_id.strip_prefix("placeholder_")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    let accent = Color32::from_rgb(90, 90, 115);
                    let is_hovered = mouse_pos.map(|p| clip_rect.contains(p)).unwrap_or(false);
                    let border_color = if is_hovered { Color32::from_rgb(160, 160, 210) } else { accent };
                    painter.rect(clip_rect, 3.0,
                        Color32::from_rgba_unmultiplied(35, 35, 50, 140),
                        Stroke::new(1.5, border_color));
                    // Штрихована заливка
                    let dash_step = 10.0_f32;
                    let num = ((cw / dash_step) as usize).max(1);
                    for d in 0..num {
                        if d % 2 == 0 { continue; }
                        let x0 = clip_rect.left() + d as f32 * dash_step;
                        let x1 = (x0 + dash_step).min(clip_rect.right());
                        painter.rect_filled(
                            Rect::from_min_max(
                                Pos2::new(x0, clip_rect.top() + 4.0),
                                Pos2::new(x1, clip_rect.bottom() - 4.0),
                            ),
                            0.0, Color32::from_rgba_unmultiplied(80, 80, 100, 40),
                        );
                    }
                    if cw > 28.0 {
                        let label = format!("+ #{}", seg_idx + 1);
                        painter.text(
                            clip_rect.center(), Align2::CENTER_CENTER, &label,
                            egui::FontId::proportional(10.0), Color32::from_rgb(160, 160, 190),
                        );
                    }
                    let clip_resp = ui.allocate_rect(clip_rect, Sense::click());
                    if clip_resp.clicked() {
                        editor.selected_clip_id = Some(clip.id.clone());
                        editor.pending_open_stock_picker = Some(seg_idx);
                    }
                    if is_hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    continue;
                }

                let is_anim = clip.path.as_ref()
                    .map(|p| anim_loading.lock().unwrap().contains(p) || regen_paths.contains(p))
                    .unwrap_or(false);

                let is_sel = editor.selected_clip_id.as_deref() == Some(clip.id.as_str());
                // Прозорість: мінімум 30% видимості, щоб кліп не зникав
                let op = clip.opacity.clamp(0.0, 1.0);
                let bg_alpha = ((op * 0.7 + 0.3) * 255.0) as u8;
                let (bg, accent) = match clip.kind {
                    ClipKind::Video => (
                        Color32::from_rgba_unmultiplied(18, 32, 55, bg_alpha),
                        Color32::from_rgb(9, 100, 220),
                    ),
                    ClipKind::Image => (
                        Color32::from_rgba_unmultiplied(30, 22, 48, bg_alpha),
                        Color32::from_rgb(120, 70, 200),
                    ),
                    ClipKind::Audio => (
                        Color32::from_rgba_unmultiplied(20, 40, 28, bg_alpha),
                        Color32::from_rgb(39, 160, 80),
                    ),
                };
                let border = if is_sel { Color32::WHITE } else { accent.linear_multiply(op * 0.7 + 0.3) };
                painter.rect(clip_rect, 3.0, bg, Stroke::new(if is_sel { 2.0 } else { 1.2 }, border));

                // Індикатор оживлення поверх кліпу (painter-based, надійно в ScrollArea)
                if is_anim {
                    painter.rect_filled(clip_rect, 3.0, Color32::from_black_alpha(155));
                    let t = ui.ctx().input(|i| i.time) as f32;
                    let center = clip_rect.center();
                    let r = (cw * 0.5).min(clip_rect.height() * 0.38).min(9.0).max(4.0);
                    let segs = 20usize;
                    for s in 0..segs {
                        let a0 = t * std::f32::consts::TAU + s as f32 * std::f32::consts::TAU / segs as f32;
                        let a1 = a0 + std::f32::consts::TAU / segs as f32;
                        let alpha = (s as f32 / segs as f32 * 200.0) as u8 + 30;
                        painter.line_segment(
                            [center + Vec2::new(a0.cos() * r, a0.sin() * r),
                             center + Vec2::new(a1.cos() * r, a1.sin() * r)],
                            Stroke::new(1.8, Color32::from_rgba_unmultiplied(255, 180, 50, alpha)),
                        );
                    }
                    ui.ctx().request_repaint();
                }

                let handle_w = 6.0;

                // ─── Thumbnail першого кадру ─────────────────────────────────
                if !is_anim && !matches!(clip.kind, ClipKind::Audio) && cw > 32.0 {
                    if let Some(texture) = editor.pool_thumbnails.get(&clip.media_id) {
                        let tex_size = texture.size_vec2();
                        if tex_size.x > 0.0 && tex_size.y > 0.0 {
                            let th = clip_rect.height() - 4.0; // висота thumbnail
                            let tw = (th * tex_size.x / tex_size.y).min(cw - handle_w * 2.0 - 2.0);
                            let img_rect = Rect::from_min_size(
                                Pos2::new(clip_rect.left() + handle_w + 1.0, clip_rect.top() + 2.0),
                                Vec2::new(tw, th),
                            );
                            painter.image(
                                texture.id(),
                                img_rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                // Напівпрозорий щоб фонова кольорова заливка та текст залишались читабельними
                                Color32::from_rgba_unmultiplied(255, 255, 255, 180),
                            );
                        }
                    }
                }

                let handle_col = if is_sel { Color32::WHITE } else { accent.linear_multiply(0.6) };
                painter.rect_filled(
                    Rect::from_min_size(clip_rect.min, Vec2::new(handle_w, clip_rect.height())),
                    2.0, handle_col,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(clip_rect.max.x - handle_w, clip_rect.min.y),
                        Vec2::new(handle_w, clip_rect.height()),
                    ),
                    2.0, handle_col,
                );

                // ─── Смужка прозорості ───────────────────────────────────────
                // Відступ зверху/знизу щоб смужка не зливалась з рамкою кліпу
                let opacity_strip_h = 3.0;
                let pad_top = 4.0;
                let pad_bot = 3.0;
                let inner_h = clip_rect.height() - opacity_strip_h - pad_top - pad_bot;
                let strip_y = clip_rect.top() + pad_top + (1.0 - clip.opacity.clamp(0.0, 1.0)) * inner_h;
                let strip_rect = Rect::from_min_size(
                    Pos2::new(clip_rect.left() + handle_w, strip_y),
                    Vec2::new(clip_rect.width() - handle_w * 2.0, opacity_strip_h),
                );
                let strip_alpha = if is_sel { 230u8 } else { 140u8 };
                painter.rect_filled(strip_rect, 1.0, Color32::from_rgba_unmultiplied(255, 255, 255, strip_alpha));

                if cw > 18.0 {
                    let icon = match clip.kind { ClipKind::Video => "🎬", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                    let label = if clip.name.chars().count() > 16 {
                        format!("{} {}…", icon, clip.name.chars().take(13).collect::<String>())
                    } else {
                        format!("{} {}", icon, clip.name)
                    };
                    painter.text(
                        Pos2::new(clip_rect.left() + handle_w + 2.0, clip_rect.top() + 5.0),
                        Align2::LEFT_TOP, &label,
                        egui::FontId::proportional(10.0), Color32::from_rgb(200, 200, 215),
                    );
                    if cw > 50.0 {
                        painter.text(
                            Pos2::new(clip_rect.left() + handle_w + 2.0, clip_rect.top() + 19.0),
                            Align2::LEFT_TOP, format!("{:.1}s", clip.duration),
                            egui::FontId::proportional(9.0), Color32::from_rgb(120, 120, 130),
                        );
                    }
                }

                // Курсор залежно від зони: горизонтальний trim, вертикальний (смужка opacity)
                if let Some(pos) = mouse_pos {
                    if clip_rect.contains(pos) {
                        let rx = pos.x - clip_rect.left();
                        let ry = pos.y - clip_rect.top();
                        let strip_y_rel = pad_top + (1.0 - clip.opacity.clamp(0.0, 1.0)) * inner_h;
                        let on_strip = (ry - strip_y_rel).abs() < 5.0
                            && rx >= handle_w
                            && rx <= clip_rect.width() - handle_w;
                        if on_strip {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                        } else if rx < 8.0 || rx > clip_rect.width() - 8.0 {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                    }
                }

                let clip_resp = ui.allocate_rect(clip_rect, Sense::click_and_drag());
                if clip_resp.double_clicked() {
                    if let Some(ref path) = clip.path {
                        editor.pool_preview = Some(path.clone());
                        editor.pool_preview_texture = None;
                    }
                } else if clip_resp.clicked() {
                    editor.selected_clip_id = Some(clip.id.clone());
                }
                if clip_resp.drag_started() {
                    // press_origin() — де саме натиснули, а не де зараз курсор
                    let start = ui.input(|i| i.pointer.press_origin()).or(mouse_pos);
                    if let Some(pos) = start {
                        let rx = pos.x - clip_rect.left();
                        let ry = pos.y - clip_rect.top();
                        let strip_y_rel = pad_top + (1.0 - clip.opacity.clamp(0.0, 1.0)) * inner_h;
                        // 8px зони захоплення — надійніший за 5px
                        let on_strip = (ry - strip_y_rel).abs() < 8.0
                            && rx >= handle_w
                            && rx <= clip_rect.width() - handle_w;

                        if on_strip && editor.clip_drag_state.is_none() {
                            editor.opacity_drag = Some(OpacityDragState {
                                clip_id: clip.id.clone(),
                                initial_opacity: clip.opacity,
                                initial_mouse_y: pos.y,
                                clip_height: inner_h,
                            });
                            editor.selected_clip_id = Some(clip.id.clone());
                        } else if editor.opacity_drag.is_none() {
                            let mode = if rx < 8.0 {
                                DragMode::TrimLeft
                            } else if rx > clip_rect.width() - 8.0 {
                                DragMode::TrimRight
                            } else {
                                DragMode::Move
                            };
                            editor.clip_drag_state = Some(ClipDragState {
                                clip_id: clip.id.clone(),
                                mode,
                                initial_start: clip.start_secs,
                                initial_duration: clip.duration,
                                initial_mouse_x: pos.x,
                                initial_track_idx: clip.track_idx,
                                snap_line_secs: None,
                            });
                            editor.selected_clip_id = Some(clip.id.clone());
                        }
                    }
                }

                // Контекстне меню (правий клік)
                if let Some(ref path) = clip.path {
                    let clip_path = path.clone();
                    let clip_kind = clip.kind.clone();
                    let clip_stock_seg = clip.stock_seg_idx;
                    let is_animating = anim_loading.lock().unwrap().contains(&clip_path) || regen_paths.contains(&clip_path);
                    clip_resp.context_menu(|ui| {
                        if let Some(seg_idx) = clip_stock_seg {
                            if ui.button(translate(language, "montage_editor_replace_stock")).clicked() {
                                editor.pending_open_stock_picker = Some(seg_idx);
                                ui.close_menu();
                            }
                            ui.separator();
                        }
                        if matches!(clip_kind, ClipKind::Image) {
                            if is_animating {
                                ui.add_enabled(false, egui::Button::new(format!("⏳ {}", translate(language, "gallery_regen_loading"))));
                            } else if ui.button(translate(language, "montage_editor_animate")).clicked() {
                                editor.pending_animate_paths.push(clip_path.clone());
                                ui.close_menu();
                            }
                            ui.separator();
                        }
                        if ui.button(translate(language, "montage_editor_regen_same")).clicked() {
                            editor.pending_regen = Some((clip_path.clone(), false));
                            ui.close_menu();
                        }
                        if ui.button(translate(language, "montage_editor_regen_custom")).clicked() {
                            editor.pending_regen = Some((clip_path.clone(), true));
                            ui.close_menu();
                        }
                    });
                }
            }

            // ─── Голосова доріжка (інтерлівінг серед звичайних) ────────────
            if let Some(ref ap) = editor.audio_path.clone() {
                let vo_vis = vo_pos.min(total_rows - 1);
                let audio_y = vis_y(rect, vo_vis) + 2.0;
                let audio_w = editor.audio_duration * zoom;
                let audio_x = rect.left() + editor.audio_start_secs * zoom;
                let audio_rect = Rect::from_min_size(Pos2::new(audio_x, audio_y), Vec2::new(audio_w, track_h - 4.0));
                painter.rect(audio_rect, 3.0, Color32::from_rgb(20, 48, 30), Stroke::new(1.2, Color32::from_rgb(39, 174, 96)));

                let audio_resp = ui.allocate_rect(audio_rect, Sense::click_and_drag());
                if audio_resp.dragged() {
                    let delta = audio_resp.drag_delta();
                    // Горизонтальне перетягування: зсув старту
                    editor.audio_start_secs = (editor.audio_start_secs + delta.x / zoom).max(0.0);
                    // Вертикальне перетягування: позиція миші → цільова доріжка
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let rel_y = mouse_pos.y - (rect.top() + ruler_h);
                        let vis_idx = (rel_y / (track_h + 2.0)).max(0.0) as usize;
                        let new_pos = vis_idx.min(editor.num_tracks);
                        editor.voiceover_track_idx = new_pos;
                    }
                    editor.active_audios.clear();
                }

                if audio_w > 20.0 {
                    let aname = ap.file_name().and_then(|n| n.to_str()).unwrap_or("audio");
                    painter.text(
                        Pos2::new(audio_rect.left() + 6.0, audio_rect.top() + 8.0),
                        Align2::LEFT_TOP, format!("♪ {}", aname),
                        egui::FontId::proportional(11.0), Color32::from_rgb(150, 210, 160),
                    );
                }
            }

            // ─── Зони накладання (жовта сітка переходів) ─────────────
            for ti in 0..editor.num_tracks {
                let visual_idx = track_visual(ti);
                let track_y = vis_y(rect, visual_idx);
                // Збираємо кліпи цієї доріжки, сортуємо за start_secs
                let mut track_clips: Vec<&EditorClip> = editor.clips.iter()
                    .filter(|c| c.track_idx == ti)
                    .collect();
                track_clips.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap());

                // Шукаємо накладання (як у kadr TransitionZones)
                for i in 1..track_clips.len() {
                    let b = track_clips[i];
                    let b_start = b.start_secs;
                    let b_end = b.start_secs + b.duration;
                    let mut cover_end = 0.0_f32;
                    for j in 0..i {
                        let a = track_clips[j];
                        let a_end = a.start_secs + a.duration;
                        if a.start_secs < b_start && a_end > b_start {
                            cover_end = cover_end.max(a_end);
                        }
                    }
                    let zone_to = cover_end.min(b_end);
                    if zone_to > b_start + 0.001 {
                        let zx = rect.left() + b_start * zoom;
                        let zw = ((zone_to - b_start) * zoom).max(2.0);
                        let zone_rect = Rect::from_min_size(
                            Pos2::new(zx, track_y + 2.0),
                            Vec2::new(zw, track_h - 4.0),
                        );
                        // Жовта напівпрозора підкладка
                        painter.rect_filled(zone_rect, 3.0,
                            Color32::from_rgba_unmultiplied(255, 182, 72, 22));
                        // Жовта рамка
                        painter.rect_stroke(zone_rect, 3.0,
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 182, 72, 180)));
                        // Діагональна сітка (crosshatch): малюємо X-подібні відрізки
                        let cell = 6.0_f32;
                        let line_color = Color32::from_rgba_unmultiplied(255, 182, 72, 120);
                        let zl = zone_rect.left();
                        let zt = zone_rect.top();
                        let zr = zone_rect.right();
                        let zb = zone_rect.bottom();
                        let mut y = zt;
                        while y < zb {
                            let mut x = zl;
                            while x < zr {
                                let cx = x.min(zr - cell);
                                let cy = y.min(zb - cell);
                                // ╲
                                painter.line_segment(
                                    [Pos2::new(cx, cy), Pos2::new(cx + cell, cy + cell)],
                                    Stroke::new(1.0, line_color),
                                );
                                // ╱
                                painter.line_segment(
                                    [Pos2::new(cx + cell, cy), Pos2::new(cx, cy + cell)],
                                    Stroke::new(1.0, line_color),
                                );
                                x += cell;
                            }
                            y += cell;
                        }

                        // Кнопка вибору переходу ◈ у правій частині зони
                        let btn_size = 14.0;
                        let btn_center = if zw > btn_size + 8.0 {
                            Pos2::new(zr - btn_size / 2.0 - 4.0, track_y + track_h / 2.0)
                        } else {
                            Pos2::new(zl + zw / 2.0 + btn_size / 2.0 + 1.0, track_y + track_h / 2.0)
                        };
                        let btn_rect = Rect::from_center_size(btn_center, Vec2::new(btn_size, btn_size));
                        let btn_hover = ui.ctx().input(|inp| {
                            inp.pointer.hover_pos().map_or(false, |pos| btn_rect.contains(pos))
                        });
                        // Курсор-вказівник при наведенні
                        if btn_hover {
                            ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        let btn_color = if btn_hover {
                            Color32::from_rgba_unmultiplied(255, 182, 72, 240)
                        } else {
                            Color32::from_rgba_unmultiplied(255, 182, 72, 160)
                        };
                        painter.rect_filled(btn_rect, 2.0, btn_color);
                        painter.text(
                            btn_center,
                            Align2::CENTER_CENTER,
                            "◈",
                            egui::FontId::proportional(11.0),
                            Color32::BLACK,
                        );
                        // Клік по кнопці відкриває випадаючий список
                        // primary_pressed (натискання) — щоб уникнути конфлікту
                        // з primary_released (відпускання) у закритті попапу
                        if btn_hover && ui.ctx().input(|inp| inp.pointer.primary_pressed()) {
                            editor.overlap_transition_popup = Some((b.id.clone(), btn_center));
                        }
                    }
                }
            }

            // Плейхед
            let ph_x = rect.left() + editor.playhead * zoom;
            if ph_x >= rect.left() && ph_x <= rect.right() {
                painter.line_segment(
                    [Pos2::new(ph_x, rect.top()), Pos2::new(ph_x, rect.bottom())],
                    Stroke::new(2.0, Color32::from_rgb(9, 123, 244)),
                );
                let ph_top = rect.top() + ruler_h;
                let p1 = Pos2::new(ph_x - 6.0, ph_top - 6.0);
                let p2 = Pos2::new(ph_x + 6.0, ph_top - 6.0);
                let p3 = Pos2::new(ph_x, ph_top);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    Color32::from_rgb(9, 123, 244),
                    Stroke::NONE,
                ));
            }

            // Скраб плейхеда кліком по лінійці
            if let Some(pos) = mouse_pos {
                if ruler_rect.contains(pos) && ui.input(|i| i.pointer.primary_down()) {
                    let new_ph = ((pos.x - rect.left()) / zoom).clamp(0.0, total_dur);
                    if (new_ph - editor.playhead).abs() > 0.05 {
                        editor.active_audios.clear();
                    }
                    editor.playhead = new_ph;
                }
            }

            // Клік по порожньому місцю — знімаємо виділення
            if resp.clicked() {
                let pos = resp.interact_pointer_pos().unwrap_or_default();
                let hit = editor.clips.iter().any(|clip| {
                    let visual_idx = track_visual(clip.track_idx);
                    let track_y = vis_y(rect, visual_idx);
                    let cx = rect.left() + clip.start_secs * zoom;
                    let cw = clip.duration * zoom;
                    let cr = Rect::from_min_size(Pos2::new(cx, track_y + 2.0), Vec2::new(cw, track_h - 4.0));
                    cr.contains(pos)
                });
                if !hit { editor.selected_clip_id = None; }
            }

            // ─── Візуальна лінія снапу при перетягуванні ──────────────
            if let Some(ref drag) = editor.clip_drag_state {
                if let Some(snap_secs) = drag.snap_line_secs {
                    let snap_x = rect.left() + snap_secs * zoom;
                    if snap_x >= rect.left() && snap_x <= rect.right() {
                        let top_y = rect.top() + ruler_h;
                        let total_rows = editor.num_tracks + if has_vo { 1 } else { 0 };
                        let bottom_y = top_y + total_rows as f32 * (track_h + 2.0) + 4.0;
                        let snap_color = Color32::from_rgb(255, 182, 72);
                        painter.line_segment(
                            [Pos2::new(snap_x, top_y), Pos2::new(snap_x, bottom_y)],
                            Stroke::new(2.0, snap_color.gamma_multiply(0.5)),
                        );
                        let mut y = top_y;
                        let step = 8.0;
                        while y < bottom_y {
                            let seg_end = (y + step * 0.5).min(bottom_y);
                            painter.line_segment(
                                [Pos2::new(snap_x, y), Pos2::new(snap_x, seg_end)],
                                Stroke::new(1.5, snap_color),
                            );
                            y += step;
                        }
                    }
                }
            }
        });

        // Sticky лейбли доріжок — малюємо поверх через painter з урахуванням vertical offset
        let v_off = output.state.offset.y;
        let painter = ui.painter_at(labels_rect);
        painter.rect_filled(labels_rect, 0.0, Color32::from_rgb(18, 18, 22));
        painter.rect_filled(
            Rect::from_min_size(labels_rect.min, Vec2::new(label_w, ruler_h)),
            0.0, Color32::from_rgb(20, 20, 24),
        );
        // Рахуємо кількості відео/аудіо доріжок для нумерації
        let mut video_num = 0usize;
        let mut audio_num = 0usize;
        for vis in 0..total_rows {
            let track_y = vis_y(labels_rect, vis) - v_off;
            if track_y + track_h < labels_rect.top() || track_y > labels_rect.bottom() { continue; }
            let is_vo_row = has_vo && vis == vo_pos;
            let (label, bg, border, text_color, bar_color) = if is_vo_row {
                ("♪ Voice".to_string(),
                 Color32::from_rgb(22, 32, 26),
                 Color32::from_rgb(35, 52, 40),
                 Color32::from_rgb(100, 170, 120),
                 Color32::from_rgb(39, 160, 80))
            } else {
                let ti = if vis < vo_pos { vis } else { vis - 1 };
                let kind = editor.track_kinds.get(ti).copied().unwrap_or(TrackKind::Video);
                match kind {
                    TrackKind::Video => {
                        video_num += 1;
                        (format!("V{}", video_num),
                         Color32::from_rgb(28, 28, 32),
                         Color32::from_rgb(42, 42, 48),
                         Color32::from_rgb(160, 160, 170),
                         Color32::from_rgb(9, 100, 220))
                    }
                    TrackKind::Audio => {
                        audio_num += 1;
                        (format!("A{}", audio_num),
                         Color32::from_rgb(22, 32, 26),
                         Color32::from_rgb(35, 52, 40),
                         Color32::from_rgb(100, 170, 120),
                         Color32::from_rgb(39, 160, 80))
                    }
                }
            };
            let lrect = Rect::from_min_size(Pos2::new(labels_rect.left(), track_y), Vec2::new(label_w, track_h));
            painter.rect(lrect, 0.0, bg, Stroke::new(1.0, border));

            // Назва доріжки (верхня частина)
            let name_rect = Rect::from_min_size(lrect.min, Vec2::new(label_w, track_h * 0.6));
            painter.text(name_rect.center(), Align2::CENTER_CENTER, &label, egui::FontId::proportional(11.0), text_color);

            // ─── Повзунок гучності (нижня частина лейблу) ────────────────────
            let ti_opt = if is_vo_row {
                None // голосова
            } else {
                Some(if vis < vo_pos { vis } else { vis - 1 })
            };

            // Поточне значення гучності
            let current_vol = if is_vo_row {
                editor.voiceover_volume
            } else if let Some(ti) = ti_opt {
                editor.track_volumes.get(ti).copied().unwrap_or(1.0)
            } else {
                1.0
            };

            // Малюємо фонову доріжку слайдера
            let slider_y = lrect.top() + track_h * 0.62;
            let slider_h = 3.0;
            let pad_x = 6.0;
            let slider_track = Rect::from_min_size(
                Pos2::new(lrect.left() + pad_x, slider_y),
                Vec2::new(label_w - pad_x * 2.0, slider_h),
            );
            painter.rect_filled(slider_track, 1.5, Color32::from_rgb(40, 40, 48));

            // Заливка від 0 до поточного значення (max = 2.0 → 50% ширини = 100%)
            let fill_w = (current_vol / 2.0).clamp(0.0, 1.0) * (label_w - pad_x * 2.0);
            let fill_color = if (current_vol - 1.0).abs() < 0.05 {
                bar_color
            } else if current_vol > 1.0 {
                Color32::from_rgb(230, 140, 30) // помаранчевий якщо > 100%
            } else {
                bar_color.linear_multiply(0.6)
            };
            if fill_w > 0.5 {
                let fill_rect = Rect::from_min_size(
                    Pos2::new(slider_track.left(), slider_y),
                    Vec2::new(fill_w, slider_h),
                );
                painter.rect_filled(fill_rect, 1.5, fill_color);
            }

            // Мітка 100% (вертикальна лінія посередині)
            let mid_x = slider_track.left() + (label_w - pad_x * 2.0) * 0.5;
            painter.line_segment(
                [Pos2::new(mid_x, slider_y - 2.0), Pos2::new(mid_x, slider_y + slider_h + 2.0)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
            );

            // Ручка на поточній позиції
            let handle_x = slider_track.left() + fill_w;
            let handle_center = Pos2::new(handle_x, slider_y + slider_h * 0.5);
            painter.circle_filled(handle_center, 5.0, fill_color);
            painter.circle_stroke(handle_center, 5.0, Stroke::new(1.0, Color32::from_rgb(180, 180, 200)));

            // Інтерактивна зона (трохи ширша за візуальний слайдер)
            let interact_rect = Rect::from_min_size(
                Pos2::new(lrect.left(), slider_y - 6.0),
                Vec2::new(label_w, slider_h + 12.0),
            );
            let bar_resp = ui.allocate_rect(interact_rect, Sense::drag());

            if bar_resp.dragged() {
                // Обчислюємо нову гучність за x-позицією кліка/перетягування
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let rel_x = (pos.x - slider_track.left()).clamp(0.0, label_w - pad_x * 2.0);
                    let new_vol = (rel_x / (label_w - pad_x * 2.0)) * 2.0;
                    let new_vol = new_vol.clamp(0.0, 2.0);
                    if is_vo_row {
                        editor.voiceover_volume = new_vol;
                    } else if let Some(ti) = ti_opt {
                        while editor.track_volumes.len() <= ti { editor.track_volumes.push(1.0); }
                        editor.track_volumes[ti] = new_vol;
                    }
                    // Зупиняємо активне аудіо щоб воно перезапустилось з новою гучністю
                    editor.active_audios.retain(|a| {
                        if is_vo_row {
                            editor.audio_path.as_deref() != Some(a.path.as_path())
                        } else if let Some(ti) = ti_opt {
                            // Зупиняємо аудіо кліпи цієї доріжки
                            !editor.clips.iter().any(|c| c.track_idx == ti && c.path.as_deref() == Some(a.path.as_path()))
                        } else {
                            true
                        }
                    });
                }
                ui.ctx().request_repaint();
            }
            if bar_resp.drag_stopped() {
                editor.save_to_timeline().ok();
            }

            // Tooltip з поточним значенням
            // Курсор при наведенні на слайдер
            if bar_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            bar_resp.on_hover_text(format!("Vol: {:.0}%", current_vol * 100.0));
        }
    });

    // ─── Випадаючий список вибору overlap-переходу ─────────────────────
    if let Some((ref clip_id, pos)) = editor.overlap_transition_popup.clone() {
        // Знаходимо поточний overlap_transition для цього кліпу
        let current_trans = editor.clips.iter()
            .find(|c| c.id == *clip_id)
            .map(|c| c.overlap_transition.clone())
            .unwrap_or_else(|| "fade".to_string());

        let popup_id = egui::Id::new("overlap_transition_popup");
        let mut close_popup = false;

        egui::Area::new(popup_id)
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                // Обмежуємо попап висотою екрану з прокруткою
                let screen_h = ui.ctx().screen_rect().height();
                let max_popup_h = (screen_h - pos.y - 20.0).max(100.0).min(400.0);
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(translate(language, "montage_editor_overlap_transition"));
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(max_popup_h)
                        .show(ui, |ui| {
                            for &name in crate::gui::pipeline::editing::XFADE_TRANSITIONS {
                                let selected = current_trans == name;
                                if ui.selectable_label(selected, name).clicked() {
                                    // Оновлюємо overlap_transition для incoming кліпу
                                    if let Some(clip) = editor.clips.iter_mut().find(|c| c.id == *clip_id) {
                                        clip.overlap_transition = name.to_string();
                                        editor.save_to_timeline().ok();
                                    }
                                    close_popup = true;
                                }
                            }
                        });
                });
            });

        // Закриваємо попап при натисканні Escape
        if ui.ctx().input(|inp| inp.key_pressed(egui::Key::Escape)) {
            close_popup = true;
        }

        if close_popup {
            editor.overlap_transition_popup = None;
        }
    }
}

// ─── Логіка перетягування смужки прозорості ─────────────────────────────────

fn update_opacity_drag(ui: &mut egui::Ui, editor: &mut MontageEditorState) {
    if editor.opacity_drag.is_none() { return; }

    let released = ui.input(|i| !i.pointer.any_down());
    if released {
        editor.opacity_drag = None;
        return;
    }

    // Поки opacity_drag активний — clip_drag не повинен запускатись
    editor.clip_drag_state = None;

    let Some(ref state) = editor.opacity_drag else { return };
    let Some(pos) = ui.input(|i| i.pointer.hover_pos()) else { return };

    let dy = pos.y - state.initial_mouse_y;
    // Рух вниз → зменшення прозорості, вгору → збільшення
    let new_opacity = (state.initial_opacity - dy / state.clip_height.max(1.0)).clamp(0.0, 1.0);
    let clip_id = state.clip_id.clone();
    if let Some(clip) = editor.clips.iter_mut().find(|c| c.id == clip_id) {
        clip.opacity = new_opacity;
    }
    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    ui.ctx().request_repaint();
}

// ─── Логіка перетягування кліпів (move / trim) ───────────────────────────────

fn update_clip_drag(
    ctx: &egui::Context,
    editor: &mut MontageEditorState,
    rect: Rect,
    ruler_h: f32,
    track_h: f32,
    zoom: f32,
    has_vo: bool,
    vo_pos: usize,
    _total_rows: usize,
) {
    // Якщо opacity_drag активний — clip_drag не обробляємо
    if editor.opacity_drag.is_some() { return; }

    if ctx.input(|i| i.pointer.any_released()) {
        editor.clip_drag_state = None;
        return;
    }

    let drag = match &editor.clip_drag_state {
        Some(d) => ClipDragState {
            clip_id: d.clip_id.clone(),
            mode: d.mode,
            initial_start: d.initial_start,
            initial_duration: d.initial_duration,
            initial_mouse_x: d.initial_mouse_x,
            initial_track_idx: d.initial_track_idx,
            snap_line_secs: d.snap_line_secs,
        },
        None => return,
    };

    let pos = match ctx.input(|i| i.pointer.hover_pos()) {
        Some(p) => p,
        None => return,
    };

    let dx = (pos.x - drag.initial_mouse_x) / zoom;

    let clip_idx = match editor.clips.iter().position(|c| c.id == drag.clip_id) {
        Some(idx) => idx,
        None => return,
    };

    let dy = pos.y - (rect.top() + ruler_h);
    let vis_idx = (dy / (track_h + 2.0)).max(0.0) as usize;
    // Конвертуємо візуальну позицію в track_idx, пропускаючи голосову
    let new_track = if has_vo && vis_idx == vo_pos {
        // На голосову доріжку не можна — беремо найближчу
        if vis_idx > 0 { vis_idx - 1 } else { 0 }
    } else {
        let ti = if vis_idx < vo_pos { vis_idx } else { vis_idx - 1 };
        ti.min(editor.num_tracks - 1)
    };

    let raw_start = (drag.initial_start + dx).max(0.0);
    let clip_dur = drag.initial_duration;
    let raw_end = raw_start + clip_dur;

    // Магнітний snap до сусідніх кліпів + плейхеда
    let snap_threshold = 10.0 / zoom; // ~10 px
    let mut best_snap: Option<f32> = None;
    let mut best_dist = snap_threshold;
    let mut snap_is_end = false; // true якщо снап по кінцю кліпу

    // Snap до плейхеда (початок або кінець кліпу)
    let ph = editor.playhead;
    let d_ph_start = (raw_start - ph).abs();
    if d_ph_start < best_dist { best_dist = d_ph_start; best_snap = Some(ph); snap_is_end = false; }
    let d_ph_end = (raw_end - ph).abs();
    if d_ph_end < best_dist { best_dist = d_ph_end; best_snap = Some((ph - clip_dur).max(0.0)); snap_is_end = true; }

    for (i, other) in editor.clips.iter().enumerate() {
        if i == clip_idx { continue; }
        // Снап тільки в межах тієї ж доріжки
        if other.track_idx != new_track { continue; }

        let other_end = other.start_secs + other.duration;

        // Початок кліпу до кінця іншого (стиковка)
        let d1 = (raw_start - other_end).abs();
        if d1 < best_dist { best_dist = d1; best_snap = Some(other_end); snap_is_end = false; }
        // Початок кліпу до початку іншого (вирівнювання)
        let d2 = (raw_start - other.start_secs).abs();
        if d2 < best_dist { best_dist = d2; best_snap = Some(other.start_secs); snap_is_end = false; }
        // Кінець кліпу до початку іншого (стиковка справа)
        let d3 = (raw_end - other.start_secs).abs();
        if d3 < best_dist { best_dist = d3; best_snap = Some((other.start_secs - clip_dur).max(0.0)); snap_is_end = true; }
    }

    // Візуальний індикатор снапу — позиція на таймлінії де краї збігаються
    let snapped_start = best_snap.unwrap_or(raw_start);
    if let Some(ref mut ds) = editor.clip_drag_state {
        ds.snap_line_secs = best_snap.map(|s| if snap_is_end { s + clip_dur } else { s });
    }

    let clip = &mut editor.clips[clip_idx];
    match drag.mode {
        DragMode::Move => {
            clip.start_secs = snapped_start;
            clip.track_idx = new_track;
        }
        DragMode::TrimLeft => {
            let max_dx = drag.initial_duration - 0.1;
            let bounded_dx = dx.clamp(-drag.initial_start, max_dx);
            clip.start_secs = drag.initial_start + bounded_dx;
            clip.duration = drag.initial_duration - bounded_dx;
        }
        DragMode::TrimRight => {
            clip.duration = (drag.initial_duration + dx).max(0.1);
        }
    }

    ctx.request_repaint();
}
