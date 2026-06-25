use eframe::egui;

use super::{Tab, VideoMakerApp};
use crate::localization::translate;

impl VideoMakerApp {
    /// Перевіряє результат фонової перевірки CLI-інструментів і відкриває welcome-вікно за потреби.
    pub(super) fn poll_pending_tool_check(&mut self) {
        if let Some(ref service) = self.pending_tool_check {
            let gemini = self.tool_checks.gemini.lock().unwrap().clone();
            let claude = self.tool_checks.claude.lock().unwrap().clone();

            let mut check_done = false;
            let mut needs_install = false;

            if service == "Gemini CLI" {
                match &gemini {
                    crate::gui::welcome::ToolStatus::Checking => {
                        // Перевірка ще триває
                    }
                    crate::gui::welcome::ToolStatus::NotInstalled => {
                        needs_install = true;
                        check_done = true;
                    }
                    _ => {
                        check_done = true;
                    }
                }
            } else if service == "Claude Code" {
                match &claude {
                    crate::gui::welcome::ToolStatus::Checking => {
                        // Перевірка ще триває
                    }
                    crate::gui::welcome::ToolStatus::NotInstalled => {
                        needs_install = true;
                        check_done = true;
                    }
                    _ => {
                        check_done = true;
                    }
                }
            }

            if check_done {
                if needs_install {
                    self.welcome_open = true;
                }
                self.pending_tool_check = None;
            }
        }
    }

    /// Малює глобальні стартові вікна: привітання та сповіщення про оновлення.
    pub(super) fn draw_startup_windows(&mut self, ctx: &egui::Context) {
        // Вікно привітання — відображається при першому запуску
        if self.welcome_open {
            let closed = crate::gui::welcome::draw_welcome_dialog(
                ctx,
                &mut self.welcome_open,
                &mut self.welcome_dont_show,
                &self.tool_checks,
                self.language,
            );
            // Якщо щойно натиснуто "Закрити" і стоїть галочка — зберігаємо show_welcome=false
            if closed && self.welcome_dont_show {
                let mut new_settings = self.last_saved_settings.clone();
                new_settings.show_welcome = false;
                crate::gui::settings::storage::save_settings(&new_settings);
                self.last_saved_settings = new_settings;
            }
        }

        // Перевіряємо результат фонової перевірки оновлень і відкриваємо діалог
        {
            let has_update = self.update_info.lock().unwrap().is_some();
            if has_update && !self.update_dialog_open {
                self.update_dialog_open = true;
            }
        }
        if self.update_dialog_open {
            crate::gui::update_dialog::draw_update_dialog(
                ctx,
                self.language,
                &self.update_info,
                &mut self.update_dialog_open,
            );
        }
    }

    /// Малює верхню панель, вікно балансів і статус потоків.
    pub(super) fn draw_topbar_windows(&mut self, ctx: &egui::Context) {
        // Верхня панель для навігації між вкладками
        crate::gui::topbar::draw_navigation_bar(
            ctx,
            &mut self.active_tab,
            &self.jobs,
            self.language,
            &self.openrouter_balance,
            &self.voicebot_balance,
            &self.googler_balance,
            &mut self.balance_window_open,
        );

        // Плаваюче вікно з детальними балансами
        crate::gui::topbar::draw_balance_window(
            ctx,
            &mut self.balance_window_open,
            self.language,
            &self.openrouter_key,
            &self.openrouter_balance,
            &self.voicebot_key,
            &self.voicebot_balance,
            &self.googler_key,
            &self.googler_balance,
        );

        crate::gui::topbar::draw_threads_window(
            ctx,
            &mut self.threads_window_open,
            self.language,
            &mut self.openrouter_max_threads,
            &mut self.claude_max_threads,
            &mut self.gemini_max_threads,
            &mut self.codex_max_threads,
            &mut self.agy_max_threads,
            &mut self.pi_max_threads,
            &self.voicebot_balance,
            &mut self.edge_tts_max_threads,
            &mut self.googler_image_max_threads,
            &mut self.googler_video_max_threads,
            &mut self.ffmpeg_max_threads,
        );

        // Нижній рядок статусу потоків (реєструємо ДО SidePanel, щоб займав повну ширину)
        crate::gui::topbar::draw_status_bar(
            ctx,
            self.openrouter_max_threads,
            self.claude_max_threads,
            self.gemini_max_threads,
            self.codex_max_threads,
            self.agy_max_threads,
            self.pi_max_threads,
            self.edge_tts_max_threads,
            self.ffmpeg_max_threads,
            self.googler_image_max_threads,
            self.googler_video_max_threads,
            &mut self.threads_window_open,
        );
    }

    /// Малює відкриті користувачем вікна логів задач.
    pub(super) fn draw_job_logs_windows(&mut self, ctx: &egui::Context) {
        let log_ids: Vec<(u64, String)> = self
            .open_job_logs
            .iter()
            .map(|(&id, name)| (id, name.clone()))
            .collect();
        let mut to_close_logs = Vec::new();
        for (job_id, job_name) in log_ids {
            if !crate::gui::logs::draw_job_logs_window(
                ctx,
                self.language,
                job_id,
                &job_name,
                &mut self.auto_scroll_logs,
                &mut self.copied_toast,
            ) {
                to_close_logs.push(job_id);
            }
        }
        for id in to_close_logs {
            self.open_job_logs.remove(&id);
        }
    }

    /// Синхронізує автоматичні вікна контролю і перехід до галереї для media-control задач.
    pub(super) fn sync_control_windows(&mut self) {
        // Авто-відкриття вікна контролю коли задача переходить в AwaitingControl
        if self.pipeline_control_auto_open {
            for job in &self.jobs {
                if !self.control_dismissed.contains(&job.id)
                    && *job.status.lock().unwrap() == crate::queue::JobStatus::AwaitingControl
                    && !self.open_job_controls.contains_key(&job.id)
                {
                    let text = job
                        .translated_text
                        .lock()
                        .unwrap()
                        .clone()
                        .unwrap_or_default();
                    self.open_job_controls.insert(
                        job.id,
                        crate::gui::pipeline::translation_control::TranslationControlWindowState::new_with_text(text),
                    );
                }
            }
        }

        // Авто-перехід на вкладку Галерея при першій появі AwaitingMediaControl (лише один раз на задачу)
        for job in &self.jobs {
            if *job.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMediaControl
                && !self.media_control_notified.contains(&job.id)
            {
                self.media_control_notified.insert(job.id);
                self.active_tab = Tab::Gallery;
            }
        }
    }

    /// Малює та обробляє Stock Picker поверх інших вікон.
    pub(super) fn draw_stock_picker_window(&mut self, ctx: &egui::Context) {
        if let Some(ref mut picker_state) = self.stock_picker_state {
            let action =
                crate::gui::stock_picker::draw_stock_picker(ctx, self.language, picker_state);
            match action {
                crate::gui::stock_picker::StockPickerAction::Close => {
                    self.stock_picker_state = None;
                }
                crate::gui::stock_picker::StockPickerAction::Confirmed(maybe_media) => {
                    self.stock_picker_state = None;
                    if let Some(ref mut editor) = self.montage_editor_state {
                        // Переносимо MediaItem з trim-редактора в пул — вже містить витягнуті кадри
                        if let Some(media) = maybe_media {
                            editor.media_pool.retain(|m| m.path != media.path);
                            editor.media_pool.push(media);
                        }
                        editor.needs_stock_refresh = true;
                    }
                }
                crate::gui::stock_picker::StockPickerAction::None => {}
            }
        }
    }

    /// Малює вікна контролю перекладу та чатів з агентом.
    pub(super) fn draw_control_and_chat_windows(&mut self, ctx: &egui::Context) {
        // Спливаючі вікна контролю перекладу (по одному на задачу)
        crate::gui::pipeline::translation_control::draw_translation_control_windows(
            ctx,
            self.language,
            &self.jobs,
            &mut self.open_job_controls,
            &mut self.control_dismissed,
            &self.openrouter_models,
            &self.openrouter_models_loading,
        );

        // Спливаючі вікна чату з агентом (по одному на задачу)
        crate::gui::agent_chat_window::draw_agent_chat_windows(
            ctx,
            self.language,
            &self.jobs,
            &mut self.open_agent_chats,
        );
    }

    /// Відображає спливаюче сповіщення про копіювання.
    pub(super) fn draw_copied_toast(&mut self, ctx: &egui::Context) {
        if let Some((_, instant)) = &self.copied_toast {
            if instant.elapsed().as_secs_f32() < 2.0 {
                egui::Area::new(egui::Id::new("copied_toast"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_black_alpha(220))
                            .rounding(8.0)
                            .stroke(egui::Stroke::new(1.0, self.accent_color))
                            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(translate(
                                        self.language,
                                        "logs_copied_toast",
                                    ))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                                    .size(13.0),
                                );
                            });
                    });

                // Просимо eframe перемалювати екран, щоб таймер оновлювався плавно
                ctx.request_repaint();
            }
        }
    }

    /// Малює вкладку системних логів роботи додатку.
    pub(super) fn draw_logs_tab(&mut self, ui: &mut egui::Ui) {
        crate::gui::logs::draw_logs_tab(
            ui,
            self.language,
            &mut self.auto_scroll_logs,
            &mut self.copied_toast,
        );
    }
}
