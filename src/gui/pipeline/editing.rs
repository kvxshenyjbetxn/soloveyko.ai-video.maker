use crate::localization::{translate, Language};
use eframe::egui;

/// Всі доступні xfade переходи FFmpeg.
pub const XFADE_TRANSITIONS: &[&str] = &[
    "fade",
    "wipeleft",
    "wiperight",
    "wipeup",
    "wipedown",
    "slideleft",
    "slideright",
    "slideup",
    "slidedown",
    "smoothleft",
    "smoothright",
    "smoothup",
    "smoothdown",
    "circlecrop",
    "rectcrop",
    "distance",
    "fadeblack",
    "fadewhite",
    "fadegrays",
    "pixelize",
    "diagtl",
    "diagtr",
    "diagbl",
    "diagbr",
    "hlslice",
    "hrslice",
    "vuslice",
    "vdslice",
    "dissolve",
    "hblur",
    "hlwind",
    "hrwind",
    "vuwind",
    "vdwind",
    "coverleft",
    "coverright",
    "coverup",
    "coverdown",
    "revealleft",
    "revealright",
    "revealup",
    "revealdown",
    "zoomin",
    "squeezeh",
    "squeezev",
    "horzopen",
    "horzclose",
    "vertopen",
    "vertclose",
    "circleopen",
    "circleclose",
    "radial",
];

/// Малює секцію "Монтаж" на панелі пайплайну.
pub fn draw_editing_section(
    ui: &mut egui::Ui,
    language: Language,
    capcut_enabled: &mut bool,
    capcut_draft_path: &mut String,
    montage_service: &mut String,
    montage_fps: &mut u32,
    montage_preset: &mut String,
    montage_bitrate: &mut u32,
    montage_transition: &mut String,
    montage_transition_duration: &mut f32,
    montage_image_zoom_enabled: &mut bool,
    montage_image_zoom_mode: &mut String,
    montage_image_zoom_scale: &mut f32,
    montage_image_shake_enabled: &mut bool,
    montage_image_shake_intensity: &mut f32,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Перемикач: CapCut або FFmpeg
        ui.horizontal(|ui| {
            crate::gui::pipeline::toggle_switch(ui, capcut_enabled);
            ui.label(egui::RichText::new(translate(language, "capcut_toggle_label")).strong());
        });

        if *capcut_enabled {
            // ─── Режим CapCut ────────────────────────────────────────────────
            ui.add_space(8.0);
            ui.label(egui::RichText::new(translate(language, "capcut_draft_path_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 60.0).max(60.0);
                ui.add(
                    egui::TextEdit::singleline(capcut_draft_path)
                        .hint_text(translate(language, "capcut_draft_path_hint"))
                        .desired_width(w),
                );
                if ui
                    .button("📁")
                    .on_hover_text(translate(language, "capcut_draft_path_hint"))
                    .clicked()
                {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        *capcut_draft_path = folder.to_string_lossy().into_owned();
                    }
                }
            });
            ui.add_space(6.0);
            return;
        }

        ui.add_space(8.0);

        // Вибір сервісу (FFmpeg)
        ui.label(egui::RichText::new(translate(language, "montage_service_label")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("montage_service_combo")
            .selected_text(montage_service.as_str())
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(montage_service, "FFmpeg".to_string(), "FFmpeg");
            });

        // Налаштування FFmpeg
        if montage_service.as_str() == "FFmpeg" {
            ui.add_space(8.0);

            // FPS
            ui.label(egui::RichText::new(translate(language, "montage_fps_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(montage_fps, 1..=120).suffix(" fps"));
            });

            ui.add_space(8.0);

            // Пресет
            ui.label(egui::RichText::new(translate(language, "montage_preset_label")).strong());
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("montage_preset_combo")
                .selected_text(montage_preset.as_str())
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    for p in &[
                        "ultrafast",
                        "superfast",
                        "veryfast",
                        "faster",
                        "fast",
                        "medium",
                        "slow",
                        "slower",
                        "veryslow",
                    ] {
                        ui.selectable_value(montage_preset, p.to_string(), *p);
                    }
                });

            ui.add_space(8.0);

            // Бітрейт
            ui.label(egui::RichText::new(translate(language, "montage_bitrate_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(montage_bitrate)
                        .range(1..=100)
                        .suffix(" MB/s"),
                );
            });

            ui.add_space(8.0);

            // Перехід між кліпами
            ui.label(egui::RichText::new(translate(language, "montage_transition_label")).strong());
            ui.add_space(4.0);

            let selected_text = match montage_transition.as_str() {
                "none" => translate(language, "montage_transition_none").to_string(),
                "random" => translate(language, "montage_transition_random").to_string(),
                other => other.to_string(),
            };

            egui::ComboBox::from_id_salt("montage_transition_combo")
                .selected_text(selected_text)
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        montage_transition,
                        "none".to_string(),
                        translate(language, "montage_transition_none"),
                    );
                    ui.selectable_value(
                        montage_transition,
                        "random".to_string(),
                        translate(language, "montage_transition_random"),
                    );
                    ui.separator();
                    for &t in XFADE_TRANSITIONS {
                        ui.selectable_value(montage_transition, t.to_string(), t);
                    }
                });

            // Тривалість переходу — тільки якщо перехід увімкнено
            if montage_transition.as_str() != "none" {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(translate(language, "montage_transition_duration_label"))
                        .strong(),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(montage_transition_duration)
                            .range(0.1..=3.0)
                            .speed(0.05)
                            .suffix(" s"),
                    );
                });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Підказка про те, що ефекти тільки для зображень
            ui.label(
                egui::RichText::new(translate(language, "montage_image_effects_note"))
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );

            ui.add_space(8.0);

            // Ефект зуму
            ui.horizontal(|ui| {
                crate::gui::pipeline::toggle_switch(ui, montage_image_zoom_enabled);
                ui.label(
                    egui::RichText::new(translate(language, "montage_image_zoom_label")).strong(),
                );
            });

            if *montage_image_zoom_enabled {
                ui.add_space(4.0);
                ui.label(translate(language, "montage_image_zoom_mode_label"));
                ui.horizontal(|ui| {
                    let is_alternate = montage_image_zoom_mode == "alternate";
                    if ui
                        .selectable_label(
                            is_alternate,
                            translate(language, "montage_image_zoom_mode_alternate"),
                        )
                        .clicked()
                    {
                        *montage_image_zoom_mode = "alternate".to_string();
                    }
                    if ui
                        .selectable_label(
                            !is_alternate,
                            translate(language, "montage_image_zoom_mode_oscillate"),
                        )
                        .clicked()
                    {
                        *montage_image_zoom_mode = "oscillate".to_string();
                    }
                });
                ui.add_space(4.0);
                ui.label(translate(language, "montage_image_zoom_scale_label"));
                ui.add(
                    egui::Slider::new(montage_image_zoom_scale, 1.1..=2.0)
                        .step_by(0.05)
                        .show_value(true),
                );
            }

            ui.add_space(8.0);

            // Ефект покачування
            ui.horizontal(|ui| {
                crate::gui::pipeline::toggle_switch(ui, montage_image_shake_enabled);
                ui.label(
                    egui::RichText::new(translate(language, "montage_image_shake_label")).strong(),
                );
            });

            if *montage_image_shake_enabled {
                ui.add_space(4.0);
                ui.label(translate(language, "montage_image_shake_intensity_label"));
                ui.add(
                    egui::Slider::new(montage_image_shake_intensity, 0.1..=1.0)
                        .step_by(0.05)
                        .show_value(true),
                );
            }
        }

        ui.add_space(6.0);
    });
}
