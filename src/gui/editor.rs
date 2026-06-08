use eframe::egui;
use crate::localization::{Language, translate};

use std::hash::{Hash, Hasher};

fn get_encoder() -> &'static tiktoken::CoreBpe {
    tiktoken::get_encoding("cl100k_base").unwrap()
}

/// Динамічно рахує токени для вказаного тексту за допомогою кодування cl100k_base.
pub fn count_tokens(text: &str) -> usize {
    get_encoder().count(text)
}

/// Допоміжна функція для швидкого обчислення хешу тексту, щоб визначити, чи змінився вміст.
fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Структура для збереження кешованої статистики тексту сценарію.
/// Запобігає важким повторним обчисленням tiktoken та split_text на кожному кадрі.
#[derive(Clone)]
pub struct EditorStats {
    pub char_count: usize,
    pub paragraph_count: usize,
    pub token_count: usize,
    pub fragments_paragraphs: usize,
    pub fragments_sentences: usize,
    pub fragments_char_limit: usize,
    
    // Поля для відстеження змін та інвалідації кешу
    pub last_text_hash: u64,
    pub last_char_limit: usize,
}

impl Default for EditorStats {
    fn default() -> Self {
        Self {
            char_count: 0,
            paragraph_count: 0,
            token_count: 0,
            fragments_paragraphs: 0,
            fragments_sentences: 0,
            fragments_char_limit: 0,
            last_text_hash: 0,
            last_char_limit: 0,
        }
    }
}

/// Відображає редактор сценарію на всю доступну висоту та ширину.
///
/// Використовує `ScrollArea` з вимкненим автозменшенням (`auto_shrink`)
/// та `TextEdit` без стандартних рамок (`frame(false)`), щоб забезпечити
/// ефект "чистого аркуша" на всю висоту робочої області.
pub fn draw_editor(
    ui: &mut egui::Ui,
    text: &mut String,
    language: Language,
    text_split_char_limit: usize,
    stats: &mut EditorStats,
) {
    // 1. Обчислення статистики (виконується ліниво тільки при зміні вмісту тексту або ліміту символів)
    let current_hash = calculate_hash(text);
    if stats.last_text_hash != current_hash || stats.last_char_limit != text_split_char_limit {
        stats.last_text_hash = current_hash;
        stats.last_char_limit = text_split_char_limit;
        
        stats.char_count = text.chars().count();
        stats.paragraph_count = text.lines().filter(|line| !line.trim().is_empty()).count();
        stats.token_count = count_tokens(text);
        stats.fragments_paragraphs = crate::core::pipeline::timeline::text_splitter::split_text(text, "paragraphs", 0).len();
        stats.fragments_sentences  = crate::core::pipeline::timeline::text_splitter::split_text(text, "sentences",  0).len();
        stats.fragments_char_limit = crate::core::pipeline::timeline::text_splitter::split_text(text, "char_limit", text_split_char_limit).len();
    }

    let char_count = stats.char_count;
    let paragraph_count = stats.paragraph_count;
    let token_count = stats.token_count;
    let fragments_paragraphs = stats.fragments_paragraphs;
    let fragments_sentences = stats.fragments_sentences;
    let fragments_char_limit = stats.fragments_char_limit;

    // 2. Рендеринг панелі статистики (фіксована вгорі)
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);

        let text_color = ui.visuals().widgets.noninteractive.text_color();
        let accent_color = ui.visuals().selection.bg_fill;
        let bullet_color = text_color.linear_multiply(0.3);

        // Символи
        ui.label(egui::RichText::new(translate(language, "stats_chars")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", char_count)).size(16.0).strong().color(accent_color));

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

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Фрагменти по абзацах
        ui.label(egui::RichText::new(translate(language, "stats_fragments_paragraphs")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", fragments_paragraphs)).size(16.0).strong().color(accent_color));

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Фрагменти по реченнях
        ui.label(egui::RichText::new(translate(language, "stats_fragments_sentences")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", fragments_sentences)).size(16.0).strong().color(accent_color));

        // Роздільник
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        // Фрагменти по ліміту символів
        ui.label(egui::RichText::new(translate(language, "stats_fragments_chars")).size(16.0).color(text_color));
        ui.label(egui::RichText::new(format!(" {}", fragments_char_limit)).size(16.0).strong().color(accent_color));
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
