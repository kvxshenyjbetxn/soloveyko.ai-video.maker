use eframe::egui;

use super::{Tab, VideoMakerApp};

impl VideoMakerApp {
    /// Малює нижню панель черги задач, якщо вона доступна для поточної вкладки.
    pub(super) fn draw_queue_panel_host(&mut self, ctx: &egui::Context) {
        // Нижня панель черги задач (тільки якщо є задачі і ми не на Gallery)
        if !self.jobs.is_empty() && self.active_tab != Tab::Gallery {
            let minimized = self.queue_panel_collapsed || self.queue_panel_fullscreen;
            let mut panel = egui::TopBottomPanel::bottom("queue_panel").resizable(!minimized);
            panel = if minimized {
                panel.exact_height(32.0)
            } else {
                panel
                    .min_height(140.0)
                    .default_height(160.0)
                    .max_height(350.0)
            };
            panel.show(ctx, |ui| {
                crate::gui::queue::draw_queue_panel(
                    ui,
                    self.language,
                    &mut self.jobs,
                    &mut self.job_counter,
                    &mut self.open_job_logs,
                    &mut self.open_job_controls,
                    &self.whisper_model_download,
                    &mut self.retry_request,
                    &mut self.queue_cancel_confirm_job,
                    &mut self.open_agent_chats,
                    &mut self.montage_editor_open_job,
                    &mut self.queue_panel_collapsed,
                    &mut self.queue_panel_fullscreen,
                );
            });

            if self.jobs.is_empty() {
                self.job_counter = 0;
                self.queue_panel_collapsed = false;
                self.queue_panel_fullscreen = false;
                self.queue_cancel_confirm_job = None;
            }
        }
    }

    /// Малює чергу в центральній області, коли увімкнено повноекранний режим.
    pub(super) fn draw_fullscreen_queue_if_needed(&mut self, ui: &mut egui::Ui) -> bool {
        if self.queue_panel_fullscreen && !self.jobs.is_empty() && self.active_tab != Tab::Gallery {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 8.0,
                    right: 0.0,
                    top: 8.0,
                    bottom: 0.0,
                })
                .show(ui, |ui| {
                    crate::gui::queue::draw_queue_jobs_list(
                        ui,
                        self.language,
                        &mut self.jobs,
                        &mut self.open_job_logs,
                        &mut self.open_job_controls,
                        &mut self.retry_request,
                        &mut self.queue_cancel_confirm_job,
                        &mut self.open_agent_chats,
                        &mut self.montage_editor_open_job,
                    );
                });

            if self.jobs.is_empty() {
                self.job_counter = 0;
                self.queue_panel_collapsed = false;
                self.queue_panel_fullscreen = false;
                self.queue_cancel_confirm_job = None;
            }
            return true;
        }

        false
    }

    /// Обробляє запит на повтор конкретного етапу задачі.
    pub(super) fn handle_retry_request(&mut self, ctx: &egui::Context) {
        if let Some((target_id, stage)) = self.retry_request.take() {
            if let Some(job) = self.jobs.iter().find(|j| j.id == target_id) {
                crate::core::pipeline::retry_from_stage(
                    stage,
                    job.id,
                    job.name.clone(),
                    job.settings.clone(),
                    std::sync::Arc::clone(&job.status),
                    std::sync::Arc::clone(&job.translation_stage),
                    std::sync::Arc::clone(&job.voiceover_stage),
                    std::sync::Arc::clone(&job.video_stage),
                    std::sync::Arc::clone(&job.subtitles_stage),
                    std::sync::Arc::clone(&job.montage_stage),
                    std::sync::Arc::clone(&job.translated_text),
                    std::sync::Arc::clone(&job.total_cost),
                    std::sync::Arc::clone(&job.audio_duration),
                    std::sync::Arc::clone(&job.prompts_progress),
                    std::sync::Arc::clone(&job.media_progress),
                    std::sync::Arc::clone(&job.montage_progress),
                    std::sync::Arc::clone(&job.montage_file_size),
                    std::sync::Arc::clone(&job.media_control_resume),
                    std::sync::Arc::clone(&job.montage_control_resume),
                    std::sync::Arc::clone(&job.agent_control_resume),
                    std::sync::Arc::clone(&job.agent_chat),
                    std::sync::Arc::clone(&job.agent_session),
                    std::sync::Arc::clone(&job.capcut_mode_override),
                    ctx.clone(),
                );
            }
        }
    }
}
