use crate::gui;
use crate::gui::settings::storage::{AppSettings, load_settings, save_settings};
use eframe::egui;
use crate::theme::AppTheme;
use crate::localization::{Language, translate};

/// Перерахування для представлення доступних вкладок програми.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Основна вкладка
    Main,
    /// Галерея згенерованих медіафайлів
    Gallery,
    /// Вкладка налаштувань
    Settings,
    /// Вкладка логів
    Logs,
}

/// Головна структура нашого GUI додатку, що зберігає його поточний стан.
pub struct VideoMakerApp {
    /// Поточна активна вкладка програми.
    active_tab: Tab,
    /// Текст сценарію, який вводить користувач.
    text_input: String,
    /// Поточна обрана тема оформлення додатку.
    theme: AppTheme,
    /// Обраний акцентний колір для елементів інтерфейсу.
    accent_color: egui::Color32,
    /// Поточна збережена ширина бічної панелі пайплайну.
    pipeline_width: f32,
    /// Поточна вибрана мова інтерфейсу програми.
    pub language: Language,
    /// Копія останніх збережених налаштувань на диску для відстеження змін у реальному часі.
    pub last_saved_settings: AppSettings,
    /// Ключ API для OpenRouter.
    pub openrouter_key: String,
    /// Тимчасовий статус перевірки OpenRouter API ключа.
    pub openrouter_status: Option<String>,
    /// Введення імені шаблону
    pub template_name_input: String,
    /// Доступні шаблони на диску
    pub saved_templates: Vec<String>,
    /// Статус роботи з шаблонами
    pub template_status: Option<String>,
    /// Ключ API для Voice Bot.
    pub voicebot_key: String,
    /// Статус перевірки Voice Bot API ключа.
    pub voicebot_status: Option<String>,
    /// Ключ API для Googler.
    pub googler_key: String,
    /// Статус перевірки Googler API ключа.
    pub googler_status: Option<String>,
    /// Результат фонового тесту API ключа Googler.
    pub googler_test_result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Баланс Googler для відображення у топбарі.
    pub googler_balance: std::sync::Arc<std::sync::Mutex<Option<crate::api::googler::GooglerBalance>>>,
    /// Обраний провайдер озвучки.
    pub voiceover_provider: String,
    /// UUID обраного шаблону озвучки.
    pub voiceover_template_uuid: String,
    /// Завантажені шаблони Voice Bot.
    pub voicebot_templates: std::sync::Arc<std::sync::Mutex<Option<Result<Vec<crate::gui::pipeline::voiceover::VoiceBotTemplate>, String>>>>,
    /// Прапорець завантаження шаблонів Voice Bot.
    pub voicebot_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Результат фонового тесту API ключа Voice Bot.
    pub voicebot_test_result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Чи увімкнено етап "Переклад" у пайплайні.
    pub pipeline_translation_enabled: bool,
    /// Чи увімкнено контроль перекладу у пайплайні.
    pub pipeline_translation_control_enabled: bool,
    /// Чи відкривати вікно контролю автоматично при переході задачі в AwaitingControl.
    pub pipeline_control_auto_open: bool,
    /// Чи увімкнено контроль зображень (пауза після відеоряду для перегляду).
    pub pipeline_media_control_enabled: bool,
    /// Кеш текстур для галереї медіафайлів. None означає помилку завантаження.
    pub gallery_textures: std::collections::HashMap<std::path::PathBuf, Option<egui::TextureHandle>>,
    /// Зображення, яке зараз відкрите у повноекранному перегляді.
    pub gallery_preview: Option<std::path::PathBuf>,
    /// Набір шляхів зображень, які зараз анімуються у фоні (image-to-video).
    pub gallery_anim_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Прапорець виконання перегенерації медіафайлу у фоні.
    pub media_regen_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Результат перегенерації. None = ще не завершено.
    pub media_regen_result: std::sync::Arc<std::sync::Mutex<Option<Result<(), String>>>>,
    /// Файл, що зараз перегенеровується.
    pub media_regen_target: Option<std::path::PathBuf>,
    /// Чи відкрите вікно кастомної перегенерації.
    pub media_regen_window_open: bool,
    /// Базові налаштування задачі (googler_key тощо) для перегенерації.
    pub media_regen_base_settings: Option<crate::queue::JobSettings>,
    /// Тип медіа для кастомної перегенерації.
    pub media_regen_media_type: String,
    /// Пріоритет провайдерів зображень для перегенерації.
    pub media_regen_image_priority: Vec<String>,
    /// Пріоритет провайдерів відео для перегенерації.
    pub media_regen_video_priority: Vec<String>,
    /// Кастомний промт для перегенерації (порожній = зчитати зі збереженого).
    pub media_regen_prompt: String,
    /// Помилка перегенерації для відображення.
    pub media_regen_error: Option<String>,
    /// ID задачі до якої належить медіафайл що перегенеровується.
    pub media_regen_job_id: u64,
    /// Назва задачі до якої належить медіафайл що перегенеровується.
    pub media_regen_job_name: String,
    /// Чи увімкнено етап "Озвучка" у пайплайні.
    pub pipeline_voiceover_enabled: bool,
    /// Чи увімкнено етап "Відеоряд" у пайплайні.
    pub pipeline_video_enabled: bool,
    /// Чи увімкнено етап "Субтитри" у пайплайні.
    pub pipeline_subtitles_enabled: bool,
    /// Чи увімкнено етап "Монтаж" у пайплайні.
    pub pipeline_editing_enabled: bool,
    /// Промт для моделі перекладу.
    pub translation_prompt: String,
    /// ID обраної моделі OpenRouter для перекладу.
    pub translation_model: String,
    /// ID обраної моделі OpenRouter.
    pub translation_model_openrouter: String,
    /// ID обраної моделі Claude.
    pub translation_model_claude: String,
    /// ID обраної моделі Gemini.
    pub translation_model_gemini: String,
    /// Рядок пошуку у дропдауні вибору моделі (ephemeral UI state).
    pub translation_model_search: String,
    /// Список моделей OpenRouter, завантажених у фоні.
    pub openrouter_models: std::sync::Arc<std::sync::Mutex<Option<Result<Vec<crate::gui::pipeline::translation::OpenRouterModel>, String>>>>,
    /// Прапорець завантаження моделей OpenRouter.
    pub openrouter_models_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Баланс OpenRouter для відображення у топбарі.
    pub openrouter_balance: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Баланс VoiceBot для відображення у топбарі.
    pub voicebot_balance: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Обраний сервіс для генерації відеоряду.
    pub video_service: String,
    /// Тип медіа для генерації: "image" або "video"
    pub video_media_type: String,
    /// Режим нарізання тексту: "paragraphs" | "sentences" | "char_limit" | "full"
    pub text_split_mode: String,
    /// Ліміт символів для режиму char_limit.
    pub text_split_char_limit: usize,
    /// Промт для генерації зображень відеоряду.
    pub video_prompt: String,
    /// Сервіс ЛЛМ для генерації промтів відеоряду.
    pub video_llm_service: String,
    /// Активна модель ЛЛМ для відео-промтів.
    pub video_llm_model: String,
    /// Модель OpenRouter для відео-промтів.
    pub video_llm_model_openrouter: String,
    /// Модель Claude для відео-промтів.
    pub video_llm_model_claude: String,
    /// Модель Gemini для відео-промтів.
    pub video_llm_model_gemini: String,
    /// Температура ЛЛМ для відео-промтів.
    pub video_llm_temperature: f32,
    /// Рядок пошуку у дропдауні вибору моделі ЛЛМ відеоряду.
    pub video_llm_model_search: String,
    /// Пріоритетний список провайдерів зображень Googler.
    pub googler_image_priority: Vec<String>,
    /// Пріоритетний список провайдерів відео Googler.
    pub googler_video_priority: Vec<String>,
    /// Температура моделі для перекладу (0.0 — 2.0).
    pub translation_temperature: f32,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code").
    pub translation_service: String,
    /// Чи відкрите вікно балансів.
    pub balance_window_open: bool,
    /// Чи відкрите вікно потоків.
    pub threads_window_open: bool,
    /// Шлях збереження для macOS.
    pub save_path_macos: String,
    /// Шлях збереження для Windows.
    pub save_path_windows: String,
    /// Черга задач пайплайну.
    pub jobs: Vec<crate::queue::PipelineJob>,
    /// Лічильник ID задач.
    pub job_counter: u64,
    /// Повідомлення про помилку валідації перед додаванням в чергу.
    pub queue_error: Option<String>,
    /// Обрана задача для перегляду її логів
    pub selected_job_logs: Option<(u64, String)>,
    /// Задача, для якої зараз відкрито вікно контролю перекладу
    pub selected_job_control: Option<u64>,
    /// Текстовий буфер для редагування перекладу під час контролю
    pub control_text_input: String,
    /// Чи відкрите вікно розширеної перегенерації
    pub control_regen_extended_open: bool,
    /// Одноразовий сервіс перекладу для перегенерації
    pub control_regen_service: String,
    /// Одноразова активна модель для перегенерації
    pub control_regen_model: String,
    /// Одноразова модель OpenRouter для перегенерації
    pub control_regen_model_openrouter: String,
    /// Одноразова модель Claude для перегенерації
    pub control_regen_model_claude: String,
    /// Одноразова модель Gemini для перегенерації
    pub control_regen_model_gemini: String,
    /// Рядок пошуку моделі для перегенерації
    pub control_regen_model_search: String,
    /// Одноразовий промт для перегенерації
    pub control_regen_prompt: String,
    /// Одноразова температура для перегенерації
    pub control_regen_temperature: f32,
    /// Результат фонової перегенерації (текст, вартість)
    pub control_regen_result: std::sync::Arc<std::sync::Mutex<Option<Result<(String, Option<f64>), String>>>>,
    /// Прапорець виконання перегенерації
    pub control_regen_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Помилка перегенерації
    pub control_regen_error: Option<String>,
    /// Задачі, для яких користувач вручну закрив вікно контролю (авто-відкриття їх пропускає).
    pub control_dismissed: std::collections::HashSet<u64>,
    /// Чи відкрите вікно введення назви задачі.
    pub job_name_dialog_open: bool,
    /// Поточний текст у полі введення назви задачі.
    pub job_name_input: String,
    /// Максимальна кількість потоків для OpenRouter.
    pub openrouter_max_threads: usize,
    /// Максимальна кількість потоків для Claude Code.
    pub claude_max_threads: usize,
    /// Максимальна кількість потоків для Gemini CLI.
    pub gemini_max_threads: usize,
    /// Чи відкрите вікно привітання.
    pub welcome_open: bool,
    /// Галочка "не показувати при наступному запуску" у вікні привітання.
    pub welcome_dont_show: bool,
    /// Стани перевірки CLI-інструментів для вікна привітання.
    pub tool_checks: crate::gui::welcome::ToolChecks,
    /// Сервіс, для якого очікується фонова перевірка CLI інструментів.
    pub pending_tool_check: Option<String>,
    /// Обраний голос для Edge TTS.
    pub edge_tts_voice: String,
    /// Темп для Edge TTS.
    pub edge_tts_rate: String,
    /// Тональність для Edge TTS.
    pub edge_tts_pitch: String,
    /// Гучність для Edge TTS.
    pub edge_tts_volume: String,
    /// Максимальна кількість потоків для Edge TTS.
    pub edge_tts_max_threads: usize,
    /// Максимальна кількість одночасних процесів FFmpeg.
    pub ffmpeg_max_threads: usize,
    /// Завантажені голоси Edge TTS.
    pub edge_tts_voices: std::sync::Arc<std::sync::Mutex<Option<Result<Vec<crate::api::edgetts::EdgeTTSVoice>, String>>>>,
    /// Прапорець завантаження голосів Edge TTS.
    pub edge_tts_loading_voices: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Показувати всі мови для Edge TTS.
    pub edge_tts_show_all_languages: bool,
    /// Максимальна кількість потоків зображень Googler.
    pub googler_image_max_threads: usize,
    /// Максимальна кількість потоків відео Googler.
    pub googler_video_max_threads: usize,
    /// Конвертувати аудіо в WAV після озвучки.
    pub voiceover_convert_to_wav: bool,
    /// Обраний сервіс для генерації субтитрів.
    pub subtitles_service: String,
    /// Мова розпізнавання для Whisper.
    pub whisper_language: String,
    /// Модель Whisper.
    pub whisper_model: String,
    /// Максимальна кількість символів у сегменті субтитрів (0 = без обмеження).
    pub whisper_max_line_width: usize,
    /// Стан завантаження ggml-моделі whisper.cpp у фоні.
    pub whisper_model_download: std::sync::Arc<std::sync::Mutex<crate::gui::welcome::BinaryDownload>>,
    /// Сервіс монтажу ("FFmpeg").
    pub montage_service: String,
    /// FPS для монтажу.
    pub montage_fps: u32,
    /// Пресет кодування FFmpeg.
    pub montage_preset: String,
    /// Бітрейт відео у МБ/с.
    pub montage_bitrate: u32,
    /// Тип переходу між кліпами ("none", "random" або назва xfade).
    pub montage_transition: String,
    /// Тривалість переходу в секундах.
    pub montage_transition_duration: f32,
    /// Сповіщення про успішне копіювання (текст, час копіювання).
    pub copied_toast: Option<(String, std::time::Instant)>,
    /// Чи увімкнене автоматичне прокручування логу донизу.
    pub auto_scroll_logs: bool,
    /// Кеш кадрів hover-анімації відео: path → список текстур кадрів.
    pub video_hover_frames: std::collections::HashMap<std::path::PathBuf, Vec<egui::TextureHandle>>,
    /// Стан hover-анімації відео: path → (поточний кадр, час останнього переходу).
    pub video_hover_state: std::collections::HashMap<std::path::PathBuf, (usize, std::time::Instant)>,
    /// Набір шляхів відео, для яких зараз витягуються hover-кадри.
    pub video_hover_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Результат фонового витягування hover-кадрів.
    pub video_hover_result: std::sync::Arc<std::sync::Mutex<Option<(std::path::PathBuf, Vec<egui::TextureHandle>)>>>,
    /// Кеш першого кадру відео як thumbnail: path → текстура.
    pub video_thumbnails: std::collections::HashMap<std::path::PathBuf, Option<egui::TextureHandle>>,
    /// Набір шляхів відео, для яких зараз витягується thumbnail.
    pub video_thumb_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Результат фонового витягування thumbnail.
    pub video_thumb_result: std::sync::Arc<std::sync::Mutex<Option<(std::path::PathBuf, Option<egui::TextureHandle>)>>>,
    /// Активний повноекранний відеоплеєр (якщо відео відкрите).
    pub video_player: Option<crate::gui::gallery::video_player::VideoPlayer>,
}

impl Default for VideoMakerApp {
    fn default() -> Self {
        let default_settings = AppSettings::default();
        Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme: AppTheme::Dark, // Сучасна темна тема за замовчуванням
            accent_color: egui::Color32::from_rgb(0, 122, 255), // Синій колір за замовчуванням
            pipeline_width: 450.0,
            language: Language::Uk,
            openrouter_key: String::new(),
            openrouter_status: None,
            template_name_input: String::new(),
            saved_templates: crate::gui::settings::storage::load_saved_templates(),
            template_status: None,
            voicebot_key: String::new(),
            voicebot_status: None,
            googler_key: String::new(),
            googler_status: None,
            googler_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            googler_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voiceover_provider: "Voice Bot".to_string(),
            voiceover_template_uuid: String::new(),
            voicebot_templates: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            voicebot_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pipeline_translation_enabled: true,
            pipeline_translation_control_enabled: false,
            pipeline_control_auto_open: false,
            pipeline_media_control_enabled: false,
            gallery_textures: std::collections::HashMap::new(),
            gallery_preview: None,
            gallery_anim_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            media_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            media_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            media_regen_target: None,
            media_regen_window_open: false,
            media_regen_base_settings: None,
            media_regen_media_type: "image".to_string(),
            media_regen_image_priority: vec!["flow_IMAGEN_3_5".to_string(), "flow_GEM_PIX_2".to_string(), "flow_NARWHAL".to_string(), "flower".to_string(), "grok".to_string(), "openai".to_string()],
            media_regen_video_priority: vec!["flow".to_string(), "flower".to_string(), "grok".to_string()],
            media_regen_prompt: String::new(),
            media_regen_error: None,
            media_regen_job_id: 0,
            media_regen_job_name: String::new(),
            pipeline_voiceover_enabled: true,
            pipeline_video_enabled: true,
            pipeline_subtitles_enabled: true,
            pipeline_editing_enabled: true,
            translation_prompt: String::new(),
            translation_model: String::new(),
            translation_model_openrouter: String::new(),
            translation_model_claude: "sonnet".to_string(),
            translation_model_gemini: "gemini-2.5-flash".to_string(),
            translation_model_search: String::new(),
            openrouter_models: std::sync::Arc::new(std::sync::Mutex::new(None)),
            openrouter_models_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            openrouter_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_service: "Googler".to_string(),
            video_media_type: "image".to_string(),
            text_split_mode: "paragraphs".to_string(),
            text_split_char_limit: 500,
            video_prompt: String::new(),
            video_llm_service: "None".to_string(),
            video_llm_model: String::new(),
            video_llm_model_openrouter: String::new(),
            video_llm_model_claude: "sonnet".to_string(),
            video_llm_model_gemini: "gemini-2.5-flash".to_string(),
            video_llm_temperature: 0.7,
            video_llm_model_search: String::new(),
            googler_image_priority: vec!["flow_IMAGEN_3_5".to_string(), "flow_GEM_PIX_2".to_string(), "flow_NARWHAL".to_string(), "flower".to_string(), "grok".to_string(), "openai".to_string()],
            googler_video_priority: vec!["flow".to_string(), "flower".to_string(), "grok".to_string()],
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            balance_window_open: false,
            threads_window_open: false,
            save_path_macos: String::new(),
            save_path_windows: String::new(),
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
            selected_job_control: None,
            control_text_input: String::new(),
            control_regen_extended_open: false,
            control_regen_service: String::new(),
            control_regen_model: String::new(),
            control_regen_model_openrouter: String::new(),
            control_regen_model_claude: "sonnet".to_string(),
            control_regen_model_gemini: "gemini-2.5-flash".to_string(),
            control_regen_model_search: String::new(),
            control_regen_prompt: String::new(),
            control_regen_temperature: 0.7,
            control_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            control_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            control_regen_error: None,
            control_dismissed: std::collections::HashSet::new(),
            job_name_dialog_open: false,
            job_name_input: String::new(),
            openrouter_max_threads: 5,
            claude_max_threads: 5,
            gemini_max_threads: 5,
            welcome_open: false,
            welcome_dont_show: false,
            tool_checks: crate::gui::welcome::ToolChecks::new(),
            pending_tool_check: None,
            edge_tts_voice: default_settings.edge_tts_voice.clone(),
            edge_tts_rate: default_settings.edge_tts_rate.clone(),
            edge_tts_pitch: default_settings.edge_tts_pitch.clone(),
            edge_tts_volume: default_settings.edge_tts_volume.clone(),
            edge_tts_max_threads: default_settings.edge_tts_max_threads,
            ffmpeg_max_threads: default_settings.ffmpeg_max_threads,
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: default_settings.googler_image_max_threads,
            googler_video_max_threads: default_settings.googler_video_max_threads,
            voiceover_convert_to_wav: false,
            subtitles_service: "Whisper".to_string(),
            whisper_language: "auto".to_string(),
            whisper_model: "base".to_string(),
            whisper_max_line_width: 42,
            whisper_model_download: std::sync::Arc::new(std::sync::Mutex::new(crate::gui::welcome::BinaryDownload::Idle)),
            montage_service: "FFmpeg".to_string(),
            montage_fps: 30,
            montage_preset: "medium".to_string(),
            montage_bitrate: 8,
            montage_transition: "none".to_string(),
            montage_transition_duration: 0.5,
            copied_toast: None,
            auto_scroll_logs: true,
            last_saved_settings: default_settings,
            video_hover_frames: std::collections::HashMap::new(),
            video_hover_state: std::collections::HashMap::new(),
            video_hover_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            video_hover_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_thumbnails: std::collections::HashMap::new(),
            video_thumb_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            video_thumb_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_player: None,
        }
    }
}


impl VideoMakerApp {
    /// Створює новий екземпляр додатку, завантажуючи збережені налаштування з диска.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Завантажуємо збережені налаштування користувача з файлу settings.json
        let saved = load_settings();
        
        // Конвертуємо назву теми (String) у тип AppTheme
        let theme = match saved.theme.as_str() {
            "Light" => AppTheme::Light,
            "Amoled" => AppTheme::Amoled,
            _ => AppTheme::Dark,
        };
        
        // Відновлюємо колір акценту з масиву [r, g, b, a]
        let accent_color = egui::Color32::from_rgba_unmultiplied(
            saved.accent_color[0],
            saved.accent_color[1],
            saved.accent_color[2],
            saved.accent_color[3],
        );

        let pipeline_width = saved.pipeline_width;

        // Конвертуємо назву мови (String) у тип Language
        let language = match saved.language.as_str() {
            "En" => Language::En,
            "Ru" => Language::Ru,
            _ => Language::Uk,
        };

        let openrouter_key = saved.openrouter_key.clone();
        let voicebot_key = saved.voicebot_key.clone();
        let googler_key = saved.googler_key.clone();
        let voiceover_provider = saved.voiceover_provider.clone();
        let voiceover_template_uuid = saved.voiceover_template_uuid.clone();
        let pipeline_translation_enabled = saved.pipeline_translation_enabled;
        let pipeline_translation_control_enabled = saved.pipeline_translation_control_enabled;
        let pipeline_control_auto_open = saved.pipeline_control_auto_open;
        let pipeline_media_control_enabled = saved.pipeline_media_control_enabled;
        let pipeline_voiceover_enabled = saved.pipeline_voiceover_enabled;
        let pipeline_video_enabled = saved.pipeline_video_enabled;
        let pipeline_subtitles_enabled = saved.pipeline_subtitles_enabled;
        let pipeline_editing_enabled = saved.pipeline_editing_enabled;
        let translation_prompt = saved.translation_prompt.clone();
        let translation_model = saved.translation_model.clone();
        let translation_service = saved.translation_service.clone();
        let mut translation_model_openrouter = saved.translation_model_openrouter.clone();
        let mut translation_model_claude = saved.translation_model_claude.clone();
        let mut translation_model_gemini = saved.translation_model_gemini.clone();

        // Зворотна сумісність: якщо завантажені окремі слоти порожні, але є загальне поле translation_model
        if translation_model_openrouter.is_empty() && translation_service == "OpenRouter" {
            translation_model_openrouter = translation_model.clone();
        }
        if translation_model_claude.is_empty() {
            translation_model_claude = if translation_service == "Claude Code" {
                translation_model.clone()
            } else {
                "sonnet".to_string()
            };
        }
        if translation_model_gemini.is_empty() {
            translation_model_gemini = if translation_service == "Gemini CLI" {
                translation_model.clone()
            } else {
                "gemini-2.5-flash".to_string()
            };
        }

        let video_service = saved.video_service.clone();
        let video_media_type = saved.video_media_type.clone();
        let text_split_mode = saved.text_split_mode.clone();
        let text_split_char_limit = saved.text_split_char_limit;
        let video_prompt = saved.video_prompt.clone();
        let video_llm_service = saved.video_llm_service.clone();
        let mut video_llm_model_openrouter = saved.video_llm_model_openrouter.clone();
        let video_llm_model_claude = saved.video_llm_model_claude.clone();
        let video_llm_model_gemini = saved.video_llm_model_gemini.clone();
        let video_llm_temperature = saved.video_llm_temperature;
        // Відновлюємо активну модель залежно від збереженого сервісу
        let video_llm_model = match video_llm_service.as_str() {
            "OpenRouter" => {
                if saved.video_llm_model.is_empty() {
                    video_llm_model_openrouter = saved.video_llm_model.clone();
                }
                saved.video_llm_model.clone()
            }
            "Claude Code" => video_llm_model_claude.clone(),
            "Gemini CLI"  => video_llm_model_gemini.clone(),
            _             => saved.video_llm_model.clone(),
        };
        let googler_image_priority = saved.googler_image_priority.clone();
        let googler_video_priority = saved.googler_video_priority.clone();
        let translation_temperature = saved.translation_temperature;
        let save_path_macos = saved.save_path_macos.clone();
        let save_path_windows = saved.save_path_windows.clone();
        let openrouter_max_threads = saved.openrouter_max_threads;
        let claude_max_threads = saved.claude_max_threads;
        let gemini_max_threads = saved.gemini_max_threads;
        let show_welcome = saved.show_welcome;

        // Налаштовуємо глобальний лімітер одночасних запитів OpenRouter
        crate::api::openrouter::OpenRouterLimiter::get().set_max_threads(openrouter_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Claude Code
        crate::api::claude::ClaudeLimiter::get().set_max_threads(claude_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Gemini CLI
        crate::api::gemini::GeminiLimiter::get().set_max_threads(gemini_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Edge TTS
        crate::api::edgetts::EdgeTTSLimiter::get().set_max_threads(saved.edge_tts_max_threads);
        // Налаштовуємо глобальний лімітер одночасних процесів FFmpeg
        crate::api::ffmpeg::FfmpegLimiter::get().set_max_threads(saved.ffmpeg_max_threads);

        let saved_templates = crate::gui::settings::storage::load_saved_templates();

        // Ініціалізуємо вікно привітання та одразу запускаємо фонові перевірки CLI
        let tool_checks = crate::gui::welcome::ToolChecks::new();
        if show_welcome {
            tool_checks.start(cc.egui_ctx.clone());
        }

        let openrouter_balance = std::sync::Arc::new(std::sync::Mutex::new(None));
        let voicebot_balance = std::sync::Arc::new(std::sync::Mutex::new(None));
        let googler_balance = std::sync::Arc::new(std::sync::Mutex::new(None));

        // Завантажуємо баланси у фоні при старті, якщо ключі вже збережені
        if !openrouter_key.is_empty() && openrouter_key.starts_with("sk-or-") {
            crate::api::openrouter::fetch_balance(
                openrouter_key.clone(),
                std::sync::Arc::clone(&openrouter_balance),
                cc.egui_ctx.clone(),
            );
        }
        if !voicebot_key.is_empty() {
            crate::api::voicebot::fetch_balance(
                voicebot_key.clone(),
                std::sync::Arc::clone(&voicebot_balance),
                cc.egui_ctx.clone(),
            );
        }
        if !googler_key.is_empty() {
            crate::api::googler::fetch_balance(
                googler_key.clone(),
                std::sync::Arc::clone(&googler_balance),
                cc.egui_ctx.clone(),
            );
        }

        Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme,
            accent_color,
            pipeline_width,
            language,
            openrouter_key,
            openrouter_status: None,
            template_name_input: saved.last_template.clone(),
            saved_templates,
            template_status: None,
            voicebot_key,
            voicebot_status: None,
            googler_key,
            googler_status: None,
            googler_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            googler_balance,
            voiceover_provider,
            voiceover_template_uuid,
            voicebot_templates: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            voicebot_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pipeline_translation_enabled,
            pipeline_translation_control_enabled,
            pipeline_control_auto_open,
            pipeline_media_control_enabled,
            gallery_textures: std::collections::HashMap::new(),
            gallery_preview: None,
            gallery_anim_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            media_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            media_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            media_regen_target: None,
            media_regen_window_open: false,
            media_regen_base_settings: None,
            media_regen_media_type: "image".to_string(),
            media_regen_image_priority: vec!["flow_IMAGEN_3_5".to_string(), "flow_GEM_PIX_2".to_string(), "flow_NARWHAL".to_string(), "flower".to_string(), "grok".to_string(), "openai".to_string()],
            media_regen_video_priority: vec!["flow".to_string(), "flower".to_string(), "grok".to_string()],
            media_regen_prompt: String::new(),
            media_regen_error: None,
            media_regen_job_id: 0,
            media_regen_job_name: String::new(),
            pipeline_voiceover_enabled,
            pipeline_video_enabled,
            pipeline_subtitles_enabled,
            pipeline_editing_enabled,
            translation_prompt,
            translation_model,
            translation_model_openrouter,
            translation_model_claude,
            translation_model_gemini,
            translation_model_search: String::new(),
            openrouter_models: std::sync::Arc::new(std::sync::Mutex::new(None)),
            openrouter_models_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            openrouter_balance,
            voicebot_balance,
            video_service,
            video_media_type,
            text_split_mode,
            text_split_char_limit,
            video_prompt,
            video_llm_service,
            video_llm_model,
            video_llm_model_openrouter,
            video_llm_model_claude,
            video_llm_model_gemini,
            video_llm_temperature,
            video_llm_model_search: String::new(),
            googler_image_priority,
            googler_video_priority,
            translation_temperature,
            translation_service,
            balance_window_open: false,
            threads_window_open: false,
            save_path_macos,
            save_path_windows,
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
            selected_job_control: None,
            control_text_input: String::new(),
            control_regen_extended_open: false,
            control_regen_service: String::new(),
            control_regen_model: String::new(),
            control_regen_model_openrouter: String::new(),
            control_regen_model_claude: "sonnet".to_string(),
            control_regen_model_gemini: "gemini-2.5-flash".to_string(),
            control_regen_model_search: String::new(),
            control_regen_prompt: String::new(),
            control_regen_temperature: 0.7,
            control_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            control_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            control_regen_error: None,
            control_dismissed: std::collections::HashSet::new(),
            job_name_dialog_open: false,
            job_name_input: String::new(),
            openrouter_max_threads,
            claude_max_threads,
            gemini_max_threads,
            welcome_open: show_welcome,
            welcome_dont_show: false,
            tool_checks,
            pending_tool_check: None,
            edge_tts_voice: saved.edge_tts_voice.clone(),
            edge_tts_rate: saved.edge_tts_rate.clone(),
            edge_tts_pitch: saved.edge_tts_pitch.clone(),
            edge_tts_volume: saved.edge_tts_volume.clone(),
            edge_tts_max_threads: saved.edge_tts_max_threads,
            ffmpeg_max_threads: saved.ffmpeg_max_threads,
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: saved.googler_image_max_threads,
            googler_video_max_threads: saved.googler_video_max_threads,
            voiceover_convert_to_wav: saved.voiceover_convert_to_wav,
            subtitles_service: saved.subtitles_service.clone(),
            whisper_language: saved.whisper_language.clone(),
            whisper_model: saved.whisper_model.clone(),
            whisper_max_line_width: saved.whisper_max_line_width,
            whisper_model_download: std::sync::Arc::new(std::sync::Mutex::new(crate::gui::welcome::BinaryDownload::Idle)),
            montage_service: saved.montage_service.clone(),
            montage_fps: saved.montage_fps,
            montage_preset: saved.montage_preset.clone(),
            montage_bitrate: saved.montage_bitrate,
            montage_transition: saved.montage_transition.clone(),
            montage_transition_duration: saved.montage_transition_duration,
            copied_toast: None,
            auto_scroll_logs: true,
            last_saved_settings: saved,
            video_hover_frames: std::collections::HashMap::new(),
            video_hover_state: std::collections::HashMap::new(),
            video_hover_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            video_hover_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_thumbnails: std::collections::HashMap::new(),
            video_thumb_loading: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            video_thumb_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_player: None,
        }
    }

    /// Малює вкладку системних логів роботи додатку.
    fn draw_logs_tab(&mut self, ui: &mut egui::Ui) {
        crate::gui::logs::draw_logs_tab(
            ui,
            self.language,
            &mut self.auto_scroll_logs,
            &mut self.copied_toast,
        );
    }
}

impl eframe::App for VideoMakerApp {
    /// Викликається кожного разу, коли інтерфейс потребує оновлення та перемальовування.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Перевірка статусів CLI для фонової перевірки
        if let Some(ref service) = self.pending_tool_check {
            let gemini = self.tool_checks.gemini.lock().unwrap().clone();
            let claude = self.tool_checks.claude.lock().unwrap().clone();

            let mut check_done = false;
            let mut needs_install = false;

            if service == "Gemini CLI" {
                match &gemini {
                    crate::gui::welcome::ToolStatus::Checking => {
                        // Перевірка ще триває
                    }
                    crate::gui::welcome::ToolStatus::NotInstalled => {
                        needs_install = true;
                        check_done = true;
                    }
                    _ => {
                        check_done = true;
                    }
                }
            } else if service == "Claude Code" {
                match &claude {
                    crate::gui::welcome::ToolStatus::Checking => {
                        // Перевірка ще триває
                    }
                    crate::gui::welcome::ToolStatus::NotInstalled => {
                        needs_install = true;
                        check_done = true;
                    }
                    _ => {
                        check_done = true;
                    }
                }
            }

            if check_done {
                if needs_install {
                    self.welcome_open = true;
                }
                self.pending_tool_check = None;
            }
        }

        // Динамічно застосовуємо обрану тему оформлення та акцентний колір до поточного контексту
        crate::theme::apply_theme(ctx, self.theme, self.accent_color);

        // Вікно привітання — відображається при першому запуску
        if self.welcome_open {
            let closed = crate::gui::welcome::draw_welcome_dialog(
                ctx,
                &mut self.welcome_open,
                &mut self.welcome_dont_show,
                &self.tool_checks,
                self.language,
            );
            // Якщо щойно натиснуто "Закрити" і стоїть галочка — зберігаємо show_welcome=false
            if closed && self.welcome_dont_show {
                let mut new_settings = self.last_saved_settings.clone();
                new_settings.show_welcome = false;
                crate::gui::settings::storage::save_settings(&new_settings);
                self.last_saved_settings = new_settings;
            }
        }

        // Верхня панель для навігації між вкладками
        crate::gui::topbar::draw_navigation_bar(
            ctx,
            &mut self.active_tab,
            &self.jobs,
            self.language,
            &self.openrouter_balance,
            &self.voicebot_balance,
            &self.googler_balance,
            &mut self.balance_window_open,
        );

        // Плаваюче вікно з детальними балансами
        crate::gui::topbar::draw_balance_window(
            ctx,
            &mut self.balance_window_open,
            self.language,
            &self.openrouter_key,
            &self.openrouter_balance,
            &self.voicebot_key,
            &self.voicebot_balance,
            &self.googler_key,
            &self.googler_balance,
        );

        crate::gui::topbar::draw_threads_window(
            ctx,
            &mut self.threads_window_open,
            self.language,
            &mut self.openrouter_max_threads,
            &mut self.claude_max_threads,
            &mut self.gemini_max_threads,
            &self.voicebot_balance,
            &mut self.edge_tts_max_threads,
            &self.googler_balance,
            &mut self.googler_image_max_threads,
            &mut self.googler_video_max_threads,
            &mut self.ffmpeg_max_threads,
        );

        // Нижній рядок статусу потоків (реєструємо ДО SidePanel, щоб займав повну ширину)
        crate::gui::topbar::draw_status_bar(
            ctx,
            self.openrouter_max_threads,
            self.claude_max_threads,
            self.gemini_max_threads,
            self.edge_tts_max_threads,
            self.ffmpeg_max_threads,
            self.googler_image_max_threads,
            self.googler_video_max_threads,
            &self.googler_balance,
            &mut self.threads_window_open,
        );

        // Відображаємо бічну панель пайплайну ТІЛЬКИ на вкладці "Основна"
        if self.active_tab == Tab::Main {
            let prev_translation_service = self.translation_service.clone();

            // default_width передається лише як початкове значення при першому запуску.
            // egui::Memory зберігає ширину між кадрами сам — нічого читати назад не потрібно.
            let side_frame = egui::Frame::side_top_panel(ctx.style().as_ref())
                .inner_margin(egui::Margin::same(0.0));
            egui::SidePanel::right("pipeline_panel")
                .frame(side_frame)
                .default_width(self.pipeline_width)
                .width_range(100.0..=750.0)
                .resizable(true)
                .show(ctx, |ui| {
                    gui::pipeline::draw_pipeline_panel(
                        ui,
                        self.language,
                        &mut self.openrouter_key,
                        &mut self.openrouter_status,
                        &self.openrouter_balance,
                        &mut self.voicebot_key,
                        &mut self.voicebot_status,
                        &self.voicebot_test_result,
                        &self.voicebot_balance,
                        &mut self.googler_key,
                        &mut self.googler_status,
                        &self.googler_test_result,
                        &self.googler_balance,
                        &mut self.voiceover_provider,
                        &mut self.voiceover_template_uuid,
                        &self.voicebot_templates,
                        &self.voicebot_loading,
                        &mut self.edge_tts_voice,
                        &mut self.edge_tts_rate,
                        &mut self.edge_tts_pitch,
                        &mut self.edge_tts_volume,
                        &self.edge_tts_voices,
                        &self.edge_tts_loading_voices,
                        &mut self.edge_tts_show_all_languages,
                        &mut self.template_name_input,
                        &mut self.saved_templates,
                        &mut self.template_status,
                        &mut self.pipeline_translation_enabled,
                        &mut self.pipeline_translation_control_enabled,
                        &mut self.pipeline_control_auto_open,
                        &mut self.pipeline_media_control_enabled,
                        &mut self.pipeline_voiceover_enabled,
                        &mut self.pipeline_video_enabled,
                        &mut self.pipeline_subtitles_enabled,
                        &mut self.pipeline_editing_enabled,
                        &mut self.translation_prompt,
                        &mut self.translation_model,
                        &mut self.translation_model_openrouter,
                        &mut self.translation_model_claude,
                        &mut self.translation_model_gemini,
                        &mut self.translation_model_search,
                        &self.openrouter_models,
                        &self.openrouter_models_loading,
                        &mut self.video_service,
                        &mut self.video_media_type,
                        &mut self.text_split_mode,
                        &mut self.text_split_char_limit,
                        &mut self.video_prompt,
                        &mut self.video_llm_service,
                        &mut self.video_llm_model,
                        &mut self.video_llm_model_openrouter,
                        &mut self.video_llm_model_claude,
                        &mut self.video_llm_model_gemini,
                        &mut self.video_llm_temperature,
                        &mut self.video_llm_model_search,
                        &mut self.translation_temperature,
                        &mut self.translation_service,
                        &mut self.save_path_macos,
                        &mut self.save_path_windows,
                        &mut self.googler_image_max_threads,
                        &mut self.googler_video_max_threads,
                        &mut self.voiceover_convert_to_wav,
                        &mut self.googler_image_priority,
                        &mut self.googler_video_priority,
                        &mut self.subtitles_service,
                        &mut self.whisper_language,
                        &mut self.whisper_model,
                        &mut self.whisper_max_line_width,
                        &self.whisper_model_download,
                        &mut self.montage_service,
                        &mut self.montage_fps,
                        &mut self.montage_preset,
                        &mut self.montage_bitrate,
                        &mut self.montage_transition,
                        &mut self.montage_transition_duration,
                        &self.text_input,
                        &mut self.jobs,
                        &mut self.job_counter,
                        &mut self.queue_error,
                        &mut self.job_name_dialog_open,
                        &mut self.job_name_input,
                    );
                });

            if self.translation_service != prev_translation_service {
                if self.translation_service == "Gemini CLI" || self.translation_service == "Claude Code" {
                    self.tool_checks.start(ctx.clone());
                    self.pending_tool_check = Some(self.translation_service.clone());
                } else {
                    self.pending_tool_check = None;
                }
            }
        }

        // Нижня панель черги задач (тільки якщо є задачі і ми не на Gallery)
        if !self.jobs.is_empty() && self.active_tab != Tab::Gallery {
            egui::TopBottomPanel::bottom("queue_panel")
                .min_height(140.0)
                .default_height(160.0)
                .max_height(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    crate::gui::queue::draw_queue_panel(
                        ui,
                        self.language,
                        &mut self.jobs,
                        &mut self.selected_job_logs,
                        &mut self.selected_job_control,
                        &mut self.control_text_input,
                        &self.whisper_model_download,
                        &mut self.active_tab,
                    );
                });
        }


        // Конфігуруємо фрейм для центральної панелі.
        // Для редактора (Main) прибираємо відступи (margin), щоб поле було на всю висоту та ширину.
        // Для вкладки налаштувань (Settings) залишаємо стандартні відступи.
        let frame = if self.active_tab == Tab::Main {
            egui::Frame::none()
                .fill(ctx.style().visuals.panel_fill) // Насичуємо фоновим кольором поточної теми
                .inner_margin(egui::Margin::same(0.0))
        } else {
            egui::Frame::central_panel(ctx.style().as_ref())
                .stroke(egui::Stroke::NONE)
        };

        // Central Panel
        let mut regen_action: Option<crate::gui::gallery::RegenAction> = None;
        let mut animate_all = false;
        let mut hover_extract_request: Option<std::path::PathBuf> = None;
        let mut thumb_requests: Vec<std::path::PathBuf> = Vec::new();
        let hover_loading_snapshot = self.video_hover_loading.lock().unwrap().clone();
        let thumb_loading_snapshot = self.video_thumb_loading.lock().unwrap().clone();
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Main => {
                        gui::editor::draw_editor(ui, &mut self.text_input, self.language, self.text_split_char_limit);
                    }
                    Tab::Gallery => {
                        crate::gui::gallery::draw_gallery_tab(
                            ui, self.language, &self.jobs,
                            &mut self.gallery_textures,
                            &mut self.gallery_preview,
                            &self.media_regen_loading,
                            &self.media_regen_target,
                            &mut regen_action,
                            &self.gallery_anim_loading,
                            &mut animate_all,
                            &self.video_hover_frames,
                            &mut self.video_hover_state,
                            &hover_loading_snapshot,
                            &mut hover_extract_request,
                            &self.video_thumbnails,
                            &thumb_loading_snapshot,
                            &mut thumb_requests,
                        );
                    }
                    Tab::Settings => {
                        let welcome_changed = gui::settings::draw_settings(
                            ui,
                            &mut self.theme,
                            &mut self.accent_color,
                            &mut self.language,
                            &mut self.last_saved_settings.show_welcome,
                        );
                        if welcome_changed {
                            let new_settings = self.last_saved_settings.clone();
                            crate::gui::settings::storage::save_settings(&new_settings);
                        }
                    }
                    Tab::Logs => {
                        self.draw_logs_tab(ui);
                    }
                }
            });

        // Спливаюче вікно з логами обраної задачі
        // Обробка кнопок перегенерації з галереї
        if let Some((file, settings, is_custom, job_id, job_name)) = regen_action {
            if is_custom {
                self.media_regen_target = Some(file.clone());
                self.media_regen_media_type = settings.video_media_type.clone();
                self.media_regen_image_priority = settings.googler_image_priority.clone();
                self.media_regen_video_priority = settings.googler_video_priority.clone();
                self.media_regen_prompt = crate::core::pipeline::read_prompt_for_file(&file);
                self.media_regen_base_settings = Some(settings);
                self.media_regen_job_id = job_id;
                self.media_regen_job_name = job_name;
                self.media_regen_error = None;
                self.media_regen_window_open = true;
            } else {
                let priority = if settings.video_media_type == "video" {
                    settings.googler_video_priority.clone()
                } else {
                    settings.googler_image_priority.clone()
                };
                self.media_regen_target = Some(file.clone());
                self.media_regen_error = None;
                self.gallery_textures.remove(&file);
                crate::core::pipeline::regenerate_single_media(
                    file,
                    settings.video_media_type.clone(),
                    priority,
                    settings.googler_key.clone(),
                    None,
                    job_id,
                    job_name,
                    ctx.clone(),
                    std::sync::Arc::clone(&self.media_regen_result),
                    std::sync::Arc::clone(&self.media_regen_loading),
                );
            }
        }

        // Очищення текстур для видалених файлів (після анімації .jpg → .mp4)
        self.gallery_textures.retain(|path, _| path.exists());

        // Запуск hover-витягування кадрів, якщо галерея це запитала
        if let Some(path) = hover_extract_request {
            crate::gui::gallery::video_player::start_hover_extraction(
                path,
                ctx.clone(),
                std::sync::Arc::clone(&self.video_hover_loading),
                std::sync::Arc::clone(&self.video_hover_result),
            );
        }

        // Запуск thumbnail-витягування для нових відео
        for path in thumb_requests {
            if !self.video_thumbnails.contains_key(&path) {
                self.video_thumbnails.insert(path.clone(), None); // Резервуємо місце
                crate::gui::gallery::video_player::start_thumbnail_extraction(
                    path,
                    ctx.clone(),
                    std::sync::Arc::clone(&self.video_thumb_loading),
                    std::sync::Arc::clone(&self.video_thumb_result),
                );
            }
        }

        // Обробка результату hover-витягування
        if let Some((path, frames)) = self.video_hover_result.lock().unwrap().take() {
            self.video_hover_frames.insert(path, frames);
        }

        // Обробка результату thumbnail-витягування
        if let Some((path, tex)) = self.video_thumb_result.lock().unwrap().take() {
            self.video_thumbnails.insert(path, tex);
        }

        // Обробка кнопки "Анімувати все"
        if animate_all {
            let anim_loading = std::sync::Arc::clone(&self.gallery_anim_loading);
            for job in &self.jobs {
                let media_dir = std::path::Path::new(&job.settings.save_path).join("media");
                if !media_dir.exists() { continue; }
                if let Ok(entries) = std::fs::read_dir(&media_dir) {
                    let images: Vec<_> = entries
                        .filter_map(Result::ok)
                        .filter(|e| {
                            matches!(
                                e.path().extension().and_then(|x| x.to_str()),
                                Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
                            )
                        })
                        .map(|e| e.path())
                        .collect();
                    for img_path in images {
                        if anim_loading.lock().unwrap().contains(&img_path) { continue; }
                        self.gallery_textures.remove(&img_path);
                        crate::core::pipeline::animate_single_image(
                            img_path,
                            self.googler_video_priority.clone(),
                            self.googler_key.clone(),
                            job.id,
                            job.name.clone(),
                            ctx.clone(),
                            std::sync::Arc::clone(&anim_loading),
                        );
                    }
                }
            }
        }

        // Обробка результату перегенерації
        {
            let result = self.media_regen_result.lock().unwrap().take();
            if let Some(outcome) = result {
                match outcome {
                    Ok(()) => {
                        // Скидаємо кеш щоб нове зображення підтягнулось
                        if let Some(ref path) = self.media_regen_target {
                            self.gallery_textures.remove(path);
                        }
                        self.media_regen_target = None;
                        self.media_regen_error = None;
                    }
                    Err(e) => {
                        self.media_regen_error = Some(e);
                        self.media_regen_target = None;
                    }
                }
            }
        }

        // Вікно кастомної перегенерації
        crate::gui::gallery::draw_media_regen_window(
            ctx,
            self.language,
            &mut self.media_regen_window_open,
            &self.media_regen_target,
            &mut self.media_regen_media_type,
            &mut self.media_regen_image_priority,
            &mut self.media_regen_video_priority,
            &mut self.media_regen_prompt,
            &self.media_regen_loading,
            &self.media_regen_base_settings,
            &mut self.media_regen_error,
            self.media_regen_job_id,
            &self.media_regen_job_name,
            &mut self.gallery_textures,
            &self.media_regen_result,
        );

        if let Some((job_id, job_name)) = self.selected_job_logs.clone() {
            if !crate::gui::logs::draw_job_logs_window(
                ctx,
                self.language,
                job_id,
                &job_name,
                &mut self.auto_scroll_logs,
                &mut self.copied_toast,
            ) {
                self.selected_job_logs = None;
            }
        }

        // Авто-відкриття вікна контролю коли задача переходить в AwaitingControl
        if self.pipeline_control_auto_open && self.selected_job_control.is_none() {
            if let Some(job) = self.jobs.iter().find(|j| {
                !self.control_dismissed.contains(&j.id)
                    && *j.status.lock().unwrap() == crate::queue::JobStatus::AwaitingControl
            }) {
                let job_id = job.id;
                let translated_text = job.translated_text.lock().unwrap().clone();
                self.selected_job_control = Some(job_id);
                self.control_text_input = translated_text.unwrap_or_default();
            }
        }

        // Авто-перехід на вкладку Галерея коли задача переходить в AwaitingMediaControl
        if self.jobs.iter().any(|j| {
            *j.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMediaControl
        }) {
            self.active_tab = Tab::Gallery;
        }

        // Спливаюче вікно контролю перекладу
        crate::gui::pipeline::translation_control::draw_translation_control_window(
            ctx,
            self.language,
            &self.jobs,
            &mut self.selected_job_control,
            &mut self.control_text_input,
            &mut self.control_regen_extended_open,
            &mut self.control_regen_service,
            &mut self.control_regen_model,
            &mut self.control_regen_model_openrouter,
            &mut self.control_regen_model_claude,
            &mut self.control_regen_model_gemini,
            &mut self.control_regen_model_search,
            &mut self.control_regen_prompt,
            &mut self.control_regen_temperature,
            &self.control_regen_result,
            &self.control_regen_loading,
            &mut self.control_regen_error,
            &mut self.control_dismissed,
            &self.openrouter_models,
            &self.openrouter_models_loading,
        );

        // АВТОЗБЕРЕЖЕННЯ:
        // Перевіряємо, чи користувач наразі не перетягує панель (миша відпущена).
        // Це запобігає надмірному навантаженню на диск та гарантує запис файлу лише після відпускання миші.
        let is_pointer_down = ctx.input(|i| i.pointer.any_down());
        
        if !is_pointer_down {
            let current_theme_str = match self.theme {
                AppTheme::Light => "Light".to_string(),
                AppTheme::Dark => "Dark".to_string(),
                AppTheme::Amoled => "Amoled".to_string(),
            };
            let current_color_arr = self.accent_color.to_array();
            let current_language_str = match self.language {
                Language::En => "En".to_string(),
                Language::Ru => "Ru".to_string(),
                _ => "Uk".to_string(),
            };

            // Перевіряємо зміни значень (з дельтою для ширини панелі)
            if current_theme_str != self.last_saved_settings.theme
                || current_color_arr != self.last_saved_settings.accent_color
                || current_language_str != self.last_saved_settings.language
                || self.openrouter_key != self.last_saved_settings.openrouter_key
                || self.voicebot_key != self.last_saved_settings.voicebot_key
                || self.voiceover_provider != self.last_saved_settings.voiceover_provider
                || self.voiceover_template_uuid != self.last_saved_settings.voiceover_template_uuid
                || self.template_name_input != self.last_saved_settings.last_template
                || self.pipeline_translation_enabled != self.last_saved_settings.pipeline_translation_enabled
                || self.pipeline_translation_control_enabled != self.last_saved_settings.pipeline_translation_control_enabled
                || self.pipeline_control_auto_open != self.last_saved_settings.pipeline_control_auto_open
                || self.pipeline_media_control_enabled != self.last_saved_settings.pipeline_media_control_enabled
                || self.pipeline_voiceover_enabled != self.last_saved_settings.pipeline_voiceover_enabled
                || self.pipeline_video_enabled != self.last_saved_settings.pipeline_video_enabled
                || self.pipeline_subtitles_enabled != self.last_saved_settings.pipeline_subtitles_enabled
                || self.pipeline_editing_enabled != self.last_saved_settings.pipeline_editing_enabled
                || self.translation_prompt != self.last_saved_settings.translation_prompt
                || self.translation_model != self.last_saved_settings.translation_model
                || self.translation_model_openrouter != self.last_saved_settings.translation_model_openrouter
                || self.translation_model_claude != self.last_saved_settings.translation_model_claude
                || self.translation_model_gemini != self.last_saved_settings.translation_model_gemini
                || self.googler_key != self.last_saved_settings.googler_key
                || self.video_service != self.last_saved_settings.video_service
                || self.video_media_type != self.last_saved_settings.video_media_type
                || self.video_prompt != self.last_saved_settings.video_prompt
                || self.video_llm_service != self.last_saved_settings.video_llm_service
                || self.video_llm_model != self.last_saved_settings.video_llm_model
                || self.video_llm_model_openrouter != self.last_saved_settings.video_llm_model_openrouter
                || self.video_llm_model_claude != self.last_saved_settings.video_llm_model_claude
                || self.video_llm_model_gemini != self.last_saved_settings.video_llm_model_gemini
                || self.video_llm_temperature != self.last_saved_settings.video_llm_temperature
                || self.text_split_mode != self.last_saved_settings.text_split_mode
                || self.text_split_char_limit != self.last_saved_settings.text_split_char_limit
                || self.translation_service != self.last_saved_settings.translation_service
                || self.save_path_macos != self.last_saved_settings.save_path_macos
                || self.save_path_windows != self.last_saved_settings.save_path_windows
                || self.openrouter_max_threads != self.last_saved_settings.openrouter_max_threads
                || self.claude_max_threads != self.last_saved_settings.claude_max_threads
                || self.gemini_max_threads != self.last_saved_settings.gemini_max_threads
                || self.edge_tts_voice != self.last_saved_settings.edge_tts_voice
                || self.edge_tts_rate != self.last_saved_settings.edge_tts_rate
                || self.edge_tts_pitch != self.last_saved_settings.edge_tts_pitch
                || self.edge_tts_volume != self.last_saved_settings.edge_tts_volume
                || self.edge_tts_max_threads != self.last_saved_settings.edge_tts_max_threads
                || self.ffmpeg_max_threads != self.last_saved_settings.ffmpeg_max_threads
                || self.googler_image_max_threads != self.last_saved_settings.googler_image_max_threads
                || self.googler_video_max_threads != self.last_saved_settings.googler_video_max_threads
                || self.voiceover_convert_to_wav != self.last_saved_settings.voiceover_convert_to_wav
                || self.googler_image_priority != self.last_saved_settings.googler_image_priority
                || self.googler_video_priority != self.last_saved_settings.googler_video_priority
                || self.translation_temperature != self.last_saved_settings.translation_temperature
                || self.subtitles_service != self.last_saved_settings.subtitles_service
                || self.whisper_language != self.last_saved_settings.whisper_language
                || self.whisper_model != self.last_saved_settings.whisper_model
                || self.whisper_max_line_width != self.last_saved_settings.whisper_max_line_width
                || self.montage_service != self.last_saved_settings.montage_service
                || self.montage_fps != self.last_saved_settings.montage_fps
                || self.montage_preset != self.last_saved_settings.montage_preset
                || self.montage_bitrate != self.last_saved_settings.montage_bitrate
                || self.montage_transition != self.last_saved_settings.montage_transition
                || self.montage_transition_duration != self.last_saved_settings.montage_transition_duration
            {
                let new_settings = AppSettings {
                    theme: current_theme_str,
                    accent_color: current_color_arr,
                    pipeline_width: self.pipeline_width,
                    language: current_language_str,
                    openrouter_key: self.openrouter_key.clone(),
                    voicebot_key: self.voicebot_key.clone(),
                    googler_key: self.googler_key.clone(),
                    voiceover_provider: self.voiceover_provider.clone(),
                    voiceover_template_uuid: self.voiceover_template_uuid.clone(),
                    last_template: self.template_name_input.clone(),
                    pipeline_translation_enabled: self.pipeline_translation_enabled,
                    pipeline_translation_control_enabled: self.pipeline_translation_control_enabled,
                    pipeline_control_auto_open: self.pipeline_control_auto_open,
                    pipeline_media_control_enabled: self.pipeline_media_control_enabled,
                    pipeline_voiceover_enabled: self.pipeline_voiceover_enabled,
                    pipeline_video_enabled: self.pipeline_video_enabled,
                    pipeline_subtitles_enabled: self.pipeline_subtitles_enabled,
                    pipeline_editing_enabled: self.pipeline_editing_enabled,
                    translation_prompt: self.translation_prompt.clone(),
                    translation_model: self.translation_model.clone(),
                    translation_model_openrouter: self.translation_model_openrouter.clone(),
                    translation_model_claude: self.translation_model_claude.clone(),
                    translation_model_gemini: self.translation_model_gemini.clone(),
                    video_service: self.video_service.clone(),
                    video_media_type: self.video_media_type.clone(),
                    text_split_mode: self.text_split_mode.clone(),
                    text_split_char_limit: self.text_split_char_limit,
                    video_prompt: self.video_prompt.clone(),
                    video_llm_service: self.video_llm_service.clone(),
                    video_llm_model: self.video_llm_model.clone(),
                    video_llm_model_openrouter: self.video_llm_model_openrouter.clone(),
                    video_llm_model_claude: self.video_llm_model_claude.clone(),
                    video_llm_model_gemini: self.video_llm_model_gemini.clone(),
                    video_llm_temperature: self.video_llm_temperature,
                    translation_temperature: self.translation_temperature,
                    translation_service: self.translation_service.clone(),
                    save_path_macos: self.save_path_macos.clone(),
                    save_path_windows: self.save_path_windows.clone(),
                    save_path: String::new(),
                    openrouter_max_threads: self.openrouter_max_threads,
                    claude_max_threads: self.claude_max_threads,
                    gemini_max_threads: self.gemini_max_threads,
                    edge_tts_voice: self.edge_tts_voice.clone(),
                    edge_tts_rate: self.edge_tts_rate.clone(),
                    edge_tts_pitch: self.edge_tts_pitch.clone(),
                    edge_tts_volume: self.edge_tts_volume.clone(),
                    edge_tts_max_threads: self.edge_tts_max_threads,
                    ffmpeg_max_threads: self.ffmpeg_max_threads,
                    googler_image_max_threads: self.googler_image_max_threads,
                    googler_video_max_threads: self.googler_video_max_threads,
                    voiceover_convert_to_wav: self.voiceover_convert_to_wav,
                    googler_image_priority: self.googler_image_priority.clone(),
                    googler_video_priority: self.googler_video_priority.clone(),
                    subtitles_service: self.subtitles_service.clone(),
                    whisper_language: self.whisper_language.clone(),
                    whisper_model: self.whisper_model.clone(),
                    whisper_max_line_width: self.whisper_max_line_width,
                    montage_service: self.montage_service.clone(),
                    montage_fps: self.montage_fps,
                    montage_preset: self.montage_preset.clone(),
                    montage_bitrate: self.montage_bitrate,
                    montage_transition: self.montage_transition.clone(),
                    montage_transition_duration: self.montage_transition_duration,
                    show_welcome: self.last_saved_settings.show_welcome,
                };
                
                // Зберігаємо оновлені налаштування у файл JSON на диску
                save_settings(&new_settings);
                
                // Оновлюємо копію останніх збережених параметрів у пам'яті
                self.last_saved_settings = new_settings;
            }
        }

        // Повноекранний перегляд медіафайлу з галереї
        if let Some(path) = self.gallery_preview.clone() {
            let is_video = matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("mp4") | Some("webm") | Some("mov")
            );

            if is_video {
                // Якщо плеєр для іншого файлу — скидаємо
                if self.video_player.as_ref().map_or(false, |p| p.path != path) {
                    self.video_player = None;
                }

                // Якщо плеєра ще немає — створюємо і запускаємо streaming
                if self.video_player.is_none() {
                    let player = crate::gui::gallery::video_player::VideoPlayer::new(path.clone(), 10.0);
                    crate::gui::gallery::video_player::start_fullscreen_extraction(
                        &player, path.clone(), ctx.clone(),
                    );
                    self.video_player = Some(player);
                }

                // Дренуємо нові кадри та відображаємо
                if let Some(ref mut player) = self.video_player {
                    player.drain_pending();
                    let keep_open = crate::gui::gallery::video_player::draw_video_player(ctx, player);
                    if !keep_open {
                        self.gallery_preview = None;
                        self.video_player = None;
                    }
                }
            } else {
                // Зображення — існуючий перегляд
                let tex = self.gallery_textures.get(&path).and_then(|t| t.as_ref()).cloned();
                if let Some(texture) = tex {
                    let regen_loading_this = *self.media_regen_loading.lock().unwrap()
                        && self.media_regen_target.as_deref() == Some(path.as_path());
                    let (keep_open, regen_kind) = crate::gui::gallery::draw_image_preview(ctx, &texture, regen_loading_this);
                    if !keep_open {
                        self.gallery_preview = None;
                    }
                    if let Some(is_custom) = regen_kind {
                        let job_info = self.jobs.iter().find(|j| {
                            let media_dir = std::path::Path::new(&j.settings.save_path).join("media");
                            path.starts_with(&media_dir)
                        }).map(|j| (j.id, j.name.clone(), j.settings.clone()));

                        if let Some((job_id, job_name, settings)) = job_info {
                            if is_custom {
                                self.media_regen_target = Some(path.clone());
                                self.media_regen_media_type = settings.video_media_type.clone();
                                self.media_regen_image_priority = settings.googler_image_priority.clone();
                                self.media_regen_video_priority = settings.googler_video_priority.clone();
                                self.media_regen_prompt = crate::core::pipeline::read_prompt_for_file(&path);
                                self.media_regen_base_settings = Some(settings);
                                self.media_regen_job_id = job_id;
                                self.media_regen_job_name = job_name;
                                self.media_regen_error = None;
                                self.media_regen_window_open = true;
                            } else {
                                let priority = if settings.video_media_type == "video" {
                                    settings.googler_video_priority.clone()
                                } else {
                                    settings.googler_image_priority.clone()
                                };
                                self.media_regen_target = Some(path.clone());
                                self.media_regen_error = None;
                                self.gallery_textures.remove(&path);
                                crate::core::pipeline::regenerate_single_media(
                                    path,
                                    settings.video_media_type.clone(),
                                    priority,
                                    settings.googler_key.clone(),
                                    None,
                                    job_id,
                                    job_name,
                                    ctx.clone(),
                                    std::sync::Arc::clone(&self.media_regen_result),
                                    std::sync::Arc::clone(&self.media_regen_loading),
                                );
                            }
                        }
                    }
                } else {
                    self.gallery_preview = None;
                }
            }
        }

        // Відображаємо спливаюче сповіщення (Toast) про копіювання
        if let Some((_, instant)) = &self.copied_toast {
            if instant.elapsed().as_secs_f32() < 2.0 {
                egui::Area::new(egui::Id::new("copied_toast"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-20.0, -20.0])
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_black_alpha(220))
                            .rounding(8.0)
                            .stroke(egui::Stroke::new(1.0, self.accent_color))
                            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(translate(self.language, "logs_copied_toast"))
                                        .strong()
                                        .color(egui::Color32::WHITE)
                                        .size(13.0)
                                );
                            });
                    });
                
                // Просимо eframe перемалювати екран, щоб таймер оновлювався плавно
                ctx.request_repaint();
            }
        }
    }
}
