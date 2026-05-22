use std::sync::{Arc, Mutex};

/// Статус виконання задачі пайплайну.
#[derive(Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed(String),
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
    pub openrouter_key: String,
}

/// Одна задача в черзі пайплайну.
pub struct PipelineJob {
    pub id: u64,
    /// Назва задачі, яку ввів користувач
    pub name: String,
    pub status: Arc<Mutex<JobStatus>>,
    /// Знімок налаштувань — зберігається для можливого перезапуску задачі
    pub settings: JobSettings,
}

impl PipelineJob {
    pub fn new(id: u64, name: String, settings: JobSettings) -> Self {
        Self {
            id,
            name,
            status: Arc::new(Mutex::new(JobStatus::Pending)),
            settings,
        }
    }
}
