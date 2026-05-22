use std::sync::{Arc, Mutex};

/// Статус виконання задачі пайплайну.
#[derive(Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
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
    pub translation_prompt: String,
    pub translation_model: String,
    pub translation_temperature: f32,
    pub translation_service: String,
    pub openrouter_key: String,
    pub voiceover_enabled: bool,
    pub voicebot_key: String,
    pub voiceover_template_uuid: String,
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
        }
    }
}
