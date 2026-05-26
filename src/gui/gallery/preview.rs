use eframe::egui;
use super::icons::{draw_menu_icon, draw_refresh_icon};

/// Завантажує зображення з диску та повертає TextureHandle для egui.
pub fn load_image_texture(ctx: &egui::Context, path: &std::path::Path) -> Option<egui::TextureHandle> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("img");
    Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
}

/// Повноекранний перегляд зображення поверх інтерфейсу.
/// Повертає (keep_open, regen_kind): keep_open=false → закрити;
/// regen_kind: Some(false)=ті ж налаштування, Some(true)=кастомні.
pub fn draw_image_preview(
    ctx: &egui::Context,
    texture: &egui::TextureHandle,
    regen_loading: bool,
) -> (bool, Option<bool>) {
    let mut keep_open = true;
    let mut regen_kind: Option<bool> = None;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return (false, None);
    }

    let screen_rect = ctx.screen_rect();
    let padding = 40.0;

    let max_w = screen_rect.width() - padding * 2.0;
    let max_h = screen_rect.height() - padding * 2.0;
    let img_size = texture.size_vec2();
    let scale = (max_w / img_size.x).min(max_h / img_size.y);
    let display_size = img_size * scale;
    let img_rect = egui::Rect::from_center_size(screen_rect.center(), display_size);

    egui::Area::new(egui::Id::new("gallery_preview_area"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let bg = ui.allocate_rect(screen_rect, egui::Sense::click());
            ui.painter().rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(215));

            ui.put(img_rect, egui::Image::from_texture(texture).fit_to_exact_size(display_size));

            if regen_loading {
                ui.painter().rect_filled(img_rect, 0.0, egui::Color32::from_black_alpha(120));
                ui.put(img_rect, egui::Spinner::new().size(32.0));
            }

            let btn_size = egui::vec2(36.0, 36.0);
            let top_y = screen_rect.top() + 22.0;

            // Кнопка X (закрити)
            let close_center = egui::pos2(screen_rect.right() - 22.0, top_y);
            let close_rect   = egui::Rect::from_center_size(close_center, btn_size);
            let close_resp   = ui.interact(close_rect, egui::Id::new("gp_close"), egui::Sense::click());
            let close_color  = if close_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_gray(160) };
            let xs = egui::Stroke::new(2.0, close_color);
            let r = 8.0;
            ui.painter().line_segment([close_center + egui::vec2(-r,-r), close_center + egui::vec2(r,r)], xs);
            ui.painter().line_segment([close_center + egui::vec2(r,-r), close_center + egui::vec2(-r,r)], xs);

            // Кнопка "Кастомна перегенерація" (≡)
            let custom_center = egui::pos2(screen_rect.right() - 66.0, top_y);
            let custom_rect   = egui::Rect::from_center_size(custom_center, btn_size);
            let custom_resp   = ui.interact(custom_rect, egui::Id::new("gp_custom"), egui::Sense::click());
            let custom_color  = if custom_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_gray(160) };
            draw_menu_icon(ui.painter(), custom_center, 8.0, egui::Stroke::new(2.0, custom_color));

            // Кнопка "Та сама перегенерація" (↻)
            let same_center = egui::pos2(screen_rect.right() - 110.0, top_y);
            let same_rect   = egui::Rect::from_center_size(same_center, btn_size);
            let same_resp   = ui.interact(same_rect, egui::Id::new("gp_same"), egui::Sense::click());
            let same_color  = if same_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_gray(160) };
            draw_refresh_icon(ui.painter(), same_center, 9.0, egui::Stroke::new(2.0, same_color));

            if close_resp.clicked() {
                keep_open = false;
            } else if same_resp.clicked() {
                regen_kind = Some(false);
            } else if custom_resp.clicked() {
                regen_kind = Some(true);
            } else if bg.clicked() {
                if let Some(pos) = bg.interact_pointer_pos() {
                    if !img_rect.contains(pos) {
                        keep_open = false;
                    }
                }
            }
        });

    (keep_open, regen_kind)
}
