use eframe::egui;
use crate::localization::{Language, translate};

/// Всі доступні xfade переходи FFmpeg.
pub const XFADE_TRANSITIONS: &[&str] = &[
    "fade", "wipeleft", "wiperight", "wipeup", "wipedown",
    "slideleft", "slideright", "slideup", "slidedown",
    "smoothleft", "smoothright", "smoothup", "smoothdown",
    "circlecrop", "rectcrop", "distance",
    "fadeblack", "fadewhite", "fadegrays",
    "pixelize", "diagtl", "diagtr", "diagbl", "diagbr",
    "hlslice", "hrslice", "vuslice", "vdslice",
    "dissolve", "hblur",
    "hlwind", "hrwind", "vuwind", "vdwind",
    "coverleft", "coverright", "coverup", "coverdown",
    "revealleft", "revealright", "revealup", "revealdown",
    "zoomin", "squeezeh", "squeezev",
    "horzopen", "horzclose", "vertopen", "vertclose",
    "circleopen", "circleclose", "radial",
];

/// Малює секцію "Монтаж" на панелі пайплайну.
pub fn draw_editing_section(
    ui: &mut egui::Ui,
    language: Language,
    montage_service: &mut String,
    montage_fps: &mut u32,
    montage_preset: &mut String,
    montage_bitrate: &mut u32,
    montage_transition: &mut String,
    montage_transition_duration: &mut f32,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Вибір сервісу
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
                ui.add(
                    egui::Slider::new(montage_fps, 1..=120)
                        .suffix(" fps"),
                );
            });

            ui.add_space(8.0);

            // Пресет
            ui.label(egui::RichText::new(translate(language, "montage_preset_label")).strong());
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("montage_preset_combo")
                .selected_text(montage_preset.as_str())
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    for p in &["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"] {
                        ui.selectable_value(montage_preset, p.to_string(), *p);
                    }
                });

            ui.add_space(8.0);

            // Бітрейт
            ui.label(egui::RichText::new(translate(language, "montage_bitrate_label")).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(montage_bitrate).range(1..=100).suffix(" MB/s"));
            });

            ui.add_space(8.0);

            // Перехід між кліпами
            ui.label(egui::RichText::new(translate(language, "montage_transition_label")).strong());
            ui.add_space(4.0);

            let selected_text = match montage_transition.as_str() {
                "none"   => translate(language, "montage_transition_none").to_string(),
                "random" => translate(language, "montage_transition_random").to_string(),
                other    => other.to_string(),
            };

            egui::ComboBox::from_id_salt("montage_transition_combo")
                .selected_text(selected_text)
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(montage_transition, "none".to_string(),
                        translate(language, "montage_transition_none"));
                    ui.selectable_value(montage_transition, "random".to_string(),
                        translate(language, "montage_transition_random"));
                    ui.separator();
                    for &t in XFADE_TRANSITIONS {
                        ui.selectable_value(montage_transition, t.to_string(), t);
                    }
                });

            // Тривалість переходу — тільки якщо перехід увімкнено
            if montage_transition.as_str() != "none" {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(translate(language, "montage_transition_duration_label")).strong());
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
        }

        ui.add_space(6.0);
    });
}
