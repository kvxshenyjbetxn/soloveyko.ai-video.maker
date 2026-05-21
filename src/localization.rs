use std::fmt;

/// Підтримувані мови інтерфейсу програми.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    /// Українська мова
    Uk,
    /// Англійська мова
    En,
    /// Російська мова
    Ru,
}

impl Default for Language {
    fn default() -> Self {
        Self::Uk
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uk => write!(f, "Українська"),
            Self::En => write!(f, "English"),
            Self::Ru => write!(f, "Русский"),
        }
    }
}

/// Повертає локалізований рядок за його ключем та обраною мовою.
pub fn translate(lang: Language, key: &str) -> &'static str {
    match lang {
        Language::Uk => match key {
            "app_title" => "Soloveyko.ai Video Maker",
            "tab_main" => "Основна",
            "tab_settings" => "Налаштування",
            "pipeline_settings" => "Налаштування пайплайну",
            "templates" => "Шаблони",
            "storage" => "Шлях збереження",
            "api" => "АПІ",
            "translation" => "Переклад",
            "voiceover" => "Озвучка",
            "video" => "Відеоряд",
            "editing" => "Монтаж",
            "editor_hint" => "Введіть або вставте сюди текст вашого майбутнього відео сценарію...",
            "settings_general" => "Основні",
            "settings_general_title" => "Основні налаштування",
            "settings_theme" => "Тема оформлення",
            "settings_theme_desc" => "Виберіть колірну схему графічного інтерфейсу:",
            "settings_theme_light" => "Світла тема",
            "settings_theme_dark" => "Темна тема",
            "settings_theme_amoled" => "Чорна AMOLED тема",
            "settings_accent" => "Колір акценту",
            "settings_accent_desc" => "Виберіть колір виділень, активних елементів та навігації:",
            "settings_accent_quick" => "Швидкий вибір:",
            "settings_accent_custom" => "Власний колір з палітри:",
            "settings_data" => "Керування даними",
            "settings_data_desc" => "Відкрити локальну папку з файлом налаштувань settings.json:",
            "settings_open_folder" => "Відкрити папку користувача",
            "settings_lang" => "Мова інтерфейсу",
            "settings_lang_desc" => "Оберіть мову відображення програми:",
            "settings_lang_uk" => "Українська",
            "settings_lang_en" => "English",
            "settings_lang_ru" => "Русский",
            "color_blue" => "Синій",
            "color_green" => "Зелений",
            "color_red" => "Червоний",
            "color_orange" => "Помаранчевий",
            "color_purple" => "Фіолетовий",
            _ => "",
        },
        Language::En => match key {
            "app_title" => "Soloveyko.ai Video Maker",
            "tab_main" => "Main",
            "tab_settings" => "Settings",
            "pipeline_settings" => "Pipeline Settings",
            "templates" => "Templates",
            "storage" => "Storage Path",
            "api" => "API",
            "translation" => "Translation",
            "voiceover" => "Voiceover",
            "video" => "Video Sequence",
            "editing" => "Editing",
            "editor_hint" => "Enter or paste the text of your future video script here...",
            "settings_general" => "General",
            "settings_general_title" => "General Settings",
            "settings_theme" => "Theme",
            "settings_theme_desc" => "Select GUI color scheme:",
            "settings_theme_light" => "Light Theme",
            "settings_theme_dark" => "Dark Theme",
            "settings_theme_amoled" => "AMOLED Black Theme",
            "settings_accent" => "Accent Color",
            "settings_accent_desc" => "Select color for highlights, active elements, and navigation:",
            "settings_accent_quick" => "Quick select:",
            "settings_accent_custom" => "Custom color from palette:",
            "settings_data" => "Data Management",
            "settings_data_desc" => "Open local folder containing settings.json:",
            "settings_open_folder" => "Open User Folder",
            "settings_lang" => "Interface Language",
            "settings_lang_desc" => "Select application display language:",
            "settings_lang_uk" => "Ukrainian",
            "settings_lang_en" => "English",
            "settings_lang_ru" => "Russian",
            "color_blue" => "Blue",
            "color_green" => "Green",
            "color_red" => "Red",
            "color_orange" => "Orange",
            "color_purple" => "Purple",
            _ => "",
        },
        Language::Ru => match key {
            "app_title" => "Soloveyko.ai Video Maker",
            "tab_main" => "Основная",
            "tab_settings" => "Настройки",
            "pipeline_settings" => "Настройки пайплайна",
            "templates" => "Шаблоны",
            "storage" => "Путь сохранения",
            "api" => "API",
            "translation" => "Перевод",
            "voiceover" => "Озвучка",
            "video" => "Видеоряд",
            "editing" => "Монтаж",
            "editor_hint" => "Введите или вставьте сюда текст вашего будущего видеосценария...",
            "settings_general" => "Основные",
            "settings_general_title" => "Основные настройки",
            "settings_theme" => "Тема оформления",
            "settings_theme_desc" => "Выберите цветовую схему графического интерфейса:",
            "settings_theme_light" => "Светлая тема",
            "settings_theme_dark" => "Темная тема",
            "settings_theme_amoled" => "Черная AMOLED тема",
            "settings_accent" => "Цвет акцента",
            "settings_accent_desc" => "Выберите цвет выделений, активных элементов и навигации:",
            "settings_accent_quick" => "Быстрый выбор:",
            "settings_accent_custom" => "Собственный цвет из палитры:",
            "settings_data" => "Управление данными",
            "settings_data_desc" => "Открыть локальную папку с файлом настроек settings.json:",
            "settings_open_folder" => "Открыть папку пользователя",
            "settings_lang" => "Язык интерфейса",
            "settings_lang_desc" => "Выберите язык отображения программы:",
            "settings_lang_uk" => "Украинский",
            "settings_lang_en" => "Английский",
            "settings_lang_ru" => "Русский",
            "color_blue" => "Синий",
            "color_green" => "Зеленый",
            "color_red" => "Красный",
            "color_orange" => "Оранжевый",
            "color_purple" => "Фиолетовый",
            _ => "",
        }
    }
}
