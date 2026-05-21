use crate::theme::AppTheme;
use eframe::egui;

/// Малює секцію загальних налаштувань програми, вибір теми та акцентного кольору.
pub fn draw_general_settings(ui: &mut egui::Ui, current_theme: &mut AppTheme, accent_color: &mut egui::Color32) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        
        // Заголовок підвкладки "Основні"
        ui.heading("Основні налаштування");
        ui.separator();
        
        ui.add_space(12.0);
        ui.strong("Тема оформлення");
        ui.small("Виберіть колірну схему графічного інтерфейсу:");
        
        ui.add_space(8.0);
        
        // Контейнер вибору тем (без рамок)
        ui.vertical(|ui| {
            ui.radio_value(current_theme, AppTheme::Light, "Світла тема");
            ui.add_space(6.0);
            ui.radio_value(current_theme, AppTheme::Dark, "Темна тема");
            ui.add_space(6.0);
            ui.radio_value(current_theme, AppTheme::Amoled, "Чорна AMOLED тема");
        });

        ui.add_space(16.0);
        ui.strong("Колір акценту");
        ui.small("Виберіть колір виділень, активних елементів та навігації:");
        
        ui.add_space(8.0);

        // Блок налаштування кольору акценту (без рамок)
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Список готових стильних кольорів для швидкого вибору
                let presets = [
                    ("Синій", egui::Color32::from_rgb(0, 122, 255)),
                    ("Зелений", egui::Color32::from_rgb(46, 204, 113)),
                    ("Червоний", egui::Color32::from_rgb(231, 76, 60)),
                    ("Помаранчевий", egui::Color32::from_rgb(230, 126, 34)),
                    ("Фіолетовий", egui::Color32::from_rgb(155, 89, 182)),
                ];

                ui.label("Швидкий вибір:");
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
                ui.label("Власний колір з палітри:");
                ui.add_space(8.0);
                
                // Повна палітра вільного вибору для точного налаштування кольору
                ui.color_edit_button_srgba(accent_color);
            });
        });

        ui.add_space(24.0);
        ui.separator();
        ui.add_space(12.0);
        
        ui.strong("Керування даними");
        ui.small("Відкрити локальну папку з файлом налаштувань settings.json:");
        ui.add_space(8.0);
        
        if ui.button("Відкрити папку користувача").clicked() {
            super::storage::open_settings_folder();
        }
    });
}
