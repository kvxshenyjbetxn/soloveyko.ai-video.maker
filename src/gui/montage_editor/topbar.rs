use eframe::egui;
use egui::{Color32, Layout};
use crate::localization::{Language, translate};
use super::state::MontageEditorState;

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
    ui.horizontal(|ui| {
        ui.label(translate(language, "montage_editor_zoom"));
        ui.add(egui::Slider::new(&mut editor.timeline_zoom, 10.0..=300.0).show_value(false));

        let total_dur = editor.total_dur();
        let dm = (total_dur / 60.0) as u32;
        let ds = (total_dur % 60.0) as u32;
        ui.label(
            egui::RichText::new(format!("{} кліпів | {:02}:{:02}", editor.clips.len(), dm, ds))
                .weak().size(11.0)
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let btn = ui.add_enabled(
                is_awaiting,
                egui::Button::new(
                    egui::RichText::new(translate(language, "montage_editor_continue"))
                        .strong()
                        .color(if is_awaiting { Color32::WHITE } else { Color32::GRAY }),
                )
                .fill(if is_awaiting { Color32::from_rgb(39, 174, 96) } else { Color32::from_rgb(30, 30, 35) })
            );
            if btn.clicked() {
                let job_name = editor.job_name.clone();
                let save_path_str = editor.save_path.display().to_string();
                let clip_count = editor.clips.iter().filter(|c| c.track_idx == 0).count();
                match editor.save_to_timeline() {
                    Ok(_) => crate::logger::log_job(
                        job_id, &job_name,
                        &format!("Montage editor: timeline saved ({} clips, path: {})", clip_count, save_path_str),
                    ),
                    Err(e) => crate::logger::log_job(
                        job_id, &job_name,
                        &format!("Montage editor: SAVE FAILED: {} (path: {})", e, save_path_str),
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

            // Кнопка розгортання на весь екран / згортання у вікно
            let max_text = if editor.maximized { "🗗" } else { "🗖" };
            let max_tooltip = if editor.maximized {
                translate(language, "montage_editor_restore")
            } else {
                translate(language, "montage_editor_maximize")
            };
            if ui.button(egui::RichText::new(max_text).size(14.0))
                .on_hover_text(max_tooltip)
                .clicked()
            {
                editor.maximized = !editor.maximized;
            }

            ui.add_space(4.0);

            // Кнопка перебудови таймлінії (читає зміни агента і перезавантажує редактор)
            if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                let rebuild_arc = std::sync::Arc::clone(&job.timeline_rebuild_requested);
                if ui.button(egui::RichText::new("🔄").size(13.0))
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
