use eframe::egui;
use crate::localization::{Language, translate};

/// Малює секцію "АПІ" на панелі пайплайну з підтримкою OpenRouter.
pub fn draw_api_section(
    ui: &mut egui::Ui,
    language: Language,
    openrouter_key: &mut String,
    openrouter_status: &mut Option<String>,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        
        // Відображаємо мітку сервісу
        ui.label(egui::RichText::new("OpenRouter").strong());
        ui.add_space(4.0);
        
        // Зберігаємо доступну ширину ДО створення горизонтального рядка.
        // Це критично важливо, оскільки всередині ui.horizontal() доступна ширина розраховується інакше,
        // що призводить до самовільного розширення бічної панелі пайплайну.
        let available_width = ui.available_width();
        
        ui.horizontal(|ui| {
            // Текстове поле для введення ключа. Маскуємо його як пароль для безпеки.
            // Задаємо ширину на основі збереженої available_width, залишаючи місце під кнопку та відступи.
            let response = ui.add(
                egui::TextEdit::singleline(openrouter_key)
                    .password(true)
                    .hint_text("sk-or-...")
                    .desired_width((available_width - 90.0).max(100.0))
            );
            
            // Якщо користувач почав редагувати поле, скидаємо статус перевірки для динамічності інтерфейсу
            if response.changed() {
                *openrouter_status = None;
            }

            // Невелика кнопка перевірки збоку від поля
            let check_btn = ui.add_sized(
                [70.0, 20.0],
                egui::Button::new(translate(language, "api_check_btn"))
            );
            
            if check_btn.clicked() {
                let trimmed = openrouter_key.trim();
                if trimmed.is_empty() {
                    // Статус, якщо ключ порожній
                    *openrouter_status = Some(translate(language, "api_status_empty").to_string());
                } else if trimmed.starts_with("sk-or-") && trimmed.len() >= 15 {
                    // Статус успішної локальної перевірки
                    *openrouter_status = Some(translate(language, "api_status_success").to_string());
                } else {
                    // Статус невірного формату ключа
                    *openrouter_status = Some(translate(language, "api_status_invalid").to_string());
                }
            }
        });

        // Відображаємо повідомлення про статус перевірки з увімкненим переносом тексту,
        // щоб довгі повідомлення не розсували панель.
        if let Some(status) = openrouter_status {
            ui.add_space(4.0);
            
            // Визначаємо колір тексту залежно від успішності перевірки (успішний статус починається з ✔)
            let is_success = status.starts_with('✔');
            let text_color = if is_success {
                egui::Color32::from_rgb(46, 204, 113) // Приємний зелений
            } else {
                egui::Color32::from_rgb(231, 76, 60) // Приємний червоний
            };
            
            // Використовуємо Label з .wrap() для запобігання горизонтальному розширенню панелі
            ui.add(
                egui::Label::new(
                    egui::RichText::new(status.as_str())
                        .color(text_color)
                        .size(12.0)
                ).wrap()
            );
        }
        
        ui.add_space(6.0);
    });
}
