use eframe::egui;

/// Доступні теми оформлення додатку.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    /// Стандартна світла тема
    Light,
    /// Стандартна темна тема
    Dark,
    /// Повністю чорна AMOLED тема
    Amoled,
}

/// Застосовує обрану тему та колір акценту до контексту egui.
/// 
/// Виконує перемикання колірної палітри інтерфейсу між світлою,
/// темною та глибокою чорною AMOLED (без використання сірих тонів),
/// а також встановлює кастомний колір акценту для виділень та посилань.
pub fn apply_theme(ctx: &egui::Context, theme: AppTheme, accent_color: egui::Color32) {
    let mut visuals = match theme {
        AppTheme::Light => egui::Visuals::light(),
        AppTheme::Dark => egui::Visuals::dark(),
        AppTheme::Amoled => {
            let mut v = egui::Visuals::dark();
            
            // Налаштовуємо повністю чорний колір фону для AMOLED екранів (#000000)
            v.panel_fill = egui::Color32::from_rgb(0, 0, 0);
            v.window_fill = egui::Color32::from_rgb(0, 0, 0);
            v.extreme_bg_color = egui::Color32::from_rgb(6, 6, 6);
            v
        }
    };
    
    // Застосовуємо обраний колір акценту для елементів виділення та гіперпосилань
    visuals.selection.bg_fill = accent_color;
    visuals.hyperlink_color = accent_color;
    
    // Застосовуємо оновлені параметри візуалів
    ctx.set_visuals(visuals);

    // Забороняємо виділення тексту в інтерфейсі (прибирає курсор I-beam)
    ctx.style_mut(|s| s.interaction.selectable_labels = false);
}
