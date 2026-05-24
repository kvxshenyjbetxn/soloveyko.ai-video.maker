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
    /// Пріоритетний список провайдерів зображень Googler.
    pub googler_image_priority: Vec<String>,
    /// Пріоритетний список провайдерів відео Googler.
    pub googler_video_priority: Vec<String>,
    /// Температура моделі для перекладу (0.0 — 2.0).
    pub translation_temperature: f32,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code").
    pub translation_service: String,
    /// Чи відкрите вікно детальних балансів.
    pub balance_window_open: bool,
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
            googler_image_priority: vec!["flow_IMAGEN_3_5".to_string(), "flow_GEM_PIX_2".to_string(), "flow_NARWHAL".to_string(), "flower".to_string(), "grok".to_string(), "openai".to_string()],
            googler_video_priority: vec!["flow".to_string(), "flower".to_string(), "grok".to_string()],
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            balance_window_open: false,
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
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: default_settings.googler_image_max_threads,
            googler_video_max_threads: default_settings.googler_video_max_threads,
            voiceover_convert_to_wav: false,
            subtitles_service: "Whisper".to_string(),
            whisper_language: "auto".to_string(),
            whisper_model: "base".to_string(),
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
        }
    }
}

/// Малює компактний чіп з балансом. При наведенні підсвічується і змінює курсор.
fn draw_balance_chip(ui: &mut egui::Ui, prefix: &str, value: &str) -> egui::Response {
    let text = format!("{}: {}", prefix, value);
    let font_id = egui::FontId::new(13.0, egui::FontFamily::Proportional);
    let text_color = ui.visuals().text_color();

    let galley = ui.fonts(|f| f.layout_no_wrap(text, font_id, text_color));

    let padding = egui::vec2(8.0, 4.0);
    let desired_size = galley.rect.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() {
            ui.visuals().widgets.hovered.weak_bg_fill
        } else {
            ui.visuals().faint_bg_color
        };
        ui.painter().rect_filled(rect, egui::Rounding::same(4.0), fill);
        ui.painter().galley(rect.min + padding, galley, text_color);
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
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
            googler_image_priority,
            googler_video_priority,
            translation_temperature,
            translation_service,
            balance_window_open: false,
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
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: saved.googler_image_max_threads,
            googler_video_max_threads: saved.googler_video_max_threads,
            voiceover_convert_to_wav: saved.voiceover_convert_to_wav,
            subtitles_service: saved.subtitles_service.clone(),
            whisper_language: saved.whisper_language.clone(),
            whisper_model: saved.whisper_model.clone(),
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
        }
    }

    /// Малює вкладку системних логів роботи додатку.
    fn draw_logs_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            
            // Заголовок та кнопки керування логом у верхній панелі
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new(translate(self.language, "tab_logs")).strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Кнопка очищення логів
                    let clear_btn = egui::Button::new(
                        egui::RichText::new(translate(self.language, "logs_clear"))
                            .color(egui::Color32::from_rgb(239, 83, 80))
                    )
                    .frame(true)
                    .rounding(4.0);
                    
                    if ui.add(clear_btn).clicked() {
                        crate::logger::clear_logs();
                    }
                    
                    ui.add_space(8.0);
                    
                    // Кнопка копіювання логів
                    let copy_btn = egui::Button::new(translate(self.language, "logs_copy"))
                        .frame(true)
                        .rounding(4.0);
                        
                    if ui.add(copy_btn).clicked() {
                        let all_logs = crate::logger::get_logs().join("\n");
                        ui.ctx().copy_text(all_logs.clone());
                        self.copied_toast = Some((all_logs, std::time::Instant::now()));
                    }

                    ui.add_space(12.0);

                    // Чекбокс автопрокрутки
                    ui.checkbox(&mut self.auto_scroll_logs, translate(self.language, "logs_autoscroll"));
                });
            });
            
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Отримуємо поточні записи логу
            let logs = crate::logger::get_logs();
            
            if logs.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(translate(self.language, "logs_empty"))
                            .weak()
                            .size(14.0)
                    );
                });
            } else {
                // Преміальна темно-вугільна панель терміналу з внутрішніми відступами
                let terminal_bg = if ui.visuals().dark_mode {
                    egui::Color32::from_rgb(15, 15, 15) // Глибокий чорний для AMOLED/Dark теми
                } else {
                    egui::Color32::from_rgb(30, 30, 30) // Темно-вугільний навіть у світлій темі для стильного вигляду
                };
                
                egui::Frame::none()
                    .fill(terminal_bg)
                    .rounding(6.0)
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(f32::INFINITY)
                            .stick_to_bottom(self.auto_scroll_logs)
                            .show(ui, |ui| {
                                for log_line in logs {
                                    // Парсимо часову мітку та повідомлення
                                    let (time_part, msg_part) = if log_line.starts_with('[') && log_line.chars().nth(9) == Some(']') {
                                        (&log_line[0..10], &log_line[10..])
                                    } else {
                                        ("", log_line.as_str())
                                    };
                                    
                                    // Визначаємо колір тексту залежно від типу події
                                    let is_error = msg_part.contains("помилка") 
                                        || msg_part.contains("failed") 
                                        || msg_part.contains("STDERR") 
                                        || msg_part.contains("Error")
                                        || msg_part.contains("Err");
                                        
                                    let is_success = msg_part.contains("успішно") 
                                        || msg_part.contains("success")
                                        || msg_part.contains("Ok");
                                        
                                    let is_command = msg_part.contains("Виконується:") 
                                        || msg_part.contains("Запуск")
                                        || msg_part.contains("Running");
                                        
                                    let text_color = if is_error {
                                        egui::Color32::from_rgb(239, 83, 80) // М'який червоний
                                    } else if is_success {
                                        egui::Color32::from_rgb(102, 187, 106) // М'який зелений
                                    } else if is_command {
                                        egui::Color32::from_rgb(129, 212, 250) // М'який блакитний
                                    } else {
                                        egui::Color32::from_rgb(220, 220, 220) // Світло-сірий
                                    };
                                    
                                    // Створюємо єдиний LayoutJob для правильного переносу та вирівнювання тексту
                                    let mut job = egui::text::LayoutJob::default();
                                    
                                    if !time_part.is_empty() {
                                        job.append(
                                            time_part,
                                            0.0,
                                            egui::TextFormat {
                                                font_id: egui::FontId::monospace(11.0),
                                                color: egui::Color32::from_gray(110),
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    
                                    job.append(
                                        msg_part,
                                        0.0,
                                        egui::TextFormat {
                                            font_id: egui::FontId::monospace(11.0),
                                            color: text_color,
                                            ..Default::default()
                                        },
                                    );
                                    
                                    // Виводимо весь рядок як клікабельний лейбл
                                    let label_resp = ui.add(
                                        egui::Label::new(job)
                                            .wrap()
                                            .sense(egui::Sense::click())
                                    );
                                    
                                    if label_resp.clicked() {
                                        ui.ctx().copy_text(log_line.clone());
                                        self.copied_toast = Some((log_line.clone(), std::time::Instant::now()));
                                    }
                                    
                                    label_resp.on_hover_text(translate(self.language, "logs_click_to_copy"));
                                    
                                    ui.add_space(3.0);
                                }
                            });
                    });
            }
        });
    }
}

fn draw_balance_window(
    ctx: &egui::Context,
    open: &mut bool,
    language: crate::localization::Language,
    openrouter_key: &str,
    openrouter_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    openrouter_max_threads: &mut usize,
    claude_max_threads: &mut usize,
    gemini_max_threads: &mut usize,
    voicebot_key: &str,
    voicebot_balance: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
    googler_key: &str,
    googler_balance: &std::sync::Arc<std::sync::Mutex<Option<crate::api::googler::GooglerBalance>>>,
    edge_tts_max_threads: &mut usize,
    googler_image_max_threads: &mut usize,
    googler_video_max_threads: &mut usize,
) {
    use crate::localization::translate;
    use std::sync::Arc;

    egui::Window::new(translate(language, "balance_window_title"))
        .open(open)
        .resizable(false)
        .collapsible(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            // --- OpenRouter ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OpenRouter").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !openrouter_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::openrouter::fetch_balance(
                                openrouter_key.to_string(),
                                Arc::clone(openrouter_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = openrouter_balance.try_lock() {
                    match guard.as_ref() {
                        Some(text) => { ui.label(text.as_str()); }
                        None if openrouter_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                        }
                    }
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_openrouter_threads"));
                    let mut val = *openrouter_max_threads;
                    let slider = ui.add(egui::Slider::new(&mut val, 1..=25));
                    if slider.changed() {
                        *openrouter_max_threads = val;
                        crate::api::openrouter::OpenRouterLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Claude Code ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Claude Code").strong());
                });
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_claude_threads"));
                    let mut val = *claude_max_threads;
                    let slider = ui.add(egui::Slider::new(&mut val, 1..=25));
                    if slider.changed() {
                        *claude_max_threads = val;
                        crate::api::claude::ClaudeLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Gemini CLI ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Gemini CLI").strong());
                });
                ui.separator();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_gemini_threads"));
                    let mut val = *gemini_max_threads;
                    let slider = ui.add(egui::Slider::new(&mut val, 1..=25));
                    if slider.changed() {
                        *gemini_max_threads = val;
                        crate::api::gemini::GeminiLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- VoiceBot ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("VoiceBot").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !voicebot_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::voicebot::fetch_balance(
                                voicebot_key.to_string(),
                                Arc::clone(voicebot_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = voicebot_balance.try_lock() {
                    match guard.as_ref() {
                        Some(text) => { ui.label(text.as_str()); }
                        None if voicebot_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                        }
                    }
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new(translate(language, "balance_voicebot_limit")).weak());
            });

            ui.add_space(4.0);

            // --- Edge TTS ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Edge TTS").strong());
                });
                ui.separator();
                ui.label(egui::RichText::new(translate(language, "balance_edge_tts_status")).weak());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(translate(language, "settings_edge_tts_threads"));
                    let mut val = *edge_tts_max_threads;
                    let slider = ui.add(egui::Slider::new(&mut val, 1..=25));
                    if slider.changed() {
                        *edge_tts_max_threads = val;
                        crate::api::edgetts::EdgeTTSLimiter::get().set_max_threads(val);
                    }
                });
            });

            ui.add_space(4.0);

            // --- Googler ---
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Googler").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !googler_key.is_empty(),
                            egui::Button::new(translate(language, "balance_refresh")).small(),
                        ).clicked() {
                            crate::api::googler::fetch_balance(
                                googler_key.to_string(),
                                Arc::clone(googler_balance),
                                ui.ctx().clone(),
                            );
                        }
                    });
                });
                ui.separator();
                if let Ok(guard) = googler_balance.try_lock() {
                    match guard.as_ref() {
                        Some(bal) => {
                            egui::Grid::new("googler_balance_grid")
                                .num_columns(2)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(translate(language, "balance_img_per_hour"));
                                    ui.label(format!("{} / {}", bal.img_used, bal.img_limit));
                                    ui.end_row();
                                    ui.label(translate(language, "balance_video_per_hour"));
                                    ui.label(format!("{} / {}", bal.video_used, bal.video_limit));
                                    ui.end_row();
                                    ui.label(translate(language, "balance_img_threads"));
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{} /", bal.img_threads_active));
                                        let mut val = *googler_image_max_threads;
                                        let slider = ui.add(egui::Slider::new(&mut val, 5..=25));
                                        if slider.changed() {
                                            *googler_image_max_threads = val;
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(translate(language, "balance_video_threads"));
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{} /", bal.video_threads_active));
                                        let mut val = *googler_video_max_threads;
                                        let slider = ui.add(egui::Slider::new(&mut val, 5..=25));
                                        if slider.changed() {
                                            *googler_video_max_threads = val;
                                        }
                                    });
                                    ui.end_row();
                                });
                        }
                        None => {
                            if googler_key.is_empty() {
                                ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                            } else {
                                ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
                            }
                            
                            ui.add_space(8.0);
                            egui::Grid::new("googler_threads_offline_grid")
                                .num_columns(2)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(translate(language, "balance_img_threads"));
                                    ui.horizontal(|ui| {
                                        ui.label("0 /");
                                        let mut val = *googler_image_max_threads;
                                        let slider = ui.add(egui::Slider::new(&mut val, 5..=25));
                                        if slider.changed() {
                                            *googler_image_max_threads = val;
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(translate(language, "balance_video_threads"));
                                    ui.horizontal(|ui| {
                                        ui.label("0 /");
                                        let mut val = *googler_video_max_threads;
                                        let slider = ui.add(egui::Slider::new(&mut val, 5..=25));
                                        if slider.changed() {
                                            *googler_video_max_threads = val;
                                        }
                                    });
                                    ui.end_row();
                                });
                        }
                    }
                }
            });

            ui.add_space(6.0);

            // Кнопка "Оновити всі"
            ui.vertical_centered(|ui| {
                if ui.button(translate(language, "balance_refresh_all")).clicked() {
                    let ctx2 = ui.ctx().clone();
                    if !openrouter_key.is_empty() {
                        crate::api::openrouter::fetch_balance(
                            openrouter_key.to_string(),
                            Arc::clone(openrouter_balance),
                            ctx2.clone(),
                        );
                    }
                    if !voicebot_key.is_empty() {
                        crate::api::voicebot::fetch_balance(
                            voicebot_key.to_string(),
                            Arc::clone(voicebot_balance),
                            ctx2.clone(),
                        );
                    }
                    if !googler_key.is_empty() {
                        crate::api::googler::fetch_balance(
                            googler_key.to_string(),
                            Arc::clone(googler_balance),
                            ctx2,
                        );
                    }
                }
            });
        });
}

/// Повертає колір для відображення статусу конкретного етапу пайплайну.
fn format_file_size(bytes: u64) -> String {
    format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
}

fn stage_color(stage: &crate::queue::StageStatus, ui: &egui::Ui) -> egui::Color32 {
    match stage {
        crate::queue::StageStatus::Pending => ui.visuals().weak_text_color(),
        crate::queue::StageStatus::Running => egui::Color32::from_rgb(255, 200, 0),
        crate::queue::StageStatus::Done    => egui::Color32::from_rgb(46, 204, 113),
        crate::queue::StageStatus::Failed  => egui::Color32::from_rgb(231, 76, 60),
    }
}

/// Малює нижню панель черги задач пайплайну.
fn draw_queue_panel(
    ui: &mut egui::Ui,
    language: crate::localization::Language,
    jobs: &mut Vec<crate::queue::PipelineJob>,
    selected_job_logs: &mut Option<(u64, String)>,
    selected_job_control: &mut Option<u64>,
    control_text_input: &mut String,
    whisper_model_download: &std::sync::Arc<std::sync::Mutex<crate::gui::welcome::BinaryDownload>>,
) {
    ui.add_space(4.0);

    // Загальна вартість всіх OpenRouter запитів у черзі
    let total_cost: f64 = jobs.iter()
        .filter_map(|j| *j.translation_cost.lock().unwrap())
        .sum();

    // Верхній рядок керування
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(translate(language, "queue_panel_title")).strong().size(13.0));
        ui.label(egui::RichText::new(format!("({})", jobs.len())).weak().size(11.0));

        if total_cost > 0.0 {
            ui.label(
                egui::RichText::new(format!("${:.5}", total_cost))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(46, 204, 113)),
            );
        }

        let has_pending = jobs.iter().any(|j| {
            *j.status.lock().unwrap() == crate::queue::JobStatus::Pending
        });

        // Перевіряємо, чи всі потрібні моделі завантажені для задач у черзі
        let model_download_state = whisper_model_download.lock().unwrap().clone();
        let whisper_blocked: Option<String> = jobs.iter()
            .filter(|j| *j.status.lock().unwrap() == crate::queue::JobStatus::Pending)
            .find(|j| {
                j.settings.subtitles_enabled
                    && j.settings.subtitles_service == "Whisper"
                    && !crate::bundle::whisper_model_exists(&j.settings.whisper_model)
            })
            .map(|j| {
                let is_downloading = matches!(model_download_state, crate::gui::welcome::BinaryDownload::Downloading(_));
                if is_downloading {
                    format!("⏳ Модель Whisper '{}' ще завантажується...", j.settings.whisper_model)
                } else {
                    format!("⚠ Модель Whisper '{}' не завантажена. Завантажте її в секції Субтитри.", j.settings.whisper_model)
                }
            });

        let can_run = has_pending && whisper_blocked.is_none();

        let mut clicked = false;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let run_btn = ui.add_enabled(
                can_run,
                egui::Button::new(egui::RichText::new(translate(language, "queue_run_btn")).strong()),
            );
            if run_btn.clicked() {
                clicked = true;
            }
            if let Some(ref msg) = whisper_blocked {
                run_btn.on_disabled_hover_text(msg);
            }

            // Малюємо загальний прогресбар черги всередині right_to_left макету,
            // щоб він зайняв весь доступний простір по центру.
            if !jobs.is_empty() {
                ui.add_space(8.0);

                let total_jobs = jobs.len();
                let overall_progress = if total_jobs > 0 {
                    let sum: f32 = jobs.iter().map(|j| {
                        let status = j.status.lock().unwrap().clone();
                        match status {
                            crate::queue::JobStatus::Done => 1.0,
                            crate::queue::JobStatus::Running | crate::queue::JobStatus::AwaitingControl => {
                                let (prog, _, _) = j.calculate_progress();
                                prog
                            }
                            _ => 0.0,
                        }
                    }).sum();
                    sum / total_jobs as f32
                } else {
                    0.0
                };

                let is_running = jobs.iter().any(|j| {
                    *j.status.lock().unwrap() == crate::queue::JobStatus::Running
                });

                let pct_label = egui::RichText::new(format!("{:.0}%", overall_progress * 100.0))
                    .size(11.0)
                    .weak();
                ui.label(pct_label);
                ui.add_space(4.0);

                let bar_width = ui.available_width() - 8.0;
                if bar_width > 30.0 {
                    let bar = egui::ProgressBar::new(overall_progress)
                        .animate(is_running)
                        .desired_height(6.0);
                    ui.add_sized([bar_width, 6.0], bar);
                }
            }
        });

        if clicked {
            let ctx = ui.ctx().clone();
            for job in jobs.iter() {
                if *job.status.lock().unwrap() != crate::queue::JobStatus::Pending {
                    continue;
                }
                crate::core::pipeline::run_pipeline(
                    job.id,
                    job.name.clone(),
                    job.settings.clone(),
                    std::sync::Arc::clone(&job.status),
                    std::sync::Arc::clone(&job.translation_stage),
                    std::sync::Arc::clone(&job.voiceover_stage),
                    std::sync::Arc::clone(&job.video_stage),
                    std::sync::Arc::clone(&job.subtitles_stage),
                    std::sync::Arc::clone(&job.montage_stage),
                    std::sync::Arc::clone(&job.translated_text),
                    std::sync::Arc::clone(&job.translation_cost),
                    std::sync::Arc::clone(&job.audio_duration),
                    std::sync::Arc::clone(&job.media_progress),
                    std::sync::Arc::clone(&job.montage_progress),
                    std::sync::Arc::clone(&job.montage_file_size),
                    ctx.clone(),
                );
            }
        }
    });

    ui.add_space(10.0);

    // Список задач з горизонтальною прокруткою
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for job in jobs.iter() {
                    let status = job.status.lock().unwrap().clone();
                    let translation_stage = job.translation_stage.lock().unwrap().clone();
                    let voiceover_stage = job.voiceover_stage.lock().unwrap().clone();
                    let video_stage = job.video_stage.lock().unwrap().clone();
                    let subtitles_stage = job.subtitles_stage.lock().unwrap().clone();
                    let montage_stage = job.montage_stage.lock().unwrap().clone();
                    let montage_pct = *job.montage_progress.lock().unwrap();

                    let (status_text, status_color): (String, egui::Color32) = match &status {
                        crate::queue::JobStatus::Pending => (
                            translate(language, "queue_status_pending").to_string(),
                            ui.visuals().weak_text_color(),
                        ),
                        crate::queue::JobStatus::Running => {
                            let (prog, _, _) = job.calculate_progress();
                            (
                                format!("{} ({:.0}%)", translate(language, "queue_status_running"), prog * 100.0),
                                egui::Color32::from_rgb(255, 200, 0),
                            )
                        }
                        crate::queue::JobStatus::AwaitingControl => {
                            let (prog, _, _) = job.calculate_progress();
                            (
                                format!("{} ({:.0}%)", translate(language, "queue_status_awaiting_control"), prog * 100.0),
                                egui::Color32::from_rgb(155, 89, 182),
                            )
                        }
                        crate::queue::JobStatus::Done => (
                            translate(language, "queue_status_done").to_string(),
                            egui::Color32::from_rgb(46, 204, 113),
                        ),
                        crate::queue::JobStatus::Failed(_) => (
                            translate(language, "queue_status_failed").to_string(),
                            egui::Color32::from_rgb(231, 76, 60),
                        ),
                    };

                    let group_frame = egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 6.0, bottom: 6.0 });
                    let response = group_frame.show(ui, |ui| {
                        ui.set_width(190.0);

                        ui.vertical(|ui| {
                            ui.add_space(3.0);

                            // Назва завдання
                            ui.label(egui::RichText::new(
                                format!("#{} {}", job.id + 1, &job.name)
                            ).strong().size(15.0));

                            ui.add_space(3.0);

                            // Активні етапи — кожен з нового рядка з кольором за статусом
                            let orig_text = &job.settings.text;
                            let orig_chars = orig_text.chars().count();
                            let orig_tokens = crate::gui::editor::count_tokens(orig_text);

                            if job.settings.translation_enabled {
                                // Етап перекладу: "Переклад" з кольором за stage статусом
                                let translated_opt = job.translated_text.lock().unwrap();
                                let cost_opt = job.translation_cost.lock().unwrap();
                                
                                let cost_str = if let Some(cost) = *cost_opt {
                                    format!(", ${:.5}", cost)
                                } else {
                                    String::new()
                                };

                                let translation_label = if let Some(ref trans_text) = *translated_opt {
                                    let trans_chars = trans_text.chars().count();
                                    let trans_tokens = crate::gui::editor::count_tokens(trans_text);
                                    format!(
                                        "{} ({} {}, {} {}{})",
                                        translate(language, "translation"),
                                        trans_tokens,
                                        translate(language, "stats_tokens_short"),
                                        trans_chars,
                                        translate(language, "stats_chars_short"),
                                        cost_str
                                    )
                                } else {
                                    format!(
                                        "{} ({} {}, {} {}{})",
                                        translate(language, "translation"),
                                        orig_tokens,
                                        translate(language, "stats_tokens_short"),
                                        orig_chars,
                                        translate(language, "stats_chars_short"),
                                        cost_str
                                    )
                                };
                                ui.label(
                                    egui::RichText::new(translation_label)
                                        .color(stage_color(&translation_stage, ui))
                                        .size(12.5),
                                );
                            } else if job.settings.voiceover_enabled {
                                // Переклад вимкнено, але озвучка увімкнена → показуємо "Оригінал" (завжди зелений)
                                let original_label = format!(
                                    "{} ({} {}, {} {})",
                                    translate(language, "voiceover_text_source_original"),
                                    orig_tokens,
                                    translate(language, "stats_tokens_short"),
                                    orig_chars,
                                    translate(language, "stats_chars_short")
                                );
                                ui.label(
                                    egui::RichText::new(original_label)
                                        .color(egui::Color32::from_rgb(46, 204, 113))
                                        .size(12.5),
                                );
                            }

                            if job.settings.voiceover_enabled {
                                // Етап озвучки: "Озвучка" з кольором за stage статусом
                                let voice_label = if voiceover_stage == crate::queue::StageStatus::Done {
                                    // Після завершення показуємо тривалість аудіо
                                    let dur_opt = job.audio_duration.lock().unwrap();
                                    if let Some(secs) = *dur_opt {
                                        let total_s = secs as u64;
                                        let h = total_s / 3600;
                                        let m = (total_s % 3600) / 60;
                                        let s = total_s % 60;
                                        let dur_str = if h > 0 {
                                            format!(
                                                "{}{}{}{}{}{}",
                                                h, translate(language, "time_hours_short"),
                                                m, translate(language, "time_mins_short"),
                                                s, translate(language, "time_secs_short")
                                            )
                                        } else if m > 0 {
                                            format!(
                                                "{}{}{}{}",
                                                m, translate(language, "time_mins_short"),
                                                s, translate(language, "time_secs_short")
                                            )
                                        } else {
                                            format!("{}{}", s, translate(language, "time_secs_short"))
                                        };
                                        format!("{} ({})", translate(language, "voiceover"), dur_str)
                                    } else {
                                        translate(language, "voiceover").to_string()
                                    }
                                } else if job.settings.translation_enabled {
                                    let translated_opt = job.translated_text.lock().unwrap();
                                    if translated_opt.is_some() {
                                        translate(language, "voiceover").to_string()
                                    } else {
                                        format!(
                                            "{} ({})",
                                            translate(language, "voiceover"),
                                            translate(language, "queue_waiting_translation")
                                        )
                                    }
                                } else {
                                    translate(language, "voiceover").to_string()
                                };

                                ui.label(
                                    egui::RichText::new(voice_label)
                                        .color(stage_color(&voiceover_stage, ui))
                                        .size(12.5),
                                );
                            }

                            if job.settings.video_enabled {
                                let video_label = match &video_stage {
                                    crate::queue::StageStatus::Running | crate::queue::StageStatus::Done => {
                                        if let Some((done, total)) = *job.media_progress.lock().unwrap() {
                                            format!("{} ({}/{})", translate(language, "video"), done, total)
                                        } else {
                                            translate(language, "video").to_string()
                                        }
                                    }
                                    _ => translate(language, "video").to_string(),
                                };
                                ui.label(
                                    egui::RichText::new(video_label)
                                        .color(stage_color(&video_stage, ui))
                                        .size(12.5),
                                );
                            }

                            if job.settings.subtitles_enabled {
                                ui.label(
                                    egui::RichText::new(translate(language, "subtitles"))
                                        .color(stage_color(&subtitles_stage, ui))
                                        .size(12.5),
                                );
                            }

                            if job.settings.montage_enabled {
                                let montage_label = match &montage_stage {
                                    crate::queue::StageStatus::Running => {
                                        match montage_pct {
                                            Some(pct) => format!("{} ({:.0}%)", translate(language, "editing"), pct * 100.0),
                                            None => translate(language, "editing").to_string(),
                                        }
                                    }
                                    crate::queue::StageStatus::Done => {
                                        let size_str = job.montage_file_size.lock().unwrap()
                                            .map(format_file_size)
                                            .unwrap_or_default();
                                        if size_str.is_empty() {
                                            format!("{} (100%)", translate(language, "editing"))
                                        } else {
                                            format!("{} (100%  {})", translate(language, "editing"), size_str)
                                        }
                                    }
                                    _ => translate(language, "editing").to_string(),
                                };
                                ui.label(
                                    egui::RichText::new(montage_label)
                                        .color(stage_color(&montage_stage, ui))
                                        .size(12.5),
                                );
                            }

                            ui.horizontal(|ui| {
                                // Загальний статус задачі
                                ui.label(
                                    egui::RichText::new(status_text)
                                        .color(status_color)
                                        .size(13.0),
                                );

                                // Показуємо помилку (hover підказка)
                                if let crate::queue::JobStatus::Failed(err) = &status {
                                    ui.label(
                                        egui::RichText::new("⚠ помилка")
                                            .color(egui::Color32::from_rgb(231, 76, 60))
                                            .size(12.0),
                                    ).on_hover_text(err);
                                }
                            });

                            ui.add_space(3.0);

                            // Індивідуальний прогрес бар картки задачі
                            let (prog, _, _) = job.calculate_progress();
                            let is_job_running = status == crate::queue::JobStatus::Running;

                            ui.horizontal(|ui| {
                                let pct_text = format!("{:.0}%", prog * 100.0);
                                let pct_galley = ui.painter().layout_no_wrap(
                                    pct_text.clone(),
                                    egui::FontId::proportional(11.0),
                                    ui.visuals().weak_text_color(),
                                );
                                let pct_width = pct_galley.size().x + 4.0;
                                let bar_width = (ui.available_width() - pct_width - ui.spacing().item_spacing.x).max(20.0);

                                let bar = egui::ProgressBar::new(prog)
                                    .animate(is_job_running)
                                    .desired_height(6.0);
                                ui.add_sized([bar_width, 6.0], bar);

                                ui.label(
                                    egui::RichText::new(pct_text)
                                        .size(11.0)
                                        .weak(),
                                );
                            });
                        });
                    });

                    // Робимо групу клікабельною для показу логів задачі
                    let interact = ui.interact(
                        response.response.rect,
                        ui.make_persistent_id(format!("job_click_{}", job.id)),
                        egui::Sense::click()
                    );
                    
                    if interact.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if interact.clicked() {
                        if status == crate::queue::JobStatus::AwaitingControl {
                            *selected_job_control = Some(job.id);
                            if let Some(text) = job.translated_text.lock().unwrap().as_ref() {
                                *control_text_input = text.clone();
                            } else {
                                *control_text_input = String::new();
                            }
                        } else {
                            *selected_job_logs = Some((job.id, job.name.clone()));
                        }
                    }

                    ui.add_space(4.0);
                }
            });
        });
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
        egui::TopBottomPanel::top("navigation_bar")
            .min_height(40.0)
            .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Main, egui::RichText::new(translate(self.language, "tab_main")).size(14.0));
                ui.selectable_value(&mut self.active_tab, Tab::Settings, egui::RichText::new(translate(self.language, "tab_settings")).size(14.0));
                ui.selectable_value(&mut self.active_tab, Tab::Logs, egui::RichText::new(translate(self.language, "tab_logs")).size(14.0));

                // Баланс-чіпи з правого боку (RTL: перший доданий — крайній правий)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if let Ok(guard) = self.openrouter_balance.try_lock() {
                        if let Some(text) = guard.as_ref() {
                            if draw_balance_chip(ui, "OpenRouter", text).clicked() {
                                self.balance_window_open = true;
                            }
                        }
                    }
                    if let Ok(guard) = self.voicebot_balance.try_lock() {
                        if let Some(text) = guard.as_ref() {
                            // Показуємо лише числову частину, без слова "символів"
                            let display = text.split_whitespace().next().unwrap_or(text.as_str());
                            if draw_balance_chip(ui, "VoiceBot", display).clicked() {
                                self.balance_window_open = true;
                            }
                        }
                    }
                    if let Ok(guard) = self.googler_balance.try_lock() {
                        if let Some(bal) = guard.as_ref() {
                            let text = format!(
                                "img: {}/{} vid: {}/{} th: i{}/{} v{}/{}",
                                bal.img_used, bal.img_limit,
                                bal.video_used, bal.video_limit,
                                bal.img_threads_active, self.googler_image_max_threads,
                                bal.video_threads_active, self.googler_video_max_threads,
                            );
                            if draw_balance_chip(ui, "Googler", &text).clicked() {
                                self.balance_window_open = true;
                            }
                        }
                    }
                });
            });
        });

        // Плаваюче вікно з детальними балансами
        draw_balance_window(
            ctx,
            &mut self.balance_window_open,
            self.language,
            &self.openrouter_key,
            &self.openrouter_balance,
            &mut self.openrouter_max_threads,
            &mut self.claude_max_threads,
            &mut self.gemini_max_threads,
            &self.voicebot_key,
            &self.voicebot_balance,
            &self.googler_key,
            &self.googler_balance,
            &mut self.edge_tts_max_threads,
            &mut self.googler_image_max_threads,
            &mut self.googler_video_max_threads,
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

        // Нижня панель черги задач (тільки якщо є задачі)
        if !self.jobs.is_empty() {
            egui::TopBottomPanel::bottom("queue_panel")
                .min_height(140.0)
                .default_height(160.0)
                .max_height(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    draw_queue_panel(
                        ui,
                        self.language,
                        &mut self.jobs,
                        &mut self.selected_job_logs,
                        &mut self.selected_job_control,
                        &mut self.control_text_input,
                        &self.whisper_model_download,
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
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Main => {
                        gui::editor::draw_editor(ui, &mut self.text_input, self.language, self.text_split_char_limit);
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
        if let Some((job_id, job_name)) = self.selected_job_logs.clone() {
            let mut is_open = true;
            let mut copied_toast_data = None;
            
            egui::Window::new(format!("{} #{}: {}", translate(self.language, "job_logs_title"), job_id + 1, job_name))
                .open(&mut is_open)
                .resizable(true)
                .default_size([550.0, 350.0])
                .show(ctx, |ui| {
                    let job_logs = crate::logger::get_job_logs(job_id);
                    if job_logs.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.colored_label(ui.visuals().weak_text_color(), translate(self.language, "job_logs_empty"));
                            ui.add_space(40.0);
                        });
                    } else {
                        // Верхня панель з кнопкою копіювання всього логу та чекбоксом автопрокрутки
                        ui.horizontal(|ui| {
                            // Чекбокс автопрокрутки зліва
                            ui.checkbox(&mut self.auto_scroll_logs, translate(self.language, "logs_autoscroll"));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let copy_all_btn = egui::Button::new(translate(self.language, "job_logs_copy_all"))
                                    .frame(true)
                                    .rounding(4.0);
                                    
                                if ui.add(copy_all_btn).clicked() {
                                    let all_job_logs = job_logs.join("\n");
                                    ui.ctx().copy_text(all_job_logs.clone());
                                    copied_toast_data = Some((all_job_logs, std::time::Instant::now()));
                                }
                            });
                        });
                        
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Преміальна темно-вугільна панель терміналу з внутрішніми відступами
                        let terminal_bg = if ui.visuals().dark_mode {
                            egui::Color32::from_rgb(15, 15, 15) // Глибокий чорний для AMOLED/Dark теми
                        } else {
                            egui::Color32::from_rgb(30, 30, 30) // Темно-вугільний навіть у світлій темі
                        };

                        egui::Frame::none()
                            .fill(terminal_bg)
                            .rounding(6.0)
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .stick_to_bottom(self.auto_scroll_logs)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            for log_line in job_logs {
                                                // Парсимо часову мітку та повідомлення
                                                let (time_part, msg_part) = if log_line.starts_with('[') && log_line.chars().nth(9) == Some(']') {
                                                    (&log_line[0..10], &log_line[10..])
                                                } else {
                                                    ("", log_line.as_str())
                                                };
                                                
                                                // Визначаємо колір тексту залежно від типу події
                                                let is_error = msg_part.contains("помилка") 
                                                    || msg_part.contains("failed") 
                                                    || msg_part.contains("STDERR") 
                                                    || msg_part.contains("Error")
                                                    || msg_part.contains("Err");
                                                    
                                                let is_success = msg_part.contains("успішно") 
                                                    || msg_part.contains("success")
                                                    || msg_part.contains("Ok");
                                                    
                                                let is_command = msg_part.contains("Виконується:") 
                                                    || msg_part.contains("Запуск")
                                                    || msg_part.contains("Running");
                                                    
                                                let text_color = if is_error {
                                                    egui::Color32::from_rgb(239, 83, 80) // М'який червоний
                                                } else if is_success {
                                                    egui::Color32::from_rgb(102, 187, 106) // М'який зелений
                                                } else if is_command {
                                                    egui::Color32::from_rgb(129, 212, 250) // М'який блакитний
                                                } else {
                                                    egui::Color32::from_rgb(220, 220, 220) // Світло-сірий
                                                };

                                                // Створюємо єдиний LayoutJob для правильного переносу та вирівнювання тексту
                                                let mut job = egui::text::LayoutJob::default();
                                                
                                                if !time_part.is_empty() {
                                                    job.append(
                                                        time_part,
                                                        0.0,
                                                        egui::TextFormat {
                                                            font_id: egui::FontId::monospace(11.0),
                                                            color: egui::Color32::from_gray(110),
                                                            ..Default::default()
                                                        },
                                                    );
                                                }
                                                
                                                job.append(
                                                    msg_part,
                                                    0.0,
                                                    egui::TextFormat {
                                                        font_id: egui::FontId::monospace(11.0),
                                                        color: text_color,
                                                        ..Default::default()
                                                    },
                                                );

                                                // Виводимо весь рядок як клікабельний лейбл
                                                let label_resp = ui.add(
                                                    egui::Label::new(job)
                                                        .wrap()
                                                        .sense(egui::Sense::click())
                                                );
                                                
                                                if label_resp.clicked() {
                                                    ui.ctx().copy_text(log_line.clone());
                                                    copied_toast_data = Some((log_line.clone(), std::time::Instant::now()));
                                                }
                                                
                                                label_resp.on_hover_text(translate(self.language, "logs_click_to_copy"));
                                                
                                                ui.add_space(3.0);
                                            }
                                        });
                                    });
                            });
                    }
                });
            
            if let Some(toast) = copied_toast_data {
                self.copied_toast = Some(toast);
            }

            if !is_open {
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

        // Спливаюче вікно контролю перекладу
        if let Some(job_id) = self.selected_job_control {
            let mut is_open = true;
            let mut should_continue = false;
            let mut is_confirmed = false;

            // Знаходимо задачу в черзі
            if let Some(job_idx) = self.jobs.iter().position(|j| j.id == job_id) {
                let job_name = self.jobs[job_idx].name.clone();
                let job_save_path = self.jobs[job_idx].settings.save_path.clone();
                let translated_text_arc = std::sync::Arc::clone(&self.jobs[job_idx].translated_text);
                let translation_cost_arc = std::sync::Arc::clone(&self.jobs[job_idx].translation_cost);
                let audio_duration_arc = std::sync::Arc::clone(&self.jobs[job_idx].audio_duration);
                let status_arc = std::sync::Arc::clone(&self.jobs[job_idx].status);
                let translation_stage_arc = std::sync::Arc::clone(&self.jobs[job_idx].translation_stage);
                let voiceover_stage_arc = std::sync::Arc::clone(&self.jobs[job_idx].voiceover_stage);
                let video_stage_arc = std::sync::Arc::clone(&self.jobs[job_idx].video_stage);
                let subtitles_stage_arc = std::sync::Arc::clone(&self.jobs[job_idx].subtitles_stage);
                let media_progress_arc = std::sync::Arc::clone(&self.jobs[job_idx].media_progress);
                let job_settings = self.jobs[job_idx].settings.clone();

                // Перевіряємо результат фонової перегенерації
                {
                    let result = self.control_regen_result.lock().unwrap().take();
                    if let Some(res) = result {
                        match res {
                            Ok((text, cost)) => {
                                self.control_text_input = text;
                                // Додаємо ціну перегенерації до накопиченої вартості задачі
                                if let Some(new_cost) = cost {
                                    let mut existing = translation_cost_arc.lock().unwrap();
                                    *existing = Some(existing.unwrap_or(0.0) + new_cost);
                                }
                                self.control_regen_error = None;
                            }
                            Err(e) => {
                                self.control_regen_error = Some(e);
                            }
                        }
                    }
                }

                let mut control_closed = false;
                let mut trigger_simple_regen = false;
                let mut open_extended = false;

                egui::Window::new(format!("{} — {}", translate(self.language, "control_window_title"), job_name))
                    .open(&mut is_open)
                    .resizable(true)
                    .default_size([500.0, 350.0])
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(translate(self.language, "control_window_text")).strong().size(12.0));
                            ui.add_space(4.0);

                            // Текстове поле для редагування перекладу
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.control_text_input)
                                            .hint_text("Перекладений текст...")
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(10)
                                    );
                                });

                            ui.add_space(4.0);

                            // Статистика перекладеного тексту
                            let translated_char_count = self.control_text_input.chars().count();
                            let translated_word_count = self.control_text_input.split_whitespace().count();
                            let translated_token_count = crate::gui::editor::count_tokens(&self.control_text_input);
                            let cost_snapshot = *translation_cost_arc.lock().unwrap();

                            let text_color = ui.visuals().widgets.noninteractive.text_color();
                            let accent_color = ui.visuals().selection.bg_fill;
                            let bullet_color = text_color.linear_multiply(0.3);

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(translate(self.language, "stats_chars")).size(12.0).color(text_color));
                                ui.label(egui::RichText::new(format!(" {}", translated_char_count)).size(12.0).strong().color(accent_color));
                                ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                                ui.label(egui::RichText::new(translate(self.language, "stats_words")).size(12.0).color(text_color));
                                ui.label(egui::RichText::new(format!(" {}", translated_word_count)).size(12.0).strong().color(accent_color));
                                ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                                ui.label(egui::RichText::new(translate(self.language, "stats_tokens")).size(12.0).color(text_color));
                                ui.label(egui::RichText::new(format!(" {}", translated_token_count)).size(12.0).strong().color(accent_color));
                                if let Some(cost) = cost_snapshot {
                                    ui.label(egui::RichText::new("  •  ").size(12.0).color(bullet_color));
                                    ui.label(egui::RichText::new(translate(self.language, "control_window_cost")).size(12.0).color(text_color));
                                    ui.label(egui::RichText::new(format!(" ${:.5}", cost)).size(12.0).strong().color(accent_color));
                                }
                            });

                            // Помилка перегенерації
                            if let Some(ref err) = self.control_regen_error {
                                ui.add_space(4.0);
                                ui.add(egui::Label::new(
                                    egui::RichText::new(format!("{} {}", translate(self.language, "control_regen_error"), err))
                                        .color(egui::Color32::from_rgb(231, 76, 60))
                                        .size(11.0)
                                ).wrap());
                            }

                            ui.add_space(8.0);

                            let is_regen_loading = *self.control_regen_loading.lock().unwrap();

                            ui.horizontal(|ui| {
                                if ui.button(translate(self.language, "job_name_cancel_btn")).clicked() {
                                    control_closed = true;
                                }

                                // Перегенерувати з тим самим промтом задачі
                                if ui.add_enabled(
                                    !is_regen_loading,
                                    egui::Button::new(translate(self.language, "control_regen_btn")),
                                ).clicked() {
                                    trigger_simple_regen = true;
                                }

                                // Відкрити вікно розширеної перегенерації
                                if ui.add_enabled(
                                    !is_regen_loading,
                                    egui::Button::new(translate(self.language, "control_regen_extended_btn")),
                                ).clicked() {
                                    open_extended = true;
                                }

                                if is_regen_loading {
                                    ui.label(
                                        egui::RichText::new(translate(self.language, "control_regen_loading"))
                                            .weak()
                                            .size(11.0),
                                    );
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(translate(self.language, "control_window_continue_btn")).strong()
                                        )
                                    ).clicked() {
                                        should_continue = true;
                                    }
                                });
                            });
                        });
                    });

                if control_closed {
                    is_open = false;
                    is_confirmed = true;
                }

                // Запускаємо просту перегенерацію з оригінальними налаштуваннями задачі
                if trigger_simple_regen {
                    let result_arc = std::sync::Arc::clone(&self.control_regen_result);
                    let loading_arc = std::sync::Arc::clone(&self.control_regen_loading);
                    let ctx_clone = ctx.clone();
                    let text = job_settings.text.clone();
                    let service = job_settings.translation_service.clone();
                    let model = job_settings.translation_model.clone();
                    let prompt = job_settings.translation_prompt.clone();
                    let temperature = job_settings.translation_temperature;
                    let key = job_settings.openrouter_key.clone();
                    let job_info = Some((job_id, job_name.clone()));

                    self.control_regen_error = None;
                    *loading_arc.lock().unwrap() = true;
                    std::thread::spawn(move || {
                        let result = crate::core::pipeline::translate::translate_text(
                            &service, &key, &model, &prompt, &text, temperature, job_info,
                        );
                        *result_arc.lock().unwrap() = Some(result);
                        *loading_arc.lock().unwrap() = false;
                        ctx_clone.request_repaint();
                    });
                }

                // Ініціалізуємо та відкриваємо вікно розширеної перегенерації
                if open_extended && !self.control_regen_extended_open {
                    self.control_regen_service = job_settings.translation_service.clone();
                    self.control_regen_model = job_settings.translation_model.clone();
                    self.control_regen_model_openrouter = if job_settings.translation_service == "OpenRouter" {
                        job_settings.translation_model.clone()
                    } else {
                        self.control_regen_model_openrouter.clone()
                    };
                    self.control_regen_model_claude = if job_settings.translation_service == "Claude Code" {
                        if job_settings.translation_model.is_empty() { "sonnet".to_string() } else { job_settings.translation_model.clone() }
                    } else {
                        self.control_regen_model_claude.clone()
                    };
                    self.control_regen_model_gemini = if job_settings.translation_service == "Gemini CLI" {
                        if job_settings.translation_model.is_empty() { "gemini-2.5-flash".to_string() } else { job_settings.translation_model.clone() }
                    } else {
                        self.control_regen_model_gemini.clone()
                    };
                    self.control_regen_prompt = job_settings.translation_prompt.clone();
                    self.control_regen_temperature = job_settings.translation_temperature;
                    self.control_regen_model_search.clear();
                    self.control_regen_extended_open = true;
                }

                // Вікно розширеної перегенерації з одноразовими налаштуваннями
                if self.control_regen_extended_open {
                    let openrouter_models_arc = std::sync::Arc::clone(&self.openrouter_models);
                    let openrouter_models_loading_arc = std::sync::Arc::clone(&self.openrouter_models_loading);
                    let text_to_translate = job_settings.text.clone();
                    let openrouter_key_ext = job_settings.openrouter_key.clone();
                    let job_info_ext = Some((job_id, job_name.clone()));

                    let mut ext_is_open = true;
                    let mut trigger_ext_regen = false;

                    egui::Window::new(translate(self.language, "control_regen_extended_title"))
                        .open(&mut ext_is_open)
                        .resizable(true)
                        .default_size([450.0, 500.0])
                        .show(ctx, |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(translate(self.language, "control_regen_settings_note"))
                                    .weak()
                                    .size(11.0),
                            ).wrap());
                            ui.add_space(6.0);
                            ui.separator();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.push_id("control_regen_translation", |ui| {
                                    crate::gui::pipeline::translation::draw_translation_section(
                                        ui,
                                        self.language,
                                        &mut self.control_regen_prompt,
                                        &mut self.control_regen_model,
                                        &mut self.control_regen_model_search,
                                        &openrouter_models_arc,
                                        &openrouter_models_loading_arc,
                                        &mut self.control_regen_temperature,
                                        &mut self.control_regen_service,
                                        &mut self.control_regen_model_openrouter,
                                        &mut self.control_regen_model_claude,
                                        &mut self.control_regen_model_gemini,
                                    );
                                });
                            });

                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            let is_regen_loading = *self.control_regen_loading.lock().unwrap();

                            if is_regen_loading {
                                ui.label(
                                    egui::RichText::new(translate(self.language, "control_regen_loading"))
                                        .weak(),
                                );
                            } else if ui.button(translate(self.language, "control_regen_run_btn")).clicked() {
                                trigger_ext_regen = true;
                            }
                        });

                    if !ext_is_open {
                        self.control_regen_extended_open = false;
                    }

                    if trigger_ext_regen {
                        let result_arc = std::sync::Arc::clone(&self.control_regen_result);
                        let loading_arc = std::sync::Arc::clone(&self.control_regen_loading);
                        let ctx_clone = ctx.clone();
                        let service = self.control_regen_service.clone();
                        let model = self.control_regen_model.clone();
                        let prompt = self.control_regen_prompt.clone();
                        let temperature = self.control_regen_temperature;

                        self.control_regen_error = None;
                        *loading_arc.lock().unwrap() = true;
                        std::thread::spawn(move || {
                            let result = crate::core::pipeline::translate::translate_text(
                                &service, &openrouter_key_ext, &model, &prompt, &text_to_translate, temperature, job_info_ext,
                            );
                            *result_arc.lock().unwrap() = Some(result);
                            *loading_arc.lock().unwrap() = false;
                            ctx_clone.request_repaint();
                        });
                    }
                }

                if should_continue {
                    // 1. Оновлюємо перекладений текст у задачі
                    *translated_text_arc.lock().unwrap() = Some(self.control_text_input.clone());

                    // 2. Зберігаємо оновлений текст на диску
                    let dir = std::path::Path::new(&job_save_path);
                    let _ = std::fs::write(dir.join("text.txt"), &self.control_text_input);

                    // 3. Змінюємо статус задачі на Running
                    *status_arc.lock().unwrap() = crate::queue::JobStatus::Running;

                    // 4. Запускаємо пайплайн знову
                    let ctx_clone = ctx.clone();
                    crate::core::pipeline::run_pipeline(
                        job_id,
                        job_name,
                        job_settings,
                        status_arc,
                        translation_stage_arc,
                        voiceover_stage_arc,
                        video_stage_arc,
                        subtitles_stage_arc,
                        std::sync::Arc::clone(&self.jobs[job_idx].montage_stage),
                        translated_text_arc,
                        translation_cost_arc,
                        audio_duration_arc,
                        media_progress_arc,
                        std::sync::Arc::clone(&self.jobs[job_idx].montage_progress),
                        std::sync::Arc::clone(&self.jobs[job_idx].montage_file_size),
                        ctx_clone,
                    );

                    // 5. Закриваємо вікно контролю
                    is_open = false;
                }
            } else {
                is_open = false;
            }

            if !is_open {
                if !is_confirmed {
                    self.control_dismissed.insert(job_id);
                }
                self.selected_job_control = None;
                self.control_text_input.clear();
                self.control_regen_extended_open = false;
                self.control_regen_error = None;
            }
        }

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
                || self.googler_image_max_threads != self.last_saved_settings.googler_image_max_threads
                || self.googler_video_max_threads != self.last_saved_settings.googler_video_max_threads
                || self.voiceover_convert_to_wav != self.last_saved_settings.voiceover_convert_to_wav
                || self.googler_image_priority != self.last_saved_settings.googler_image_priority
                || self.googler_video_priority != self.last_saved_settings.googler_video_priority
                || self.translation_temperature != self.last_saved_settings.translation_temperature
                || self.subtitles_service != self.last_saved_settings.subtitles_service
                || self.whisper_language != self.last_saved_settings.whisper_language
                || self.whisper_model != self.last_saved_settings.whisper_model
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
                    googler_image_max_threads: self.googler_image_max_threads,
                    googler_video_max_threads: self.googler_video_max_threads,
                    voiceover_convert_to_wav: self.voiceover_convert_to_wav,
                    googler_image_priority: self.googler_image_priority.clone(),
                    googler_video_priority: self.googler_video_priority.clone(),
                    subtitles_service: self.subtitles_service.clone(),
                    whisper_language: self.whisper_language.clone(),
                    whisper_model: self.whisper_model.clone(),
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
