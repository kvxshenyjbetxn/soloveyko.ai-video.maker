use super::state::MontageEditorState;
use super::types::ClipKind;
use crate::localization::{Language, translate};
use eframe::egui;

// ─── Інспектор ───────────────────────────────────────────────────────────────

pub(super) fn draw_inspector(
    ui: &mut egui::Ui,
    language: Language,
    editor: &mut MontageEditorState,
) {
    ui.label(
        egui::RichText::new(format!(
            "⚙ {}",
            translate(language, "montage_editor_inspector")
        ))
        .strong(),
    );
    ui.separator();

    let sel_id = editor.selected_clip_id.clone();
    if let Some(ref id) = sel_id {
        if let Some(idx) = editor.clips.iter().position(|c| c.id == *id) {
            let num_tracks = editor.num_tracks;
            let clip = &mut editor.clips[idx];

            if clip.is_placeholder {
                let seg_idx = clip.stock_seg_idx.or_else(|| {
                    clip.media_id
                        .strip_prefix("placeholder_")
                        .and_then(|s| s.parse::<usize>().ok())
                });
                let title = if let Some(seg_idx) = seg_idx {
                    format!(
                        "{} #{}",
                        translate(language, "montage_editor_placeholder_title"),
                        seg_idx + 1
                    )
                } else {
                    translate(language, "montage_editor_placeholder_title").to_string()
                };
                ui.label(egui::RichText::new(title).size(12.0).strong());
                ui.add_space(6.0);
                ui.label(clip.name.as_str());
                ui.add_space(6.0);
                ui.weak(translate(language, "montage_editor_placeholder_hint"));
                ui.add_space(8.0);
                ui.label(format!(
                    "{} {:.2}",
                    translate(language, "montage_editor_clip_start"),
                    clip.start_secs
                ));
                ui.label(format!(
                    "{} {:.2}",
                    translate(language, "montage_editor_clip_dur"),
                    clip.duration
                ));
                if let Some(seg_idx) = seg_idx {
                    ui.add_space(8.0);
                    if ui
                        .button(translate(language, "montage_editor_replace_stock"))
                        .clicked()
                    {
                        editor.pending_open_stock_picker = Some(seg_idx);
                    }
                    if ui
                        .button(translate(language, "montage_editor_regen_same"))
                        .clicked()
                    {
                        editor.pending_placeholder_regen = Some((seg_idx, false));
                    }
                    if ui
                        .button(translate(language, "montage_editor_regen_custom"))
                        .clicked()
                    {
                        editor.pending_placeholder_regen = Some((seg_idx, true));
                    }
                }
                return;
            }

            ui.label(egui::RichText::new(&clip.name).size(12.0).strong());
            ui.add_space(6.0);

            // ── Час/тривалість/доріжка ───────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_start"));
                ui.add(
                    egui::DragValue::new(&mut clip.start_secs)
                        .speed(0.05)
                        .range(0.0..=3600.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_dur"));
                ui.add(
                    egui::DragValue::new(&mut clip.duration)
                        .speed(0.05)
                        .range(0.1..=3600.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_track"));
                let mut t = clip.track_idx as i32;
                if ui
                    .add(
                        egui::DragValue::new(&mut t)
                            .speed(1.0)
                            .range(0..=(num_tracks as i32 - 1)),
                    )
                    .changed()
                {
                    clip.track_idx = t as usize;
                }
            });

            // ── Трансформ (масштаб + позиція) ───────────────────────────────
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(translate(language, "montage_editor_transform"))
                    .strong()
                    .size(11.0),
            );
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_scale"));
                ui.add(egui::Slider::new(&mut clip.scale, 0.05..=3.0).step_by(0.01));
            });
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_pos_x"));
                ui.add(egui::Slider::new(&mut clip.pos_x, -2.0..=2.0).step_by(0.01));
            });
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_pos_y"));
                ui.add(egui::Slider::new(&mut clip.pos_y, -2.0..=2.0).step_by(0.01));
            });

            ui.add_space(4.0);
            if ui
                .small_button(translate(language, "montage_editor_reset_transform"))
                .clicked()
            {
                clip.scale = 1.0;
                clip.pos_x = 0.0;
                clip.pos_y = 0.0;
            }

            // ── Прозорість ──────────────────────────────────────────────────
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(translate(language, "montage_editor_clip_opacity"));
                ui.add(egui::Slider::new(&mut clip.opacity, 0.0..=1.0).step_by(0.01));
            });

            // ── Ефекти (лише для зображень) ─────────────────────────────────
            if matches!(clip.kind, ClipKind::Image) {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(translate(language, "montage_editor_effects"))
                        .strong()
                        .size(11.0),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut clip.zoom_enabled,
                        translate(language, "montage_editor_clip_zoom"),
                    );
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut clip.shake_enabled,
                        translate(language, "montage_editor_clip_shake"),
                    );
                });
            }

            ui.add_space(8.0);
            if ui
                .button(translate(language, "montage_editor_delete_clip"))
                .clicked()
            {
                editor.clips.remove(idx);
                editor.selected_clip_id = None;
            }
        } else {
            editor.selected_clip_id = None;
        }
    } else {
        ui.weak(translate(language, "montage_editor_no_selection"));
    }
}
