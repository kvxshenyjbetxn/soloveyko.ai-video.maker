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
    /// Знімок налаштувань — зберігається для можливого перезапуску задачі
    pub settings: JobSettings,
    /// Збережений перекладений текст (заповнюється після перекладу)
    pub translated_text: Arc<Mutex<Option<String>>>,
    /// Вартість перекладу (якщо використовується OpenRouter)
    pub translation_cost: Arc<Mutex<Option<f64>>>,
    /// Тривалість аудіо після озвучки (в секундах)
    pub audio_duration: Arc<Mutex<Option<f64>>>,
}

impl PipelineJob {
    pub fn new(id: u64, name: String, settings: JobSettings) -> Self {
        Self {
            id,
            name,
            status: Arc::new(Mutex::new(JobStatus::Pending)),
            translation_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            voiceover_stage: Arc::new(Mutex::new(StageStatus::Pending)),
            settings,
            translated_text: Arc::new(Mutex::new(None)),
            translation_cost: Arc::new(Mutex::new(None)),
            audio_duration: Arc::new(Mutex::new(None)),
        }
    }
}
