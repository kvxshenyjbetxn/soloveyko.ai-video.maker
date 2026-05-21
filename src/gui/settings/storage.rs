use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            accent_color: [0, 122, 255, 255], // Стандартний синій колір
            pipeline_width: 450.0,            // Дефолтна ширина
            language: "Uk".to_string(),       // Стандартна мова — Українська
            openrouter_key: String::new(),
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
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
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
pub struct PipelineTemplate {
    /// Збережений API ключ для OpenRouter
    pub openrouter_key: String,
}

/// Повертає шлях до підпапки templates всередині директорії налаштувань додатку.
pub fn get_templates_dir() -> Option<PathBuf> {
    get_settings_dir().map(|mut path| {
        path.push("templates");
        path
    })
}

/// Зберігає поточні налаштування пайплайну як шаблон у файл <name>.json.
pub fn save_template(name: &str, openrouter_key: &str) -> Result<(), std::io::Error> {
    if let Some(dir) = get_templates_dir() {
        // Гарантуємо існування папки шаблонів
        fs::create_dir_all(&dir)?;
        
        let mut path = dir;
        path.push(format!("{}.json", name));
        
        let template = PipelineTemplate {
            openrouter_key: openrouter_key.to_string(),
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
                if let Ok(template) = serde_json::from_str::<PipelineTemplate>(&content) {
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
