/// Розбиває текст на частини за обраним режимом.
/// mode: "paragraphs" | "sentences" | "char_limit" | "full"
pub fn split_text(text: &str, mode: &str, char_limit: usize) -> Vec<String> {
    match mode {
        "sentences" => split_by_sentences(text),
        "char_limit" => split_by_char_limit(text, char_limit),
        "full" => {
            if text.trim().is_empty() {
                vec![]
            } else {
                vec![text.to_string()]
            }
        }
        _ => split_by_paragraphs(text), // "paragraphs" та за замовчуванням
    }
}

/// Розбиває на абзаци (подвійний перенос рядка, або одинарний якщо подвійного немає).
fn split_by_paragraphs(text: &str) -> Vec<String> {
    let chunks: Vec<String> = if text.contains("\n\n") {
        text.split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        text.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    chunks
}

/// Розбиває на речення по '.', '!', '?'.
/// Враховує що за знаком має йти пробіл або кінець тексту.
fn split_by_sentences(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        current.push(ch);

        if matches!(ch, '.' | '!' | '?') {
            let at_end = i + 1 >= len;
            let next_is_boundary =
                at_end || chars[i + 1] == ' ' || chars[i + 1] == '\n' || chars[i + 1] == '\r';

            if next_is_boundary {
                let sentence = current.trim().to_string();
                if !sentence.is_empty() {
                    results.push(sentence);
                }
                current = String::new();
            }
        }
        i += 1;
    }

    let remainder = current.trim().to_string();
    if !remainder.is_empty() {
        results.push(remainder);
    }

    results
}

/// Розбиває по ліміту символів, зупиняючись на знаку пунктуації або пробілі.
/// Не розриває слова.
fn split_by_char_limit(text: &str, limit: usize) -> Vec<String> {
    if limit == 0 {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text.trim_start();

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();
        if char_count <= limit {
            let chunk = remaining.trim().to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            break;
        }

        // Байтова позиція символу на межі ліміту
        let limit_byte = remaining
            .char_indices()
            .nth(limit)
            .map(|(b, _)| b)
            .unwrap_or(remaining.len());

        let slice = &remaining[..limit_byte];

        // Шукаємо останній знак пунктуації перед лімітом (включаємо його у чанк)
        let split_byte =
            slice
                .char_indices()
                .rev()
                .find_map(|(byte_idx, ch)| {
                    if matches!(ch, '.' | '!' | '?' | ',' | ';' | ':') {
                        Some(byte_idx + ch.len_utf8())
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    // Немає пунктуації — шукаємо пробіл (розрив між словами)
                    slice.char_indices().rev().find_map(|(byte_idx, ch)| {
                        if ch == ' ' { Some(byte_idx) } else { None }
                    })
                })
                .unwrap_or(limit_byte);

        let chunk = remaining[..split_byte].trim().to_string();
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        remaining = remaining[split_byte..].trim_start();
    }

    chunks
}
