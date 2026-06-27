use std::fmt;

pub mod en;
pub mod ru;
pub mod uk;

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
        Language::Uk => uk::translate_uk(key),
        Language::En => en::translate_en(key),
        Language::Ru => ru::translate_ru(key),
    }
}
