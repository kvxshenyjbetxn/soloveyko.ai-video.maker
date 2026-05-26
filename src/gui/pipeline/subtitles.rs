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
pub fn draw_subtitles_section(
    ui: &mut egui::Ui,
    language: Language,
    subtitles_service: &mut String,
    whisper_language: &mut String,
    whisper_model: &mut String,
    whisper_max_line_width: &mut usize,
    whisper_model_download: &Arc<Mutex<BinaryDownload>>,
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
