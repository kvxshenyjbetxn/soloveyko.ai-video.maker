use eframe::egui;
use crate::localization::{Language, translate};

/// Відображає назву провайдера для ComboBox.
/// Формат значення: "provider" або "flow_MODEL" для flow-провайдера.
fn provider_display(provider: &str) -> &'static str {
    match provider {
        "flow_IMAGEN_3_5" => "Imagen 4 - Flow",
        "flow_GEM_PIX_2"  => "Nano Banana Pro - Flow",
        "flow_NARWHAL"    => "Nano Banana 2 - Flow",
        "grok"            => "Grok",
        "flower"          => "Nano Banana 2 - Flower",
        "openai"          => "ChatGPT Images 2.0",
        _                 => "Imagen 4 - Flow",
    }
}

/// Малює секцію "Відеоряд" на панелі пайплайну.
pub fn draw_video_section(
    ui: &mut egui::Ui,
    language: Language,
    video_service: &mut String,
    googler_image_provider: &mut String,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        ui.label(egui::RichText::new(translate(language, "video_service_label")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("video_service_combo")
            .selected_text(video_service.as_str())
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(video_service, "Googler".to_string(), "Googler");
            });

        ui.add_space(8.0);

        if video_service.as_str() == "Googler" {
            ui.label(egui::RichText::new(translate(language, "googler_image_provider_label")).strong());
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("googler_image_provider_combo")
                .selected_text(provider_display(googler_image_provider))
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        googler_image_provider,
                        "flow_IMAGEN_3_5".to_string(),
                        "Imagen 4 - Flow",
                    );
                    ui.selectable_value(
                        googler_image_provider,
                        "flow_GEM_PIX_2".to_string(),
                        "Nano Banana Pro - Flow",
                    );
                    ui.selectable_value(
                        googler_image_provider,
                        "flow_NARWHAL".to_string(),
                        "Nano Banana 2 - Flow",
                    );
                    ui.selectable_value(
                        googler_image_provider,
                        "grok".to_string(),
                        "Grok",
                    );
                    ui.selectable_value(
                        googler_image_provider,
                        "flower".to_string(),
                        "Nano Banana 2 - Flower",
                    );
                    ui.selectable_value(
                        googler_image_provider,
                        "openai".to_string(),
                        "ChatGPT Images 2.0",
                    );
                });
        }

        ui.add_space(6.0);
    });
}
