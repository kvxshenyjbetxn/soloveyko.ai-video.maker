use crate::localization::{translate, Language};
use eframe::egui;

/// Файли попередньої обробки, знайдені в кінцевій папці задачі.
pub struct FoundFiles {
    pub text_txt: bool,
    pub voice_file: bool,
    pub subtitle_srt: bool,
    /// Вихідний файл агента — якщо є, агент вже відпрацював
    pub segments_json: bool,
    /// Повний таймлайн з медіа (після assign_media_to_timeline)
    pub timeline_json: bool,
    pub media_images: usize,
    pub media_videos: usize,
    pub output_video: bool,
    /// Кількість сегментів (очікувана кількість медіафайлів)
    pub expected_media: Option<usize>,
}

/// Дані для діалогу відновлення: назва задачі, знайдені файли, знімок налаштувань
/// та стан чекбоксів "зберегти цей етап".
pub struct ResumePendingData {
    pub task_name: String,
    pub found: FoundFiles,
    pub settings: crate::queue::JobSettings,
    /// Зберегти наявний файл озвучки (не перезапускати цей етап)
    pub keep_voiceover: bool,
    /// Зберегти наявні субтитри (вимкнено якщо озвучка не зберігається)
    pub keep_subtitles: bool,
    /// Зберегти timeline.json (пропустити запуск агента)
    pub keep_timeline: bool,
    /// Зберегти наявні медіафайли
    pub keep_video: bool,
}

impl ResumePendingData {
    pub fn new(task_name: String, found: FoundFiles, settings: crate::queue::JobSettings) -> Self {
        let keep_voiceover = found.voice_file;
        let keep_subtitles = found.subtitle_srt;
        // keep_timeline = true якщо є хоч один з файлів агента (segments.json або timeline.json)
        let keep_timeline = found.segments_json || found.timeline_json;
        let keep_video = found.media_images > 0 || found.media_videos > 0;
        Self {
            task_name,
            found,
            settings,
            keep_voiceover,
            keep_subtitles,
            keep_timeline,
            keep_video,
        }
    }

    /// Визначає з якого етапу фактично запустити пайплайн, з урахуванням чекбоксів.
    pub fn effective_resume_stage(&self) -> Option<crate::queue::RetryStage> {
        let agent_file_exists = self.found.segments_json || self.found.timeline_json;

        if self.found.voice_file && !self.keep_voiceover {
            return Some(crate::queue::RetryStage::Voiceover);
        }
        if self.found.subtitle_srt && !self.keep_subtitles {
            return Some(crate::queue::RetryStage::Subtitles);
        }
        // Файл агента знято → перезапустити агента (Video з агентом)
        if agent_file_exists && !self.keep_timeline {
            return Some(crate::queue::RetryStage::Video);
        }
        if (self.found.media_images > 0 || self.found.media_videos > 0) && !self.keep_video {
            return Some(crate::queue::RetryStage::Video);
        }
        // Якщо медіа зберігаємо але неповне — довантажити відсутні
        if self.keep_video && agent_file_exists {
            if let Some(expected) = self.found.expected_media {
                let present = self.found.media_images + self.found.media_videos;
                if present < expected {
                    return Some(crate::queue::RetryStage::Video);
                }
            }
        }
        self.found.resume_stage()
    }
}

impl FoundFiles {
    /// Сканує папку задачі та повертає знайдені файли попередньої обробки.
    pub fn scan(task_dir: &std::path::Path, task_name: &str) -> Self {
        let text_txt = task_dir.join("text.txt").exists();
        let voice_file = task_dir.join("voice.mp3").exists() || task_dir.join("voice.wav").exists();
        let subtitle_srt = task_dir.join("subtitle.srt").exists();
        let segments_json = task_dir.join("segments.json").exists();
        let timeline_json = task_dir.join("timeline.json").exists();

        let media_dir = task_dir.join("media");
        let (media_images, media_videos) = if media_dir.is_dir() {
            let mut imgs = 0usize;
            let mut vids = 0usize;
            if let Ok(entries) = std::fs::read_dir(&media_dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                        match ext.to_lowercase().as_str() {
                            "jpg" | "jpeg" | "png" | "webp" => imgs += 1,
                            "mp4" | "webm" | "mov" => vids += 1,
                            _ => {}
                        }
                    }
                }
            }
            (imgs, vids)
        } else {
            (0, 0)
        };

        // Та сама логіка санітайзингу що в montage.rs
        let safe_name: String = task_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let output_video = task_dir.join(format!("{}.mp4", safe_name.trim())).exists();

        // Рахуємо очікувану кількість медіа: спочатку з timeline.json, потім з segments.json
        let expected_media = if timeline_json {
            std::fs::read_to_string(task_dir.join("timeline.json"))
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| {
                    v["segments"]
                        .as_array()
                        .map(|a| a.iter().filter(|seg| !seg["media"].is_null()).count())
                })
        } else if segments_json {
            std::fs::read_to_string(task_dir.join("segments.json"))
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v["segments"].as_array().map(|a| a.len()))
        } else {
            None
        };

        Self {
            text_txt,
            voice_file,
            subtitle_srt,
            segments_json,
            timeline_json,
            media_images,
            media_videos,
            output_video,
            expected_media,
        }
    }

    pub fn has_any(&self) -> bool {
        // Фінальне відео не рахується — перевіряємо лише проміжні файли
        self.text_txt
            || self.voice_file
            || self.subtitle_srt
            || self.segments_json
            || self.timeline_json
            || self.media_images > 0
            || self.media_videos > 0
    }

    /// Визначає з якого етапу продовжити виходячи лише з наявних файлів
    /// (без урахування чекбоксів — для початкового стану).
    pub fn resume_stage(&self) -> Option<crate::queue::RetryStage> {
        // output_video не враховується — визначаємо по проміжних файлах
        if self.media_images > 0 || self.media_videos > 0 {
            Some(crate::queue::RetryStage::Montage)
        } else if self.segments_json || self.timeline_json {
            // Агент вже відпрацював — продовжуємо з пошуку медіа (без агента)
            Some(crate::queue::RetryStage::Video)
        } else if self.subtitle_srt {
            Some(crate::queue::RetryStage::Video)
        } else if self.voice_file {
            Some(crate::queue::RetryStage::Subtitles)
        } else if self.text_txt {
            Some(crate::queue::RetryStage::Voiceover)
        } else {
            None
        }
    }
}

/// Малює модальне вікно перевірки відновлення задачі.
/// Показує знайдені файли з чекбоксами, залежності між етапами та пропонує вибір дії.
pub fn draw_resume_dialog(
    ctx: &egui::Context,
    language: Language,
    open: &mut bool,
    pending: &mut Option<ResumePendingData>,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
) {
    if !*open {
        return;
    }

    let mut do_continue = false;
    let mut do_fresh = false;
    let mut do_fill_missing = false;

    // Витягуємо дані та оновлюємо чекбокси в окремому скопі щоб уникнути конфліктів позик
    {
        let data = match pending.as_mut() {
            Some(d) => d,
            None => {
                *open = false;
                return;
            }
        };

        // Копіюємо значення для відображення всередині closure
        let task_name = data.task_name.clone();
        let text_txt = data.found.text_txt;
        let voice_file = data.found.voice_file;
        let subtitle_srt = data.found.subtitle_srt;
        let segments_json = data.found.segments_json;
        let timeline_json = data.found.timeline_json;
        let media_images = data.found.media_images;
        let media_videos = data.found.media_videos;
        let output_video = data.found.output_video;
        let expected_media = data.found.expected_media;
        let agent_file_exists = segments_json || timeline_json;

        let mut keep_vo = data.keep_voiceover;
        let mut keep_su = data.keep_subtitles;
        let mut keep_tl = data.keep_timeline;
        let mut keep_vi = data.keep_video;

        // Обчислюємо поточний resume stage для відображення
        let resume_stage = {
            let temp = ResumePendingData {
                task_name: String::new(),
                found: FoundFiles {
                    text_txt,
                    voice_file,
                    subtitle_srt,
                    segments_json,
                    timeline_json,
                    media_images,
                    media_videos,
                    output_video,
                    expected_media,
                },
                settings: data.settings.clone(),
                keep_voiceover: keep_vo,
                keep_subtitles: keep_su,
                keep_timeline: keep_tl,
                keep_video: keep_vi,
            };
            temp.effective_resume_stage()
        };

        let green = egui::Color32::from_rgb(46, 204, 113);
        let weak = egui::Color32::from_rgb(140, 140, 140);

        egui::Window::new(translate(language, "resume_dialog_title"))
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.label(translate(language, "resume_dialog_folder"));
                    ui.label(egui::RichText::new(&task_name).strong().monospace());
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // Переклад — завжди інформаційно, без чекбоксу
                if text_txt {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0); // відступ замість чекбоксу
                        ui.label(egui::RichText::new("✓").color(green));
                        ui.label(translate(language, "translation"));
                        ui.label(egui::RichText::new("text.txt").color(weak).size(11.0));
                    });
                }

                // Озвучка — з чекбоксом
                if voice_file {
                    ui.horizontal(|ui| {
                        let resp = ui.checkbox(&mut keep_vo, "");
                        if resp.changed() && !keep_vo {
                            // Залежність: вимкнено озвучку → субтитри теж вимкнено
                            keep_su = false;
                        }
                        ui.label(translate(language, "voiceover"));
                        ui.label(
                            egui::RichText::new("voice.mp3 / voice.wav")
                                .color(weak)
                                .size(11.0),
                        );
                    });
                }

                // Субтитри — з чекбоксом; вимкнено якщо озвучка не зберігається
                if subtitle_srt {
                    let subtitles_enabled = !voice_file || keep_vo;
                    if !subtitles_enabled {
                        keep_su = false;
                    }
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(subtitles_enabled, |ui| {
                            ui.checkbox(&mut keep_su, "");
                        });
                        let label = egui::RichText::new(translate(language, "subtitles")).color(
                            if subtitles_enabled {
                                ui.visuals().text_color()
                            } else {
                                ui.visuals().weak_text_color()
                            },
                        );
                        ui.label(label);
                        ui.label(egui::RichText::new("subtitle.srt").color(weak).size(11.0));
                        if !subtitles_enabled {
                            ui.label(
                                egui::RichText::new(translate(language, "resume_depends_on_voice"))
                                    .color(ui.visuals().weak_text_color())
                                    .size(10.0),
                            );
                        }
                    });
                }

                // Файл агента (segments.json або timeline.json) — з чекбоксом
                if agent_file_exists {
                    let file_hint = match (segments_json, timeline_json) {
                        (true, true) => "segments.json + timeline.json",
                        (true, false) => "segments.json",
                        _ => "timeline.json",
                    };
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut keep_tl, "");
                        ui.label(translate(language, "resume_timeline"));
                        ui.label(egui::RichText::new(file_hint).color(weak).size(11.0));
                    });
                }

                // Відеоряд — з чекбоксом
                if media_images > 0 || media_videos > 0 {
                    let mut parts = Vec::new();
                    if media_images > 0 {
                        parts.push(format!(
                            "{} {}",
                            media_images,
                            translate(language, "resume_images")
                        ));
                    }
                    if media_videos > 0 {
                        parts.push(format!(
                            "{} {}",
                            media_videos,
                            translate(language, "resume_videos")
                        ));
                    }
                    // Якщо є timeline.json — показуємо скільки є / скільки очікується
                    let count_label = if let Some(expected) = expected_media {
                        let present = media_images + media_videos;
                        format!("{} ({}/{})", parts.join(", "), present, expected)
                    } else {
                        parts.join(", ")
                    };
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut keep_vi, "");
                        ui.label(translate(language, "video"));
                        ui.label(egui::RichText::new(count_label).color(weak).size(11.0));
                    });
                }

                // Готове відео — завжди інформаційно, без чекбоксу
                if output_video {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("✓").color(green));
                        ui.label(translate(language, "resume_output_video"));
                        ui.label(
                            egui::RichText::new(format!("{}.mp4", task_name))
                                .color(weak)
                                .size(11.0),
                        );
                    });
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // Поточний ефективний resume stage
                match &resume_stage {
                    Some(stage) => {
                        let stage_name = match stage {
                            crate::queue::RetryStage::Translation => {
                                translate(language, "translation")
                            }
                            crate::queue::RetryStage::Voiceover => translate(language, "voiceover"),
                            crate::queue::RetryStage::Subtitles => translate(language, "subtitles"),
                            crate::queue::RetryStage::Video => translate(language, "video"),
                            crate::queue::RetryStage::Montage => translate(language, "editing"),
                        };
                        ui.horizontal(|ui| {
                            ui.label(translate(language, "resume_resume_from"));
                            ui.label(
                                egui::RichText::new(stage_name)
                                    .strong()
                                    .color(egui::Color32::from_rgb(52, 152, 219)),
                            );
                        });
                    }
                    None => {
                        ui.label(
                            egui::RichText::new(translate(language, "resume_all_done"))
                                .color(green),
                        );
                    }
                }

                ui.add_space(10.0);

                // Кнопка "Догенерувати відсутні" — якщо є файл агента і медіа неповні
                let missing_count = expected_media.and_then(|exp| {
                    let present = media_images + media_videos;
                    if agent_file_exists && present < exp {
                        Some(exp - present)
                    } else {
                        None
                    }
                });

                if let Some(missing) = missing_count {
                    ui.add_space(4.0);
                    let fill_label = format!(
                        "{} ({})",
                        translate(language, "resume_fill_missing_btn"),
                        missing
                    );
                    if ui
                        .add_sized(
                            [ui.available_width(), 26.0],
                            egui::Button::new(egui::RichText::new(fill_label).strong()),
                        )
                        .clicked()
                    {
                        do_fill_missing = true;
                    }
                }

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [140.0, 26.0],
                            egui::Button::new(translate(language, "resume_fresh_btn")),
                        )
                        .clicked()
                    {
                        do_fresh = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if resume_stage.is_some() {
                            if ui
                                .add_sized(
                                    [140.0, 26.0],
                                    egui::Button::new(
                                        egui::RichText::new(translate(
                                            language,
                                            "resume_continue_btn",
                                        ))
                                        .strong(),
                                    ),
                                )
                                .clicked()
                            {
                                do_continue = true;
                            }
                        }
                    });
                });

                ui.add_space(2.0);
            });

        // Зберігаємо змінені стани чекбоксів назад
        data.keep_voiceover = keep_vo;
        data.keep_subtitles = keep_su;
        data.keep_timeline = keep_tl;
        data.keep_video = keep_vi;
    } // data borrow ends here

    if do_continue {
        if let Some(data) = pending.take() {
            enqueue_with_resume(data, jobs, job_counter);
        }
        *open = false;
    } else if do_fresh {
        if let Some(data) = pending.take() {
            enqueue_fresh(data, jobs, job_counter);
        }
        *open = false;
    } else if do_fill_missing {
        if let Some(data) = pending.take() {
            enqueue_fill_missing(data, jobs, job_counter);
        }
        *open = false;
    }
}

/// Додає задачу в чергу з відновленням відповідно до чекбоксів.
fn enqueue_with_resume(
    data: ResumePendingData,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
) {
    let resume_stage = data.effective_resume_stage();

    // Витягуємо скалярні значення до часткового переміщення
    let text_txt_exists = data.found.text_txt;
    let keep_vo = data.keep_voiceover;
    let keep_su = data.keep_subtitles;
    let keep_tl = data.keep_timeline;
    let keep_vi = data.keep_video;
    let save_path = data.settings.save_path.clone();

    // Часткові переміщення: found → settings → task_name
    let found = data.found;
    let mut settings = data.settings;
    settings.resume_from_stage = resume_stage;
    // Якщо timeline.json зберігаємо — пропускаємо агента при Video retry
    settings.skip_agent_on_resume = keep_tl;
    // Якщо медіа неповне і ми йдемо від Video — пропускаємо вже наявні медіафайли
    if keep_vi && found.timeline_json {
        if let Some(expected) = found.expected_media {
            let present = found.media_images + found.media_videos;
            if present < expected {
                settings.skip_existing_media = true;
            }
        }
    }

    let id = *job_counter;
    *job_counter += 1;
    let job = crate::queue::PipelineJob::new(id, data.task_name, settings);

    // Зчитуємо перекладений текст для озвучки при відновленні
    if text_txt_exists {
        if let Ok(text) = std::fs::read_to_string(std::path::Path::new(&save_path).join("text.txt"))
        {
            *job.translated_text.lock().unwrap() = Some(text);
        }
    }

    // Відновлюємо чат з агентом
    let task_dir = std::path::Path::new(&save_path);
    if let Ok(text) = std::fs::read_to_string(task_dir.join("agent_chat.json")) {
        if let Ok(msgs) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
            let mut chat = job.agent_chat.lock().unwrap();
            for msg in msgs {
                if let (Some(role), Some(content)) = (msg["role"].as_str(), msg["content"].as_str())
                {
                    chat.push(crate::queue::AgentChatMessage {
                        role: role.to_string(),
                        content: content.to_string(),
                    });
                }
            }
        }
    }

    // Відновлюємо сесію агента (щоб можна було продовжити чат після перезапуску)
    if let Ok(text) = std::fs::read_to_string(task_dir.join("agent_session.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let (Some(session_id), Some(service), Some(model)) = (
                v["session_id"].as_str(),
                v["service"].as_str(),
                v["model"].as_str(),
            ) {
                *job.agent_session.lock().unwrap() = Some(crate::queue::AgentSessionInfo {
                    session_id: session_id.to_string(),
                    service: service.to_string(),
                    model: model.to_string(),
                });
            }
        }
    }

    // Позначаємо завершені етапи для коректного відображення в черзі
    pre_mark_stages(&job, &found, keep_vo, keep_su, keep_tl, keep_vi);

    jobs.push(job);
}

/// Додає задачу в чергу для запуску з нуля, ігноруючи наявні файли.
fn enqueue_fresh(
    mut data: ResumePendingData,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
) {
    data.settings.resume_from_stage = None;
    let id = *job_counter;
    *job_counter += 1;
    jobs.push(crate::queue::PipelineJob::new(
        id,
        data.task_name,
        data.settings,
    ));
}

/// Додає задачу в чергу для догенерації відсутніх медіафайлів.
/// Зберігає timeline.json, пропускає агента, пропускає вже наявні файли.
fn enqueue_fill_missing(
    data: ResumePendingData,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    job_counter: &mut u64,
) {
    let text_txt_exists = data.found.text_txt;
    let save_path = data.settings.save_path.clone();
    let found = data.found;

    let mut settings = data.settings;
    settings.resume_from_stage = Some(crate::queue::RetryStage::Video);
    settings.skip_agent_on_resume = true;
    settings.skip_existing_media = true;

    let id = *job_counter;
    *job_counter += 1;
    let job = crate::queue::PipelineJob::new(id, data.task_name, settings);

    if text_txt_exists {
        if let Ok(text) = std::fs::read_to_string(std::path::Path::new(&save_path).join("text.txt"))
        {
            *job.translated_text.lock().unwrap() = Some(text);
        }
    }

    // Позначаємо завершені етапи (озвучка, субтитри, таймлайн — Done; відео — pending для генерації)
    use crate::queue::StageStatus;
    let s = &job.settings;
    if found.text_txt && s.translation_enabled {
        *job.translation_stage.lock().unwrap() = StageStatus::Done;
    }
    if found.voice_file && s.voiceover_enabled {
        *job.voiceover_stage.lock().unwrap() = StageStatus::Done;
    }
    if found.subtitle_srt && s.voiceover_enabled {
        *job.subtitles_stage.lock().unwrap() = StageStatus::Done;
    }

    jobs.push(job);
}

/// Попередньо позначає завершені етапи в задачі (для UI до запуску).
fn pre_mark_stages(
    job: &crate::queue::PipelineJob,
    found: &FoundFiles,
    keep_voiceover: bool,
    keep_subtitles: bool,
    keep_timeline: bool,
    keep_video: bool,
) {
    let _ = keep_timeline; // timeline.json не є окремим етапом в UI — враховується через skip_agent_on_resume
    use crate::queue::StageStatus;
    let s = &job.settings;

    if found.text_txt && s.translation_enabled {
        *job.translation_stage.lock().unwrap() = StageStatus::Done;
    }
    if found.voice_file && keep_voiceover && s.voiceover_enabled {
        *job.voiceover_stage.lock().unwrap() = StageStatus::Done;
    }
    if found.subtitle_srt && keep_subtitles && s.voiceover_enabled {
        *job.subtitles_stage.lock().unwrap() = StageStatus::Done;
    }
    if (found.media_images > 0 || found.media_videos > 0) && keep_video && s.video_enabled {
        *job.video_stage.lock().unwrap() = StageStatus::Done;
    }
    if found.output_video && s.montage_enabled {
        *job.montage_stage.lock().unwrap() = StageStatus::Done;
    }
}
