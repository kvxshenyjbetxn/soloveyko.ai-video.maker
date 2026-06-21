use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{Align2, Color32, Pos2, Rect, ScrollArea, Sense, Stroke, Vec2};
use crate::localization::{Language, translate};
use super::state::MontageEditorState;
use super::types::{ClipKind, DragMode, ClipDragState, EditorClip, OpacityDragState, TrackDragState, TrackKind};
use super::utils::uuid_str;

// ─── Розріз кліпу ──────────────────────────────────────────────────────────

/// Розрізає кліп на два в точці `split_time` (секунди).
/// Якщо кліп має парне аудіо — розрізає і його.
/// Повертає true якщо розріз виконано успішно.
fn split_clip_at(editor: &mut MontageEditorState, clip_id: &str, split_time: f32) -> bool {
    // Знаходимо індекс кліпу
    let idx = match editor.clips.iter().position(|c| c.id == clip_id) {
        Some(i) => i,
        None => return false,
    };

    // Збираємо всі необхідні дані до мутації
    let (clip_start, clip_end, pair_id, audio_linked, is_emb_audio) = {
        let clip = &editor.clips[idx];
        let start = clip.start_secs;
        let end = clip.end_secs();
        let pid = clip.pair_id.clone();
        let linked = clip.audio_linked;
        let emb = clip.is_embedded_audio;
        (start, end, pid, linked, emb)
    };

    // Перевіряємо що точка розрізу всередині кліпу (з запасом 0.05с)
    if split_time <= clip_start + 0.05 || split_time >= clip_end - 0.05 {
        return false;
    }

    editor.push_undo();

    // Дані для створення правого кліпу (клонуємо)
    let right_clip = {
        let clip = &editor.clips[idx];
        EditorClip {
            id: uuid_str(),
            media_id: clip.media_id.clone(),
            path: clip.path.clone(),
            name: clip.name.clone(),
            start_secs: split_time,
            duration: clip_end - split_time,
            track_idx: clip.track_idx,
            kind: clip.kind.clone(),
            scale: clip.scale,
            pos_x: clip.pos_x,
            pos_y: clip.pos_y,
            zoom_enabled: clip.zoom_enabled,
            shake_enabled: clip.shake_enabled,
            is_placeholder: false,
            trim_start: clip.trim_start + (split_time - clip_start),
            stock_seg_idx: clip.stock_seg_idx,
            overlap_transition: clip.overlap_transition.clone(),
            opacity: clip.opacity,
            pair_id: pair_id.clone(),
            audio_linked,
            is_embedded_audio: is_emb_audio,
        }
    };

    // Вкорочуємо лівий кліп
    editor.clips[idx].duration = split_time - clip_start;

    // Додаємо правий кліп
    editor.clips.push(right_clip);

    // Якщо є парне аудіо — розрізаємо і його
    if let (Some(pid), true) = (pair_id, audio_linked) {
        if let Some(audio_idx) = editor.clips.iter().position(|c| {
            c.pair_id.as_deref() == Some(pid.as_str()) && c.id != clip_id
        }) {
            let a_start = editor.clips[audio_idx].start_secs;
            let a_end = editor.clips[audio_idx].end_secs();
            let a_dur = (a_end - a_start).max(0.1);
            // Пропорційний розріз аудіо
            let ratio = (split_time - clip_start) / (clip_end - clip_start);
            let a_split = a_start + a_dur * ratio;

            if a_split > a_start + 0.05 && a_split < a_end - 0.05 {
                let paired_kind = editor.clips[audio_idx].kind.clone();
                let a_right = EditorClip {
                    id: uuid_str(),
                    media_id: editor.clips[audio_idx].media_id.clone(),
                    path: editor.clips[audio_idx].path.clone(),
                    name: editor.clips[audio_idx].name.clone(),
                    start_secs: a_split,
                    duration: a_end - a_split,
                    track_idx: editor.clips[audio_idx].track_idx,
                    kind: paired_kind,
                    scale: editor.clips[audio_idx].scale,
                    pos_x: editor.clips[audio_idx].pos_x,
                    pos_y: editor.clips[audio_idx].pos_y,
                    zoom_enabled: editor.clips[audio_idx].zoom_enabled,
                    shake_enabled: editor.clips[audio_idx].shake_enabled,
                    is_placeholder: false,
                    trim_start: editor.clips[audio_idx].trim_start + (a_split - a_start),
                    stock_seg_idx: editor.clips[audio_idx].stock_seg_idx,
                    overlap_transition: editor.clips[audio_idx].overlap_transition.clone(),
                    opacity: editor.clips[audio_idx].opacity,
                    pair_id: Some(pid.clone()),
                    audio_linked: true,
                    is_embedded_audio: editor.clips[audio_idx].is_embedded_audio,
                };
                editor.clips[audio_idx].duration = (a_split - a_start).max(0.1);
                editor.clips.push(a_right);
            }
        }
    }

    // Зберігаємо зміни
    editor.save_to_timeline().ok();
    true
}

// ─── Переміщення доріжки (зміна порядку) ────────────────────────────────────

/// Переміщує доріжку з позиції `from` на позицію `to`.
/// Дозволяється тільки між доріжками одного типу (Video↔Video, Audio↔Audio).
fn move_track(editor: &mut MontageEditorState, from: usize, to: usize) {
    if from == to || from >= editor.track_kinds.len() || to >= editor.track_kinds.len() {
        return;
    }
    editor.push_undo();
    if from < to {
        editor.track_kinds[from..=to].rotate_left(1);
        if editor.track_volumes.len() > to {
            editor.track_volumes[from..=to].rotate_left(1);
        }
        for clip in &mut editor.clips {
            if clip.track_idx == from {
                clip.track_idx = to;
            } else if clip.track_idx > from && clip.track_idx <= to {
                clip.track_idx -= 1;
            }
        }
    } else {
        editor.track_kinds[to..=from].rotate_right(1);
        if editor.track_volumes.len() > from {
            editor.track_volumes[to..=from].rotate_right(1);
        }
        for clip in &mut editor.clips {
            if clip.track_idx == from {
                clip.track_idx = to;
            } else if clip.track_idx >= to && clip.track_idx < from {
                clip.track_idx += 1;
            }
        }
    }
    editor.save_to_timeline().ok();
}

// ─── Таймлінія ───────────────────────────────────────────────────────────────

/// Знаходить найближчий край кліпу до `raw_secs` в межах `threshold` секунд.
/// Перевіряє краї кліпів на всіх доріжках (cross-track snap).
fn find_snap_secs(raw_secs: f32, clips: &[EditorClip], threshold: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    let mut best_dist = threshold;
    for clip in clips {
        if clip.is_placeholder { continue; }
        let d_start = (raw_secs - clip.start_secs).abs();
        if d_start < best_dist { best_dist = d_start; best = Some(clip.start_secs); }
        let end = clip.end_secs();
        let d_end = (raw_secs - end).abs();
        if d_end < best_dist { best_dist = d_end; best = Some(end); }
    }
    best
}

/// Перевіряє чи тип кліпу сумісний з типом доріжки.
/// Відео та зображення — тільки на відео-доріжки, аудіо — тільки на аудіо.
fn clip_fits_track(clip_kind: &ClipKind, track_kind: Option<&TrackKind>) -> bool {
    match track_kind {
        Some(TrackKind::Video) => matches!(clip_kind, ClipKind::Video | ClipKind::Image),
        Some(TrackKind::Audio) => matches!(clip_kind, ClipKind::Audio),
        None => false,
    }
}

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

    // ─── Клавіша S: розріз на плейхеді (працює завжди) ──────────────
    if ui.input(|i| i.key_pressed(egui::Key::S)) {
        // Знаходимо кліп під плейхедом
        let ph = editor.playhead;
        if let Some(clip_id) = editor.clips.iter()
            .find(|c| c.start_secs <= ph && ph < c.end_secs() && !c.is_placeholder)
            .map(|c| c.id.clone())
        {
            split_clip_at(editor, &clip_id, ph);
        }
    }

    // Escape — вихід з інструменту розрізу
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) && editor.split_tool_active {
        editor.split_tool_active = false;
    }

    // C — увімкнення/вимкнення інструменту розрізу
    if ui.input(|i| i.key_pressed(egui::Key::C)) {
        editor.split_tool_active = !editor.split_tool_active;
    }

    // ─── Кнопки додавання доріжок + інструмент розрізу ──────────────────
    ui.horizontal(|ui| {
        let spacing = 6.0;

        // Кнопка "+ Відео"
        if ui.add_sized([70.0, 20.0], egui::Button::new(translate(language, "montage_editor_add_video_track"))).clicked() {
            editor.push_undo();
            // Зсуваємо всі кліпи вниз, щоб нова доріжка з'явилась зверху
            for clip in &mut editor.clips {
                clip.track_idx += 1;
            }
            editor.num_tracks += 1;
            editor.track_kinds.insert(0, TrackKind::Video);
        }
        ui.add_space(spacing);
        // Кнопка "+ Аудіо"
        if ui.add_sized([70.0, 20.0], egui::Button::new(translate(language, "montage_editor_add_audio_track"))).clicked() {
            editor.push_undo();
            // Аудіо-доріжки додаються знизу (після відео)
            editor.num_tracks += 1;
            editor.track_kinds.push(TrackKind::Audio);
        }
        ui.weak(format!("{} {}", editor.num_tracks, translate(language, "montage_editor_tracks_count")));

        ui.separator();

        // Кнопки скасування / повторення
        let can_undo = !editor.undo_stack.is_empty();
        let can_redo = !editor.redo_stack.is_empty();
        let undo_btn = ui.add_enabled(
            can_undo,
            egui::Button::new(translate(language, "montage_editor_undo"))
                .min_size([80.0, 20.0].into())
                .fill(Color32::from_rgb(35, 35, 42)),
        ).on_hover_text("Ctrl+Z");
        if undo_btn.clicked() {
            editor.undo();
        }
        ui.add_space(spacing);
        let redo_btn = ui.add_enabled(
            can_redo,
            egui::Button::new(translate(language, "montage_editor_redo"))
                .min_size([80.0, 20.0].into())
                .fill(Color32::from_rgb(35, 35, 42)),
        ).on_hover_text("Ctrl+Y");
        if redo_btn.clicked() {
            editor.redo();
        }

        ui.separator();

        // Кнопка інструменту розрізу (лезо) — C для активації
        let split_btn_text = if editor.split_tool_active {
            egui::RichText::new(translate(language, "montage_editor_split_tool"))
                .strong()
                .color(Color32::WHITE)
        } else {
            egui::RichText::new(translate(language, "montage_editor_split_tool"))
                .weak()
        };
        let split_fill = if editor.split_tool_active {
            Color32::from_rgb(200, 60, 40)
        } else {
            Color32::from_rgb(35, 35, 42)
        };
        if ui.add_sized([80.0, 20.0], egui::Button::new(split_btn_text).fill(split_fill))
            .on_hover_text("C")
            .clicked()
        {
            editor.split_tool_active = !editor.split_tool_active;
        }

        // Масштаб — правий край панелі
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_sized([80.0, 20.0], egui::Slider::new(&mut editor.timeline_zoom, 10.0..=500.0)
                .logarithmic(true)
                .show_value(false));
            ui.weak("🔍");
        });
    });

    let avail_h = ui.available_height();
    let total_rows = editor.num_tracks;
    let total_tracks_h = ruler_h + (track_h + 2.0) * total_rows as f32;
    let timeline_w = (total_dur + 10.0) * zoom;

    // ─── Колесо миші → горизонтальний скрол таймлінії ───────────────────────────
    if let Some(mouse_pos) = ui.ctx().pointer_hover_pos() {
        if ui.clip_rect().contains(mouse_pos) {
            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_y.abs() > 0.1 {
                ui.ctx().input_mut(|i| {
                    i.smooth_scroll_delta.x += i.smooth_scroll_delta.y;
                    i.smooth_scroll_delta.y = 0.0;
                    i.raw_scroll_delta.x += i.raw_scroll_delta.y;
                    i.raw_scroll_delta.y = 0.0;
                });
            }
        }
    }

    // Візуальна позиція доріжки = track_idx (без зсуву)
    let track_visual = |ti: usize| -> usize { ti };
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

        // Авто-скрол за плейхедом під час відтворення:
        // якщо плейхед виходить за межі видимої зони — підтягуємо таймлінію.
        let scroll_visible_w = ui.available_width();
        if editor.is_playing {
            let ph_x = editor.playhead * zoom;
            let right_edge = editor.timeline_scroll_x + scroll_visible_w;
            if ph_x > right_edge - 60.0 || ph_x < editor.timeline_scroll_x {
                editor.timeline_scroll_x = (ph_x - scroll_visible_w * 0.25).max(0.0);
            }
        }

        // scroll_offset передається кожного кадру: враховує авто-скрол, зум і ручний скрол.
        // output.state.offset.x зберігається назад після show() щоб відстежити ручний скрол.
        let output = ScrollArea::both()
            .id_salt("editor_timeline_scroll")
            .max_height(avail_h)
            .auto_shrink([false, true])
            .scroll_offset(egui::Vec2::new(editor.timeline_scroll_x, 0.0))
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
                let track_bg = if editor.drop_target_track == Some(vis) {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 20)
                } else {
                    let ti = vis;
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

            // Глобальний курсор для інструменту розрізу (поверх усіх кліпів)
            if editor.split_tool_active {
                if let Some(pos) = mouse_pos {
                    if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                        let total_rows = editor.num_tracks;
                        let tracks_bottom = rect.top() + ruler_h + total_rows as f32 * (track_h + 2.0);
                        if pos.y <= tracks_bottom {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                        }
                    }
                }
            }

            // Ghost-preview при drag з медіа-пулу
            if let Some(ref drag_id) = editor.dragged_media_id.clone() {
                if let Some(pos) = mouse_pos {
                    if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                        let click_t = ((pos.x - rect.left()) / zoom).max(0.0);
                        let rel_y = pos.y - (rect.top() + ruler_h);
                        let vis_idx = ((rel_y / (track_h + 2.0)) as usize).min(total_rows - 1);
                        let t_idx = vis_idx;
                        let target_kind = editor.track_kinds.get(t_idx);

                        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
                            // Перевіряємо сумісність медіа з доріжкою
                            if clip_fits_track(&media.kind, target_kind) {
                                editor.drop_target_track = Some(t_idx);
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
                                    format!("{:.1}s → {}{}", click_t, if matches!(target_kind, Some(TrackKind::Audio)) { "A" } else { "V" }, t_idx + 1),
                                    egui::FontId::proportional(10.0), Color32::WHITE,
                                );
                            } else {
                                editor.drop_target_track = None;
                            }
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
                                vis_idx
                            };
                            let start = ((pos.x - rect.left()) / zoom).max(0.0);
                            // Збираємо дані до мутації (уникаємо конфлікту запозичень)
                            let drop_info = {
                                let tk = editor.track_kinds.get(t_idx).copied();
                                editor.media_pool.iter().find(|m| m.id == drag_id).and_then(|media| {
                                    if clip_fits_track(&media.kind, tk.as_ref()) {
                                        Some((media.kind.clone(), media.has_audio, media.name.clone(),
                                              media.path.clone(), media.duration_secs, media.id.clone(), tk))
                                    } else {
                                        None
                                    }
                                })
                            };
                            if let Some((kind, has_audio, name, media_path, duration, media_id, _target_kind)) = drop_info {
                                {
                                editor.push_undo();
                                // (kind, has_audio, name, media_path, duration, media_id вже клоновані)
                                // Якщо відео з аудіо — генеруємо спільний pair_id
                                let pair_uuid = if matches!(kind, ClipKind::Video) && has_audio {
                                    Some(uuid_str())
                                } else {
                                    None
                                };
                                let new_id = uuid_str();
                                editor.selected_clip_id = Some(new_id.clone());
                                editor.clips.push(EditorClip {
                                    id: new_id,
                                    media_id: media_id.clone(),
                                    path: Some(media_path.clone()),
                                    name: name.clone(),
                                    start_secs: start,
                                    duration,
                                    track_idx: t_idx,
                                    kind: kind.clone(),
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
                                    pair_id: pair_uuid.clone(),
                                    audio_linked: pair_uuid.is_some(),
                                    is_embedded_audio: false,
                                });
                                // Автоматично додаємо аудіо-кліп для відео з вбудованим аудіо
                                if let Some(ref pid) = pair_uuid {
                                    let new_end = start + duration;
                                    // Шукаємо аудіо-доріжку без перетину з новим кліпом,
                                    // щоб аудіо різних відео не накладались на одну доріжку.
                                    let audio_track_idx = {
                                        let free = editor.track_kinds.iter().enumerate()
                                            .find(|(ti, k)| {
                                                **k == super::types::TrackKind::Audio
                                                    && !editor.clips.iter().any(|c| {
                                                        c.track_idx == *ti
                                                            && c.start_secs < new_end
                                                            && c.end_secs() > start
                                                    })
                                            })
                                            .map(|(ti, _)| ti);
                                        free.unwrap_or_else(|| {
                                            let idx = editor.num_tracks;
                                            editor.num_tracks += 1;
                                            editor.track_kinds.push(super::types::TrackKind::Audio);
                                            while editor.track_volumes.len() < editor.num_tracks {
                                                editor.track_volumes.push(1.0);
                                            }
                                            idx
                                        })
                                    };
                                    editor.clips.push(EditorClip {
                                        id: uuid_str(),
                                        media_id,
                                        path: Some(media_path.clone()),
                                        name: format!("A: {}", name),
                                        start_secs: start,
                                        duration,
                                        track_idx: audio_track_idx,
                                        kind: ClipKind::Audio,
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
                                        pair_id: Some(pid.clone()),
                                        audio_linked: true,
                                        is_embedded_audio: true,
                                    });
                                    // Одразу запускаємо витягування WAV у фоні
                                    super::audio::extract_embedded_audio_async(
                                        media_path.clone(),
                                        editor.save_path.clone(),
                                    );
                                }
                                editor.save_to_timeline().ok();
                                } // closes else
                            }
                        }
                    }
                    editor.drop_target_track = None;
                }
            }

            // Обробка перетягування кліпів (move / trim)
            update_clip_drag(ui.ctx(), editor, rect, ruler_h, track_h, zoom);

            // Обробка drag смужки прозорості
            update_opacity_drag(ui, editor);

            // Оновлюємо снапнуту позицію для інструменту розрізу (cross-track)
            if editor.split_tool_active {
                let threshold = 10.0 / zoom;
                editor.split_snap_secs = mouse_pos
                    .filter(|p| rect.contains(*p) && p.y >= rect.top() + ruler_h)
                    .and_then(|p| find_snap_secs((p.x - rect.left()) / zoom, &editor.clips, threshold));
            } else {
                editor.split_snap_secs = None;
            }

            // Кліпи
            let clips_snapshot: Vec<EditorClip> = editor.clips.clone();
            // Прапорець щоб не розрізати кілька кліпів за один кадр
            let mut split_done_this_frame = false;
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
                let (bg, accent) = if clip.is_embedded_audio {
                    // Вбудоване аудіо відеофайлу — бірюзовий відтінок
                    (
                        Color32::from_rgba_unmultiplied(18, 42, 50, bg_alpha),
                        Color32::from_rgb(30, 155, 160),
                    )
                } else {
                    match clip.kind {
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
                    }
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
                    // Іконка з індикатором зв'язку для пар
                    let base_icon = match clip.kind { ClipKind::Video => "🎬", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                    let link_suffix = if clip.pair_id.is_some() {
                        if clip.audio_linked { " 🔗" } else { " 🔓" }
                    } else { "" };
                    let display_name = if clip.is_embedded_audio {
                        clip.name.strip_prefix("A: ").unwrap_or(&clip.name)
                    } else {
                        &clip.name
                    };
                    let label = if display_name.chars().count() > 14 {
                        format!("{} {}…{}", base_icon, display_name.chars().take(11).collect::<String>(), link_suffix)
                    } else {
                        format!("{} {}{}", base_icon, display_name, link_suffix)
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
                // Або хрестик якщо активний інструмент розрізу
                if let Some(pos) = mouse_pos {
                    if clip_rect.contains(pos) {
                        if editor.split_tool_active {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                        } else {
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
                }

                let clip_resp = if editor.split_tool_active {
                    ui.allocate_rect(clip_rect, Sense::click())
                } else {
                    ui.allocate_rect(clip_rect, Sense::click_and_drag())
                };

                // Інструмент розрізу: клік по кліпу = розріз у місці кліку
                if editor.split_tool_active {
                    // Примусово хрестик поверх PointingHand від Sense::click()
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    if !split_done_this_frame && clip_resp.clicked() {
                        if let Some(pos) = mouse_pos {
                            if clip_rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                                // Використовуємо снапнуту позицію якщо є (cross-track snap)
                                let click_time = editor.split_snap_secs
                                    .unwrap_or_else(|| (pos.x - rect.left()) / zoom)
                                    .max(0.0);
                                let clip_id = clip.id.clone();
                                split_clip_at(editor, &clip_id, click_time);
                                split_done_this_frame = true;
                            }
                        }
                    }
                    // Не обробляємо подвійний клік, drag та інше в режимі розрізу
                    continue;
                }

                if clip_resp.double_clicked() {
                    if let Some(ref path) = clip.path {
                        editor.pool_preview = Some(path.clone());
                        editor.pool_preview_texture = None;
                    }
                } else if clip_resp.clicked() {
                    editor.selected_clip_id = Some(clip.id.clone());
                }
                if clip_resp.drag_started() {
                    editor.push_undo();
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
                            // Шукаємо початкову позицію парного кліпу
                            let paired_initial_start = clip.pair_id.as_ref()
                                .filter(|_| clip.audio_linked)
                                .and_then(|pid| {
                                    clips_snapshot.iter()
                                        .find(|c| c.pair_id.as_deref() == Some(pid.as_str()) && c.id != clip.id)
                                        .map(|p| p.start_secs)
                                });
                            editor.clip_drag_state = Some(ClipDragState {
                                clip_id: clip.id.clone(),
                                mode,
                                initial_start: clip.start_secs,
                                initial_duration: clip.duration,
                                initial_trim_start: clip.trim_start,
                                initial_mouse_x: pos.x,
                                initial_track_idx: clip.track_idx,
                                snap_line_secs: None,
                                paired_initial_start,
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
                    let clip_pair_id = clip.pair_id.clone();
                    let clip_audio_linked = clip.audio_linked;
                    let clip_is_embedded = clip.is_embedded_audio;
                    let clip_id_str = clip.id.clone();
                    let is_animating = anim_loading.lock().unwrap().contains(&clip_path) || regen_paths.contains(&clip_path);
                    clip_resp.context_menu(|ui| {
                        // Пункти для пов'язаних аудіо/відео пар
                        if clip_pair_id.is_some() {
                            if clip_audio_linked {
                                if ui.button(translate(language, "montage_editor_unlink_audio")).clicked() {
                                    editor.push_undo();
                                    // Розв'язуємо обидва кліпи пари
                                    let pid = clip_pair_id.as_deref().unwrap_or("");
                                    for c in &mut editor.clips {
                                        if c.pair_id.as_deref() == Some(pid) {
                                            c.audio_linked = false;
                                        }
                                    }
                                    editor.save_to_timeline().ok();
                                    ui.close_menu();
                                }
                            } else if ui.button(translate(language, "montage_editor_link_audio")).clicked() {
                                editor.push_undo();
                                let pid = clip_pair_id.as_deref().unwrap_or("");
                                for c in &mut editor.clips {
                                    if c.pair_id.as_deref() == Some(pid) {
                                        c.audio_linked = true;
                                    }
                                }
                                editor.save_to_timeline().ok();
                                ui.close_menu();
                            }
                            if clip_is_embedded {
                                // На аудіо-кліпі: "Видалити аудіо"
                                if ui.button(translate(language, "montage_editor_delete_audio_clip")).clicked() {
                                    editor.push_undo();
                                    editor.clips.retain(|c| c.id != clip_id_str);
                                    // Знімаємо pair_id у відео-кліпу
                                    if let Some(pid) = clip_pair_id.as_deref() {
                                        for c in &mut editor.clips {
                                            if c.pair_id.as_deref() == Some(pid) {
                                                c.pair_id = None;
                                                c.audio_linked = false;
                                            }
                                        }
                                    }
                                    editor.save_to_timeline().ok();
                                    ui.close_menu();
                                }
                            } else {
                                // На відео-кліпі: "Видалити аудіо" видаляє парний аудіо-кліп
                                if ui.button(translate(language, "montage_editor_delete_audio_clip")).clicked() {
                                    editor.push_undo();
                                    if let Some(pid) = clip_pair_id.as_deref() {
                                        editor.clips.retain(|c| {
                                            !(c.pair_id.as_deref() == Some(pid) && c.is_embedded_audio)
                                        });
                                        // Знімаємо pair_id у відео-кліпу
                                        for c in &mut editor.clips {
                                            if c.id == clip_id_str {
                                                c.pair_id = None;
                                                c.audio_linked = false;
                                            }
                                        }
                                    }
                                    editor.save_to_timeline().ok();
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                        }
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
                        ui.separator();
                        if ui.button(translate(language, "montage_editor_delete_clip")).clicked() {
                            editor.push_undo();
                            // Якщо відео з прив'язаним вбудованим аудіо — видаляємо і його
                            if !clip_is_embedded {
                                if let Some(pid) = clip_pair_id.as_deref() {
                                    if clip_audio_linked {
                                        editor.clips.retain(|c| {
                                            !(c.pair_id.as_deref() == Some(pid) && c.is_embedded_audio)
                                        });
                                    }
                                }
                            }
                            editor.clips.retain(|c| c.id != clip_id_str);
                            if editor.selected_clip_id.as_deref() == Some(clip_id_str.as_str()) {
                                editor.selected_clip_id = None;
                            }
                            editor.save_to_timeline().ok();
                            ui.close_menu();
                        }
                    });
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

            // Індикатор лінії розрізу при активному інструменті (поверх кліпів)
            if editor.split_tool_active {
                if let Some(pos) = mouse_pos {
                    if rect.contains(pos) && pos.y >= rect.top() + ruler_h {
                        let is_snapped = editor.split_snap_secs.is_some();
                        let secs = editor.split_snap_secs
                            .unwrap_or_else(|| (pos.x - rect.left()) / zoom);
                        let split_x = rect.left() + secs * zoom;
                        let top_y = rect.top() + ruler_h;
                        let total_rows = editor.num_tracks;
                        let bottom_y = top_y + total_rows as f32 * (track_h + 2.0);
                        // Помаранчевий при снапі, червоний без нього
                        let line_color = if is_snapped {
                            Color32::from_rgb(255, 182, 72)
                        } else {
                            Color32::from_rgb(240, 50, 40)
                        };
                        painter.line_segment(
                            [Pos2::new(split_x, top_y), Pos2::new(split_x, bottom_y)],
                            Stroke::new(2.0, line_color),
                        );
                        // Маленький кружок-індикатор на лінійці при снапі
                        if is_snapped {
                            painter.circle_filled(
                                Pos2::new(split_x, top_y - 4.0),
                                4.0,
                                line_color,
                            );
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

            // Скраб плейхеда кліком по лінійці.
            // Натискання на лінійці починає drag; плейхед рухається поки кнопка затиснута,
            // навіть якщо мишка вийшла на кліпи нижче.
            let primary_down = ui.input(|i| i.pointer.primary_down());
            if let Some(pos) = mouse_pos {
                if ruler_rect.contains(pos) && ui.input(|i| i.pointer.primary_pressed()) {
                    editor.playhead_dragging = true;
                }
            }
            if !primary_down {
                editor.playhead_dragging = false;
            }
            if editor.playhead_dragging && primary_down {
                if let Some(pos) = mouse_pos {
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
                        let total_rows = editor.num_tracks;
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

        // Запам'ятовуємо поточний горизонтальний scroll для авто-прокрутки наступного кадру
        editor.timeline_scroll_x = output.state.offset.x;

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
            let ti = vis;
            let kind = editor.track_kinds.get(ti).copied().unwrap_or(TrackKind::Video);
            let (label, bg, border, text_color, bar_color) = match kind {
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
            };
            let lrect = Rect::from_min_size(Pos2::new(labels_rect.left(), track_y), Vec2::new(label_w, track_h));
            painter.rect(lrect, 0.0, bg, Stroke::new(1.0, border));

            // Підсвічуємо фон доріжки, яку перетягують
            let is_dragging_this = editor.track_drag.as_ref().map_or(false, |d| d.from_track == ti);
            if is_dragging_this {
                painter.rect_filled(lrect, 0.0, Color32::from_rgba_unmultiplied(9, 123, 244, 30));
            }

            // Назва доріжки + іконка захоплення (верхня частина)
            let name_rect = Rect::from_min_size(lrect.min, Vec2::new(label_w, track_h * 0.6));
            let grab_alpha = if is_dragging_this { 220u8 } else { 70u8 };
            painter.text(
                Pos2::new(lrect.left() + 4.0, name_rect.center().y),
                Align2::LEFT_CENTER,
                "⠿",
                egui::FontId::proportional(10.0),
                Color32::from_rgba_unmultiplied(text_color.r(), text_color.g(), text_color.b(), grab_alpha),
            );
            painter.text(name_rect.center(), Align2::CENTER_CENTER, &label, egui::FontId::proportional(11.0), text_color);

            // Інтерактивна зона для перетягування доріжки (верхні 50% лейблу, над слайдером)
            let track_drag_rect = Rect::from_min_size(lrect.min, Vec2::new(label_w, track_h * 0.50));
            let drag_resp = ui.allocate_rect(track_drag_rect, Sense::drag());

            if drag_resp.drag_started() && editor.track_drag.is_none() {
                editor.track_drag = Some(TrackDragState {
                    from_track: ti,
                    hover_track: ti,
                });
            }

            if is_dragging_this {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if drag_resp.hovered() && editor.track_drag.is_none() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }

            // ─── Повзунок гучності (нижня частина лейблу) ────────────────────

            // Поточне значення гучності
            let current_vol = editor.track_volumes.get(ti).copied().unwrap_or(1.0);

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
                    while editor.track_volumes.len() <= ti { editor.track_volumes.push(1.0); }
                    editor.track_volumes[ti] = new_vol;
                    // Зупиняємо активне аудіо щоб воно перезапустилось з новою гучністю
                    let save_path = editor.save_path.clone();
                    editor.active_audios.retain(|a| {
                        !editor.clips.iter().any(|c| {
                            if c.track_idx != ti { return false; }
                            // Прямий збіг шляху
                            if c.path.as_deref() == Some(a.path.as_path()) { return true; }
                            // Вбудоване аудіо: clip.path = .mp4, active.path = .wav кеш
                            if c.is_embedded_audio {
                                if let Some(ref cp) = c.path {
                                    let cached = super::audio::embedded_audio_cache_path(cp, &save_path);
                                    return cached == a.path;
                                }
                            }
                            false
                        })
                    });
                }
                ui.ctx().request_repaint();
            }
            if bar_resp.drag_stopped() {
                // Тільки гучності — не перезаписуємо весь timeline.json (щоб не знищити дані агента)
                editor.save_volumes_only().ok();
            }

            // Tooltip з поточним значенням
            // Курсор при наведенні на слайдер
            if bar_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            bar_resp.on_hover_text(format!("Vol: {:.0}%", current_vol * 100.0));
        }

        // ─── Оновлення та обробка drag-перетягування доріжок ─────────────────
        if editor.track_drag.is_some() {
            // Оновлюємо hover_track за Y-позицією курсору
            if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let raw = (hover_pos.y - labels_rect.top() - ruler_h + v_off) / (track_h + 2.0);
                let hover_vis = (raw.max(0.0) as usize).min(total_rows.saturating_sub(1));
                if let Some(ref mut drag) = editor.track_drag {
                    drag.hover_track = hover_vis;
                }
            }

            // Читаємо стан для малювання (перед мутацією)
            let (from_track, to_track, is_valid) = if let Some(ref drag) = editor.track_drag {
                let from_kind = editor.track_kinds.get(drag.from_track).copied();
                let to_kind = editor.track_kinds.get(drag.hover_track).copied();
                let valid = from_kind == to_kind && drag.from_track != drag.hover_track;
                (drag.from_track, drag.hover_track, valid)
            } else {
                (0, 0, false)
            };
            let _ = from_track; // використовується вище

            // Малюємо індикатор місця вставки
            if is_valid {
                let indicator_y = labels_rect.top() + ruler_h
                    + (to_track as f32 + 0.5) * (track_h + 2.0) - v_off;
                if indicator_y >= labels_rect.top() && indicator_y <= labels_rect.bottom() {
                    painter.line_segment(
                        [Pos2::new(labels_rect.left(), indicator_y),
                         Pos2::new(labels_rect.right(), indicator_y)],
                        Stroke::new(2.5, Color32::from_rgb(9, 123, 244)),
                    );
                }
            }

            // Відпускання кнопки — застосовуємо переміщення
            if ui.input(|i| !i.pointer.any_down()) {
                if let Some(drag) = editor.track_drag.take() {
                    let from = drag.from_track;
                    let to = drag.hover_track;
                    if from != to {
                        let from_kind = editor.track_kinds.get(from).copied();
                        let to_kind = editor.track_kinds.get(to).copied();
                        if from_kind.is_some() && from_kind == to_kind {
                            move_track(editor, from, to);
                        }
                    }
                }
            }

            ui.ctx().request_repaint();
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
        editor.save_to_timeline().ok();
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
) {
    // Якщо opacity_drag активний — clip_drag не обробляємо
    if editor.opacity_drag.is_some() { return; }

    if ctx.input(|i| i.pointer.any_released()) {
        if editor.clip_drag_state.is_some() {
            editor.save_to_timeline().ok();
        }
        editor.clip_drag_state = None;
        return;
    }

    let drag = match &editor.clip_drag_state {
        Some(d) => ClipDragState {
            clip_id: d.clip_id.clone(),
            mode: d.mode,
            initial_start: d.initial_start,
            initial_duration: d.initial_duration,
            initial_trim_start: d.initial_trim_start,
            initial_mouse_x: d.initial_mouse_x,
            initial_track_idx: d.initial_track_idx,
            snap_line_secs: d.snap_line_secs,
            paired_initial_start: d.paired_initial_start,
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
    // Конвертуємо візуальну позицію в track_idx
    let hovered_track = vis_idx.min(editor.num_tracks - 1);

    // Перевіряємо сумісність типу кліпу з цільовою доріжкою
    let clip_kind = editor.clips[clip_idx].kind.clone();
    let target_kind = editor.track_kinds.get(hovered_track);
    // Якщо доріжка несумісна — шукаємо найближчу сумісну
    let new_track = if clip_fits_track(&clip_kind, target_kind) {
        hovered_track
    } else {
        // Знаходимо найближчу сумісну доріжку
        let mut best: Option<usize> = None;
        let mut best_dist = usize::MAX;
        for (ti, tk) in editor.track_kinds.iter().enumerate() {
            if clip_fits_track(&clip_kind, Some(tk)) {
                let dist = if ti > hovered_track { ti - hovered_track } else { hovered_track - ti };
                if dist < best_dist {
                    best_dist = dist;
                    best = Some(ti);
                }
            }
        }
        best.unwrap_or(drag.initial_track_idx)
    };

    let clip_dur = drag.initial_duration;
    // Сирі позиції лівого та правого краю для кожного режиму
    let raw_left = (drag.initial_start + dx).max(0.0);   // Move / TrimLeft
    let raw_right = drag.initial_start + clip_dur + dx;   // Move / TrimRight

    // Snap обох країв незалежно (cross-track, всі кліпи + плейхед)
    let snap_threshold = 10.0 / zoom;
    let mut snap_left: Option<f32> = None;
    let mut snap_right: Option<f32> = None;
    let mut dist_left = snap_threshold;
    let mut dist_right = snap_threshold;

    let check = |val: f32, candidate: f32, best: &mut Option<f32>, dist: &mut f32| {
        let d = (val - candidate).abs();
        if d < *dist { *dist = d; *best = Some(candidate); }
    };

    check(raw_left,  editor.playhead, &mut snap_left,  &mut dist_left);
    check(raw_right, editor.playhead, &mut snap_right, &mut dist_right);

    for (i, other) in editor.clips.iter().enumerate() {
        if i == clip_idx { continue; }
        let other_end = other.end_secs();
        for &candidate in &[other.start_secs, other_end] {
            check(raw_left,  candidate, &mut snap_left,  &mut dist_left);
            check(raw_right, candidate, &mut snap_right, &mut dist_right);
        }
    }

    // Вибираємо знапнуту позицію та snap_line залежно від режиму
    let snapped_start;
    let snapped_right; // тільки для TrimRight
    let snap_line_secs;

    match drag.mode {
        DragMode::Move => {
            // Беремо той край, що ближче
            if snap_left.is_some() && (snap_right.is_none() || dist_left <= dist_right) {
                snapped_start = snap_left.unwrap();
                snap_line_secs = snap_left;
            } else if let Some(sr) = snap_right {
                snapped_start = (sr - clip_dur).max(0.0);
                snap_line_secs = Some(sr);
            } else {
                snapped_start = raw_left;
                snap_line_secs = None;
            }
            snapped_right = snapped_start + clip_dur;
        }
        DragMode::TrimLeft => {
            snapped_start = snap_left.unwrap_or(raw_left);
            snapped_right = drag.initial_start + clip_dur; // правий край незмінний
            snap_line_secs = snap_left;
        }
        DragMode::TrimRight => {
            snapped_start = raw_left;
            snapped_right = snap_right.unwrap_or(raw_right);
            snap_line_secs = snap_right;
        }
    }
    let _ = snapped_right; // використовується нижче у TrimRight

    if let Some(ref mut ds) = editor.clip_drag_state {
        ds.snap_line_secs = snap_line_secs;
    }

    // Зчитуємо дані пари до мутабельного запозичення
    let (pair_id, is_linked, this_clip_id) = {
        let c = &editor.clips[clip_idx];
        (c.pair_id.clone(), c.audio_linked, c.id.clone())
    };

    let clip = &mut editor.clips[clip_idx];
    match drag.mode {
        DragMode::Move => {
            clip.start_secs = snapped_start;
            clip.track_idx = new_track;
        }
        DragMode::TrimLeft => {
            let snapped_dx = snapped_start - drag.initial_start;
            let max_dx = drag.initial_duration - 0.1;
            let bounded_dx = snapped_dx.clamp(-drag.initial_start, max_dx);
            clip.start_secs = drag.initial_start + bounded_dx;
            clip.duration = drag.initial_duration - bounded_dx;
            clip.trim_start = (drag.initial_trim_start + bounded_dx).max(0.0);
        }
        DragMode::TrimRight => {
            let new_dur = (snap_right.unwrap_or(raw_right) - drag.initial_start).max(0.1);
            clip.duration = new_dur;
        }
    }
    // перше запозичення закінчилось

    // Синхронно рухаємо парний кліп (якщо прив'язаний)
    if matches!(drag.mode, DragMode::Move) && is_linked {
        if let (Some(pid), Some(paired_init)) = (pair_id, drag.paired_initial_start) {
            if let Some(paired) = editor.clips.iter_mut().find(|c| {
                c.pair_id.as_deref() == Some(pid.as_str()) && c.id != this_clip_id
            }) {
                paired.start_secs = (paired_init + dx).max(0.0);
            }
        }
    }

    ctx.request_repaint();
}
