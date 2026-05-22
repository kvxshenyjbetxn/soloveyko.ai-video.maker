use eframe::egui;
use crate::localization::{Language, translate};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct VoiceBotTemplate {
    pub uuid: String,
    pub name: String,
}

/// Малює секцію "Озвучка" на панелі пайплайну.
#[allow(clippy::too_many_arguments)]
pub fn draw_voiceover_section(
    ui: &mut egui::Ui,
    language: Language,
    voicebot_key: &str,
    voiceover_provider: &mut String,
    voiceover_template_uuid: &mut String,
    voicebot_templates: &Arc<Mutex<Option<Result<Vec<VoiceBotTemplate>, String>>>>,
    voicebot_loading: &Arc<Mutex<bool>>,
    edge_tts_voice: &mut String,
    edge_tts_rate: &mut String,
    edge_tts_pitch: &mut String,
    edge_tts_volume: &mut String,
    edge_tts_voices: &Arc<Mutex<Option<Result<Vec<crate::api::edgetts::EdgeTTSVoice>, String>>>>,
    edge_tts_loading_voices: &Arc<Mutex<bool>>,
    edge_tts_show_all_languages: &mut bool,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);

        ui.label(egui::RichText::new(translate(language, "voiceover_provider_label")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_salt("voiceover_provider_combo")
            .selected_text(voiceover_provider.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(voiceover_provider, "Voice Bot".to_string(), "Voice Bot");
                ui.selectable_value(voiceover_provider, "Edge TTS".to_string(), "Edge TTS");
            });

        ui.add_space(8.0);

        if voiceover_provider.as_str() == "Voice Bot" {
            let is_loading = *voicebot_loading.lock().unwrap();
            let templates_snapshot = voicebot_templates.lock().unwrap().clone();

            if is_loading {
                ui.label(
                    egui::RichText::new(translate(language, "voiceover_templates_loading"))
                        .weak()
                        .size(12.0)
                );
            } else {
                match templates_snapshot {
                    None => {
                        if voicebot_key.is_empty() {
                            ui.add(egui::Label::new(
                                egui::RichText::new(translate(language, "voicebot_key_required"))
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .size(12.0)
                            ).wrap());
                        } else {
                            // Встановлюємо прапорець до spawn, щоб наступний фрейм не тригернув ще раз
                            *voicebot_loading.lock().unwrap() = true;

                            let templates_arc = Arc::clone(voicebot_templates);
                            let loading_arc = Arc::clone(voicebot_loading);
                            let key = voicebot_key.to_string();
                            let ctx = ui.ctx().clone();

                            std::thread::spawn(move || {
                                let agent = ureq::AgentBuilder::new()
                                    .timeout_connect(std::time::Duration::from_secs(10))
                                    .timeout(std::time::Duration::from_secs(15))
                                    .build();

                                let parsed = match agent
                                    .get("https://voiceapi.csv666.ru/templates")
                                    .set("X-API-Key", &key)
                                    .set("Accept", "application/json")
                                    .call()
                                {
                                    Ok(response) => match response.into_json::<Vec<VoiceBotTemplate>>() {
                                        Ok(templates) => Ok(templates),
                                        Err(e) => Err(format!("Помилка парсингу: {}", e)),
                                    },
                                    Err(ureq::Error::Status(401, _)) => Err("Невірний ключ. Перевірте X-API-Key в секції АПІ.".to_string()),
                                    Err(ureq::Error::Status(code, _)) if code >= 500 => {
                                        Err(format!("Сервер тимчасово недоступний ({}). Спробуйте пізніше.", code))
                                    }
                                    Err(ureq::Error::Status(code, _)) => Err(format!("Помилка запиту ({})", code)),
                                    Err(_) => Err("Помилка мережі. Перевірте з'єднання.".to_string()),
                                };

                                *templates_arc.lock().unwrap() = Some(parsed);
                                *loading_arc.lock().unwrap() = false;
                                ctx.request_repaint();
                            });
                        }
                    }
                    Some(Ok(templates)) => {
                        let selected_name = templates
                            .iter()
                            .find(|t| t.uuid == *voiceover_template_uuid)
                            .map(|t| t.name.as_str())
                            .unwrap_or(translate(language, "voiceover_template_hint"));

                        egui::ComboBox::from_id_salt("voicebot_template_combo")
                            .selected_text(selected_name)
                            .show_ui(ui, |ui| {
                                for template in &templates {
                                    ui.selectable_value(
                                        voiceover_template_uuid,
                                        template.uuid.clone(),
                                        &template.name,
                                    );
                                }
                            });
                    }
                    Some(Err(error)) => {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("❌ {}", error))
                                .color(egui::Color32::from_rgb(231, 76, 60))
                                .size(12.0)
                        ).wrap());
                        ui.add_space(4.0);
                        if ui.button(translate(language, "voiceover_templates_retry")).clicked() {
                            *voicebot_templates.lock().unwrap() = None;
                        }
                    }
                }
            }
        }

        if voiceover_provider.as_str() == "Edge TTS" {
            let is_loading = *edge_tts_loading_voices.lock().unwrap();
            let voices_snapshot = edge_tts_voices.lock().unwrap().clone();

            if is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new(translate(language, "edge_tts_voices_loading"))
                            .weak()
                            .size(12.0)
                    );
                });
            } else {
                match voices_snapshot {
                    None => {
                        // Починаємо фонове завантаження голосів
                        *edge_tts_loading_voices.lock().unwrap() = true;
                        crate::api::edgetts::fetch_voices(
                            Arc::clone(edge_tts_voices),
                            Arc::clone(edge_tts_loading_voices),
                            ui.ctx().clone(),
                        );
                    }
                    Some(Ok(voices)) => {
                        // 1. Фільтрація голосів
                        let filtered_voices: Vec<&crate::api::edgetts::EdgeTTSVoice> = voices
                            .iter()
                            .filter(|v| {
                                if *edge_tts_show_all_languages {
                                    true
                                } else {
                                    // Показуємо uk-UA, en-US, en-GB, ru-RU
                                    let loc = v.locale.to_lowercase();
                                    loc == "uk-ua" || loc == "en-us" || loc == "en-gb" || loc == "ru-ru"
                                }
                            })
                            .collect();

                        // 2. ComboBox вибору голосу
                        ui.label(egui::RichText::new(translate(language, "edge_tts_voice_label")).size(12.0));
                        ui.add_space(2.0);

                        let current_voice_friendly = voices
                            .iter()
                            .find(|v| v.short_name == *edge_tts_voice)
                            .map(|v| v.friendly_name.clone())
                            .unwrap_or_else(|| edge_tts_voice.clone());

                        egui::ComboBox::from_id_salt("edge_tts_voice_combo")
                            .selected_text(current_voice_friendly)
                            .width(ui.available_width() - 10.0)
                            .show_ui(ui, |ui| {
                                for voice_item in &filtered_voices {
                                    ui.selectable_value(
                                        edge_tts_voice,
                                        voice_item.short_name.clone(),
                                        voice_item.friendly_name.clone(),
                                    );
                                }
                            });

                        ui.add_space(4.0);

                        // Прапорець "Показати всі мови"
                        ui.checkbox(edge_tts_show_all_languages, translate(language, "edge_tts_show_all"));

                        ui.add_space(8.0);

                        // 3. Параметри темпу, тональності та гучності
                        let mut rate_val: i32 = edge_tts_rate.parse::<i32>().unwrap_or(0);
                        let mut pitch_val: i32 = edge_tts_pitch.parse::<i32>().unwrap_or(0);
                        let mut volume_val: i32 = edge_tts_volume.parse::<i32>().unwrap_or(0);

                        let mut changed = false;

                        egui::Grid::new("edge_tts_params_grid")
                            .num_columns(2)
                            .spacing([8.0, 8.0])
                            .show(ui, |ui| {
                                // Темп
                                ui.label(translate(language, "edge_tts_rate_label"));
                                ui.scope(|ui| {
                                    ui.style_mut().spacing.slider_width = 120.0;
                                    if ui.add(egui::Slider::new(&mut rate_val, -100..=100).suffix("%")).changed() {
                                        changed = true;
                                    }
                                });
                                ui.end_row();

                                // Тональність
                                ui.label(translate(language, "edge_tts_pitch_label"));
                                ui.scope(|ui| {
                                    ui.style_mut().spacing.slider_width = 120.0;
                                    if ui.add(egui::Slider::new(&mut pitch_val, -100..=100).suffix("Hz")).changed() {
                                        changed = true;
                                    }
                                });
                                ui.end_row();

                                // Гучність
                                ui.label(translate(language, "edge_tts_volume_label"));
                                ui.scope(|ui| {
                                    ui.style_mut().spacing.slider_width = 120.0;
                                    if ui.add(egui::Slider::new(&mut volume_val, -100..=100).suffix("%")).changed() {
                                        changed = true;
                                    }
                                });
                                ui.end_row();
                            });

                        if changed {
                            *edge_tts_rate = rate_val.to_string();
                            *edge_tts_pitch = pitch_val.to_string();
                            *edge_tts_volume = volume_val.to_string();
                        }
                    }
                    Some(Err(error)) => {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("❌ {}", error))
                                .color(egui::Color32::from_rgb(231, 76, 60))
                                .size(12.0)
                        ).wrap());
                        ui.add_space(4.0);
                        if ui.button(translate(language, "voiceover_templates_retry")).clicked() {
                            *edge_tts_voices.lock().unwrap() = None;
                        }
                    }
                }
            }
        }

        ui.add_space(6.0);
    });
}
