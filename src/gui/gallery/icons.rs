use eframe::egui;

/// Малює іконку "оновлення" (кругова стрілка ↻) через Painter.
pub fn draw_refresh_icon(painter: &egui::Painter, center: egui::Pos2, r: f32, stroke: egui::Stroke) {
    let segments = 14;
    let arc = std::f32::consts::PI * (300.0_f32.to_radians());
    let start = -std::f32::consts::FRAC_PI_2;
    for i in 0..segments {
        let a1 = start + arc * i as f32 / segments as f32;
        let a2 = start + arc * (i + 1) as f32 / segments as f32;
        painter.line_segment(
            [center + egui::vec2(r * a1.cos(), r * a1.sin()),
             center + egui::vec2(r * a2.cos(), r * a2.sin())],
            stroke,
        );
    }
    let end = start + arc;
    let tip  = center + egui::vec2(r * end.cos(), r * end.sin());
    let tang = end + std::f32::consts::FRAC_PI_2;
    let aw   = stroke.width + 2.0;
    let left  = tip + egui::vec2(aw * (tang - 0.5).cos(), aw * (tang - 0.5).sin());
    let right = tip + egui::vec2(aw * (tang + 0.5).cos(), aw * (tang + 0.5).sin());
    painter.add(egui::Shape::convex_polygon(vec![tip, left, right], stroke.color, egui::Stroke::NONE));
}

/// Малює іконку "налаштування" (три горизонтальні лінії ≡) через Painter.
pub fn draw_menu_icon(painter: &egui::Painter, center: egui::Pos2, half_w: f32, stroke: egui::Stroke) {
    for &dy in &[-half_w * 0.55, 0.0, half_w * 0.55] {
        painter.line_segment(
            [egui::pos2(center.x - half_w, center.y + dy),
             egui::pos2(center.x + half_w, center.y + dy)],
            stroke,
        );
    }
}

/// Малює трикутник ▶ по центру заданого прямокутника.
pub fn draw_play_triangle(painter: &egui::Painter, center: egui::Pos2, size: f32) {
    let pts = vec![
        egui::pos2(center.x - size * 0.5, center.y - size * 0.8),
        egui::pos2(center.x + size,       center.y),
        egui::pos2(center.x - size * 0.5, center.y + size * 0.8),
    ];
    painter.add(egui::Shape::convex_polygon(pts, egui::Color32::from_gray(180), egui::Stroke::NONE));
}
