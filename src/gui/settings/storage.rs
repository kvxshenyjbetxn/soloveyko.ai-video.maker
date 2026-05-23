use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn default_true() -> bool { true }
fn default_video_service() -> String { "Googler".to_string() }
fn default_image_provider() -> String { "flow_IMAGEN_3_5".to_string() }
fn default_temperature() -> f32 { 0.7 }
fn default_openrouter_max_threads() -> usize { 5 }
fn default_claude_max_threads() -> usize { 5 }
fn default_gemini_max_threads() -> usize { 5 }
fn default_translation_service() -> String { "OpenRouter".to_string() }
fn default_show_welcome() -> bool { true }
fn default_model_claude() -> String { "sonnet".to_string() }
fn default_model_gemini() -> String { "gemini-2.5-flash".to_string() }
fn default_edge_tts_voice() -> String { "uk-UA-PolinaNeural".to_string() }
fn default_edge_tts_rate() -> String { "0".to_string() }
fn default_edge_tts_pitch() -> String { "0".to_string() }
fn default_edge_tts_volume() -> String { "0".to_string() }
fn default_edge_tts_max_threads() -> usize { 5 }
fn default_googler_threads() -> usize { 5 }

/// Очищає текстові параметри (темп, тональність, гучність), прибираючи відсотки, герци та інші букви
fn clean_numeric_param(s: &str) -> String {
    let cleaned: String = s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
        .collect();
    if cleaned.is_empty() {
        return "0".to_string();
    }
    if cleaned == "+0" || cleaned == "-0" || cleaned == "0" {
        return "0".to_string();
    }
    cleaned
}

/// Структура для серіалізації налаштувань у формат JSON.
///
/// Зберігає тему оформлення як рядок, колір акценту як масив [r, g, b, a], ширину бічної панелі та мову інтерфейсу.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    /// Поточна вибрана тема ("Light", "Dark", "Amoled")
    pub theme: String,
    /// Масив кольору акценту [r, g, b, a]
    pub accent_color: [u8; 4],
    /// Ширина бічної панелі пайплайну
    pub pipeline_width: f32,
    /// Поточна вибрана мова інтерфейсу ("Uk", "En")
    pub language: String,
    /// Ключ API для OpenRouter
    pub openrouter_key: String,
    /// Ключ API для Voice Bot
    pub voicebot_key: String,
    /// Ключ API для Googler
    pub googler_key: String,
    /// Поточний вибраний сервіс озвучки ("Voice Bot")
    pub voiceover_provider: String,
    /// UUID обраного шаблону озвучки
    pub voiceover_template_uuid: String,
    /// Назва останнього завантаженого шаблону пайплайну
    pub last_template: String,
    /// Чи увімкнено етап "Переклад" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_translation_enabled: bool,
    /// Чи увімкнено контроль перекладу у пайплайні
    #[serde(default)]
    pub pipeline_translation_control_enabled: bool,
    /// Чи увімкнено етап "Озвучка" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_voiceover_enabled: bool,
    /// Чи увімкнено етап "Відеоряд" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_video_enabled: bool,
    /// Чи увімкнено етап "Субтитри" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_subtitles_enabled: bool,
    /// Чи увімкнено етап "Монтаж" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_editing_enabled: bool,
    /// Промт для моделі перекладу
    pub translation_prompt: String,
    /// ID обраної активної моделі для перекладу (загальне поле)
    pub translation_model: String,
    /// Обрана модель OpenRouter
    #[serde(default)]
    pub translation_model_openrouter: String,
    /// Обрана модель Claude
    #[serde(default = "default_model_claude")]
    pub translation_model_claude: String,
    /// Обрана модель Gemini
    #[serde(default = "default_model_gemini")]
    pub translation_model_gemini: String,
    /// Обраний сервіс для генерації відеоряду ("Googler")
    #[serde(default = "default_video_service")]
    pub video_service: String,
    /// Обраний провайдер зображень для Googler ("flow", "flower", "grok", "openai")
    #[serde(default = "default_image_provider")]
    pub googler_image_provider: String,
    /// Температура моделі для перекладу (0.0 — 2.0)
    #[serde(default = "default_temperature")]
    pub translation_temperature: f32,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code")
    #[serde(default = "default_translation_service")]
    pub translation_service: String,
    /// Шлях збереження для macOS
    #[serde(default)]
    pub save_path_macos: String,
    /// Шлях збереження для Windows
    #[serde(default)]
    pub save_path_windows: String,
    /// Застаріле поле — читається лише для міграції зі старих конфігів, не записується
    #[serde(default, skip_serializing)]
    pub save_path: String,
    /// Максимальна кількість потоків для OpenRouter
    #[serde(default = "default_openrouter_max_threads")]
    pub openrouter_max_threads: usize,
    /// Максимальна кількість потоків для Claude Code
    #[serde(default = "default_claude_max_threads")]
    pub claude_max_threads: usize,
    /// Максимальна кількість потоків для Gemini CLI
    #[serde(default = "default_gemini_max_threads")]
    pub gemini_max_threads: usize,
    /// Чи показувати вікно привітання при наступному запуску
    #[serde(default = "default_show_welcome")]
    pub show_welcome: bool,
    /// Обраний голос для Edge TTS
    #[serde(default = "default_edge_tts_voice")]
    pub edge_tts_voice: String,
    /// Темп для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_rate")]
    pub edge_tts_rate: String,
    /// Тональність для Edge TTS (наприклад, "+0Hz")
    #[serde(default = "default_edge_tts_pitch")]
    pub edge_tts_pitch: String,
    /// Гучність для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_volume")]
    pub edge_tts_volume: String,
    /// Максимальна кількість потоків для Edge TTS
    #[serde(default = "default_edge_tts_max_threads")]
    pub edge_tts_max_threads: usize,
    /// Максимальна кількість потоків зображень Googler
    #[serde(default = "default_googler_threads")]
    pub googler_image_max_threads: usize,
    /// Максимальна кількість потоків відео Googler
    #[serde(default = "default_googler_threads")]
    pub googler_video_max_threads: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            accent_color: [0, 122, 255, 255],
            pipeline_width: 450.0,
            language: "Uk".to_string(),
            openrouter_key: String::new(),
            voicebot_key: String::new(),
            googler_key: String::new(),
            voiceover_provider: "Voice Bot".to_string(),
            voiceover_template_uuid: String::new(),
            last_template: String::new(),
            pipeline_translation_enabled: true,
            pipeline_translation_control_enabled: false,
            pipeline_voiceover_enabled: true,
            pipeline_video_enabled: true,
            pipeline_subtitles_enabled: true,
            pipeline_editing_enabled: true,
            translation_prompt: String::new(),
            translation_model: String::new(),
            translation_model_openrouter: String::new(),
            translation_model_claude: "sonnet".to_string(),
            translation_model_gemini: "gemini-2.5-flash".to_string(),
            video_service: "Googler".to_string(),
            googler_image_provider: "flow_IMAGEN_3_5".to_string(),
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            save_path_macos: String::new(),
            save_path_windows: String::new(),
            save_path: String::new(),
            openrouter_max_threads: 5,
            claude_max_threads: 5,
            gemini_max_threads: 5,
            show_welcome: true,
            edge_tts_voice: "uk-UA-PolinaNeural".to_string(),
            edge_tts_rate: "0".to_string(),
            edge_tts_pitch: "0".to_string(),
            edge_tts_volume: "0".to_string(),
            edge_tts_max_threads: 5,
            googler_image_max_threads: 5,
            googler_video_max_threads: 5,
        }
    }
}

/// Повертає кросплатформений шлях до папки конфігурації проєкту: <UserConfigDir>/Soloveyko.AI-Video.Maker
pub fn get_settings_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("Soloveyko.AI-Video.Maker");
        path
    })
}

/// Повертає повний шлях до файлу конфігурації settings.json
pub fn get_settings_path() -> Option<PathBuf> {
    get_settings_dir().map(|mut path| {
        path.push("settings.json");
        path
    })
}

/// Завантажує налаштування користувача з файлу settings.json.
/// 
/// Якщо файл не існує або пошкоджений, повертає налаштування за замовчуванням.
pub fn load_settings() -> AppSettings {
    if let Some(path) = get_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&content) {
                    settings.edge_tts_rate = clean_numeric_param(&settings.edge_tts_rate);
                    settings.edge_tts_pitch = clean_numeric_param(&settings.edge_tts_pitch);
                    settings.edge_tts_volume = clean_numeric_param(&settings.edge_tts_volume);
                    // Міграція з єдиного save_path у платформо-специфічні поля
                    if settings.save_path_macos.is_empty()
                        && settings.save_path_windows.is_empty()
                        && !settings.save_path.is_empty()
                    {
                        if cfg!(target_os = "macos") {
                            settings.save_path_macos = settings.save_path.clone();
                        } else {
                            settings.save_path_windows = settings.save_path.clone();
                        }
                    }
                    return settings;
                }
            }
        }
    }
    AppSettings::default()
}

/// Зберігає поточні налаштування користувача у файл settings.json.
/// 
/// Автоматично створює папку конфігурації проєкту, якщо вона ще не існує.
pub fn save_settings(settings: &AppSettings) {
    if let Some(dir) = get_settings_dir() {
        // Створюємо директорію, якщо вона відсутня
        let _ = fs::create_dir_all(&dir);
        
        if let Some(path) = get_settings_path() {
            if let Ok(json) = serde_json::to_string_pretty(settings) {
                let _ = fs::write(path, json);
            }
        }
    }
}

/// Відкриває папку налаштувань у системному файловому менеджері (Explorer / Finder / xdg-open).
pub fn open_settings_folder() {
    if let Some(dir) = get_settings_dir() {
        // Гарантуємо, що папка існує перед відкриттям
        let _ = fs::create_dir_all(&dir);
        
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("explorer").arg(dir).spawn();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(dir).spawn();
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = Command::new("xdg-open").arg(dir).spawn();
        }
    }
}

/// Структура, яка описує шаблон налаштувань пайплайну.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PipelineTemplate {
    /// Збережений API ключ для OpenRouter
    pub openrouter_key: String,
    /// Обраний провайдер озвучки
    pub voiceover_provider: String,
    /// UUID обраного шаблону озвучки
    pub voiceover_template_uuid: String,
    /// Чи увімкнено етап "Переклад"
    #[serde(default = "default_true")]
    pub pipeline_translation_enabled: bool,
    /// Чи увімкнено контроль перекладу
    #[serde(default)]
    pub pipeline_translation_control_enabled: bool,
    /// Чи увімкнено етап "Озвучка"
    #[serde(default = "default_true")]
    pub pipeline_voiceover_enabled: bool,
    /// Чи увімкнено етап "Відеоряд"
    #[serde(default = "default_true")]
    pub pipeline_video_enabled: bool,
    /// Чи увімкнено етап "Субтитри"
    #[serde(default = "default_true")]
    pub pipeline_subtitles_enabled: bool,
    /// Чи увімкнено етап "Монтаж"
    #[serde(default = "default_true")]
    pub pipeline_editing_enabled: bool,
    /// Промт для моделі перекладу
    pub translation_prompt: String,
    /// ID обраної моделі OpenRouter для перекладу
    pub translation_model: String,
    /// Обрана модель OpenRouter
    #[serde(default)]
    pub translation_model_openrouter: String,
    /// Обрана модель Claude
    #[serde(default = "default_model_claude")]
    pub translation_model_claude: String,
    /// Обрана модель Gemini
    #[serde(default = "default_model_gemini")]
    pub translation_model_gemini: String,
    /// Обраний сервіс для генерації відеоряду
    #[serde(default = "default_video_service")]
    pub video_service: String,
    /// Обраний провайдер зображень для Googler
    #[serde(default = "default_image_provider")]
    pub googler_image_provider: String,
    /// Температура моделі для перекладу (0.0 — 2.0)
    #[serde(default = "default_temperature")]
    pub translation_temperature: f32,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code")
    #[serde(default = "default_translation_service")]
    pub translation_service: String,
    /// Обраний голос для Edge TTS
    #[serde(default = "default_edge_tts_voice")]
    pub edge_tts_voice: String,
    /// Темп для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_rate")]
    pub edge_tts_rate: String,
    /// Тональність для Edge TTS (наприклад, "+0Hz")
    #[serde(default = "default_edge_tts_pitch")]
    pub edge_tts_pitch: String,
    /// Гучність для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_volume")]
    pub edge_tts_volume: String,
    /// Максимальна кількість потоків зображень Googler
    #[serde(default = "default_googler_threads")]
    pub googler_image_max_threads: usize,
    /// Максимальна кількість потоків відео Googler
    #[serde(default = "default_googler_threads")]
    pub googler_video_max_threads: usize,
}

/// Повертає шлях до підпапки templates всередині директорії налаштувань додатку.
pub fn get_templates_dir() -> Option<PathBuf> {
    get_settings_dir().map(|mut path| {
        path.push("templates");
        path
    })
}

/// Зберігає поточні налаштування пайплайну як шаблон у файл <name>.json.
#[allow(clippy::too_many_arguments)]
pub fn save_template(
    name: &str,
    openrouter_key: &str,
    voiceover_provider: &str,
    voiceover_template_uuid: &str,
    pipeline_translation_enabled: bool,
    pipeline_translation_control_enabled: bool,
    pipeline_voiceover_enabled: bool,
    pipeline_video_enabled: bool,
    pipeline_subtitles_enabled: bool,
    pipeline_editing_enabled: bool,
    translation_prompt: &str,
    translation_model: &str,
    translation_model_openrouter: &str,
    translation_model_claude: &str,
    translation_model_gemini: &str,
    video_service: &str,
    googler_image_provider: &str,
    translation_temperature: f32,
    translation_service: &str,
    edge_tts_voice: &str,
    edge_tts_rate: &str,
    edge_tts_pitch: &str,
    edge_tts_volume: &str,
    googler_image_max_threads: usize,
    googler_video_max_threads: usize,
) -> Result<(), std::io::Error> {
    if let Some(dir) = get_templates_dir() {
        fs::create_dir_all(&dir)?;

        let mut path = dir;
        path.push(format!("{}.json", name));

        let template = PipelineTemplate {
            openrouter_key: openrouter_key.to_string(),
            voiceover_provider: voiceover_provider.to_string(),
            voiceover_template_uuid: voiceover_template_uuid.to_string(),
            pipeline_translation_enabled,
            pipeline_translation_control_enabled,
            pipeline_voiceover_enabled,
            pipeline_video_enabled,
            pipeline_subtitles_enabled,
            pipeline_editing_enabled,
            translation_prompt: translation_prompt.to_string(),
            translation_model: translation_model.to_string(),
            translation_model_openrouter: translation_model_openrouter.to_string(),
            translation_model_claude: translation_model_claude.to_string(),
            translation_model_gemini: translation_model_gemini.to_string(),
            video_service: video_service.to_string(),
            googler_image_provider: googler_image_provider.to_string(),
            translation_temperature,
            translation_service: translation_service.to_string(),
            edge_tts_voice: edge_tts_voice.to_string(),
            edge_tts_rate: edge_tts_rate.to_string(),
            edge_tts_pitch: edge_tts_pitch.to_string(),
            edge_tts_volume: edge_tts_volume.to_string(),
            googler_image_max_threads,
            googler_video_max_threads,
        };

        let json = serde_json::to_string_pretty(&template)?;
        fs::write(path, json)?;
    }
    Ok(())
}

/// Завантажує налаштування пайплайну з шаблону за його назвою.
pub fn load_template(name: &str) -> Option<PipelineTemplate> {
    if let Some(dir) = get_templates_dir() {
        let mut path = dir;
        path.push(format!("{}.json", name));
        
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(mut template) = serde_json::from_str::<PipelineTemplate>(&content) {
                    template.edge_tts_rate = clean_numeric_param(&template.edge_tts_rate);
                    template.edge_tts_pitch = clean_numeric_param(&template.edge_tts_pitch);
                    template.edge_tts_volume = clean_numeric_param(&template.edge_tts_volume);
                    return Some(template);
                }
            }
        }
    }
    None
}

/// Сканує папку шаблонів та повертає список імен доступних шаблонів.
pub fn load_saved_templates() -> Vec<String> {
    let mut templates = Vec::new();
    if let Some(dir) = get_templates_dir() {
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            templates.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    templates.sort();
    templates
}
