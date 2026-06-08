use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::localization::{Language, translate};
use crate::api::updater::UpdateInfo;

/// Відображає модальне вікно з повідомленням про доступне оновлення.
pub fn draw_update_dialog(
    ctx: &egui::Context,
    lang: Language,
    update_info: &Arc<Mutex<Option<UpdateInfo>>>,
    open: &mut bool,
) {
    let info = {
        let guard = update_info.lock().unwrap();
        guard.clone()
    };

    let Some(info) = info else { return };

    egui::Window::new(translate(lang, "update_title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(480.0, 360.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(translate(lang, "update_new_version"));
                    ui.strong(&info.version);
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(translate(lang, "update_changelog"));
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut info.changelog.as_str())
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(translate(lang, "update_download_btn"))
                        .clicked()
                    {
                        crate::api::updater::open_url(&info.download_url);
                    }
                    ui.add_space(8.0);
                    if ui.button(translate(lang, "update_later_btn")).clicked() {
                        *update_info.lock().unwrap() = None;
                        *open = false;
                    }
                });

                ui.add_space(4.0);
            });
        });
}
