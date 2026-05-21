use eframe::egui;

/// Відображає редактор сценарію на всю доступну висоту та ширину.
/// 
/// Використовує `ScrollArea` з вимкненим автозменшенням (`auto_shrink`)
/// та `TextEdit` без стандартних рамок (`frame(false)`), щоб забезпечити
/// ефект "чистого аркуша" на всю висоту робочої області.
pub fn draw_editor(ui: &mut egui::Ui, text: &mut String) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2]) // Запобігає стисканню скрол-області
        .show(ui, |ui| {
            // Задаємо легкий відступ від країв для покращення читабельності
            ui.add_space(8.0);
            
            let text_edit = egui::TextEdit::multiline(text)
                .hint_text("Введіть або вставте сюди текст вашого майбутнього відео сценарію...")
                .desired_width(f32::INFINITY)
                .desired_rows(40) // Велика дефолтна кількість рядків
                .frame(false);    // Безрамковий дизайн для чистішого вигляду

            // Розтягуємо текстове поле на всю доступну область по горизонталі та вертикалі
            ui.add_sized(ui.available_size(), text_edit);
        });
}
