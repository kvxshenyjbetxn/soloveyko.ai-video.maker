use eframe::egui;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

/// Структура для опису голосу Edge TTS (збережена для сумісності з UI)
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeTTSVoice {
    pub name: String,
    pub short_name: String,
    pub gender: String,
    pub locale: String,
    pub friendly_name: String,
}

/// Лімітер одночасних запитів до Edge TTS (семафор)
pub struct EdgeTTSLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl EdgeTTSLimiter {
    /// Повертає глобальний екземпляр лімітера
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<EdgeTTSLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| EdgeTTSLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }

    /// Встановлює максимальну кількість одночасних запитів
    pub fn set_max_threads(&self, max: usize) {
        let mut max_threads = self.max_threads.lock().unwrap();
        *max_threads = max;
        self.condvar.notify_all();
    }

    /// Отримує дозвіл на виконання запиту (блокує потік, якщо досягнуто ліміту)
    pub fn acquire(&self) -> EdgeTTSPermit<'_> {
        let mut active = self.active.lock().unwrap();
        loop {
            let mut max = *self.max_threads.lock().unwrap();
            if max == 0 {
                max = 1;
            }
            if *active < max {
                break;
            }
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        EdgeTTSPermit { limiter: self }
    }

    /// Звільняє один потік та сповіщає інші очікуючі
    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }

    /// Повертає кількість активних потоків
    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

/// Дозвіл на виконання запиту Edge TTS
pub struct EdgeTTSPermit<'a> {
    limiter: &'a EdgeTTSLimiter,
}

impl<'a> Drop for EdgeTTSPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Допоміжна функція для парсингу параметрів темпу, тональності та гучності в ціле число i32.
/// Очищає рядок від не-цифрових символів (крім знаків '+' та '-') і повертає число.
fn parse_param(s: &str) -> i32 {
    let cleaned: String = s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
        .collect();
    cleaned.parse::<i32>().unwrap_or(0)
}

/// Очищає довгу назву голосу Microsoft Edge TTS для відображення в інтерфейсі.
/// Перетворює "Microsoft Dmitry Online (Natural) - Russian (Russia)" на "Dmitry (Russian, ru-RU)"
fn clean_friendly_name(friendly_name: &str, locale: &str) -> String {
    // Прибираємо "Microsoft " на початку
    let clean_pref = friendly_name.strip_prefix("Microsoft ").unwrap_or(friendly_name);
    
    // Розділяємо за допомогою " - "
    let parts: Vec<&str> = clean_pref.split(" - ").collect();
    
    let voice_name = if !parts.is_empty() {
        let first_part = parts[0];
        // Прибираємо " Online..." або " Neural..." якщо вони є
        if let Some(idx) = first_part.find(" Online") {
            first_part[..idx].trim().to_string()
        } else if let Some(idx) = first_part.find(" Neural") {
            first_part[..idx].trim().to_string()
        } else {
            first_part.trim().to_string()
        }
    } else {
        clean_pref.trim().to_string()
    };

    let language_name = if parts.len() > 1 {
        let second_part = parts[1];
        // Прибираємо дужки з країною, наприклад "Russian (Russia)" -> "Russian"
        if let Some(idx) = second_part.find(" (") {
            second_part[..idx].trim().to_string()
        } else {
            second_part.trim().to_string()
        }
    } else {
        "".to_string()
    };

    if language_name.is_empty() {
        format!("{} ({})", voice_name, locale)
    } else {
        format!("{} ({}, {})", voice_name, language_name, locale)
    }
}

/// Фоново завантажує список голосів Edge TTS за допомогою бібліотеки msedge-tts
pub fn fetch_voices(
    result: Arc<Mutex<Option<Result<Vec<EdgeTTSVoice>, String>>>>,
    loading: Arc<Mutex<bool>>,
    ctx: egui::Context,
) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    std::thread::spawn(move || {
        let res = match msedge_tts::voice::get_voices_list() {
            Ok(voices) => {
                let edge_voices = voices
                    .into_iter()
                    .map(|v| {
                        let locale = v.locale.clone().unwrap_or_else(|| "en-US".to_string());
                        let raw_friendly = v.friendly_name.clone().unwrap_or_else(|| v.name.clone());
                        let friendly_name = clean_friendly_name(&raw_friendly, &locale);
                        EdgeTTSVoice {
                            name: v.name.clone(),
                            short_name: v.short_name.clone().unwrap_or_else(|| v.name.clone()),
                            gender: v.gender.clone().unwrap_or_else(|| "Unknown".to_string()),
                            locale,
                            friendly_name,
                        }
                    })
                    .collect();
                Ok(edge_voices)
            }
            Err(e) => Err(format!("Failed to fetch voice list: {}", e)),
        };

        *result.lock().unwrap() = Some(res);
        *loading.lock().unwrap() = false;
        ctx.request_repaint();
    });
}

/// Виконує синтез тексту в аудіо через стабільне API бібліотеки msedge-tts
pub fn synthesize(
    text: &str,
    voice: &str,
    rate: &str,
    pitch: &str,
    volume: &str,
    output_path: &str,
) -> Result<(), String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    use msedge_tts::tts::client::connect;
    use msedge_tts::tts::SpeechConfig;
    use std::io::Write;

    // Конфігуруємо запис відповідно до вимог msedge-tts
    let config = SpeechConfig {
        voice_name: voice.to_string(),
        audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        pitch: parse_param(pitch),
        rate: parse_param(rate),
        volume: parse_param(volume),
    };

    // Підключаємось до Edge TTS клієнта
    let mut client = connect().map_err(|e| format!("Failed to connect to Edge TTS: {}", e))?;

    // Синтезуємо текст в аудіо
    let audio = client
        .synthesize(text, &config)
        .map_err(|e| format!("Edge TTS synthesis error: {}", e))?;

    // Створюємо вихідний файл
    let mut file = std::fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    // Write audio bytes
    file.write_all(&audio.audio_bytes)
        .map_err(|e| format!("Failed to write audio to file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_tts_synthesize() {
        let text = "Привіт! Це тестове повідомлення для перевірки працездатності синтезу Microsoft Edge TTS за допомогою нової бібліотеки.";
        let voice = "uk-UA-PolinaNeural";
        let output_path = "target/test_output.mp3";

        // Видаляємо старий файл, якщо є
        let _ = std::fs::remove_file(output_path);

        let res = synthesize(text, voice, "0", "0", "0", output_path);
        assert!(res.is_ok(), "Синтез завершився з помилкою: {:?}", res);

        let metadata = std::fs::metadata(output_path);
        assert!(metadata.is_ok(), "Файл не створено");
        let size = metadata.unwrap().len();
        assert!(size > 0, "Створено порожній файл");
        println!("Audio file generated successfully: {} bytes", size);
    }
}
