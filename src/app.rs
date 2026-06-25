use crate::gui;
use crate::gui::settings::storage::AppSettings;
use eframe::egui;
use crate::theme::AppTheme;
use crate::localization::Language;

mod settings_sync;
mod pipeline_host;
mod windows;
mod lifecycle;
mod queue_host;
mod gallery_host;
mod montage_host;

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
    /// Ключ API для AssemblyAI.
    pub assemblyai_key: String,
    /// Статус перевірки AssemblyAI API ключа.
    pub assemblyai_status: Option<String>,
    /// Результат фонового тесту API ключа AssemblyAI.
    pub assemblyai_test_result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Ключ API для Pexels Stock.
    pub pexels_key: String,
    /// Статус перевірки Pexels API ключа.
    pub pexels_status: Option<String>,
    /// Результат фонового тесту API ключа Pexels.
    pub pexels_test_result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Ключ API для Pixabay Stock.
    pub pixabay_key: String,
    /// Статус перевірки Pixabay API ключа.
    pub pixabay_status: Option<String>,
    /// Результат фонового тесту API ключа Pixabay.
    pub pixabay_test_result: std::sync::Arc<std::sync::Mutex<Option<String>>>,
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
    /// Чи увімкнено контроль монтажу (показує кнопку редактора монтажу в карточці задачі).
    pub pipeline_montage_control_enabled: bool,
    /// ID задачі, для якої відкрито редактор монтажу. None = редактор закрито.
    pub montage_editor_open_job: Option<u64>,
    /// Стан редактора монтажу (завантажений timeline, аудіо тощо).
    pub montage_editor_state: Option<crate::gui::montage_editor::MontageEditorState>,
    /// Кеш текстур для галереї медіафайлів. None означає що ще завантажується або помилка.
    pub gallery_textures: std::collections::HashMap<std::path::PathBuf, Option<egui::TextureHandle>>,
    /// Набір шляхів зображень, які зараз завантажуються у фоні.
    pub gallery_image_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Результат фонового завантаження зображень галереї.
    pub gallery_image_result: std::sync::Arc<std::sync::Mutex<Vec<(std::path::PathBuf, Option<egui::TextureHandle>)>>>,
    /// Зображення, яке зараз відкрите у повноекранному перегляді.
    pub gallery_preview: Option<std::path::PathBuf>,
    /// Набір шляхів зображень, які зараз анімуються у фоні (image-to-video).
    pub gallery_anim_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Прапорець виконання перегенерації медіафайлу у фоні (для custom regen window).
    pub media_regen_loading: std::sync::Arc<std::sync::Mutex<bool>>,
    /// Результат перегенерації для custom regen window. None = ще не завершено.
    pub media_regen_result: std::sync::Arc<std::sync::Mutex<Option<Result<(), String>>>>,
    /// Файл, що зараз перегенеровується у custom regen window.
    pub media_regen_target: Option<std::path::PathBuf>,
    /// Набір шляхів усіх файлів що зараз перегенеровуються (підтримка паралельних).
    pub media_regen_paths: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Черга результатів перегенерацій для обробки кешу (усі паралельні).
    pub media_regen_results_queue: std::sync::Arc<std::sync::Mutex<Vec<(std::path::PathBuf, Result<(), String>)>>>,
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
    /// ID обраної моделі Codex.
    pub translation_model_codex: String,
    /// ID обраної моделі AGY.
    pub translation_model_agy: String,
    /// ID обраної моделі Pi.
    pub translation_model_pi: String,
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
    /// Збережений режим нарізання для не-CLI сервісів (відновлюється після переключення з агентів)
    pub text_split_mode_openrouter: String,
    /// Ліміт символів для режиму char_limit.
    pub text_split_char_limit: usize,
    /// Промт для генерації зображень відеоряду.
    pub video_prompt: String,
    /// Чи додавати контекст сценарію в API-промт відеоряду.
    pub video_context_enabled: bool,
    /// Режим контексту: "full" або "around".
    pub video_context_mode: String,
    /// Кількість символів контексту навколо сегмента для режиму "around".
    pub video_context_chars: usize,
    /// Чи увімкнено автоматичний апскейл Googler відео
    pub googler_video_upscale_enabled: bool,
    /// Роздільна здатність апскейлу ("1080p", "2K", "4K")
    pub googler_video_upscale_resolution: String,
    /// Якість апскейлу ("fast", "balanced", "max")
    pub googler_video_upscale_quality: String,
    /// Якість превʼю редактора монтажу ("performance", "balanced", "high", "ultra")
    pub preview_quality: String,
    /// FPS превʼю редактора монтажу (15.0, 30.0, 60.0)
    pub preview_fps: f32,
    /// Системна інструкція агенту для створення timeline.json.
    pub video_agent_prompt: String,
    /// Чи увімкнено поле стилю для генерації медіа в агентному режимі.
    pub video_style_enabled: bool,
    /// Промт стилю для генерації медіа ({{text}} підставляється з timeline.json).
    pub video_style_prompt: String,

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
    /// Модель Codex для відео-промтів.
    pub video_llm_model_codex: String,
    /// Модель AGY для відео-промтів.
    pub video_llm_model_agy: String,
    /// Модель Pi для відео-промтів.
    pub video_llm_model_pi: String,
    /// Температура ЛЛМ для відео-промтів.
    pub video_llm_temperature: f32,
    /// Рядок пошуку у дропдауні вибору моделі ЛЛМ відеоряду.
    pub video_llm_model_search: String,
    /// Пріоритетний список провайдерів зображень Googler.
    pub googler_image_priority: Vec<String>,
    /// Пріоритетний список провайдерів відео Googler.
    pub googler_video_priority: Vec<String>,
    /// Вимкнені провайдери відео Googler.
    pub googler_video_disabled: Vec<String>,
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
    /// Запит на повтор конкретного етапу задачі: (job_id, stage)
    pub retry_request: Option<(u64, crate::queue::RetryStage)>,
    /// Відкриті вікна логів задач: job_id → job_name.
    pub open_job_logs: std::collections::HashMap<u64, String>,
    /// Відкриті вікна контролю перекладу: job_id → стан вікна.
    pub open_job_controls: std::collections::HashMap<u64, crate::gui::pipeline::translation_control::TranslationControlWindowState>,
    /// Відкриті вікна чату з агентом: job_id → стан вікна.
    pub open_agent_chats: std::collections::HashMap<u64, crate::gui::agent_chat_window::AgentChatWindowState>,
    /// Задачі, для яких користувач вручну закрив вікно контролю (авто-відкриття їх пропускає).
    pub control_dismissed: std::collections::HashSet<u64>,
    /// Чи відкрите вікно введення назви задачі.
    pub job_name_dialog_open: bool,
    /// Поточний текст у полі введення назви задачі.
    pub job_name_input: String,
    /// Чи відкрите вікно відновлення задачі (знайдені наявні файли).
    pub resume_dialog_open: bool,
    /// Дані для діалогу відновлення (задача в очікуванні рішення користувача).
    pub resume_pending: Option<crate::gui::pipeline::resume::ResumePendingData>,
    /// Максимальна кількість потоків для OpenRouter.
    pub openrouter_max_threads: usize,
    /// Максимальна кількість потоків для Claude Code.
    pub claude_max_threads: usize,
    /// Максимальна кількість потоків для Gemini CLI.
    pub gemini_max_threads: usize,
    /// Максимальна кількість потоків для Codex CLI.
    pub codex_max_threads: usize,
    /// Максимальна кількість потоків для AGY CLI.
    pub agy_max_threads: usize,
    /// Максимальна кількість потоків для Pi CLI.
    pub pi_max_threads: usize,
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
    /// Розмір шрифту субтитрів (пунктів).
    pub subtitle_font_size: u32,
    /// RGB колір тексту субтитрів.
    pub subtitle_color: [u8; 3],
    /// Вертикальний відступ субтитрів від нижнього краю (пікселів).
    pub subtitle_margin_v: u32,
    /// Ефект karaoke для субтитрів.
    pub subtitle_karaoke: bool,
    /// Режим karaoke: 0 = fill (\kf), 1 = switch (\k), 2 = follow.
    pub subtitle_karaoke_mode: u8,
    /// RGB колір слова що проговорюється.
    pub subtitle_karaoke_highlight_color: [u8; 3],
    /// RGB колір обводки субтитрів.
    pub subtitle_karaoke_outline_color: [u8; 3],
    /// Жирний текст для karaoke субтитрів.
    pub subtitle_karaoke_bold: bool,
    /// Масштаб поточного слова у % (режим follow).
    pub subtitle_karaoke_scale: u32,
    /// Обраний шрифт для субтитрів.
    pub subtitle_font: String,
    /// Список шрифтів, завантажених із системи (заповнюється при старті).
    pub available_subtitle_fonts: Vec<String>,
    /// Генерувати CapCut-проект замість локального FFmpeg-монтажу.
    pub capcut_enabled: bool,
    /// Шлях до кореневого каталогу чернеток CapCut.
    pub capcut_draft_path: String,
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
    /// Чи увімкнено ефект зуму для зображень.
    pub montage_image_zoom_enabled: bool,
    /// Інтенсивність зуму для зображень (0.1..1.0).
    pub montage_image_zoom_intensity: f32,
    /// Режим зуму: "alternate" або "oscillate".
    pub montage_image_zoom_mode: String,
    /// Кратність зуму (1.1..2.0).
    pub montage_image_zoom_scale: f32,
    /// Чи увімкнено ефект покачування для зображень.
    pub montage_image_shake_enabled: bool,
    /// Інтенсивність покачування для зображень (0.1..1.0).
    pub montage_image_shake_intensity: f32,
    /// Чи увімкнено тригери накладення медіа за ключовими фразами.
    pub overlay_triggers_enabled: bool,
    /// Список тригерів накладення медіа.
    pub overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
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
    pub video_hover_result: std::sync::Arc<std::sync::Mutex<Vec<(std::path::PathBuf, Vec<egui::TextureHandle>)>>>,
    /// Кеш першого кадру відео як thumbnail: path → текстура.
    pub video_thumbnails: std::collections::HashMap<std::path::PathBuf, Option<egui::TextureHandle>>,
    /// Набір шляхів відео, для яких зараз витягується thumbnail.
    pub video_thumb_loading: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>>>,
    /// Результат фонового витягування thumbnail.
    pub video_thumb_result: std::sync::Arc<std::sync::Mutex<Vec<(std::path::PathBuf, Option<egui::TextureHandle>)>>>,
    /// Активний повноекранний відеоплеєр (якщо відео відкрите).
    pub video_player: Option<crate::gui::gallery::video_player::VideoPlayer>,
    /// Текст промту, який зараз показується у popup-вікні галереї. None = вікно закрите.
    pub gallery_prompt_popup: Option<String>,
    /// Кешована статистика для текстового редактора сценарію (для оптимізації продуктивності).
    pub editor_stats: crate::gui::editor::EditorStats,
    /// Список задач з историчними налаштуваннями пайплайну.
    pub task_history: Vec<crate::gui::settings::storage::TaskHistoryEntry>,
    /// Результат фонової перевірки оновлень. None = ще не завершено або оновлень немає.
    pub update_info: std::sync::Arc<std::sync::Mutex<Option<crate::api::updater::UpdateInfo>>>,
    /// Чи відкрите вікно сповіщення про оновлення.
    pub update_dialog_open: bool,
    /// Чи згорнута нижня панель черги задач.
    pub queue_panel_collapsed: bool,
    /// Чи розгорнута черга на весь центральний екран.
    pub queue_panel_fullscreen: bool,
    /// ID задач, для яких вже було авто-перейдено на вкладку Галерея при MediaControl.
    pub media_control_notified: std::collections::HashSet<u64>,
    /// Стан відкритого вікна Stock Picker (тільки single mode з редактора монтажу).
    pub stock_picker_state: Option<crate::gui::stock_picker::StockPickerState>,
}

impl eframe::App for VideoMakerApp {
    /// Викликається кожного разу, коли інтерфейс потребує оновлення та перемальовування.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Перевірка статусів CLI для фонової перевірки
        self.poll_pending_tool_check();

        // Динамічно застосовуємо обрану тему оформлення та акцентний колір до поточного контексту
        crate::theme::apply_theme(ctx, self.theme, self.accent_color);

        self.draw_startup_windows(ctx);
        self.draw_topbar_windows(ctx);

        self.draw_main_side_panels(ctx);

        self.draw_queue_panel_host(ctx);
        self.handle_retry_request(ctx);

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
        let mut image_load_requests: Vec<std::path::PathBuf> = Vec::new();
        let mut prompt_view_request: Option<std::path::PathBuf> = None;
        let hover_loading_snapshot = self.video_hover_loading.lock().unwrap().clone();
        let thumb_loading_snapshot = self.video_thumb_loading.lock().unwrap().clone();
        let image_loading_snapshot = self.gallery_image_loading.lock().unwrap().clone();
        let regen_paths_snapshot = self.media_regen_paths.lock().unwrap().clone();
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                if self.draw_fullscreen_queue_if_needed(ui) {
                    return;
                }
                match self.active_tab {
                    Tab::Main => {
                        gui::editor::draw_editor(ui, &mut self.text_input, self.language, self.text_split_char_limit, &mut self.editor_stats);
                    }
                    Tab::Gallery => {
                        let switch_to_main = crate::gui::gallery::draw_gallery_tab(
                            ui, self.language, &self.jobs,
                            &mut self.gallery_textures,
                            &mut self.gallery_preview,
                            &regen_paths_snapshot,
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
                            &mut prompt_view_request,
                            &mut image_load_requests,
                            &image_loading_snapshot,
                        );
                        if switch_to_main {
                            self.active_tab = Tab::Main;
                        }
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

        self.handle_gallery_runtime(
            ctx,
            regen_action,
            prompt_view_request,
            hover_extract_request,
            thumb_requests,
            image_load_requests,
            animate_all,
        );

        self.draw_job_logs_windows(ctx);
        self.sync_control_windows();
        self.draw_stock_picker_window(ctx);
        self.draw_control_and_chat_windows(ctx);

        self.handle_montage_runtime(ctx, &regen_paths_snapshot);

        // АВТОЗБЕРЕЖЕННЯ:
        self.autosave_settings_if_needed(ctx);

        self.draw_gallery_preview_window(ctx);

        self.draw_copied_toast(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        crate::api::ffmpeg::ChildTracker::get().kill_all();
    }
}
