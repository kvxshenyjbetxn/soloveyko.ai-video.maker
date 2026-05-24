use std::sync::{Arc, Mutex};

/// Статус виконання задачі пайплайну.
#[derive(Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    AwaitingControl,
    Done,
    Failed(String),
}

/// Статус окремого етапу пайплайну для відображення в карточці задачі.
#[derive(Clone, PartialEq)]
pub enum StageStatus {
    /// Очікує — сірий
    Pending,
    /// Виконується — жовтий
    Running,
    /// Завершено успішно — зелений
    Done,
    /// Помилка — червоний
    Failed,
}

/// Знімок налаштувань пайплайну на момент додавання задачі в чергу.
#[derive(Clone)]
pub struct JobSettings {
    pub text: String,
    pub save_path: String,
    pub translation_enabled: bool,
    pub translation_control_enabled: bool,
    pub translation_prompt: String,
    pub translation_model: String,
    pub translation_temperature: f32,
    pub translation_service: String,
    pub openrouter_key: String,
    pub voiceover_enabled: bool,
    pub voicebot_key: String,
    pub voiceover_template_uuid: String,
    pub voiceover_provider: String,
    pub edge_tts_voice: String,
    pub edge_tts_rate: String,
    pub edge_tts_pitch: String,
    pub edge_tts_volume: String,
    pub voiceover_convert_to_wav: bool,
    pub video_enabled: bool,
    #[allow(dead_code)]
    pub video_service: String,
    pub video_prompt: String,
    pub text_split_mode: String,
    pub text_split_char_limit: usize,
    pub googler_key: String,
    pub googler_image_priority: Vec<String>,
    pub googler_video_priority: Vec<String>,
    pub googler_image_max_threads: usize,
    pub video_media_type: String,
    pub subtitles_enabled: bool,
    pub subtitles_service: String,
    pub whisper_language: String,
    pub whisper_model: String,
    pub montage_enabled: bool,
    #[allow(dead_code)]
    pub montage_service: String,
    pub montage_fps: u32,
    pub montage_preset: String,
    pub montage_bitrate: u32,
}

/// Одна задача в черзі пайплайну.
pub struct PipelineJob {
    pub id: u64,
    /// Назва задачі, яку ввів користувач
    pub name: String,
    pub status: Arc<Mutex<JobStatus>>,
    /// Статус етапу перекладу (Переклад або Оригінал)
    pub translation_stage: Arc<Mutex<StageStatus>>,
    /// Статус етапу озвучки
    pub voiceover_stage: Arc<Mutex<StageStatus>>,
    /// Статус етапу відеоряду
    pub video_stage: Arc<Mutex<StageStatus>>,
    /// Статус етапу субтитрів
    pub subtitles_stage: Arc<Mutex<StageStatus>>,
    /// Статус етапу монтажу
    pub montage_stage: Arc<Mutex<StageStatus>>,
    /// Знімок налаштувань — зберігається для можливого перезапуску задачі
    pub settings: JobSettings,
    /// Збережений перекладений текст (заповнюється після перекладу)
    pub translated_text: Arc<Mutex<Option<String>>>,
    /// Вартість перекладу (якщо використовується OpenRouter)
    pub translation_cost: Arc<Mutex<Option<f64>>>,
    /// Тривалість аудіо після озвучки (в секундах)
    pub audio_duration: Arc<Mutex<Option<f64>>>,
    /// Прогрес генерації медіа: (завершено, загалом). None — поки кількість невідома.
    pub media_progress: Arc<Mutex<Option<(usize, usize)>>>,
}

impl PipelineJob {
    pub fn new(id: u64, name: String, settings: JobSettings) -> Self {
        Self {
            id,
            name,
            status: Arc::new(Mutex::new(JobStatus::Pending)),
            translation_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            voiceover_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            video_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            subtitles_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            montage_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            settings,
            translated_text: Arc::new(Mutex::new(None)),
            translation_cost: Arc::new(Mutex::new(None)),
            audio_duration: Arc::new(Mutex::new(None)),
            media_progress: Arc::new(Mutex::new(None)),
        }
    }

    /// Повертає (прогрес [0.0..1.0], кількість завершених етапів, загальна кількість активних етапів)
    /// Спроектовано з можливістю легкого розширення у майбутньому для нових етапів.
    pub fn calculate_progress(&self) -> (f32, usize, usize) {
        let mut total_stages = 0;
        let mut completed_score = 0.0f32;
        let mut completed_count = 0;

        // Етап 1: Переклад
        if self.settings.translation_enabled {
            total_stages += 1;
            let stage = self.translation_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    completed_score += 0.5;
                }
                _ => {}
            }
        }

        // Етап 2: Озвучка
        if self.settings.voiceover_enabled {
            total_stages += 1;
            let stage = self.voiceover_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    completed_score += 0.5;
                }
                _ => {}
            }
        }

        // Етап 3: Відеоряд
        if self.settings.video_enabled {
            total_stages += 1;
            let stage = self.video_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    // Якщо відома кількість медіафайлів — гранулярний прогрес
                    let granular = self.media_progress.lock().unwrap()
                        .and_then(|(done, total)| {
                            if total > 0 { Some(done as f32 / total as f32) } else { None }
                        });
                    completed_score += granular.unwrap_or(0.1);
                }
                _ => {}
            }
        }

        // Етап 4: Субтитри
        if self.settings.subtitles_enabled {
            total_stages += 1;
            let stage = self.subtitles_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    completed_score += 0.5;
                }
                _ => {}
            }
        }

        // Етап 5: Монтаж
        if self.settings.montage_enabled {
            total_stages += 1;
            let stage = self.montage_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    completed_score += 0.5;
                }
                _ => {}
            }
        }

        if total_stages == 0 {
            // Якщо жоден етап не увімкнено, то прогрес залежить від загального статусу
            let status = self.status.lock().unwrap().clone();
            if status == JobStatus::Done {
                (1.0, 0, 0)
            } else {
                (0.0, 0, 0)
            }
        } else {
            let progress = completed_score / total_stages as f32;
            (progress.clamp(0.0, 1.0), completed_count, total_stages)
        }
    }
}
