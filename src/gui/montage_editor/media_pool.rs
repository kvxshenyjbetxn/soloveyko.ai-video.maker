use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::{Align2, Color32, Layout, Pos2, Rect, ScrollArea, Sense, Stroke, Vec2};
use crate::localization::{Language, translate};
use super::state::MontageEditorState;
use super::media::MediaItem;
use super::types::ClipKind;

// ─── Медіа-пул ───────────────────────────────────────────────────────────────

pub(super) fn draw_media_pool(
    ui: &mut egui::Ui,
    language: Language,
    editor: &mut MontageEditorState,
    anim_loading: &Arc<Mutex<HashSet<PathBuf>>>,
) {
    const VALID_EXTS: &[&str] = &["mp4", "mov", "webm", "jpg", "jpeg", "png", "webp", "mp3", "wav"];

    // ── Drag-and-drop з файлової системи ─────────────────────────────────────
    let hovered_files = ui.ctx().input(|i| i.raw.hovered_files.clone());
    let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());

    let is_hovering_media = hovered_files.iter().any(|f| {
        f.path.as_ref().map(|p| {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            VALID_EXTS.contains(&ext.as_str())
        }).unwrap_or(true) // невідоме розширення — показуємо підказку
    });

    for file in &dropped_files {
        if let Some(path) = &file.path {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if VALID_EXTS.contains(&ext.as_str()) && !editor.media_pool.iter().any(|m| m.path == *path) {
                let save_path = editor.save_path.clone();
                editor.media_pool.push(MediaItem::new(path.clone(), &save_path));
            }
        }
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("📁 {}", translate(language, "montage_editor_media_pool"))).strong());
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(translate(language, "montage_editor_add_media")).clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Media", &["mp4", "mov", "webm", "jpg", "jpeg", "png", "webp", "mp3", "wav"])
                    .pick_files()
                {
                    let save_path = editor.save_path.clone();
                    for path in paths {
                        if !editor.media_pool.iter().any(|m| m.path == path) {
                            editor.media_pool.push(MediaItem::new(path, &save_path));
                        }
                    }
                }
            }

            // Кнопка "Оживити все" — тільки якщо в пулі є зображення
            let all_image_paths: Vec<PathBuf> = editor.media_pool.iter()
                .filter(|m| matches!(m.kind, ClipKind::Image))
                .map(|m| m.path.clone())
                .collect();
            if !all_image_paths.is_empty() {
                if ui.small_button("🎬").on_hover_text(translate(language, "montage_editor_animate_all")).clicked() {
                    let loading = anim_loading.lock().unwrap();
                    for path in all_image_paths {
                        if !loading.contains(&path) {
                            editor.pending_animate_paths.push(path);
                        }
                    }
                }
            }

            // Кнопка "Оживити обрані" — тільки якщо виділені зображення
            let selected_image_paths: Vec<PathBuf> = editor.media_pool.iter()
                .filter(|m| matches!(m.kind, ClipKind::Image) && editor.selected_media_ids.contains(&m.id))
                .map(|m| m.path.clone())
                .collect();
            if !selected_image_paths.is_empty() {
                let cnt = selected_image_paths.len();
                if ui.small_button(translate(language, "montage_editor_animate_selected"))
                    .on_hover_text(format!("({cnt})"))
                    .clicked()
                {
                    let loading = anim_loading.lock().unwrap();
                    for path in selected_image_paths {
                        if !loading.contains(&path) {
                            editor.pending_animate_paths.push(path);
                        }
                    }
                }
            }
        });
    });
    ui.separator();

    ScrollArea::vertical().id_salt("editor_pool_scroll").show(ui, |ui| {
        if editor.media_pool.is_empty() {
            // Зона скидання коли пул порожній
            let drop_h = (ui.available_height() - 8.0).max(60.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), drop_h), Sense::hover());
            let stroke_col = if is_hovering_media {
                Color32::from_rgb(9, 123, 244)
            } else {
                Color32::from_rgb(55, 55, 60)
            };
            let bg_col = if is_hovering_media {
                Color32::from_rgba_unmultiplied(9, 123, 244, 18)
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect(rect, 6.0, bg_col, Stroke::new(1.5, stroke_col));
            ui.painter().text(
                rect.center() - Vec2::new(0.0, 10.0),
                Align2::CENTER_CENTER,
                "📂",
                egui::FontId::proportional(24.0),
                stroke_col,
            );
            ui.painter().text(
                rect.center() + Vec2::new(0.0, 14.0),
                Align2::CENTER_CENTER,
                translate(language, "montage_editor_drop_here"),
                egui::FontId::proportional(11.0),
                stroke_col,
            );
            return;
        }
        let mut to_remove: Option<usize> = None;
        let mut toggle_select_id: Option<String> = None;
        let mut context_animate: Option<PathBuf> = None;
        let mut context_regen: Option<(PathBuf, bool)> = None;

        for (idx, media) in editor.media_pool.iter().enumerate() {
            let item_w = (ui.available_width() - 30.0).max(80.0);
            let media_id = media.id.clone();
            let media_path = media.path.clone();
            let media_kind = media.kind.clone();
            let is_selected = editor.selected_media_ids.contains(&media.id);
            let is_animating = anim_loading.lock().unwrap().contains(&media.path);

            ui.horizontal(|ui| {
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(item_w, 26.0), Sense::click_and_drag());

                if resp.drag_started() {
                    editor.dragged_media_id = Some(media_id.clone());
                }
                if resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }

                let is_dragged = editor.dragged_media_id.as_deref() == Some(media_id.as_str());
                let is_hovered = resp.hovered();

                let bg = if is_animating {
                    Color32::from_rgba_unmultiplied(200, 140, 0, 35)
                } else if is_dragged {
                    Color32::from_rgba_unmultiplied(9, 123, 244, 40)
                } else if is_selected {
                    Color32::from_rgba_unmultiplied(39, 160, 80, 35)
                } else if is_hovered {
                    Color32::from_rgb(38, 38, 42)
                } else {
                    Color32::from_rgb(28, 28, 30)
                };
                let stroke_col = if is_selected {
                    Color32::from_rgb(39, 160, 80)
                } else if is_dragged {
                    Color32::from_rgb(9, 123, 244)
                } else {
                    Color32::from_rgb(45, 45, 50)
                };
                ui.painter().rect(rect, 4.0, bg, Stroke::new(1.0, stroke_col));

                // Спінер оживлення поверх елементу
                if is_animating {
                    ui.painter().rect_filled(rect, 4.0, Color32::from_black_alpha(150));
                    let spin_rect = Rect::from_center_size(rect.center(), Vec2::splat(14.0));
                    ui.put(spin_rect, egui::Spinner::new().size(14.0));
                    ui.ctx().request_repaint();
                }

                // Індикатор прогресу витягування кадрів / анімування
                let done = media.is_extraction_complete();
                let icon = match media.kind {
                    ClipKind::Video => "🎥",
                    ClipKind::Image => "🖼",
                    ClipKind::Audio => "🎵",
                };
                let status_dot = if is_animating { " 🎬" } else if done { "" } else { " ⏳" };
                let dur_text = format!("{:.1}s", media.duration_secs);
                let display = if media.name.chars().count() > 16 {
                    format!("{} {}…{} {}", icon, media.name.chars().take(13).collect::<String>(), status_dot, dur_text)
                } else {
                    format!("{} {}{} {}", icon, media.name, status_dot, dur_text)
                };
                let text_col = if is_dragged { Color32::from_rgb(9, 123, 244) } else { Color32::from_rgb(200, 200, 205) };
                ui.painter().text(
                    Pos2::new(rect.left() + 6.0, rect.top() + 6.0),
                    Align2::LEFT_TOP, &display,
                    egui::FontId::proportional(11.0), text_col,
                );

                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").clicked() {
                        to_remove = Some(idx);
                    }
                });

                // Клік → виділення; подвійний клік → fullscreen preview
                if resp.double_clicked() {
                    editor.pool_preview = Some(media_path.clone());
                    editor.pool_preview_texture = None;
                } else if resp.clicked() {
                    toggle_select_id = Some(media_id.clone());
                }

                // Контекстне меню (правий клік)
                resp.context_menu(|ui| {
                    if matches!(media_kind, ClipKind::Image) {
                        if is_animating {
                            ui.add_enabled(false, egui::Button::new(format!("⏳ {}", translate(language, "gallery_regen_loading"))));
                        } else if ui.button(translate(language, "montage_editor_animate")).clicked() {
                            context_animate = Some(media_path.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                    }
                    if ui.button(translate(language, "montage_editor_regen_same")).clicked() {
                        context_regen = Some((media_path.clone(), false));
                        ui.close_menu();
                    }
                    if ui.button(translate(language, "montage_editor_regen_custom")).clicked() {
                        context_regen = Some((media_path.clone(), true));
                        ui.close_menu();
                    }
                });
            });
            ui.add_space(2.0);
        }

        if let Some(id) = toggle_select_id {
            if !editor.selected_media_ids.remove(&id) {
                editor.selected_media_ids.insert(id);
            }
        }
        if let Some(path) = context_animate {
            if !anim_loading.lock().unwrap().contains(&path) {
                editor.pending_animate_paths.push(path);
            }
        }
        if let Some(regen) = context_regen {
            editor.pending_regen = Some(regen);
        }
        if let Some(idx) = to_remove {
            editor.media_pool.remove(idx);
        }
    });

    // Плаваюча картка при drag
    if let Some(ref drag_id) = editor.dragged_media_id.clone() {
        if let Some(media) = editor.media_pool.iter().find(|m| &m.id == drag_id) {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag_card"));
                let painter = ui.ctx().layer_painter(layer);
                let card_w = 140.0;
                let card_h = 28.0;
                let card_rect = Rect::from_min_size(
                    Pos2::new(pos.x + 10.0, pos.y - card_h / 2.0),
                    Vec2::new(card_w, card_h),
                );
                painter.rect(card_rect, 4.0,
                    Color32::from_rgb(28, 36, 52),
                    Stroke::new(1.5, Color32::from_rgb(9, 123, 244)));
                let icon = match media.kind { ClipKind::Video => "🎥", ClipKind::Image => "🖼", ClipKind::Audio => "🎵" };
                let label = if media.name.chars().count() > 12 {
                    format!("{} {}…", icon, media.name.chars().take(10).collect::<String>())
                } else {
                    format!("{} {}", icon, media.name)
                };
                painter.text(
                    card_rect.center(),
                    Align2::CENTER_CENTER, &label,
                    egui::FontId::proportional(11.0), Color32::WHITE,
                );
            }
        }
    }

    // Overlay коли файли перетягуються над пулом (а в пулі вже є елементи)
    if is_hovering_media && !editor.media_pool.is_empty() {
        let pool_rect = ui.clip_rect();
        let painter = ui.painter_at(pool_rect);
        painter.rect_filled(pool_rect, 0.0, Color32::from_rgba_unmultiplied(9, 123, 244, 22));
        painter.rect_stroke(pool_rect, 6.0, Stroke::new(2.0, Color32::from_rgb(9, 123, 244)));
        painter.text(
            pool_rect.center(),
            Align2::CENTER_CENTER,
            translate(language, "montage_editor_drop_here"),
            egui::FontId::proportional(13.0),
            Color32::from_rgb(9, 123, 244),
        );
    }
}
