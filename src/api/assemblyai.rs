use std::sync::{Condvar, Mutex, OnceLock};

/// Лімітер одночасних запитів до AssemblyAI (семафор, фіксовано 5 потоків).
pub struct AssemblyAILimiter {
    active: Mutex<usize>,
    condvar: Condvar,
}

impl AssemblyAILimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<AssemblyAILimiter> = OnceLock::new();
        LIMITER.get_or_init(|| AssemblyAILimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
        })
    }

    pub fn acquire(&self) -> AssemblyAIPermit<'_> {
        let mut active = self.active.lock().unwrap();
        while *active >= 5 {
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        AssemblyAIPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }

    pub fn active_count(&self) -> usize {
        *self.active.lock().unwrap()
    }
}

pub struct AssemblyAIPermit<'a> {
    limiter: &'a AssemblyAILimiter,
}

impl<'a> Drop for AssemblyAIPermit<'a> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

/// Транскрибує аудіофайл через AssemblyAI.
/// Повертає SRT-рядок та JSON з word-level timestamps (для збереження як subtitle.json).
pub fn transcribe(
    key: &str,
    audio_path: &std::path::Path,
    language: &str,
    max_line_width: usize,
) -> Result<(String, serde_json::Value), String> {
    let _permit = AssemblyAILimiter::get().acquire();

    let audio_bytes = std::fs::read(audio_path)
        .map_err(|e| format!("Failed to read audio: {}", e))?;

    let upload_url = upload_audio(key, &audio_bytes)?;
    let transcript_id = create_transcript(key, &upload_url, language)?;
    let response = poll_transcript(key, &transcript_id)?;

    let words = response
        .get("words")
        .and_then(|w| w.as_array())
        .ok_or("AssemblyAI: no words in transcript response")?;

    let srt = words_to_srt(words, max_line_width);

    Ok((srt, response))
}

fn upload_audio(key: &str, bytes: &[u8]) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(300))
        .build();

    let response = agent
        .post("https://api.assemblyai.com/v2/upload")
        .set("Authorization", key)
        .set("Content-Type", "application/octet-stream")
        .send_bytes(bytes)
        .map_err(|e| format!("AssemblyAI upload error: {}", e))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("AssemblyAI upload parse error: {}", e))?;

    json.get("upload_url")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "AssemblyAI: no upload_url in response".to_string())
}

fn create_transcript(key: &str, audio_url: &str, language: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let mut body = serde_json::json!({
        "audio_url": audio_url,
        "punctuate": true,
        "format_text": true,
    });

    if language == "auto" {
        body["language_detection"] = serde_json::json!(true);
    } else {
        body["language_code"] = serde_json::json!(language);
    }

    let response = agent
        .post("https://api.assemblyai.com/v2/transcript")
        .set("Authorization", key)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("AssemblyAI create transcript error: {}", e))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("AssemblyAI transcript response parse error: {}", e))?;

    json.get("id")
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "AssemblyAI: no transcript id in response".to_string())
}

fn poll_transcript(key: &str, id: &str) -> Result<serde_json::Value, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let url = format!("https://api.assemblyai.com/v2/transcript/{}", id);

    loop {
        let response = agent
            .get(&url)
            .set("Authorization", key)
            .call()
            .map_err(|e| format!("AssemblyAI polling error: {}", e))?;

        let json: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("AssemblyAI poll parse error: {}", e))?;

        let status = json
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        match status {
            "completed" => return Ok(json),
            "error" => {
                let err = json
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                return Err(format!("AssemblyAI transcript failed: {}", err));
            }
            _ => {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    }
}

/// Конвертує масив слів AssemblyAI у SRT-рядок.
/// max_line_width: 0 = без обмеження (розбиває лише за кінцем речення), > 0 = ліміт символів.
fn words_to_srt(words: &[serde_json::Value], max_line_width: usize) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut segments: Vec<(u64, u64, String)> = Vec::new();

    let mut current_text = String::new();
    let mut seg_start: Option<u64> = None;
    let mut seg_end: u64 = 0;

    for word in words {
        let text = word.get("text").and_then(|t| t.as_str()).unwrap_or("");
        let start = word.get("start").and_then(|s| s.as_u64()).unwrap_or(0);
        let end = word.get("end").and_then(|e| e.as_u64()).unwrap_or(0);

        if seg_start.is_none() {
            seg_start = Some(start);
        }

        let appended = if current_text.is_empty() {
            text.to_string()
        } else {
            format!("{} {}", current_text, text)
        };

        let ends_sentence = current_text.ends_with('.') || current_text.ends_with('!') || current_text.ends_with('?');
        let would_overflow = max_line_width > 0 && appended.len() > max_line_width && !current_text.is_empty();

        if (ends_sentence || would_overflow) && !current_text.is_empty() {
            segments.push((seg_start.unwrap_or(start), seg_end, current_text.clone()));
            current_text = text.to_string();
            seg_start = Some(start);
            seg_end = end;
        } else {
            current_text = appended;
            seg_end = end;
        }
    }

    if !current_text.is_empty() {
        segments.push((seg_start.unwrap_or(0), seg_end, current_text));
    }

    let mut srt = String::new();
    for (i, (start_ms, end_ms, text)) in segments.iter().enumerate() {
        srt.push_str(&format!("{}\n", i + 1));
        srt.push_str(&format!("{} --> {}\n", ms_to_srt(*start_ms), ms_to_srt(*end_ms)));
        srt.push_str(text);
        srt.push_str("\n\n");
    }
    srt
}

/// Конвертує масив слів WhisperX у SRT-рядок.
/// Формат слів WhisperX: {word, start (секунди), end (секунди), score}.
/// max_line_width: 0 = без обмеження (розбиває лише за кінцем речення), > 0 = ліміт символів.
pub fn whisperx_words_to_srt(words: &[serde_json::Value], max_line_width: usize) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut segments: Vec<(u64, u64, String)> = Vec::new();
    let mut current_text = String::new();
    let mut seg_start: Option<u64> = None;
    let mut seg_end: u64 = 0;

    for word in words {
        let text = word.get("word").and_then(|t| t.as_str()).unwrap_or("").trim();
        if text.is_empty() { continue; }

        // WhisperX зберігає час у секундах (f64), переводимо у мілісекунди
        let start = word.get("start").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let end   = word.get("end").and_then(|e| e.as_f64()).unwrap_or(0.0);
        let start_ms = (start * 1000.0) as u64;
        let end_ms   = (end   * 1000.0) as u64;

        if seg_start.is_none() {
            seg_start = Some(start_ms);
        }

        let appended = if current_text.is_empty() {
            text.to_string()
        } else {
            format!("{} {}", current_text, text)
        };

        let ends_sentence = current_text.ends_with('.') || current_text.ends_with('!') || current_text.ends_with('?');
        let would_overflow = max_line_width > 0 && appended.len() > max_line_width && !current_text.is_empty();

        if (ends_sentence || would_overflow) && !current_text.is_empty() {
            segments.push((seg_start.unwrap_or(start_ms), seg_end, current_text.clone()));
            current_text = text.to_string();
            seg_start = Some(start_ms);
            seg_end = end_ms;
        } else {
            current_text = appended;
            seg_end = end_ms;
        }
    }

    if !current_text.is_empty() {
        segments.push((seg_start.unwrap_or(0), seg_end, current_text));
    }

    let mut srt = String::new();
    for (i, (start_ms, end_ms, text)) in segments.iter().enumerate() {
        srt.push_str(&format!("{}\n", i + 1));
        srt.push_str(&format!("{} --> {}\n", ms_to_srt(*start_ms), ms_to_srt(*end_ms)));
        srt.push_str(text);
        srt.push_str("\n\n");
    }
    srt
}

fn ms_to_srt(ms: u64) -> String {
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = total_secs / 3600;
    format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, millis)
}

/// Перевіряє AssemblyAI ключ через GET /v2/account.
/// Повертає опис статусу.
pub fn check_key(key: &str) -> String {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build();

    match agent
        .get("https://api.assemblyai.com/v2/account")
        .set("Authorization", key)
        .call()
    {
        Ok(_) => "✔ Ключ валідний".to_string(),
        Err(ureq::Error::Status(401, _)) => "❌ Невірний ключ (401)".to_string(),
        Err(ureq::Error::Status(code, _)) => format!("❌ Помилка ({})", code),
        Err(_) => "❌ Помилка мережі. Перевірте з'єднання.".to_string(),
    }
}
