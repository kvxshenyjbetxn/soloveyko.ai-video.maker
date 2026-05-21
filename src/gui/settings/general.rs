use crate::theme::AppTheme;
use crate::localization::{Language, translate};
use eframe::egui;

/// Малює секцію загальних налаштувань програми, вибір теми, мови та акцентного кольору.
pub fn draw_general_settings(
    ui: &mut egui::Ui,
    current_theme: &mut AppTheme,
    accent_color: &mut egui::Color32,
    language: &mut Language,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        
        // Заголовок підвкладки "Основні"
        ui.heading(translate(*language, "settings_general_title"));
        ui.separator();
        
        ui.add_space(12.0);

        // Блок вибору мови інтерфейсу
        ui.strong(translate(*language, "settings_lang"));
        ui.small(translate(*language, "settings_lang_desc"));
        
        ui.add_space(8.0);
        
        ui.horizontal(|ui| {
            ui.radio_value(language, Language::Uk, translate(*language, "settings_lang_uk"));
            ui.add_space(16.0);
            ui.radio_value(language, Language::En, translate(*language, "settings_lang_en"));
            ui.add_space(16.0);
            ui.radio_value(language, Language::Ru, translate(*language, "settings_lang_ru"));
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);
        
        // Блок вибору теми оформлення
        ui.strong(translate(*language, "settings_theme"));
        ui.small(translate(*language, "settings_theme_desc"));
        
        ui.add_space(8.0);
        
        // Контейнер вибору тем (без рамок)
        ui.vertical(|ui| {
            ui.radio_value(current_theme, AppTheme::Light, translate(*language, "settings_theme_light"));
            ui.add_space(6.0);
            ui.radio_value(current_theme, AppTheme::Dark, translate(*language, "settings_theme_dark"));
            ui.add_space(6.0);
            ui.radio_value(current_theme, AppTheme::Amoled, translate(*language, "settings_theme_amoled"));
        });

        ui.add_space(16.0);
        ui.strong(translate(*language, "settings_accent"));
        ui.small(translate(*language, "settings_accent_desc"));
        
        ui.add_space(8.0);

        // Блок налаштування кольору акценту (без рамок)
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Список готових стильних кольорів для швидкого вибору
                let presets = [
                    (translate(*language, "color_blue"), egui::Color32::from_rgb(0, 122, 255)),
                    (translate(*language, "color_green"), egui::Color32::from_rgb(46, 204, 113)),
                    (translate(*language, "color_red"), egui::Color32::from_rgb(231, 76, 60)),
                    (translate(*language, "color_orange"), egui::Color32::from_rgb(230, 126, 34)),
                    (translate(*language, "color_purple"), egui::Color32::from_rgb(155, 89, 182)),
                ];

                ui.label(translate(*language, "settings_accent_quick"));
                ui.add_space(4.0);

                for (name, color) in presets {
                    // Робимо кнопку кольоровою, якщо вона вибрана, або стандартного фону
                    let is_selected = *accent_color == color;
                    let button = egui::Button::new(name)
                        .fill(if is_selected { color } else { ui.style().visuals.widgets.noninteractive.bg_fill });

                    if ui.add(button).clicked() {
                        *accent_color = color;
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(translate(*language, "settings_accent_custom"));
                ui.add_space(8.0);
                
                // Повна палітра вільного вибору для точного налаштування кольору
                ui.color_edit_button_srgba(accent_color);
            });
        });

        ui.add_space(24.0);
        ui.separator();
        ui.add_space(12.0);
        
        ui.strong(translate(*language, "settings_data"));
        ui.small(translate(*language, "settings_data_desc"));
        ui.add_space(8.0);
        
        if ui.button(translate(*language, "settings_open_folder")).clicked() {
            super::storage::open_settings_folder();
        }
    });
}
