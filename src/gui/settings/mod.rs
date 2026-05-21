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
}

/// Головна функція для малювання вкладки налаштувань.
/// 
/// Створює двопанельний інтерфейс: ліворуч відображає меню вибору підвкладок,
/// праворуч — вміст обраної підвкладки (наприклад, "Основні").
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
        
        // Список усіх назв підвкладок для визначення найдовшого слова
        let subtab_names = [general_label];
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

        // Розраховуємо ширину лівої панелі: довжина найдовшого слова + горизонтальні відступи кнопки + запас для запобігання переносу.
        let button_padding_x = ui.spacing().button_padding.x;
        let panel_width = max_word_width + button_padding_x * 2.0 + 12.0;

        // Встановлюємо ширину лівої панелі рівною довжині найдовшого слова в назві підвкладки з урахуванням відступів.
        ui.vertical(|ui| {
            ui.set_width(panel_width);
            ui.add_space(8.0);
            
            // Елементи меню вибору підвкладок
            ui.selectable_value(active_subtab, SettingsSubTab::General, general_label);
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
            }
        });
    });
}
