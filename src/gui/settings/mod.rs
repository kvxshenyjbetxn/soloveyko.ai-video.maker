pub mod api;
pub mod general;
pub mod storage;

use crate::theme::AppTheme;
use crate::localization::{Language, translate};
use eframe::egui;

/// Перерахування для представлення доступних підвкладок налаштувань.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSubTab {
    General,
    VoiceoverVoiceBot,
    VideoGoogler,
    ApiOpenRouter,
}

/// Головна функція для малювання вкладки налаштувань.
///
/// Створює двопанельний інтерфейс: ліворуч відображає ієрархічне меню вибору підвкладок,
/// праворуч — вміст обраної підвкладки.
pub fn draw_settings(
    ui: &mut egui::Ui,
    current_theme: &mut AppTheme,
    accent_color: &mut egui::Color32,
    active_subtab: &mut SettingsSubTab,
    language: &mut Language,
) {
    ui.horizontal(|ui| {
        let font_id = egui::TextStyle::Button.resolve(ui.style());

        let general_label = translate(*language, "settings_general");
        let api_label = translate(*language, "settings_api");
        let voiceover_label = translate(*language, "settings_api_voiceover");
        let video_label = translate(*language, "settings_api_video");
        let openrouter_label = translate(*language, "settings_api_openrouter");

        let subtab_names = [
            general_label,
            api_label,
            voiceover_label,
            video_label,
            openrouter_label,
            "Voice Bot",
            "Googler",
        ];
        let mut max_word_width = 0.0_f32;

        for name in &subtab_names {
            for word in name.split_whitespace() {
                let word_width = ui.fonts(|f| {
                    f.layout_no_wrap(word.to_string(), font_id.clone(), egui::Color32::PLACEHOLDER)
                        .size()
                        .x
                });
                if word_width > max_word_width {
                    max_word_width = word_width;
                }
            }
        }

        let button_padding_x = ui.spacing().button_padding.x;
        let panel_width = max_word_width + button_padding_x * 2.0 + 54.0;

        // Ліва панель — деревоподібне меню
        ui.vertical(|ui| {
            ui.set_width(panel_width);
            ui.add_space(8.0);

            ui.selectable_value(active_subtab, SettingsSubTab::General, general_label);
            ui.add_space(8.0);

            egui::CollapsingHeader::new(api_label)
                .default_open(true)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(4.0);

                        egui::CollapsingHeader::new(voiceover_label)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.add_space(2.0);
                                ui.selectable_value(active_subtab, SettingsSubTab::VoiceoverVoiceBot, "Voice Bot");
                            });

                        ui.add_space(6.0);

                        egui::CollapsingHeader::new(video_label)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.add_space(2.0);
                                ui.selectable_value(active_subtab, SettingsSubTab::VideoGoogler, "Googler");
                            });

                        ui.add_space(6.0);

                        ui.selectable_value(active_subtab, SettingsSubTab::ApiOpenRouter, openrouter_label);
                    });
                });
        });

        ui.separator();

        // Права панель — вміст активної підвкладки
        ui.vertical(|ui| {
            ui.add_space(8.0);

            match active_subtab {
                SettingsSubTab::General => {
                    general::draw_general_settings(ui, current_theme, accent_color, language);
                }
                SettingsSubTab::VoiceoverVoiceBot => {
                    api::voiceover::voicebot::draw(ui, *language);
                }
                SettingsSubTab::VideoGoogler => {
                    api::video::googler::draw(ui, *language);
                }
                SettingsSubTab::ApiOpenRouter => {
                    api::openrouter::draw(ui, *language);
                }
            }
        });
    });
}
