use eframe::egui;
use crate::localization::{Language, translate};

fn get_encoder() -> &'static tiktoken::CoreBpe {
    tiktoken::get_encoding("cl100k_base").unwrap()
}

/// Динамічно рахує токени для вказаного тексту за допомогою кодування cl100k_base.
pub fn count_tokens(text: &str) -> usize {
    get_encoder().count(text)
}

/// Відображає редактор сценарію на всю доступну висоту та ширину.
/// 
/// Використовує `ScrollArea` з вимкненим автозменшенням (`auto_shrink`)
/// та `TextEdit` без стандартних рамок (`frame(false)`), щоб забезпечити
/// ефект "чистого аркуша" на всю висоту робочої області.
pub fn draw_editor(ui: &mut egui::Ui, text: &mut String, language: Language) {
    // 1. Обчислення статистики
    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();
    let paragraph_count = text.lines().filter(|line| !line.trim().is_empty()).count();
    let token_count = count_tokens(text);

    // 2. Рендеринг панелі статистики (фіксована вгорі)
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0); // Легкий відступ зліва для вирівнювання

        let text_color = ui.visuals().widgets.noninteractive.text_color();
        let accent_color = ui.visuals().selection.bg_fill;
        let bullet_color = text_color.linear_multiply(0.3);

        // Символи
        ui.label(egui::RichText::new(translate(language, "stats_chars")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", char_count)).size(16.0).strong().color(accent_color));

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Слова
        ui.label(egui::RichText::new(translate(language, "stats_words")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", word_count)).size(16.0).strong().color(accent_color));

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Абзаци
        ui.label(egui::RichText::new(translate(language, "stats_paragraphs")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", paragraph_count)).size(16.0).strong().color(accent_color));

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Токени
        ui.label(egui::RichText::new(translate(language, "stats_tokens")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", token_count)).size(16.0).strong().color(accent_color));
    });
    ui.add_space(4.0);
    ui.separator();

    // 3. Область прокрутки для безрамкового текстового поля
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2]) // Запобігає стисканню скрол-області
        .show(ui, |ui| {
            // Задаємо легкий відступ від країв для покращення читабельності
            ui.add_space(8.0);
            
            let text_edit = egui::TextEdit::multiline(text)
                .hint_text(translate(language, "editor_hint"))
                .desired_width(f32::INFINITY)
                .desired_rows(40) // Велика дефолтна кількість рядків
                .frame(false);    // Безрамковий дизайн для чистішого вигляду

            // Розтягуємо текстове поле на всю доступну область по горизонталі та вертикалі
            ui.add_sized(ui.available_size(), text_edit);
        });
}
