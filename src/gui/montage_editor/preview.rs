use std::time::Instant;
use eframe::egui;
use egui::{Align2, Color32, Pos2, Rect, Sense, Vec2};
use super::state::MontageEditorState;
use super::types::{ClipKind, EditorClip, MontagePreviewSettings, PreviewDragMode, PreviewDragState};
use super::media::MediaItem;

// ─── Дані overlay-кліпу для рендеру превью ───────────────────────────────────

struct OverlayRenderItem {
    path: std::path::PathBuf,
    t_off: f32,
    scale: f32,
    pos_x: f32,
    pos_y: f32,
    duration: f32,
    kind: ClipKind,
    zoom_enabled: bool,
    shake_enabled: bool,
}

// ─── Допоміжні типи та функції для превью-ефектів ────────────────────────────

/// Візуальна категорія переходу для превью (ігнорує FFmpeg-специфіку)
enum TransitionKind {
    Fade,
    WipeLeft, WipeRight, WipeUp, WipeDown,
    SlideLeft, SlideRight, SlideUp, SlideDown,
    FadeBlack, FadeWhite,
}

fn transition_kind(name: &str) -> TransitionKind {
    match name {
        "wipeleft"  | "hlslice"   | "smoothleft"  | "diagtl" | "diagbl"
        | "circlecrop" | "rectcrop"                             => TransitionKind::WipeLeft,
        "wiperight" | "hrslice"   | "smoothright" | "diagtr" | "diagbr" => TransitionKind::WipeRight,
        "wipeup"    | "vuslice"   | "smoothup"                           => TransitionKind::WipeUp,
        "wipedown"  | "vdslice"   | "smoothdown"                         => TransitionKind::WipeDown,
        "slideleft" | "coverleft" | "revealright"
        | "horzopen" | "hlwind"                                           => TransitionKind::SlideLeft,
        "slideright"| "coverright"| "revealleft"
        | "horzclose"| "hrwind"                                           => TransitionKind::SlideRight,
        "slideup"   | "coverup"   | "revealdown"
        | "vertopen" | "vuwind"                                           => TransitionKind::SlideUp,
        "slidedown" | "coverdown" | "revealup"
        | "vertclose"| "vdwind"                                           => TransitionKind::SlideDown,
        "fadeblack"                                                        => TransitionKind::FadeBlack,
        "fadewhite" | "fadegrays"                                          => TransitionKind::FadeWhite,
        _ => TransitionKind::Fade,
    }
}

/// Обчислює коефіцієнт зуму для зображення в момент `t` секунд.
/// Відтворює логіку zoompan з montage.rs (alternate / oscillate).
fn compute_zoom(t: f32, duration: f32, settings: &MontagePreviewSettings, img_idx: usize, is_image: bool) -> f32 {
    if !settings.zoom_enabled || !is_image { return 1.0; }
    let t_norm = (t / duration.max(0.001)).clamp(0.0, 1.0);
    let z_amp = settings.zoom_scale - 1.0;
    match settings.zoom_mode.as_str() {
        "oscillate" => 1.0 + z_amp * (1.0 - (std::f32::consts::TAU * t_norm).cos()) / 2.0,
        _ => {
            // alternate: парні кліпи zoom-in, непарні zoom-out
            if img_idx % 2 == 0 {
                (1.0 + z_amp * t_norm).min(settings.zoom_scale)
            } else {
                (settings.zoom_scale - z_amp * t_norm).max(1.0)
            }
        }
    }
}

/// UV-прямокутник для центрованого crop при заданому коефіцієнті зуму.
fn zoom_uv(zoom_factor: f32) -> Rect {
    let m = (1.0 - 1.0 / zoom_factor) / 2.0;
    Rect::from_min_max(Pos2::new(m, m), Pos2::new(1.0 - m, 1.0 - m))
}

/// UV-зміщення для ефекту покачування (відповідає crop-фільтру FFmpeg).
/// Повертає зміщення в UV-просторі (0..1), а не в пікселях —
/// тому чорних країв немає: TextureWrapMode::ClampToEdge використовує крайні пікселі.
fn shake_uv(t: f32, settings: &MontagePreviewSettings, is_image: bool) -> Vec2 {
    if !settings.shake_enabled || !is_image { return Vec2::ZERO; }
    // FFmpeg: amp_f = 40 * intensity px у 1920px. Відповідний UV-дробик: amp_f / 1920
    let amp = settings.shake_intensity * 40.0 / 1920.0;
    Vec2::new(
        amp * (std::f32::consts::PI * 0.7 * t).sin(),
        amp * (std::f32::consts::PI * 0.53 * t).sin(),
    )
}

/// Малює кадр текстури у container.
/// `uv` — zoom-crop (з `zoom_uv`); `uv_shift` — додаткове UV-зміщення (shake).
/// Зображення завжди заповнює container без чорних країв.
fn render_clip_frame(
    painter: &egui::Painter,
    tex: &egui::TextureHandle,
    container: Rect,
    uv: Rect,
    uv_shift: Vec2,
    alpha: u8,
) {
    let sz = tex.size_vec2();
    let scale = (container.width() / sz.x).min(container.height() / sz.y);
    let img_rect = Rect::from_center_size(container.center(), sz * scale);
    // Зміщуємо UV-crop, а не позицію зображення → без чорних країв
    let shifted_uv = Rect::from_center_size(uv.center() + uv_shift, uv.size());
    painter.image(tex.id(), img_rect, shifted_uv, Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
}

// ─── Preview ──────────────────────────────────────────────────────────────────

/// Шукає медіа для кліпу:
/// 1. По media_id (стабільний UUID, не залежить від шляху)
/// 2. Fallback: по точному шляху
/// 3. Fallback: по стему файлу (для .jpg → .mp4 після оживлення)
fn find_media_for_clip<'a>(pool: &'a [MediaItem], clip: &EditorClip) -> Option<&'a MediaItem> {
    if !clip.media_id.is_empty() {
        if let Some(m) = pool.iter().find(|m| m.id == clip.media_id) {
            return Some(m);
        }
    }
    if let Some(ref path) = clip.path {
        if let Some(m) = pool.iter().find(|m| m.path == *path) {
            return Some(m);
        }
        let stem = path.file_stem().and_then(|s| s.to_str())?;
        let parent = path.parent()?;
        return pool.iter().find(|m| {
            m.path.parent() == Some(parent)
                && m.path.file_stem().and_then(|s| s.to_str()) == Some(stem)
        });
    }
    None
}

pub(super) fn draw_preview(ui: &mut egui::Ui, editor: &mut MontageEditorState) {
    let ph = editor.playhead;
    let settings = editor.preview_settings.clone();

    // Доріжка 0 = базова (фон). Overlay-доріжки (1+) рендеруються поверх незалежно.
    let active_track = 0usize;

    let mut sorted: Vec<EditorClip> = editor.clips.iter()
        .filter(|c| c.track_idx == active_track && c.path.is_some())
        .cloned()
        .collect();
    sorted.sort_by(|a, b| a.start_secs.partial_cmp(&b.start_secs).unwrap_or(std::cmp::Ordering::Equal));

    let active_idx = sorted.iter().position(|c| c.start_secs <= ph && ph < c.end_secs());
    let first_clip = active_idx.map(|i| sorted[i].clone());

    // ─── Виявлення накладання (overlap → автоматичний crossfade) ────────
    // active_idx дає перший кліп що містить плейхед (= outgoing).
    // Якщо є другий кліп пізніше що теж містить плейхед — це incoming, накладання є.
    let incoming = active_idx.and_then(|ai| {
        sorted[ai + 1..].iter()
            .find(|c| c.start_secs <= ph && ph < c.end_secs())
            .cloned()
    });
    let (outgoing, active) = if let Some(ref inc) = incoming {
        (first_clip, Some(inc.clone()))
    } else {
        (None, first_clip)
    };

    // Чи є налаштування переходу з налаштувань пайплайну
    let settings_prev = if outgoing.is_none() {
        active_idx.and_then(|i| if i > 0 { Some(sorted[i - 1].clone()) } else { None })
    } else {
        None
    };
    let has_settings_trans = settings.transition != "none" && settings.transition_duration > 0.0 && settings_prev.is_some();

    // Обчислюємо overlap-перехід: outgoing + active накладаються
    let overlap_progress = if let (Some(out), Some(act)) = (&outgoing, &active) {
        let overlap_start = act.start_secs;
        let overlap_end = (out.start_secs + out.duration).min(act.start_secs + act.duration);
        if overlap_end > overlap_start + 0.001 && ph >= overlap_start && ph < overlap_end {
            Some((ph - overlap_start) / (overlap_end - overlap_start))
        } else {
            None
        }
    } else {
        None
    };

    // Якщо overlap є — використовуємо його замість settings-переходу
    let prev_clip = if overlap_progress.is_some() { outgoing.clone() } else { settings_prev.clone() };
    let clip_offset = active.as_ref().map(|c| (ph - c.start_secs).max(0.0)).unwrap_or(0.0);
    let in_transition = overlap_progress.is_some() || (has_settings_trans && clip_offset < settings.transition_duration);
    // Зміщення у вихідному файлі з урахуванням trim_start (для кадрів превʼю)
    let source_offset = clip_offset + active.as_ref().map(|c| c.trim_start).unwrap_or(0.0);

    // Індекс зображення серед усіх зображень (для alternate-зуму)
    let img_idx_active = active_idx.map(|idx| {
        sorted[..idx].iter().filter(|c| matches!(c.kind, ClipKind::Image)).count()
    }).unwrap_or(0);
    let img_idx_incoming = img_idx_active
        + if outgoing.as_ref().map(|c| matches!(c.kind, ClipKind::Image)).unwrap_or(false) { 1 } else { 0 };
    let img_idx_prev = img_idx_active.saturating_sub(
        if prev_clip.as_ref().map(|c| matches!(c.kind, ClipKind::Image)).unwrap_or(false) { 1 } else { 0 }
    );

    // Визначення стану переходу (progress для overlap або settings)
    let transition_progress = overlap_progress.unwrap_or(if has_settings_trans && clip_offset < settings.transition_duration {
        clip_offset / settings.transition_duration
    } else {
        0.0
    });

    // Медіа-елементи (clone щоб розділити borrow від frame_cache)
    // Якщо не знайдено в пулі але файл існує — додаємо в пул на ходу (захист від десинхронізації)
    let active_media: Option<MediaItem> = {
        let found = active.as_ref()
            .and_then(|c| find_media_for_clip(&editor.media_pool, c))
            .cloned();
        if found.is_none() {
            if let Some(clip) = active.as_ref() {
                if let Some(path) = clip.path.as_ref() {
                    if path.exists() && !editor.media_pool.iter().any(|m| m.path == *path) {
                        let sp = editor.save_path.clone();
                        let m = MediaItem::new(path.clone(), &sp, editor.preview_render);
                        let mid = m.id.clone();
                        editor.media_pool.push(m);
                        let cid = clip.id.clone();
                        if let Some(c) = editor.clips.iter_mut().find(|c| c.id == cid) {
                            c.media_id = mid;
                        }
                    }
                }
            }
            active.as_ref()
                .and_then(|c| find_media_for_clip(&editor.media_pool, c))
                .cloned()
        } else {
            found
        }
    };
    let prev_media: Option<MediaItem> = prev_clip.as_ref()
        .and_then(|c| find_media_for_clip(&editor.media_pool, c))
        .cloned();

    // Текстури. Під час drag/playback використовуємо легкий scrub-кеш,
    // після зупинки плейхеду просимо чіткий still-кадр у фоні.
    let pointer_down = ui.ctx().input(|i| i.pointer.primary_down());
    let use_sharp_frame = !editor.is_playing && !pointer_down && !in_transition;
    let current_tex = active_media.as_ref()
        .and_then(|m| editor.frame_cache.get_frame(ui.ctx(), m, source_offset, use_sharp_frame, editor.preview_render));
    let prev_tex = if in_transition {
        prev_media.as_ref()
            .and_then(|m| {
                // Для overlap-переходу: кадр на поточній позиції плейхеду в межах outgoing кліпу
                if overlap_progress.is_some() {
                    if let Some(ref out) = outgoing {
                        let out_t = (ph - out.start_secs + out.trim_start).max(0.0).min(m.duration_secs - 0.001);
                        editor.frame_cache.get_frame(ui.ctx(), m, out_t, false, editor.preview_render)
                    } else {
                        None
                    }
                } else {
                    // Settings-перехід: останній кадр попереднього кліпу
                    let last_t = (m.duration_secs - 0.001).max(0.0);
                    editor.frame_cache.get_frame(ui.ctx(), m, last_t, false, editor.preview_render)
                }
            })
    } else {
        None
    };

    let is_extracting = active_media.as_ref()
        .map(|m| !m.is_extraction_complete())
        .unwrap_or(false);

    if is_extracting || in_transition {
        ui.ctx().request_repaint();
    }

    // ── Layout ───────────────────────────────────────────────────────────────
    const TRANSPORT_H: f32 = 44.0;
    const LABEL_H: f32 = 18.0;

    let avail_w = ui.available_width();
    let avail_h = (ui.available_height() - TRANSPORT_H - LABEL_H - 10.0).max(40.0);
    let frame_w = (avail_h * 16.0 / 9.0).min(avail_w - 4.0);
    let frame_h = frame_w * 9.0 / 16.0;
    let pad_x = ((avail_w - frame_w) / 2.0).max(0.0);

    ui.label(egui::RichText::new("📺 Попередній перегляд").size(11.0).weak());

    ui.horizontal(|ui| {
        ui.add_space(pad_x);
        let (rect, _frame_resp) = ui.allocate_exact_size(Vec2::new(frame_w, frame_h), Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, Color32::from_rgb(5, 5, 7));

        // Per-clip zoom/shake для базової (V1) доріжки
        let is_base_img = active.as_ref().map(|c| matches!(c.kind, ClipKind::Image)).unwrap_or(false);
        let base_zoom_on = is_base_img && active.as_ref().map(|c| c.zoom_enabled).unwrap_or(false);
        let base_shake_on = is_base_img && active.as_ref().map(|c| c.shake_enabled).unwrap_or(false);
        let is_prev_img = prev_clip.as_ref().map(|c| matches!(c.kind, ClipKind::Image)).unwrap_or(false);
        let prev_zoom_on = is_prev_img && prev_clip.as_ref().map(|c| c.zoom_enabled).unwrap_or(false);
        let prev_shake_on = is_prev_img && prev_clip.as_ref().map(|c| c.shake_enabled).unwrap_or(false);
        let dur_curr = active.as_ref().map(|c| c.duration).unwrap_or(1.0);
        let dur_prev = prev_clip.as_ref().map(|c| c.duration).unwrap_or(1.0);
        // Для overlap: час у межах outgoing кліпу для коректного зуму/шейку
        let prev_offset = if overlap_progress.is_some() {
            outgoing.as_ref().map(|o| (ph - o.start_secs).max(0.0)).unwrap_or(dur_prev)
        } else {
            dur_prev
        };
        let uv_curr = zoom_uv(compute_zoom(clip_offset, dur_curr, &settings,
            if overlap_progress.is_some() { img_idx_incoming } else { img_idx_active }, base_zoom_on));
        let sh_curr = shake_uv(clip_offset, &settings, base_shake_on);
        let uv_prev = zoom_uv(compute_zoom(prev_offset, dur_prev, &settings, img_idx_prev, prev_zoom_on));
        let sh_prev = shake_uv(prev_offset, &settings, prev_shake_on);

        // ── Рендер базової доріжки 0 (зі zoom/shake/переходами) ─────────────
        let trans_kind = if overlap_progress.is_some() {
            TransitionKind::Fade // overlap → завжди crossfade
        } else {
            transition_kind(&settings.transition)
        };
        if let Some(ref curr) = current_tex {
            if in_transition {
                let tp = transition_progress;
                match trans_kind {
                    TransitionKind::Fade => {
                        if let Some(ref pt) = prev_tex {
                            render_clip_frame(&painter, pt, rect, uv_prev, sh_prev, ((1.0 - tp) * 255.0) as u8);
                        }
                        render_clip_frame(&painter, curr, rect, uv_curr, sh_curr, (tp * 255.0) as u8);
                    }
                    TransitionKind::FadeBlack => {
                        if tp < 0.5 {
                            if let Some(ref pt) = prev_tex {
                                render_clip_frame(&painter, pt, rect, uv_prev, sh_prev, ((1.0 - tp * 2.0) * 255.0) as u8);
                            }
                        } else {
                            render_clip_frame(&painter, curr, rect, uv_curr, sh_curr, (((tp - 0.5) * 2.0) * 255.0) as u8);
                        }
                    }
                    TransitionKind::FadeWhite => {
                        painter.rect_filled(rect, 0.0, Color32::WHITE);
                        if tp < 0.5 {
                            if let Some(ref pt) = prev_tex {
                                render_clip_frame(&painter, pt, rect, uv_prev, sh_prev, ((1.0 - tp * 2.0) * 255.0) as u8);
                            }
                        } else {
                            render_clip_frame(&painter, curr, rect, uv_curr, sh_curr, (((tp - 0.5) * 2.0) * 255.0) as u8);
                        }
                    }
                    TransitionKind::WipeLeft => {
                        let sx = rect.left() + (1.0 - tp) * rect.width();
                        if let Some(ref pt) = prev_tex {
                            let r = Rect::from_min_max(rect.min, Pos2::new(sx, rect.max.y));
                            render_clip_frame(&ui.painter_at(r), pt, rect, uv_prev, sh_prev, 255);
                        }
                        let r = Rect::from_min_max(Pos2::new(sx, rect.min.y), rect.max);
                        render_clip_frame(&ui.painter_at(r), curr, rect, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::WipeRight => {
                        let sx = rect.left() + tp * rect.width();
                        let r_new = Rect::from_min_max(rect.min, Pos2::new(sx, rect.max.y));
                        let r_old = Rect::from_min_max(Pos2::new(sx, rect.min.y), rect.max);
                        if let Some(ref pt) = prev_tex {
                            render_clip_frame(&ui.painter_at(r_old), pt, rect, uv_prev, sh_prev, 255);
                        }
                        render_clip_frame(&ui.painter_at(r_new), curr, rect, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::WipeUp => {
                        let sy = rect.top() + (1.0 - tp) * rect.height();
                        if let Some(ref pt) = prev_tex {
                            let r = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, sy));
                            render_clip_frame(&ui.painter_at(r), pt, rect, uv_prev, sh_prev, 255);
                        }
                        let r = Rect::from_min_max(Pos2::new(rect.min.x, sy), rect.max);
                        render_clip_frame(&ui.painter_at(r), curr, rect, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::WipeDown => {
                        let sy = rect.top() + tp * rect.height();
                        let r_new = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, sy));
                        let r_old = Rect::from_min_max(Pos2::new(rect.min.x, sy), rect.max);
                        if let Some(ref pt) = prev_tex {
                            render_clip_frame(&ui.painter_at(r_old), pt, rect, uv_prev, sh_prev, 255);
                        }
                        render_clip_frame(&ui.painter_at(r_new), curr, rect, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::SlideLeft => {
                        let p = ui.painter_at(rect);
                        if let Some(ref pt) = prev_tex {
                            let c = rect.translate(Vec2::new(-tp * rect.width(), 0.0));
                            render_clip_frame(&p, pt, c, uv_prev, sh_prev, 255);
                        }
                        let c = rect.translate(Vec2::new((1.0 - tp) * rect.width(), 0.0));
                        render_clip_frame(&p, curr, c, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::SlideRight => {
                        let p = ui.painter_at(rect);
                        if let Some(ref pt) = prev_tex {
                            let c = rect.translate(Vec2::new(tp * rect.width(), 0.0));
                            render_clip_frame(&p, pt, c, uv_prev, sh_prev, 255);
                        }
                        let c = rect.translate(Vec2::new(-(1.0 - tp) * rect.width(), 0.0));
                        render_clip_frame(&p, curr, c, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::SlideUp => {
                        let p = ui.painter_at(rect);
                        if let Some(ref pt) = prev_tex {
                            let c = rect.translate(Vec2::new(0.0, -tp * rect.height()));
                            render_clip_frame(&p, pt, c, uv_prev, sh_prev, 255);
                        }
                        let c = rect.translate(Vec2::new(0.0, (1.0 - tp) * rect.height()));
                        render_clip_frame(&p, curr, c, uv_curr, sh_curr, 255);
                    }
                    TransitionKind::SlideDown => {
                        let p = ui.painter_at(rect);
                        if let Some(ref pt) = prev_tex {
                            let c = rect.translate(Vec2::new(0.0, tp * rect.height()));
                            render_clip_frame(&p, pt, c, uv_prev, sh_prev, 255);
                        }
                        let c = rect.translate(Vec2::new(0.0, -(1.0 - tp) * rect.height()));
                        render_clip_frame(&p, curr, c, uv_curr, sh_curr, 255);
                    }
                }
            } else {
                render_clip_frame(&painter, curr, rect, uv_curr, sh_curr, 255);
            }

            if is_extracting {
                let dot = Pos2::new(rect.right() - 10.0, rect.top() + 10.0);
                painter.circle_filled(dot, 5.0, Color32::from_rgba_unmultiplied(255, 180, 0, 220));
            }
            let mut effects: Vec<&str> = Vec::new();
            if settings.zoom_enabled && base_zoom_on { effects.push("zoom"); }
            if settings.shake_enabled && base_shake_on { effects.push("shake"); }
            if in_transition { effects.push(&settings.transition); }
            if !effects.is_empty() {
                let label = effects.join(" + ");
                painter.text(
                    Pos2::new(rect.left() + 6.0, rect.bottom() - 6.0), Align2::LEFT_BOTTOM,
                    &label, egui::FontId::proportional(9.0),
                    Color32::from_rgba_unmultiplied(200, 200, 100, 180),
                );
            }
        } else if is_extracting {
            ui.put(rect, egui::Spinner::new().size(36.0));
            painter.text(
                rect.center() + Vec2::new(0.0, 28.0), Align2::CENTER_CENTER,
                "Підготовка превью...",
                egui::FontId::proportional(10.0), Color32::from_rgb(180, 180, 100),
            );
        } else if active.is_some() && active_media.is_some() {
            // Медіа є в пулі, але кадрів ще немає (витягування в процесі або відео не підтримується)
            ui.ctx().request_repaint();
            painter.text(
                rect.center() - Vec2::new(0.0, 12.0), Align2::CENTER_CENTER,
                "🎬", egui::FontId::proportional(36.0), Color32::from_rgb(60, 60, 80),
            );
        } else if active.is_some() {
            painter.text(
                rect.center() - Vec2::new(0.0, 12.0), Align2::CENTER_CENTER,
                "🎬", egui::FontId::proportional(36.0), Color32::from_rgb(40, 40, 52),
            );
            painter.text(
                rect.center() + Vec2::new(0.0, 24.0), Align2::CENTER_CENTER,
                "Файл не знайдено у медіа-пулі",
                egui::FontId::proportional(10.0), Color32::from_rgb(70, 70, 88),
            );
        } else {
            painter.text(
                rect.center() - Vec2::new(0.0, 12.0), Align2::CENTER_CENTER,
                "🎬", egui::FontId::proportional(36.0), Color32::from_rgb(40, 40, 52),
            );
            painter.text(
                rect.center() + Vec2::new(0.0, 24.0), Align2::CENTER_CENTER,
                "Немає медіа під плейхедом",
                egui::FontId::proportional(10.0), Color32::from_rgb(70, 70, 88),
            );
        }

        // ── Overlay-доріжки (track 1+) поверх базової доріжки 0 ─────────────
        // Збираємо до Vec перед мутацією frame_cache (уникаємо borrow-конфлікту)
        let mut ov_sorted: Vec<&EditorClip> = editor.clips.iter()
            .filter(|c| c.track_idx > 0 && c.path.is_some())
            .filter(|c| c.start_secs <= ph && ph < c.end_secs())
            .collect();
        ov_sorted.sort_by_key(|c| c.track_idx);
        let overlay_data: Vec<OverlayRenderItem> = ov_sorted.iter()
            .map(|c| OverlayRenderItem {
                path: c.path.clone().unwrap(),
                t_off: (ph - c.start_secs).max(0.0),
                scale: c.scale, pos_x: c.pos_x, pos_y: c.pos_y,
                duration: c.duration,
                kind: c.kind.clone(),
                zoom_enabled: c.zoom_enabled,
                shake_enabled: c.shake_enabled,
            })
            .collect();

        for (ov_idx, item) in overlay_data.iter().enumerate() {
            let ov_media = editor.media_pool.iter().find(|m| m.path == item.path).cloned();
            if let Some(media) = ov_media {
                if !media.is_extraction_complete() {
                    ui.ctx().request_repaint();
                    let dot_x = rect.left() + 10.0 + ov_idx as f32 * 14.0;
                    painter.circle_filled(
                        Pos2::new(dot_x, rect.top() + 10.0),
                        5.0, Color32::from_rgba_unmultiplied(255, 200, 60, 220),
                    );
                }
                if let Some(tex) = editor.frame_cache.get_frame(ui.ctx(), &media, item.t_off, use_sharp_frame, editor.preview_render) {
                    let clip_w = rect.width() * item.scale;
                    let clip_h = rect.height() * item.scale;
                    let clip_cx = rect.center().x + item.pos_x * rect.width() / 2.0;
                    let clip_cy = rect.center().y + item.pos_y * rect.height() / 2.0;
                    let container = Rect::from_center_size(
                        Pos2::new(clip_cx, clip_cy),
                        Vec2::new(clip_w, clip_h),
                    );
                    let is_ov_img = matches!(item.kind, ClipKind::Image);
                    let ov_uv = zoom_uv(compute_zoom(item.t_off, item.duration, &settings, ov_idx,
                        is_ov_img && item.zoom_enabled));
                    let ov_shake = shake_uv(item.t_off, &settings,
                        is_ov_img && item.shake_enabled);
                    render_clip_frame(&painter, &tex, container, ov_uv, ov_shake, 255);
                }
            }
        }

        let any_shake_on = base_shake_on || overlay_data.iter().any(|item|
            matches!(item.kind, ClipKind::Image) && item.shake_enabled
        );

        // ── Ручки трансформу для виділеного кліпу ───────────────────────────
        let sel_transform = editor.selected_clip_id.as_ref().and_then(|id| {
            editor.clips.iter().find(|c| &c.id == id
                && c.start_secs <= ph && ph < c.end_secs())
                .map(|c| {
                    let w = rect.width() * c.scale;
                    let h = rect.height() * c.scale;
                    let cx = rect.center().x + c.pos_x * rect.width() / 2.0;
                    let cy = rect.center().y + c.pos_y * rect.height() / 2.0;
                    let sel_rect = Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(w, h));
                    let corners: [Pos2; 4] = [
                        sel_rect.left_top(), sel_rect.right_top(),
                        sel_rect.right_bottom(), sel_rect.left_bottom(),
                    ];
                    (sel_rect, corners, c.id.clone())
                })
        });

        if let Some((sel_rect, corners, _)) = &sel_transform {
            painter.rect_stroke(*sel_rect, 0.0, egui::Stroke::new(2.0, Color32::from_rgb(9, 123, 244)));
            for &corner in corners {
                painter.circle_filled(corner, 5.0, Color32::from_rgb(9, 123, 244));
                painter.circle_stroke(corner, 5.0, egui::Stroke::new(1.0, Color32::WHITE));
            }
        }

        draw_frame_overlay(&painter, rect, &settings, any_shake_on);

        if sel_transform.is_some() {
            let (sel_rect, corners, _) = sel_transform.as_ref().unwrap();
            update_preview_drag(ui.ctx(), editor, rect, *sel_rect, corners);
        } else if editor.preview_drag.is_some() {
            update_preview_drag(ui.ctx(), editor, rect, Rect::NOTHING, &[]);
        }
    });

    ui.add_space(4.0);

    // ── Транспортні кнопки ────────────────────────────────────────────────────
    let total = editor.total_dur();
    ui.horizontal(|ui| {
        let group_w = 260.0;
        ui.add_space(((avail_w - group_w) / 2.0).max(0.0));

        if ui.button("⏮").on_hover_text("-0.1s").clicked() {
            editor.playhead = (editor.playhead - 0.1).max(0.0);
            editor.active_audios.clear();
        }
        if ui.button("⏹").clicked() {
            editor.is_playing = false;
            editor.playhead = 0.0;
            editor.active_audios.clear();
        }
        let play_lbl = egui::RichText::new(if editor.is_playing { "⏸" } else { "▶" }).size(16.0);
        if ui.button(play_lbl).clicked() {
            editor.is_playing = !editor.is_playing;
            editor.last_frame_time = Instant::now();
            if !editor.is_playing {
                editor.active_audios.clear();
            }
        }
        if ui.button("⏭").on_hover_text("+0.1s").clicked() {
            editor.playhead = (editor.playhead + 0.1).min(total);
            editor.active_audios.clear();
        }

        ui.add_space(8.0);

        let m = (editor.playhead / 60.0) as u32;
        let s = (editor.playhead % 60.0) as u32;
        let cs = ((editor.playhead % 1.0) * 100.0) as u32;
        ui.label(
            egui::RichText::new(format!("{:02}:{:02}.{:02}", m, s, cs))
                .monospace().size(12.0).color(Color32::from_rgb(210, 210, 230))
        );
        let tm = (total / 60.0) as u32;
        let ts = (total % 60.0) as u32;
        ui.label(egui::RichText::new(format!("/ {:02}:{:02}", tm, ts)).size(10.0).weak());
    });
}

// ─── Обводка кадру + shake-зона ──────────────────────────────────────────────

/// Малює обводку кадру 1920×1080 та пунктирну "safe zone" якщо shake увімкнено.
fn draw_frame_overlay(
    painter: &egui::Painter,
    rect: Rect,
    settings: &MontagePreviewSettings,
    is_image: bool,
) {
    painter.rect_stroke(
        rect, 2.0,
        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
    );

    if !settings.shake_enabled || !is_image {
        return;
    }

    let amp = settings.shake_intensity * 40.0;
    let mx = amp / (1920.0 + 2.0 * amp) * rect.width();
    let my = amp / (1080.0 + 2.0 * amp) * rect.height();

    let safe = rect.shrink2(Vec2::new(mx, my));
    let dash_len = 6.0f32;
    let gap_len = 4.0f32;
    let safe_color = Color32::from_rgba_unmultiplied(255, 180, 30, 220);
    let stroke = egui::Stroke::new(1.0, safe_color);
    for side in 0..4u8 {
        let (start, end) = match side {
            0 => (safe.left_top(), safe.right_top()),
            1 => (safe.right_top(), safe.right_bottom()),
            2 => (safe.right_bottom(), safe.left_bottom()),
            _ => (safe.left_bottom(), safe.left_top()),
        };
        let total = (end - start).length();
        let dir = (end - start) / total;
        let mut pos = 0.0f32;
        let mut draw = true;
        while pos < total {
            let seg = if draw { dash_len } else { gap_len };
            let seg = seg.min(total - pos);
            if draw {
                let p0 = start + dir * pos;
                let p1 = start + dir * (pos + seg);
                painter.line_segment([p0, p1], stroke);
            }
            pos += seg;
            draw = !draw;
        }
    }
}

// ─── Інтерактивний трансформ на превью ───────────────────────────────────────

/// Обробляє drag-переміщення та масштабування кліпу прямо на превью.
fn update_preview_drag(
    ctx: &egui::Context,
    editor: &mut MontageEditorState,
    frame_rect: Rect,
    sel_rect: Rect,
    corners: &[Pos2],
) {
    // Не обробляємо drag коли поверх відкрито інше вікно (stock picker тощо)
    if editor.input_blocked {
        editor.preview_drag = None;
        return;
    }

    if ctx.input(|i| i.pointer.any_released()) {
        editor.preview_drag = None;
        return;
    }

    let mouse = match ctx.input(|i| i.pointer.hover_pos()) { Some(p) => p, None => return };
    let primary_down = ctx.input(|i| i.pointer.primary_down());
    let primary_pressed = ctx.input(|i| i.pointer.primary_pressed());

    if let Some(drag) = editor.preview_drag.as_ref() {
        if primary_down {
            let dx = (mouse.x - drag.initial_mouse.x) / drag.frame_rect.width() * 2.0;
            let dy = (mouse.y - drag.initial_mouse.y) / drag.frame_rect.height() * 2.0;
            let (init_px, init_py, init_sc, clip_id, drag_mode) = (
                drag.initial_pos_x, drag.initial_pos_y,
                drag.initial_scale, drag.clip_id.clone(), drag.mode,
            );
            if let Some(idx) = editor.clips.iter().position(|c| c.id == clip_id) {
                match drag_mode {
                    PreviewDragMode::Move => {
                        editor.clips[idx].pos_x = (init_px + dx).clamp(-2.5, 2.5);
                        editor.clips[idx].pos_y = (init_py + dy).clamp(-2.5, 2.5);
                    }
                    PreviewDragMode::Scale => {
                        let scale_delta = (dx - dy) * 0.5;
                        editor.clips[idx].scale = (init_sc + scale_delta).clamp(0.05, 3.0);
                    }
                }
            }
            ctx.request_repaint();
        } else {
            editor.preview_drag = None;
        }
        return;
    }

    if primary_pressed && !corners.is_empty() {
        let on_corner = corners.iter().any(|&c| (mouse - c).length() < 10.0);
        let in_interior = sel_rect.contains(mouse);

        if on_corner || in_interior {
            if let Some(sel_id) = editor.selected_clip_id.clone() {
                if let Some(clip) = editor.clips.iter().find(|c| c.id == sel_id) {
                    editor.preview_drag = Some(PreviewDragState {
                        clip_id: clip.id.clone(),
                        mode: if on_corner { PreviewDragMode::Scale } else { PreviewDragMode::Move },
                        initial_pos_x: clip.pos_x,
                        initial_pos_y: clip.pos_y,
                        initial_scale: clip.scale,
                        initial_mouse: mouse,
                        frame_rect,
                    });
                    ctx.set_cursor_icon(if on_corner {
                        egui::CursorIcon::ResizeNwSe
                    } else {
                        egui::CursorIcon::Grabbing
                    });
                }
            }
        }
        return;
    }

    if !corners.is_empty() {
        if corners.iter().any(|&c| (mouse - c).length() < 10.0) {
            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
        } else if sel_rect.contains(mouse) {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        }
    }
}
