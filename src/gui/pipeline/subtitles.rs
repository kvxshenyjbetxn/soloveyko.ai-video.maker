use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::localization::{Language, translate};
use crate::gui::welcome::BinaryDownload;

/// Доступні мови для Whisper (код → відображувана назва).
const WHISPER_LANGUAGES: &[(&str, &str)] = &[
    ("auto", ""),
    ("uk", "Ukrainian"),
    ("en", "English"),
    ("ru", "Russian"),
    ("de", "German"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("it", "Italian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("nl", "Dutch"),
    ("ja", "Japanese"),
    ("zh", "Chinese"),
    ("ar", "Arabic"),
    ("tr", "Turkish"),
];

/// Моделі для whisper.cpp (ggml-формат).
const WHISPER_MODELS: &[&str] = &[
    "tiny", "base", "small", "medium", "large-v3", "large-v3-turbo",
];

/// Моделі для WhisperX (faster-whisper / HuggingFace).
const WHISPERX_MODELS: &[&str] = &[
    "tiny", "base", "small", "medium",
    "large-v1", "large-v2", "large-v3", "large",
    "distil-large-v2", "distil-large-v3",
    "distil-medium.en", "distil-small.en",
];

/// Малює секцію "Субтитри" на панелі пайплайну.
#[allow(clippy::too_many_arguments)]
pub fn draw_subtitles_section(
    ui: &mut egui::Ui,
    language: Language,
    subtitles_service: &mut String,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
    whisper_model_download: &Arc<Mutex<BinaryDownload>>,
    subtitle_font_size: &mut u32,
    subtitle_color: &mut [u8; 3],
    subtitle_margin_v: &mut u32,
    subtitle_karaoke: &mut bool,
    subtitle_karaoke_fill: &mut bool,
    subtitle_karaoke_highlight_color: &mut [u8; 3],
    subtitle_karaoke_outline_color: &mut [u8; 3],
    subtitle_karaoke_bold: &mut bool,
    subtitle_font: &mut String,
    available_subtitle_fonts: &[String],
    ctx: egui::Context,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        // Сервіс субтитрів
        ui.label(egui::RichText::new(translate(language, "subtitles_service_label")).strong());
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("subtitles_service_combo")
            .selected_text(subtitles_service.as_str())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(subtitles_service, "Whisper".to_string(), "Whisper");
                ui.selectable_value(subtitles_service, "WhisperX".to_string(), "WhisperX");
                ui.selectable_value(subtitles_service, "AssemblyAI".to_string(), "AssemblyAI");
            });

        // Налаштування Whisper
        if subtitles_service == "Whisper" {
            draw_whisper_settings(
                ui,
                language,
                whisper_language,
                whisper_model,
                whisper_max_line_width,
                whisper_model_download,
                &ctx,
            );
        }

        // Налаштування WhisperX
        if subtitles_service == "WhisperX" {
            draw_whisperx_settings(
                ui,
                language,
                whisper_language,
                whisper_model,
                whisper_max_line_width,
            );
        }

        // Налаштування AssemblyAI
        if subtitles_service == "AssemblyAI" {
            draw_assemblyai_settings(
                ui,
                language,
                whisper_language,
                whisper_max_line_width,
            );
        }

        // Стиль субтитрів (загальний для всіх сервісів)
        draw_subtitle_style(
            ui,
            language,
            subtitles_service,
            subtitle_font_size,
            subtitle_color,
            subtitle_margin_v,
            subtitle_karaoke,
            subtitle_karaoke_fill,
            subtitle_karaoke_highlight_color,
            subtitle_karaoke_outline_color,
            subtitle_karaoke_bold,
            subtitle_font,
            available_subtitle_fonts,
        );
    });
}

/// Загальний блок вибору мови та моделі (спільний для Whisper і WhisperX).
fn draw_lang_and_model(
    ui: &mut egui::Ui,
    language: Language,
    lang_id: &str,
    model_id: &str,
    models: &[&str],
    whisper_language: &mut String,
    whisper_model: &mut String,
) {
    // Мова розпізнавання
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_lang_label")).strong());
    ui.add_space(4.0);

    let lang_display = WHISPER_LANGUAGES.iter()
        .find(|(code, _)| *code == whisper_language.as_str())
        .map(|(code, name)| {
            if *code == "auto" {
                translate(language, "subtitles_whisper_lang_auto").to_string()
            } else {
                format!("{} ({})", name, code)
            }
        })
        .unwrap_or_else(|| whisper_language.clone());

    egui::ComboBox::from_id_salt(lang_id)
        .selected_text(lang_display)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for (code, name) in WHISPER_LANGUAGES {
                let label = if *code == "auto" {
                    translate(language, "subtitles_whisper_lang_auto").to_string()
                } else {
                    format!("{} ({})", name, code)
                };
                ui.selectable_value(whisper_language, code.to_string(), label);
            }
        });

    ui.add_space(8.0);

    // Модель
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_model_label")).strong());
    ui.add_space(4.0);
    egui::ComboBox::from_id_salt(model_id)
        .selected_text(whisper_model.as_str())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for model in models {
                ui.selectable_value(whisper_model, model.to_string(), *model);
            }
        });

    ui.add_space(8.0);
}

/// Налаштування Whisper (whisper.cpp).
fn draw_whisper_settings(
    ui: &mut egui::Ui,
    language: Language,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
    whisper_model_download: &Arc<Mutex<BinaryDownload>>,
    ctx: &egui::Context,
) {
    ui.add_space(8.0);

    draw_lang_and_model(
        ui, language,
        "whisper_lang_combo", "whisper_model_combo",
        WHISPER_MODELS,
        whisper_language, whisper_model,
    );

    // Максимальна кількість символів на сегмент (--max-len)
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_max_len_label")).strong());
    ui.add_space(4.0);
    let mut max_len = *whisper_max_line_width;
    let label_text = if max_len == 0 { "∞".to_string() } else { max_len.to_string() };
    ui.horizontal(|ui| {
        if ui.add(egui::Slider::new(&mut max_len, 0..=200).show_value(false)).changed() {
            *whisper_max_line_width = max_len;
        }
        ui.label(egui::RichText::new(label_text).monospace());
    });

    ui.add_space(6.0);

    // Статус моделі та кнопка завантаження
    let download_state = whisper_model_download.lock().unwrap().clone();
    let model_exists = crate::bundle::whisper_model_exists(whisper_model);

    match download_state {
        BinaryDownload::Downloading(ref progress) => {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("⬇ {}...", progress))
                        .weak()
                        .size(11.0),
                )
                .wrap(),
            );
        }
        BinaryDownload::Failed(ref err) => {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("{} {}", translate(language, "subtitles_model_failed"), err))
                        .color(egui::Color32::from_rgb(231, 76, 60))
                        .size(11.0),
                )
                .wrap(),
            );
            ui.add_space(4.0);
            if ui.small_button(translate(language, "subtitles_model_retry")).clicked() {
                start_model_download(whisper_model, whisper_model_download, ctx);
            }
        }
        _ => {
            if model_exists {
                ui.label(
                    egui::RichText::new(translate(language, "subtitles_model_downloaded"))
                        .color(egui::Color32::from_rgb(46, 204, 113))
                        .size(11.0),
                );
            } else {
                let size_mb = crate::bundle::whisper_model_size_mb(whisper_model);
                let btn_text = if size_mb > 0.0 {
                    format!("{} (~{:.0} MB)", translate(language, "subtitles_model_download_btn"), size_mb)
                } else {
                    translate(language, "subtitles_model_download_btn").to_string()
                };

                if ui.add_sized(
                    [ui.available_width(), 22.0],
                    egui::Button::new(egui::RichText::new(btn_text).size(12.0)),
                ).clicked() {
                    start_model_download(whisper_model, whisper_model_download, ctx);
                }
            }
        }
    }
}

/// Налаштування WhisperX.
fn draw_whisperx_settings(
    ui: &mut egui::Ui,
    language: Language,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
) {
    ui.add_space(8.0);

    draw_lang_and_model(
        ui, language,
        "whisperx_lang_combo", "whisperx_model_combo",
        WHISPERX_MODELS,
        whisper_language, whisper_model,
    );

    // Максимальна кількість символів на сегмент (передається через --max_line_width)
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_max_len_label")).strong());
    ui.add_space(4.0);
    let mut max_len = *whisper_max_line_width;
    let label_text = if max_len == 0 { "∞".to_string() } else { max_len.to_string() };
    ui.horizontal(|ui| {
        if ui.add(egui::Slider::new(&mut max_len, 0..=200).show_value(false)).changed() {
            *whisper_max_line_width = max_len;
        }
        ui.label(egui::RichText::new(label_text).monospace());
    });
}

/// Налаштування AssemblyAI (cloud API — без локальних моделей).
fn draw_assemblyai_settings(
    ui: &mut egui::Ui,
    language: Language,
    whisper_language: &mut String,
    whisper_max_line_width: &mut usize,
) {
    ui.add_space(8.0);

    // Мова розпізнавання (спільна зі Whisper)
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_lang_label")).strong());
    ui.add_space(4.0);

    let lang_display = WHISPER_LANGUAGES.iter()
        .find(|(code, _)| *code == whisper_language.as_str())
        .map(|(code, name)| {
            if *code == "auto" {
                translate(language, "subtitles_whisper_lang_auto").to_string()
            } else {
                format!("{} ({})", name, code)
            }
        })
        .unwrap_or_else(|| whisper_language.clone());

    egui::ComboBox::from_id_salt("assemblyai_lang_combo")
        .selected_text(lang_display)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for (code, name) in WHISPER_LANGUAGES {
                let label = if *code == "auto" {
                    translate(language, "subtitles_whisper_lang_auto").to_string()
                } else {
                    format!("{} ({})", name, code)
                };
                ui.selectable_value(whisper_language, code.to_string(), label);
            }
        });

    ui.add_space(8.0);

    // Максимальна кількість символів на сегмент
    ui.label(egui::RichText::new(translate(language, "subtitles_whisper_max_len_label")).strong());
    ui.add_space(4.0);
    let mut max_len = *whisper_max_line_width;
    let label_text = if max_len == 0 { "∞".to_string() } else { max_len.to_string() };
    ui.horizontal(|ui| {
        if ui.add(egui::Slider::new(&mut max_len, 0..=200).show_value(false)).changed() {
            *whisper_max_line_width = max_len;
        }
        ui.label(egui::RichText::new(label_text).monospace());
    });
}

/// Малює вибір шрифту з попереднім переглядом (popup).
fn draw_font_picker(
    ui: &mut egui::Ui,
    language: Language,
    subtitle_font: &mut String,
    available_fonts: &[String],
) {
    if available_fonts.is_empty() {
        return;
    }

    ui.label(egui::RichText::new(translate(language, "subtitles_font_label")));
    ui.add_space(2.0);

    let popup_id = ui.make_persistent_id("subtitle_font_popup");

    let btn_font = if available_fonts.iter().any(|f| f == subtitle_font) {
        egui::FontId::new(14.0, egui::FontFamily::Name(subtitle_font.clone().into()))
    } else {
        egui::FontId::proportional(14.0)
    };

    let btn = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::Button::new(egui::RichText::new(subtitle_font.as_str()).font(btn_font)),
    );

    if btn.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }

    egui::popup::popup_below_widget(
        ui, popup_id, &btn,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(180.0);
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for font_name in available_fonts {
                    let font_id = egui::FontId::new(
                        16.0,
                        egui::FontFamily::Name(font_name.clone().into()),
                    );
                    let is_selected = font_name == subtitle_font;
                    let resp = ui.add(egui::SelectableLabel::new(
                        is_selected,
                        egui::RichText::new(font_name.as_str()).font(font_id),
                    ));
                    if resp.clicked() {
                        *subtitle_font = font_name.clone();
                    }
                }
            });
        },
    );
}

/// Налаштування стилю субтитрів (колір, розмір шрифту, відступ, karaoke).
#[allow(clippy::too_many_arguments)]
fn draw_subtitle_style(
    ui: &mut egui::Ui,
    language: Language,
    subtitles_service: &str,
    subtitle_font_size: &mut u32,
    subtitle_color: &mut [u8; 3],
    subtitle_margin_v: &mut u32,
    subtitle_karaoke: &mut bool,
    subtitle_karaoke_fill: &mut bool,
    subtitle_karaoke_highlight_color: &mut [u8; 3],
    subtitle_karaoke_outline_color: &mut [u8; 3],
    subtitle_karaoke_bold: &mut bool,
    subtitle_font: &mut String,
    available_subtitle_fonts: &[String],
) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(translate(language, "subtitles_style_label")).strong());
    ui.add_space(4.0);

    // Розмір шрифту
    ui.label(egui::RichText::new(translate(language, "subtitles_font_size_label")));
    ui.add_space(2.0);
    let mut font_size = *subtitle_font_size;
    ui.horizontal(|ui| {
        if ui.add(egui::Slider::new(&mut font_size, 10..=72).show_value(false)).changed() {
            *subtitle_font_size = font_size;
        }
        ui.label(egui::RichText::new(format!("{}pt", font_size)).monospace());
    });

    ui.add_space(6.0);

    // Відступ від нижнього краю
    ui.label(egui::RichText::new(translate(language, "subtitles_margin_v_label")));
    ui.add_space(2.0);
    let mut margin = *subtitle_margin_v;
    ui.horizontal(|ui| {
        if ui.add(egui::Slider::new(&mut margin, 0..=200).show_value(false)).changed() {
            *subtitle_margin_v = margin;
        }
        ui.label(egui::RichText::new(format!("{}px", margin)).monospace());
    });

    ui.add_space(6.0);

    // Шрифт
    draw_font_picker(ui, language, subtitle_font, available_subtitle_fonts);

    ui.add_space(6.0);

    // Колір тексту
    ui.label(egui::RichText::new(translate(language, "subtitles_color_label")));
    ui.add_space(2.0);
    ui.color_edit_button_srgb(subtitle_color);

    ui.add_space(6.0);

    // Колір обводки
    ui.label(egui::RichText::new(translate(language, "subtitles_outline_color_label")));
    ui.add_space(2.0);
    ui.color_edit_button_srgb(subtitle_karaoke_outline_color);

    // Karaoke (тільки для WhisperX та AssemblyAI, бо потрібні word-level timestamps)
    if subtitles_service == "WhisperX" || subtitles_service == "AssemblyAI" {
        ui.add_space(6.0);
        ui.checkbox(subtitle_karaoke, translate(language, "subtitles_karaoke_label"));

        if *subtitle_karaoke {
            ui.add_space(4.0);
            egui::Frame::none()
                .inner_margin(egui::Margin { left: 12.0, ..Default::default() })
                .show(ui, |ui| {
                    // Стиль анімації
                    ui.label(egui::RichText::new(translate(language, "subtitles_karaoke_style_label")));
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.radio_value(subtitle_karaoke_fill, true,
                            translate(language, "subtitles_karaoke_fill"));
                        ui.radio_value(subtitle_karaoke_fill, false,
                            translate(language, "subtitles_karaoke_switch"));
                    });

                    ui.add_space(6.0);

                    // Колір виділеного слова
                    ui.label(egui::RichText::new(translate(language, "subtitles_karaoke_highlight_color_label")));
                    ui.add_space(2.0);
                    ui.color_edit_button_srgb(subtitle_karaoke_highlight_color);

                    ui.add_space(6.0);

                    // Жирний текст
                    ui.checkbox(subtitle_karaoke_bold, translate(language, "subtitles_karaoke_bold_label"));
                });
        }
    }
}

/// Запускає завантаження ggml-моделі whisper.cpp у фоновому потоці.
fn start_model_download(
    model: &str,
    whisper_model_download: &Arc<Mutex<BinaryDownload>>,
    ctx: &egui::Context,
) {
    let dl_outer = Arc::clone(whisper_model_download);
    let ctx_outer = ctx.clone();
    let model_name = model.to_string();

    std::thread::spawn(move || {
        let dl_progress = Arc::clone(&dl_outer);
        let ctx_progress = ctx_outer.clone();

        *dl_outer.lock().unwrap() = BinaryDownload::Downloading("підготовка...".to_string());
        ctx_outer.request_repaint();

        let result = crate::bundle::download_whisper_model(&model_name, move |label| {
            *dl_progress.lock().unwrap() = BinaryDownload::Downloading(label);
            ctx_progress.request_repaint();
        });

        match result {
            Ok(()) => *dl_outer.lock().unwrap() = BinaryDownload::Done,
            Err(e) => *dl_outer.lock().unwrap() = BinaryDownload::Failed(e),
        }
        ctx_outer.request_repaint();
    });
}
