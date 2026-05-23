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
    /// Обраний провайдер зображень для Googler.
    pub googler_image_provider: String,
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
            googler_image_provider: "flow_IMAGEN_3_5".to_string(),
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            balance_window_open: false,
            save_path_macos: String::new(),
            save_path_windows: String::new(),
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
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
            last_saved_settings: default_settings,
        }
    }
}

/// Малює компактний чіп з балансом. При наведенні підсвічується і змінює курсор.
fn draw_balance_chip(ui: &mut egui::Ui, prefix: &str, value: &str) -> egui::Response {
    let text = format!("{}: {}", prefix, value);
    let font_size = ui.text_style_height(&egui::TextStyle::Small);
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Monospace);
    let text_color = ui.visuals().text_color();

    let galley = ui.fonts(|f| f.layout_no_wrap(text, font_id, text_color));

    let padding = egui::vec2(6.0, 2.0);
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
        let googler_image_provider = saved.googler_image_provider.clone();
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
            googler_image_provider,
            translation_temperature,
            translation_service,
            balance_window_open: false,
            save_path_macos,
            save_path_windows,
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
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
            last_saved_settings: saved,
        }
    }

    /// Малює вкладку системних логів роботи додатку.
    fn draw_logs_tab(&self, ui: &mut egui::Ui) {
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
                        ui.ctx().copy_text(all_logs);
                    }
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
                            .stick_to_bottom(true)
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
                                        egui::Color32::from_rgb(220, 220, 220) // Світло-сірий для звичайного тексту в терміналі
                                    };
                                    
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        
                                        // Виводимо часову мітку приглушеним кольором
                                        if !time_part.is_empty() {
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(time_part)
                                                        .monospace()
                                                        .size(11.0)
                                                        .color(egui::Color32::from_gray(110))
                                                )
                                            );
                                        }
                                        
                                        // Виводимо повідомлення відповідним кольором
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(msg_part)
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(text_color)
                                            )
                                            .wrap()
                                        );
                                    });
                                    
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
) {
    ui.add_space(4.0);
    
    // Верхній рядок керування
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(translate(language, "queue_panel_title")).strong().size(13.0));
        ui.label(egui::RichText::new(format!("({})", jobs.len())).weak().size(11.0));

        let has_pending = jobs.iter().any(|j| {
            *j.status.lock().unwrap() == crate::queue::JobStatus::Pending
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_enabled(
                has_pending,
                egui::Button::new(egui::RichText::new(translate(language, "queue_run_btn")).strong()),
            ).clicked() {
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
                        ctx.clone(),
                    );
                }
            }
        });
    });

    ui.add_space(2.0);
    ui.separator();
    ui.add_space(2.0);

    // Список задач з горизонтальною прокруткою
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for job in jobs.iter() {
                    let status = job.status.lock().unwrap().clone();
                    let translation_stage = job.translation_stage.lock().unwrap().clone();
                    let voiceover_stage = job.voiceover_stage.lock().unwrap().clone();

                    let (status_text, status_color) = match &status {
                        crate::queue::JobStatus::Pending => (
                            translate(language, "queue_status_pending"),
                            ui.visuals().weak_text_color(),
                        ),
                        crate::queue::JobStatus::Running => (
                            translate(language, "queue_status_running"),
                            egui::Color32::from_rgb(255, 200, 0),
                        ),
                        crate::queue::JobStatus::Done => (
                            translate(language, "queue_status_done"),
                            egui::Color32::from_rgb(46, 204, 113),
                        ),
                        crate::queue::JobStatus::Failed(_) => (
                            translate(language, "queue_status_failed"),
                            egui::Color32::from_rgb(231, 76, 60),
                        ),
                    };

                    let avail_h = ui.available_height();

                    let response = ui.group(|ui| {
                        ui.set_width(180.0);
                        ui.set_min_height((avail_h - 6.0).max(40.0));

                        ui.vertical(|ui| {
                            if avail_h > 120.0 {
                                ui.add_space(4.0);
                            }

                            // Назва завдання
                            ui.label(egui::RichText::new(
                                format!("#{} {}", job.id + 1, &job.name)
                            ).strong().size(if avail_h > 100.0 { 12.0 } else { 11.0 }));

                            if avail_h > 80.0 {
                                ui.add_space(2.0);
                            }

                            // Активні етапи — кожен з нового рядка з кольором за статусом
                            if job.settings.translation_enabled {
                                // Етап перекладу: "Переклад" з кольором за stage статусом
                                ui.label(
                                    egui::RichText::new(translate(language, "translation"))
                                        .color(stage_color(&translation_stage, ui))
                                        .size(10.0),
                                );
                            } else if job.settings.voiceover_enabled {
                                // Переклад вимкнено, але озвучка увімкнена → показуємо "Оригінал" (завжди зелений)
                                ui.label(
                                    egui::RichText::new(translate(language, "voiceover_text_source_original"))
                                        .color(egui::Color32::from_rgb(46, 204, 113))
                                        .size(10.0),
                                );
                            }

                            if job.settings.voiceover_enabled {
                                // Етап озвучки: "Озвучка" з кольором за stage статусом
                                ui.label(
                                    egui::RichText::new(translate(language, "voiceover"))
                                        .color(stage_color(&voiceover_stage, ui))
                                        .size(10.0),
                                );
                            }

                            // Загальний статус задачі
                            ui.label(
                                egui::RichText::new(status_text)
                                    .color(status_color)
                                    .size(if avail_h > 100.0 { 11.0 } else { 10.0 }),
                            );

                            // Показуємо помилку (hover підказка)
                            if let crate::queue::JobStatus::Failed(err) = &status {
                                ui.label(
                                    egui::RichText::new("⚠ помилка")
                                        .color(egui::Color32::from_rgb(231, 76, 60))
                                        .size(10.0),
                                ).on_hover_text(err);
                            }

                            if avail_h > 120.0 {
                                ui.add_space(4.0);
                            }
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
                        *selected_job_logs = Some((job.id, job.name.clone()));
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
            let npm = self.tool_checks.npm.lock().unwrap().clone();
            let gemini = self.tool_checks.gemini.lock().unwrap().clone();
            let claude = self.tool_checks.claude.lock().unwrap().clone();

            let mut check_done = false;
            let mut needs_install = false;

            if service == "Gemini CLI" {
                match (&npm, &gemini) {
                    (crate::gui::welcome::ToolStatus::Checking, _) | (_, crate::gui::welcome::ToolStatus::Checking) => {
                        // Перевірка ще триває
                    }
                    (crate::gui::welcome::ToolStatus::NotInstalled, _) | (_, crate::gui::welcome::ToolStatus::NotInstalled) => {
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
        egui::TopBottomPanel::top("navigation_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(translate(self.language, "app_title"));
                ui.separator();
                ui.selectable_value(&mut self.active_tab, Tab::Main, translate(self.language, "tab_main"));
                ui.selectable_value(&mut self.active_tab, Tab::Settings, translate(self.language, "tab_settings"));
                ui.selectable_value(&mut self.active_tab, Tab::Logs, translate(self.language, "tab_logs"));

                // Баланс-чіпи з правого боку (RTL: перший доданий — крайній правий)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                                "img: {}/{} vid: {}/{} th: {}/{} {}/{}",
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
                        &mut self.googler_image_provider,
                        &mut self.translation_temperature,
                        &mut self.translation_service,
                        &mut self.save_path_macos,
                        &mut self.save_path_windows,
                        &mut self.googler_image_max_threads,
                        &mut self.googler_video_max_threads,
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
                .min_height(75.0)
                .default_height(90.0)
                .max_height(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    draw_queue_panel(ui, self.language, &mut self.jobs, &mut self.selected_job_logs);
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
                        gui::editor::draw_editor(ui, &mut self.text_input, self.language);
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
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for log_line in job_logs {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(log_line).monospace().size(11.0));
                                        });
                                    }
                                });
                            });
                    }
                });
            if !is_open {
                self.selected_job_logs = None;
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
                || self.googler_image_provider != self.last_saved_settings.googler_image_provider
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
                    googler_image_provider: self.googler_image_provider.clone(),
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
                    show_welcome: self.last_saved_settings.show_welcome,
                };
                
                // Зберігаємо оновлені налаштування у файл JSON на диску
                save_settings(&new_settings);
                
                // Оновлюємо копію останніх збережених параметрів у пам'яті
                self.last_saved_settings = new_settings;
            }
        }
    }
}
