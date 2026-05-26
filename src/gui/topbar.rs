use eframe::egui;

/// Повертає колір для відображення навантаження потоків за відсотком зайнятості.
/// 0 → default, 1–40% → зелений, 41–75% → жовтий, 76–100% → червоний.
pub fn thread_load_color(active: usize, max: usize, default: egui::Color32) -> egui::Color32 {
    if active == 0 || max == 0 {
        return default;
    }
    let pct = active as f32 / max as f32;
    if pct <= 0.40 {
        egui::Color32::from_rgb(80, 200, 100)
    } else if pct <= 0.75 {
        egui::Color32::from_rgb(255, 200, 0)
    } else {
        egui::Color32::from_rgb(220, 70, 70)
    }
}

/// Малює клікабельний чіп з довільним кольором тексту. Основа для balance та thread чіпів.
pub fn draw_chip(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> egui::Response {
    let font_id = egui::FontId::new(13.0, egui::FontFamily::Proportional);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_id, text_color));
    let padding = egui::vec2(8.0, 4.0);
    let desired_size = galley.rect.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() {
            ui.visuals().widgets.hovered.weak_bg_fill
        } else {
            ui.visuals().faint_bg_color
        };
        ui.painter().rect_filled(rect, egui::Rounding::same(4.0), fill);
        ui.painter().galley(rect.min + padding, galley, text_color);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Малює компактний чіп з балансом. При наведенні підсвічується і змінює курсор.
pub fn draw_balance_chip(ui: &mut egui::Ui, prefix: &str, value: &str) -> egui::Response {
    draw_chip(ui, &format!("{}: {}", prefix, value), ui.visuals().text_color())
}
