use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{Align2, Color32, Pos2, Rect, ScrollArea, Sense, Stroke, Vec2};
use crate::localization::{Language, translate};
use super::state::MontageEditorState;
use super::types::{ClipKind, DragMode, ClipDragState, EditorClip};
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
    let label_w = 70.0;
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

    ui.horizontal(|ui| {
        if ui.small_button(translate(language, "montage_editor_add_track")).clicked() {
            // Зсуваємо всі кліпи вниз, щоб нова доріжка V1 з'явилась зверху
            for clip in &mut editor.clips {
                clip.track_idx += 1;
            }
            editor.num_tracks += 1;
        }
        ui.weak(format!("{} {}", editor.num_tracks, translate(language, "montage_editor_tracks_count")));
    });

    let avail_h = ui.available_height();
    let total_audio_tracks = if editor.audio_path.is_some() { 1 } else { 0 };
    let total_tracks_h = ruler_h + (track_h + 2.0) * (editor.num_tracks + total_audio_tracks) as f32;
    let timeline_w = (total_dur + 10.0) * zoom;

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

            for track_idx in 0..editor.num_tracks {
                let track_y = rect.top() + ruler_h + (track_h + 2.0) * track_idx as f32;
                let track_row = Rect::from_min_size(Pos2::new(rect.left(), track_y), Vec2::new(rect.width(), track_h));
                let track_bg = if editor.drop_target_track == Some(track_idx) {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 20)
                } else {
                    Color32::from_rgb(16, 16, 20)
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
                        let t_idx = ((rel_y / (track_h + 2.0)) as usize).min(editor.num_tracks - 1);
                        editor.drop_target_track = Some(t_idx);

                        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
                            let preview_x = rect.left() + click_t * zoom;
                            let preview_w = (media.duration_secs * zoom).max(4.0);
                            let preview_y = rect.top() + ruler_h + (track_h + 2.0) * t_idx as f32 + 2.0;
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
                                ((rel_y / (track_h + 2.0)) as usize).min(editor.num_tracks - 1)
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
                                });
                            }
                        }
                    }
                    editor.drop_target_track = None;
                }
            }

            // Обробка перетягування кліпів (move / trim)
            update_clip_drag(ui.ctx(), editor, rect, ruler_h, track_h, zoom);

            // Кліпи
            let clips_snapshot: Vec<EditorClip> = editor.clips.clone();
            for clip in &clips_snapshot {
                let track_y = rect.top() + ruler_h + (track_h + 2.0) * clip.track_idx as f32;
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
                let (bg, accent) = match clip.kind {
                    ClipKind::Video => (Color32::from_rgb(18, 32, 55), Color32::from_rgb(9, 100, 220)),
                    ClipKind::Image => (Color32::from_rgb(30, 22, 48), Color32::from_rgb(120, 70, 200)),
                    ClipKind::Audio => (Color32::from_rgb(20, 40, 28), Color32::from_rgb(39, 160, 80)),
                };
                let border = if is_sel { Color32::WHITE } else { accent };
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

                if let Some(pos) = mouse_pos {
                    if clip_rect.contains(pos) {
                        let rx = pos.x - clip_rect.left();
                        if rx < 8.0 || rx > clip_rect.width() - 8.0 {
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
                    if let Some(pos) = mouse_pos {
                        let rx = pos.x - clip_rect.left();
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
                        });
                        editor.selected_clip_id = Some(clip.id.clone());
                    }
                }

                // Контекстне меню (правий клік)
                if let Some(ref path) = clip.path {
                    let clip_path = path.clone();
                    let clip_kind = clip.kind.clone();
                    let is_animating = anim_loading.lock().unwrap().contains(&clip_path) || regen_paths.contains(&clip_path);
                    clip_resp.context_menu(|ui| {
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

            // Аудіо трек
            if let Some(ref ap) = editor.audio_path {
                let audio_y = rect.top() + ruler_h + (track_h + 2.0) * editor.num_tracks as f32 + 2.0;
                let audio_row = Rect::from_min_size(Pos2::new(rect.left(), audio_y - 2.0), Vec2::new(rect.width(), track_h));
                painter.rect_filled(audio_row, 0.0, Color32::from_rgb(14, 22, 16));
                let audio_w = editor.audio_duration * zoom;
                let audio_x = rect.left() + editor.audio_start_secs * zoom;
                let audio_rect = Rect::from_min_size(Pos2::new(audio_x, audio_y), Vec2::new(audio_w, track_h - 4.0));
                painter.rect(audio_rect, 3.0, Color32::from_rgb(20, 48, 30), Stroke::new(1.2, Color32::from_rgb(39, 174, 96)));

                let audio_resp = ui.allocate_rect(audio_rect, Sense::click_and_drag());
                if audio_resp.dragged() {
                    let delta_x = audio_resp.drag_delta().x;
                    editor.audio_start_secs = (editor.audio_start_secs + delta_x / zoom).max(0.0);
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
                    let track_y = rect.top() + ruler_h + (track_h + 2.0) * clip.track_idx as f32;
                    let cx = rect.left() + clip.start_secs * zoom;
                    let cw = clip.duration * zoom;
                    let cr = Rect::from_min_size(Pos2::new(cx, track_y + 2.0), Vec2::new(cw, track_h - 4.0));
                    cr.contains(pos)
                });
                if !hit { editor.selected_clip_id = None; }
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
        for track_idx in 0..editor.num_tracks {
            let track_y = labels_rect.top() + ruler_h + (track_h + 2.0) * track_idx as f32 - v_off;
            if track_y + track_h < labels_rect.top() || track_y > labels_rect.bottom() { continue; }
            let lrect = Rect::from_min_size(Pos2::new(labels_rect.left(), track_y), Vec2::new(label_w, track_h));
            painter.rect(lrect, 0.0, Color32::from_rgb(28, 28, 32), Stroke::new(1.0, Color32::from_rgb(42, 42, 48)));
            painter.text(lrect.center(), Align2::CENTER_CENTER, &format!("V{}", track_idx + 1), egui::FontId::proportional(11.0), Color32::from_rgb(160, 160, 170));
        }
        if editor.audio_path.is_some() {
            let audio_y = labels_rect.top() + ruler_h + (track_h + 2.0) * editor.num_tracks as f32 + 2.0 - v_off;
            if audio_y + track_h >= labels_rect.top() && audio_y <= labels_rect.bottom() {
                let arect = Rect::from_min_size(Pos2::new(labels_rect.left(), audio_y), Vec2::new(label_w, track_h));
                painter.rect(arect, 0.0, Color32::from_rgb(22, 32, 26), Stroke::new(1.0, Color32::from_rgb(35, 52, 40)));
                painter.text(arect.center(), Align2::CENTER_CENTER, "♪ Audio", egui::FontId::proportional(11.0), Color32::from_rgb(100, 170, 120));
            }
        }
    });
}

// ─── Логіка перетягування кліпів (move / trim) ───────────────────────────────

fn update_clip_drag(
    ctx: &egui::Context,
    editor: &mut MontageEditorState,
    rect: Rect,
    ruler_h: f32,
    track_h: f32,
    zoom: f32,
) {
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
    let row_idx = (dy / (track_h + 2.0)).max(0.0) as usize;
    let new_track = row_idx.min(editor.num_tracks - 1);

    let raw_start = (drag.initial_start + dx).max(0.0);
    let clip_dur = drag.initial_duration;
    let raw_end = raw_start + clip_dur;

    // Магнітний snap до сусідніх кліпів
    let snap_threshold = 8.0 / zoom;
    let mut best_snap: Option<f32> = None;
    let mut best_dist = snap_threshold;

    for (i, other) in editor.clips.iter().enumerate() {
        if i == clip_idx { continue; }

        let other_end = other.start_secs + other.duration;

        let d1 = (raw_start - other_end).abs();
        if d1 < best_dist { best_dist = d1; best_snap = Some(other_end); }

        let d2 = (raw_start - other.start_secs).abs();
        if d2 < best_dist { best_dist = d2; best_snap = Some(other.start_secs); }

        let d3 = (raw_end - other.start_secs).abs();
        if d3 < best_dist { best_dist = d3; best_snap = Some((other.start_secs - clip_dur).max(0.0)); }
    }

    let snapped_start = best_snap.unwrap_or(raw_start);

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
