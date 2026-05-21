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
    VoiceoverElevenLabs,
    VoiceoverOpenAi,
    VoiceoverPlayHt,
    
    // Провайдери Відеоряду
    VideoLeonardo,
    VideoDallE,
    VideoMidjourney,
    
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
            "ElevenLabs",
            "OpenAI TTS",
            "Play.ht",
            "Leonardo.ai",
            "DALL-E 3",
            "Midjourney",
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
                                    ui.selectable_value(active_subtab, SettingsSubTab::VoiceoverElevenLabs, "ElevenLabs");
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VoiceoverOpenAi, "OpenAI TTS");
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VoiceoverPlayHt, "Play.ht");
                                });
                            });
                        
                        ui.add_space(6.0);

                        // Підкатегорія Відеоряд (розгорнута за замовчуванням)
                        egui::CollapsingHeader::new(video_label)
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VideoLeonardo, "Leonardo.ai");
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VideoDallE, "DALL-E 3");
                                    ui.add_space(2.0);
                                    ui.selectable_value(active_subtab, SettingsSubTab::VideoMidjourney, "Midjourney");
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
                SettingsSubTab::VoiceoverElevenLabs => {
                    ui.heading(format!("{}: ElevenLabs", voiceover_label));
                    ui.separator();
                }
                SettingsSubTab::VoiceoverOpenAi => {
                    ui.heading(format!("{}: OpenAI TTS", voiceover_label));
                    ui.separator();
                }
                SettingsSubTab::VoiceoverPlayHt => {
                    ui.heading(format!("{}: Play.ht", voiceover_label));
                    ui.separator();
                }
                
                // Вкладки Відеоряду
                SettingsSubTab::VideoLeonardo => {
                    ui.heading(format!("{}: Leonardo.ai", video_label));
                    ui.separator();
                }
                SettingsSubTab::VideoDallE => {
                    ui.heading(format!("{}: DALL-E 3", video_label));
                    ui.separator();
                }
                SettingsSubTab::VideoMidjourney => {
                    ui.heading(format!("{}: Midjourney", video_label));
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
