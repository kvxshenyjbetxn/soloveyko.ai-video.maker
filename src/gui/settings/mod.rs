pub mod general;
pub mod storage;

use crate::theme::AppTheme;
use crate::localization::{Language, translate};
use eframe::egui;

/// Перерахування для представлення доступних підвкладок налаштувань.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSubTab {
    /// Основні налаштування
    General,
    
    // Провайдери Озвучки
    VoiceoverVoiceBot,
    
    // Провайдери Відеоряду
    VideoGoogler,
    
    /// Підвкладка API: OpenRouter
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
        // Визначаємо стиль шрифту для кнопок меню
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        
        let general_label = translate(*language, "settings_general");
        let api_label = translate(*language, "settings_api");
        let voiceover_label = translate(*language, "settings_api_voiceover");
        let video_label = translate(*language, "settings_api_video");
        let openrouter_label = translate(*language, "settings_api_openrouter");
        
        // Список усіх назв підвкладок для визначення найдовшого слова
        let subtab_names = [
            general_label,
            api_label,
            voiceover_label,
            video_label,
            openrouter_label,
            "Voice Bot",
            "Googler",
        ];
        let mut max_word_width = 0.0;
        
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

        // Розраховуємо ширину лівої панелі з урахуванням відступів для вкладеності 3-го рівня.
        let button_padding_x = ui.spacing().button_padding.x;
        let panel_width = max_word_width + button_padding_x * 2.0 + 54.0;

        // Ліва панель для деревоподібного меню налаштувань
        ui.vertical(|ui| {
            ui.set_width(panel_width);
            ui.add_space(8.0);
            
            // Основні налаштування
            ui.selectable_value(active_subtab, SettingsSubTab::General, general_label);
            ui.add_space(8.0);
            
            // Секція АПІ як CollapsingHeader (розгорнутий за замовчуванням)
            egui::CollapsingHeader::new(api_label)
                .default_open(true)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        
                        // Підкатегорія Озвучка (розгорнута за замовчуванням)
                        egui::CollapsingHeader::new(voiceover_label)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VoiceoverVoiceBot, "Voice Bot");
                                });
                            });
                        
                        ui.add_space(6.0);

                        // Підкатегорія Відеоряд (розгорнута за замовчуванням)
                        egui::CollapsingHeader::new(video_label)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VideoGoogler, "Googler");
                                });
                            });
                        
                        ui.add_space(6.0);

                        // OpenRouter
                        ui.selectable_value(active_subtab, SettingsSubTab::ApiOpenRouter, openrouter_label);
                    });
                });
        });

        // Вертикальний розділювач між меню та вмістом
        ui.separator();

        // Права панель для вмісту активної підвкладки
        ui.vertical(|ui| {
            ui.add_space(8.0);
            
            match active_subtab {
                SettingsSubTab::General => {
                    general::draw_general_settings(ui, current_theme, accent_color, language);
                }
                
                // Вкладки Озвучки
                SettingsSubTab::VoiceoverVoiceBot => {
                    ui.heading(format!("{}: Voice Bot", voiceover_label));
                    ui.separator();
                }
                
                // Вкладки Відеоряду
                SettingsSubTab::VideoGoogler => {
                    ui.heading(format!("{}: Googler", video_label));
                    ui.separator();
                }
                
                // OpenRouter
                SettingsSubTab::ApiOpenRouter => {
                    ui.heading(openrouter_label);
                    ui.separator();
                }
            }
        });
    });
}
