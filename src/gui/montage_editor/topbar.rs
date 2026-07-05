use super::state::MontageEditorState;
use super::types::PreviewQuality;
use crate::localization::{Language, translate};
use eframe::egui;
use egui::{Color32, Layout};
use std::collections::BTreeSet;

// ─── Топ-бар ─────────────────────────────────────────────────────────────────

pub(super) fn draw_topbar(
    ui: &mut egui::Ui,
    language: Language,
    editor: &mut MontageEditorState,
    is_awaiting: bool,
    job_id: u64,
    jobs: &[crate::queue::PipelineJob],
) -> bool {
    let mut continue_clicked = false;
    let placeholder_segments = collect_placeholder_segment_indices(editor);
    ui.horizontal(|ui| {
        draw_preview_settings(ui, language, editor);

        let total_dur = editor.total_dur();
        let dm = (total_dur / 60.0) as u32;
        let ds = (total_dur % 60.0) as u32;
        ui.label(
            egui::RichText::new(format!(
                "{} кліпів | {:02}:{:02}",
                editor.clips.len(),
                dm,
                ds
            ))
            .weak()
            .size(11.0),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // Кнопка CapCut (права)
            let capcut_btn = ui.add_enabled(
                is_awaiting,
                egui::Button::new(
                    egui::RichText::new(translate(language, "montage_render_capcut"))
                        .strong()
                        .color(if is_awaiting {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        }),
                )
                .fill(if is_awaiting {
                    Color32::from_rgb(41, 128, 185)
                } else {
                    Color32::from_rgb(30, 30, 35)
                }),
            );
            if capcut_btn.clicked() {
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    *job.capcut_mode_override.lock().unwrap() = Some(true);
                }
                let job_name = editor.job_name.clone();
                let save_path_str = editor.save_path.display().to_string();
                let clip_count = editor.clips.iter().filter(|c| c.track_idx == 0).count();
                match editor.save_to_timeline() {
                    Ok(_) => crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Montage editor: timeline saved ({} clips, path: {}) → CapCut",
                            clip_count, save_path_str
                        ),
                    ),
                    Err(e) => crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Montage editor: SAVE FAILED: {} (path: {})",
                            e, save_path_str
                        ),
                    ),
                }
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    let (lock, cvar) = &*job.montage_control_resume;
                    *lock.lock().unwrap() = true;
                    cvar.notify_one();
                }
                continue_clicked = true;
            }

            ui.add_space(4.0);

            // Кнопка FFmpeg (ліва від CapCut)
            let ffmpeg_btn = ui.add_enabled(
                is_awaiting,
                egui::Button::new(
                    egui::RichText::new(translate(language, "montage_render_ffmpeg"))
                        .strong()
                        .color(if is_awaiting {
                            Color32::WHITE
                        } else {
                            Color32::GRAY
                        }),
                )
                .fill(if is_awaiting {
                    Color32::from_rgb(39, 174, 96)
                } else {
                    Color32::from_rgb(30, 30, 35)
                }),
            );
            if ffmpeg_btn.clicked() {
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    *job.capcut_mode_override.lock().unwrap() = Some(false);
                }
                let job_name = editor.job_name.clone();
                let save_path_str = editor.save_path.display().to_string();
                let clip_count = editor.clips.iter().filter(|c| c.track_idx == 0).count();
                match editor.save_to_timeline() {
                    Ok(_) => crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Montage editor: timeline saved ({} clips, path: {}) → FFmpeg",
                            clip_count, save_path_str
                        ),
                    ),
                    Err(e) => crate::logger::log_job(
                        job_id,
                        &job_name,
                        &format!(
                            "Montage editor: SAVE FAILED: {} (path: {})",
                            e, save_path_str
                        ),
                    ),
                }
                if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                    let (lock, cvar) = &*job.montage_control_resume;
                    *lock.lock().unwrap() = true;
                    cvar.notify_one();
                }
                continue_clicked = true;
            }

            ui.add_space(8.0);

            if !placeholder_segments.is_empty() {
                let label = format!(
                    "✨ {} ({})",
                    translate(language, "montage_editor_regen_placeholders"),
                    placeholder_segments.len()
                );
                if ui
                    .button(label)
                    .on_hover_text(translate(
                        language,
                        "montage_editor_regen_placeholders_hint",
                    ))
                    .clicked()
                {
                    editor.pending_placeholder_batch_regen = placeholder_segments.clone();
                }
                ui.add_space(4.0);
            }

            // Кнопка розгортання на весь екран / згортання у вікно
            let max_text = if editor.maximized { "🗗" } else { "🗖" };
            let max_tooltip = if editor.maximized {
                translate(language, "montage_editor_restore")
            } else {
                translate(language, "montage_editor_maximize")
            };
            if ui
                .button(egui::RichText::new(max_text).size(14.0))
                .on_hover_text(max_tooltip)
                .clicked()
            {
                editor.maximized = !editor.maximized;
            }

            ui.add_space(4.0);

            // Кнопка перебудови таймлінії (читає зміни агента і перезавантажує редактор)
            if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                let rebuild_arc = std::sync::Arc::clone(&job.timeline_rebuild_requested);
                if ui
                    .button(egui::RichText::new("🔄").size(13.0))
                    .on_hover_text(translate(language, "agent_chat_rebuild_btn"))
                    .clicked()
                {
                    *rebuild_arc.lock().unwrap() = true;
                }
            }
        });
    });
    continue_clicked
}

fn collect_placeholder_segment_indices(editor: &MontageEditorState) -> Vec<usize> {
    let mut segments = BTreeSet::new();
    for clip in &editor.clips {
        if !clip.is_placeholder {
            continue;
        }

        let seg_idx = clip.stock_seg_idx.or_else(|| {
            clip.media_id
                .strip_prefix("placeholder_")
                .and_then(|value| value.parse::<usize>().ok())
        });
        if let Some(seg_idx) = seg_idx {
            segments.insert(seg_idx);
        }
    }
    segments.into_iter().collect()
}

fn draw_preview_settings(ui: &mut egui::Ui, language: Language, editor: &mut MontageEditorState) {
    let mut quality = editor.preview_render.quality;
    let mut fps = editor.preview_render.fps;

    ui.label(egui::RichText::new(translate(language, "montage_preview_settings")).size(11.0));

    egui::ComboBox::from_id_salt("montage_preview_quality")
        .width(104.0)
        .selected_text(translate(language, quality.label_key()))
        .show_ui(ui, |ui| {
            for option in [
                PreviewQuality::Performance,
                PreviewQuality::Balanced,
                PreviewQuality::High,
                PreviewQuality::Ultra,
            ] {
                ui.selectable_value(
                    &mut quality,
                    option,
                    translate(language, option.label_key()),
                );
            }
        });

    egui::ComboBox::from_id_salt("montage_preview_fps")
        .width(72.0)
        .selected_text(format!("{} FPS", fps.round() as u32))
        .show_ui(ui, |ui| {
            for option in [15.0_f32, 30.0, 60.0] {
                ui.selectable_value(&mut fps, option, format!("{} FPS", option as u32));
            }
        });

    if quality != editor.preview_render.quality || (fps - editor.preview_render.fps).abs() > 0.1 {
        editor.set_preview_render(quality, fps);
    }
}
