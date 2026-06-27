/// Тригер — медіа, яке з'являється коли в субтитрах згадується певна фраза.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default, PartialEq)]
pub struct OverlayTrigger {
    /// Ключова фраза для пошуку в субтитрах
    pub phrase: String,
    /// Шлях до медіафайлу (зображення або відео)
    pub path: String,
    /// Позиція X у відео (пікселів від лівого краю)
    pub x: i32,
    /// Позиція Y у відео (пікселів від верхнього краю)
    pub y: i32,
    /// Ширина накладки (0 = повна ширина відео)
    pub w: i32,
    /// Висота накладки (0 = повна висота відео)
    pub h: i32,
    /// Явний час початку (секунди). None = автопошук по субтитрах.
    pub start_time: Option<f64>,
    /// Тривалість відображення (секунди). None = з тривалості медіафайлу або 3.0.
    pub duration: Option<f64>,
}

/// Слово із субтитрів з розрахованим тайм-кодом початку/кінця.
struct SubWord {
    text: String,
    start: f64,
}

/// Нормалізує текст для порівняння: нижній регістр, ё→е, тільки літери/цифри, одинарні пробіли.
/// Логіка ідентична Go-версії (utils/text.go → normalize).
fn normalize(s: &str) -> String {
    let lower = s.to_lowercase().replace('ё', "е");
    let mut result = String::with_capacity(lower.len());
    let mut last_space = true;
    for c in lower.chars() {
        if c.is_alphabetic() || c.is_ascii_digit() {
            result.push(c);
            last_space = false;
        } else if !last_space {
            result.push(' ');
            last_space = true;
        }
    }
    result.trim_end().to_string()
}

/// Відстань Левенштейна між двома рядками (без обмеження довжини, як у Go).
fn levenshtein(a: &[char], b: &[char]) -> usize {
    let n = a.len();
    let m = b.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[n][m]
}

/// Перевіряє схожість двох слів: dist/maxLen ≤ threshold.
/// Аналог Go IsWordSimilar(s1, s2, 0.4) → вимагає similarity ≥ 60%.
fn is_word_similar(s1: &str, s2: &str, threshold: f64) -> bool {
    if s1 == s2 {
        return true;
    }
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return true;
    }
    let dist = levenshtein(&a, &b);
    dist as f64 / max_len as f64 <= threshold
}

/// Конвертує рядок часу формату ASS (H:MM:SS.cc) у секунди.
fn ass_time_to_secs(t: &str) -> f64 {
    let parts: Vec<&str> = t.splitn(3, ':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let h: f64 = parts[0].parse().unwrap_or(0.0);
    let m: f64 = parts[1].parse().unwrap_or(0.0);
    let s: f64 = parts[2].parse().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

/// Конвертує рядок часу формату SRT (HH:MM:SS,mmm) у секунди.
fn srt_time_to_secs(t: &str) -> f64 {
    let t = t.replace(',', ".");
    let parts: Vec<&str> = t.splitn(3, ':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let h: f64 = parts[0].parse().unwrap_or(0.0);
    let m: f64 = parts[1].parse().unwrap_or(0.0);
    let s: f64 = parts[2].parse().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

/// Шукає час початку фрази у файлі субтитрів (.ass або .srt).
///
/// Алгоритм:
/// 1. Парсить субтитри у посекундні слова з тайм-кодами
/// 2. Нормалізує текст
/// 3. Нечіткий пошук фрази з порогом схожості 60% (або 100% для коротких фраз ≤2 слова)
/// 4. Повертає час початку першого збіжного слова, або None якщо не знайдено
pub fn find_text_timing(sub_path: &std::path::Path, phrase: &str) -> Option<f64> {
    let data = std::fs::read_to_string(sub_path).ok()?;
    let is_ass = sub_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("ass"))
        .unwrap_or(false);

    let phrase_norm = normalize(phrase);
    let target_words: Vec<&str> = phrase_norm.split_whitespace().collect();
    if target_words.is_empty() {
        return None;
    }

    let mut sub_words: Vec<SubWord> = Vec::new();

    if is_ass {
        // Парсинг ASS: рядки Dialogue: 0,H:MM:SS.cc,H:MM:SS.cc,Style,,0,0,0,,text
        for line in data.lines() {
            let line = line.trim();
            if !line.starts_with("Dialogue:") {
                continue;
            }
            let parts: Vec<&str> = line.splitn(10, ',').collect();
            if parts.len() < 10 {
                continue;
            }
            let start = ass_time_to_secs(parts[1].trim());
            let end = ass_time_to_secs(parts[2].trim());
            // Видаляємо ASS-теги з тексту
            let raw_text = parts[9];
            let text = remove_ass_tags(raw_text);
            let text = text
                .replace("\\N", " ")
                .replace("\\n", " ")
                .replace("\\h", " ");
            let clean = normalize(&text);
            let words: Vec<&str> = clean.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            let word_dur = (end - start) / words.len() as f64;
            for (i, w) in words.iter().enumerate() {
                sub_words.push(SubWord {
                    text: w.to_string(),
                    start: start + i as f64 * word_dur,
                });
            }
        }
    } else {
        // Парсинг SRT: рядки з " --> " є тайм-кодами
        let mut current_start = 0.0f64;
        let mut current_end = 0.0f64;
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(arrow_pos) = line.find(" --> ") {
                current_start = srt_time_to_secs(&line[..arrow_pos]);
                current_end = srt_time_to_secs(&line[arrow_pos + 5..]);
            } else if !line.chars().all(|c| c.is_ascii_digit()) && current_end > current_start {
                let clean = normalize(line);
                let words: Vec<&str> = clean.split_whitespace().collect();
                if words.is_empty() {
                    continue;
                }
                let word_dur = (current_end - current_start) / words.len() as f64;
                for (i, w) in words.iter().enumerate() {
                    sub_words.push(SubWord {
                        text: w.to_string(),
                        start: current_start + i as f64 * word_dur,
                    });
                }
            }
        }
    }

    let threshold = if target_words.len() <= 2 { 1.0 } else { 0.6 };
    let n = target_words.len();

    for i in 0..sub_words.len().saturating_sub(n) + 1 {
        let mut match_count = 0;
        let mut current_idx = i;
        let mut first_match_idx: Option<usize> = None;

        // Логіка ідентична Go: без break — всі збіги у вікні рахуються,
        // is_word_similar з threshold=0.4 → dist/maxLen ≤ 0.4 → similarity ≥ 60%.
        for tw in &target_words {
            let limit = (current_idx + 6).min(sub_words.len());
            for j in current_idx..limit {
                if is_word_similar(&sub_words[j].text, tw, 0.4) {
                    match_count += 1;
                    current_idx = j + 1;
                    if first_match_idx.is_none() {
                        first_match_idx = Some(j);
                    }
                }
            }
        }

        let similarity = match_count as f64 / n as f64;
        if similarity >= threshold {
            if let Some(idx) = first_match_idx {
                return Some(sub_words[idx].start);
            }
        }
    }

    None
}

/// Видаляє ASS-теги вигляду {..} з рядка.
fn remove_ass_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '{' => in_tag = true,
            '}' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}
