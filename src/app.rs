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
    /// Шлях до папки збереження результатів пайплайну.
    pub save_path: String,
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
            save_path: String::new(),
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
            job_name_dialog_open: false,
            job_name_input: String::new(),
            openrouter_max_threads: 5,
            claude_max_threads: 5,
            gemini_max_threads: 5,
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
        let video_service = saved.video_service.clone();
        let googler_image_provider = saved.googler_image_provider.clone();
        let translation_temperature = saved.translation_temperature;
        let translation_service = saved.translation_service.clone();
        let save_path = saved.save_path.clone();
        let openrouter_max_threads = saved.openrouter_max_threads;
        let claude_max_threads = saved.claude_max_threads;
        let gemini_max_threads = saved.gemini_max_threads;

        // Налаштовуємо глобальний лімітер одночасних запитів OpenRouter
        crate::api::openrouter::OpenRouterLimiter::get().set_max_threads(openrouter_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Claude Code
        crate::api::claude::ClaudeLimiter::get().set_max_threads(claude_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Gemini CLI
        crate::api::gemini::GeminiLimiter::get().set_max_threads(gemini_max_threads);

        let saved_templates = crate::gui::settings::storage::load_saved_templates();

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
            save_path,
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            selected_job_logs: None,
            job_name_dialog_open: false,
            job_name_input: String::new(),
            openrouter_max_threads,
            claude_max_threads,
            gemini_max_threads,
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
                                    ui.label(format!("{} / {}", bal.img_threads_active, bal.img_threads_allowed));
                                    ui.end_row();
                                    ui.label(translate(language, "balance_video_threads"));
                                    ui.label(format!("{} / {}", bal.video_threads_active, bal.video_threads_allowed));
                                    ui.end_row();
                                });
                        }
                        None if googler_key.is_empty() => {
                            ui.label(egui::RichText::new(translate(language, "balance_no_key")).weak());
                        }
                        None => {
                            ui.label(egui::RichText::new(translate(language, "balance_not_loaded")).weak());
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
                    let (status_text, color) = match &status {
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

                            // Активні етапи — кожен з нового рядка
                            if job.settings.translation_enabled {
                                ui.label(
                                    egui::RichText::new(translate(language, "translation"))
                                        .weak()
                                        .size(10.0),
                                );
                            }
                            if job.settings.voiceover_enabled {
                                let src = if job.settings.translation_enabled {
                                    translate(language, "voiceover_text_source_translated")
                                } else {
                                    translate(language, "voiceover_text_source_original")
                                };
                                ui.label(
                                    egui::RichText::new(
                                        format!("{} ({})", translate(language, "voiceover"), src)
                                    )
                                    .weak()
                                    .size(10.0),
                                );
                            }

                            // Статус
                            ui.label(
                                egui::RichText::new(status_text)
                                    .color(color)
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
        // Динамічно застосовуємо обрану тему оформлення та акцентний колір до поточного контексту
        crate::theme::apply_theme(ctx, self.theme, self.accent_color);

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
                                bal.img_threads_active, bal.img_threads_allowed,
                                bal.video_threads_active, bal.video_threads_allowed,
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
        );

        // Відображаємо бічну панель пайплайну ТІЛЬКИ на вкладці "Основна"
        if self.active_tab == Tab::Main {
            // default_width передається лише як початкове значення при першому запуску.
            // egui::Memory зберігає ширину між кадрами сам — нічого читати назад не потрібно.
            egui::SidePanel::right("pipeline_panel")
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
                        &mut self.translation_model_search,
                        &self.openrouter_models,
                        &self.openrouter_models_loading,
                        &mut self.video_service,
                        &mut self.googler_image_provider,
                        &mut self.translation_temperature,
                        &mut self.translation_service,
                        &mut self.save_path,
                        &self.text_input,
                        &mut self.jobs,
                        &mut self.job_counter,
                        &mut self.queue_error,
                        &mut self.job_name_dialog_open,
                        &mut self.job_name_input,
                    );
                });
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
                        gui::settings::draw_settings(
                            ui,
                            &mut self.theme,
                            &mut self.accent_color,
                            &mut self.language,
                        );
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
                || self.googler_key != self.last_saved_settings.googler_key
                || self.video_service != self.last_saved_settings.video_service
                || self.googler_image_provider != self.last_saved_settings.googler_image_provider
                || (self.translation_temperature - self.last_saved_settings.translation_temperature).abs() > 0.001
                || self.translation_service != self.last_saved_settings.translation_service
                || self.save_path != self.last_saved_settings.save_path
                || self.openrouter_max_threads != self.last_saved_settings.openrouter_max_threads
                || self.claude_max_threads != self.last_saved_settings.claude_max_threads
                || self.gemini_max_threads != self.last_saved_settings.gemini_max_threads
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
                    video_service: self.video_service.clone(),
                    googler_image_provider: self.googler_image_provider.clone(),
                    translation_temperature: self.translation_temperature,
                    translation_service: self.translation_service.clone(),
                    save_path: self.save_path.clone(),
                    openrouter_max_threads: self.openrouter_max_threads,
                    claude_max_threads: self.claude_max_threads,
                    gemini_max_threads: self.gemini_max_threads,
                };
                
                // Зберігаємо оновлені налаштування у файл JSON на диску
                save_settings(&new_settings);
                
                // Оновлюємо копію останніх збережених параметрів у пам'яті
                self.last_saved_settings = new_settings;
            }
        }
    }
}
