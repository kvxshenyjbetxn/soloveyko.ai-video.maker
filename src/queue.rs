use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

/// Статус виконання задачі пайплайну.
#[derive(Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    AwaitingControl,
    /// Очікує перегляду згенерованих зображень користувачем
    AwaitingMediaControl,
    /// Очікує підтвердження монтажу від користувача
    AwaitingMontageControl,
    /// Очікує підтвердження агентного кроку від користувача
    AwaitingAgentControl,
    Done,
    Cancelled,
    Failed(String),
}

impl JobStatus {
    /// Чи вважається задача активною для кнопок керування чергою.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Running
                | Self::AwaitingControl
                | Self::AwaitingMediaControl
                | Self::AwaitingMontageControl
                | Self::AwaitingAgentControl
        )
    }
}

const JOB_CANCELLED_MESSAGE: &str = "Task cancelled by user.";

fn cancel_registry() -> &'static Mutex<HashMap<u64, Arc<AtomicBool>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<u64, Arc<AtomicBool>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Готує runtime-стан задачі перед запуском або повтором.
pub fn reset_job_runtime(job_id: u64) {
    let mut registry = cancel_registry().lock().unwrap();
    let flag = registry
        .entry(job_id)
        .or_insert_with(|| Arc::new(AtomicBool::new(false)));
    flag.store(false, Ordering::SeqCst);
}

/// Виставляє прапорець повного скасування задачі.
pub fn request_job_cancel(job_id: u64) {
    let mut registry = cancel_registry().lock().unwrap();
    let flag = registry
        .entry(job_id)
        .or_insert_with(|| Arc::new(AtomicBool::new(false)));
    flag.store(true, Ordering::SeqCst);
}

/// Повертає true, якщо задача була скасована користувачем.
pub fn is_job_cancelled(job_id: u64) -> bool {
    cancel_registry()
        .lock()
        .unwrap()
        .get(&job_id)
        .map(|flag| flag.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Забирає runtime-стан задачі, коли вона повністю більше не потрібна.
pub fn forget_job_runtime(job_id: u64) {
    cancel_registry().lock().unwrap().remove(&job_id);
}

/// Текст помилки, який використовується для кооперативного скасування задачі.
pub fn cancelled_error() -> String {
    JOB_CANCELLED_MESSAGE.to_string()
}

/// Допомагає відрізнити звичайну помилку від ручного скасування.
pub fn is_cancelled_error(error: &str) -> bool {
    error.contains(JOB_CANCELLED_MESSAGE)
}

/// Одне повідомлення в чаті з агентом.
#[derive(Clone)]
pub struct AgentChatMessage {
    pub role: String, // "user" або "agent"
    pub content: String,
}

/// Інформація про активну сесію агента для продовження чату.
#[derive(Clone)]
pub struct AgentSessionInfo {
    pub session_id: String,
    pub service: String, // "Claude Code" або "Gemini CLI"
    pub model: String,
}

/// Ідентифікатор етапу для повтору виконання з цього місця.
#[derive(Clone, PartialEq, Debug)]
pub enum RetryStage {
    Translation,
    Voiceover,
    Video,
    Subtitles,
    Montage,
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
    /// Підрежим агента відеоряду: "full" або "prompt_only"
    pub video_agent_mode: String,
    /// Додаткова користувацька інструкція агенту для роботи з segments.json
    pub video_agent_prompt: String,
    pub video_style_enabled: bool,
    pub video_style_prompt: String,
    pub video_context_enabled: bool,
    pub video_context_mode: String,
    pub video_context_chars: usize,
    pub video_llm_service: String,
    pub video_llm_model: String,
    pub video_llm_temperature: f32,
    pub text_split_mode: String,
    pub text_split_char_limit: usize,
    pub googler_key: String,
    pub googler_image_priority: Vec<String>,
    pub googler_video_priority: Vec<String>,
    pub googler_image_max_threads: usize,
    pub googler_video_upscale_enabled: bool,
    pub googler_video_upscale_resolution: String,
    pub googler_video_upscale_quality: String,
    pub video_media_type: String,

    pub assemblyai_key: String,
    pub pexels_key: String,
    pub pixabay_key: String,
    pub subtitles_enabled: bool,
    pub subtitles_service: String,
    pub whisper_language: String,
    pub whisper_model: String,
    pub whisper_max_line_width: usize,
    pub subtitle_font_size: u32,
    pub subtitle_color: [u8; 3],
    pub subtitle_margin_v: u32,
    pub subtitle_karaoke: bool,
    /// 0 = fill (\kf), 1 = switch (\k), 2 = follow (per-word highlight + повернення)
    pub subtitle_karaoke_mode: u8,
    pub subtitle_karaoke_highlight_color: [u8; 3],
    pub subtitle_karaoke_outline_color: [u8; 3],
    pub subtitle_karaoke_bold: bool,
    /// Масштаб поточного слова у %, лише для режиму follow (100 = без змін)
    pub subtitle_karaoke_scale: u32,
    pub subtitle_font: String,
    pub montage_enabled: bool,
    #[allow(dead_code)]
    pub montage_service: String,
    /// Генерувати CapCut-проект замість локального FFmpeg-монтажу.
    pub capcut_enabled: bool,
    /// Шлях до кореневого каталогу чернеток CapCut.
    pub capcut_draft_path: String,
    pub montage_fps: u32,
    pub montage_preset: String,
    pub montage_bitrate: u32,
    pub montage_transition: String,
    pub montage_transition_duration: f32,
    pub montage_image_zoom_enabled: bool,
    pub montage_image_zoom_intensity: f32,
    pub montage_image_zoom_mode: String,
    pub montage_image_zoom_scale: f32,
    pub montage_image_shake_enabled: bool,
    pub montage_image_shake_intensity: f32,
    pub media_control_enabled: bool,
    /// Чи увімкнено контроль монтажу (показує кнопку редактора монтажу в карточці задачі)
    pub montage_control_enabled: bool,
    /// Чи увімкнено тригери накладення медіа за ключовими фразами
    pub overlay_triggers_enabled: bool,
    /// Список тригерів накладення медіа
    pub overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
    /// Якщо Some — пайплайн стартує з цього етапу замість повного запуску (режим відновлення)
    pub resume_from_stage: Option<RetryStage>,
    /// Пропустити запуск агента при відновленні (timeline.json вже є)
    pub skip_agent_on_resume: bool,
    /// Пропускати сегменти, для яких медіафайл вже існує на диску (режим догенерації)
    pub skip_existing_media: bool,
}

/// Одна задача в черзі пайплайну.
impl JobSettings {
    /// Чи використовує задача CLI-агента для побудови/редагування segments.json.
    pub fn is_agent_service(&self) -> bool {
        matches!(
            self.video_llm_service.as_str(),
            "Claude Code" | "Gemini CLI" | "Codex CLI" | "AGY CLI" | "Pi CLI"
        )
    }

    /// Чи увімкнено новий режим Prompt Only.
    pub fn is_prompt_only_agent_mode(&self) -> bool {
        self.video_enabled && self.is_agent_service() && self.video_agent_mode == "prompt_only"
    }
}

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
    /// Загальна вартість всіх LLM-запитів задачі (якщо використовується OpenRouter)
    pub total_cost: Arc<Mutex<Option<f64>>>,
    /// Тривалість аудіо після озвучки (в секундах)
    pub audio_duration: Arc<Mutex<Option<f64>>>,
    /// Прогрес підготовки промтів: (завершено, загалом). None — поки кількість невідома.
    pub prompts_progress: Arc<Mutex<Option<(usize, usize)>>>,
    /// Прогрес генерації медіа: (завершено, загалом). None — поки кількість невідома.
    pub media_progress: Arc<Mutex<Option<(usize, usize)>>>,
    /// Прогрес монтажу [0.0..1.0]. None — ще не розпочато.
    pub montage_progress: Arc<Mutex<Option<f32>>>,
    /// Розмір фінального відеофайлу в байтах. None до завершення монтажу.
    pub montage_file_size: Arc<Mutex<Option<u64>>>,
    /// Condvar для відновлення пайплайну після контролю зображень
    pub media_control_resume: Arc<(Mutex<bool>, Condvar)>,
    /// Condvar для відновлення пайплайну після контролю монтажу
    pub montage_control_resume: Arc<(Mutex<bool>, Condvar)>,
    /// Condvar для відновлення пайплайну після підтвердження агентного кроку
    pub agent_control_resume: Arc<(Mutex<bool>, Condvar)>,
    /// Сигнал для перебудови таймлінії в редакторі монтажу після чату з агентом
    pub timeline_rebuild_requested: Arc<Mutex<bool>>,
    /// Повідомлення чату з агентом (зберігається між сесіями)
    pub agent_chat: Arc<Mutex<Vec<AgentChatMessage>>>,
    /// Активна сесія агента (session_id для продовження чату)
    pub agent_session: Arc<Mutex<Option<AgentSessionInfo>>>,
    /// Перевизначення режиму рендеру з редактора монтажу: None = з налаштувань, Some(true) = CapCut, Some(false) = FFmpeg
    pub capcut_mode_override: Arc<Mutex<Option<bool>>>,
}

impl PipelineJob {
    pub fn new(id: u64, name: String, settings: JobSettings) -> Self {
        reset_job_runtime(id);
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
            total_cost: Arc::new(Mutex::new(None)),
            audio_duration: Arc::new(Mutex::new(None)),
            prompts_progress: Arc::new(Mutex::new(None)),
            media_progress: Arc::new(Mutex::new(None)),
            montage_progress: Arc::new(Mutex::new(None)),
            montage_file_size: Arc::new(Mutex::new(None)),
            media_control_resume: Arc::new((Mutex::new(false), Condvar::new())),
            montage_control_resume: Arc::new((Mutex::new(false), Condvar::new())),
            agent_control_resume: Arc::new((Mutex::new(false), Condvar::new())),
            timeline_rebuild_requested: Arc::new(Mutex::new(false)),
            agent_chat: Arc::new(Mutex::new(Vec::new())),
            agent_session: Arc::new(Mutex::new(None)),
            capcut_mode_override: Arc::new(Mutex::new(None)),
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

        // Етап 3: Відеоряд (промти + медіа = 2 внутрішніх підетапи)
        if self.settings.video_enabled {
            total_stages += 1;
            let stage = self.video_stage.lock().unwrap().clone();
            match stage {
                StageStatus::Done => {
                    completed_score += 1.0;
                    completed_count += 1;
                }
                StageStatus::Running => {
                    // Підетап промтів (0..0.5) + підетап медіа (0.5..1.0)
                    let prompts_done = self
                        .prompts_progress
                        .lock()
                        .unwrap()
                        .and_then(|(done, total)| {
                            if total > 0 {
                                Some(done as f32 / total as f32)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0.0);
                    let media_done = self
                        .media_progress
                        .lock()
                        .unwrap()
                        .and_then(|(done, total)| {
                            if total > 0 {
                                Some(done as f32 / total as f32)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0.0);
                    completed_score += (prompts_done * 0.5 + media_done * 0.5).min(0.99);
                }
                _ => {}
            }
        }

        // Етап 4: Субтитри — завжди в прогресі якщо є озвучка і відповідний сервіс налаштований
        let subtitles_active = self.settings.voiceover_enabled
            && (self.settings.subtitles_service == "Whisper"
                || self.settings.subtitles_service == "WhisperX"
                || self.settings.subtitles_service == "AssemblyAI");
        if subtitles_active {
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
