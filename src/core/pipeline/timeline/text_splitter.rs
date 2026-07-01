/// Діапазон сегмента у вихідному тексті.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange {
    pub byte_start: usize,
    pub byte_end: usize,
}

impl TextRange {
    pub fn slice<'a>(&self, text: &'a str) -> &'a str {
        &text[self.byte_start..self.byte_end]
    }
}

/// Розбиває текст на частини за обраним режимом.
/// mode: "paragraphs" | "sentences" | "char_limit" | "full"
pub fn split_text(text: &str, mode: &str, char_limit: usize) -> Vec<String> {
    match mode {
        "char_limit" if char_limit == 0 => vec![text.to_string()],
        "full" => {
            if text.trim().is_empty() {
                vec![]
            } else {
                vec![text.to_string()]
            }
        }
        _ => split_text_preview_ranges(text, mode, char_limit)
            .into_iter()
            .map(|range| range.slice(text).to_string())
            .collect(),
    }
}

/// Повертає реальні діапазони сегментів у вихідному тексті.
/// Це дозволяє UI підсвічувати місця розрізу точно так само, як їх побачить пайплайн.
pub fn split_text_preview_ranges(text: &str, mode: &str, char_limit: usize) -> Vec<TextRange> {
    match mode {
        "sentences" => split_by_sentences(text),
        "char_limit" => {
            if char_limit == 0 {
                split_full_range(text)
            } else {
                split_by_char_limit(text, char_limit)
            }
        }
        "full" => split_full_range(text),
        _ => split_by_paragraphs(text),
    }
}

fn split_full_range(text: &str) -> Vec<TextRange> {
    if text.trim().is_empty() {
        vec![]
    } else {
        vec![TextRange {
            byte_start: 0,
            byte_end: text.len(),
        }]
    }
}

/// Розбиває на абзаци (подвійний перенос рядка, або одинарний якщо подвійного немає).
fn split_by_paragraphs(text: &str) -> Vec<TextRange> {
    if text.contains("\n\n") {
        split_by_separator(text, "\n\n")
    } else {
        split_by_lines(text)
    }
}

fn split_by_separator(text: &str, separator: &str) -> Vec<TextRange> {
    let mut ranges = Vec::new();
    let mut start = 0;

    while let Some(rel_idx) = text[start..].find(separator) {
        let end = start + rel_idx;
        push_trimmed_range(text, start, end, &mut ranges);
        start = end + separator.len();
    }

    push_trimmed_range(text, start, text.len(), &mut ranges);
    ranges
}

fn split_by_lines(text: &str) -> Vec<TextRange> {
    let mut ranges = Vec::new();
    let mut start = 0;

    for (byte_idx, ch) in text.char_indices() {
        if ch == '\n' {
            push_trimmed_range(text, start, byte_idx, &mut ranges);
            start = byte_idx + ch.len_utf8();
        }
    }

    push_trimmed_range(text, start, text.len(), &mut ranges);
    ranges
}

/// Розбиває на речення по '.', '!', '?'.
/// Враховує що за знаком має йти пробіл або кінець тексту.
fn split_by_sentences(text: &str) -> Vec<TextRange> {
    let mut results = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let len = chars.len();
    let mut segment_start = 0;
    let mut i = 0;

    while i < len {
        let (byte_idx, ch) = chars[i];

        if matches!(ch, '.' | '!' | '?') {
            let at_end = i + 1 >= len;
            let next_is_boundary =
                at_end || chars[i + 1].1 == ' ' || chars[i + 1].1 == '\n' || chars[i + 1].1 == '\r';

            if next_is_boundary {
                let segment_end = if at_end { text.len() } else { chars[i + 1].0 };
                push_trimmed_range(text, segment_start, segment_end, &mut results);
                segment_start = segment_end;
            }
        }

        let _ = byte_idx;
        i += 1;
    }

    push_trimmed_range(text, segment_start, text.len(), &mut results);
    results
}

/// Розбиває по ліміту символів, зупиняючись на знаку пунктуації або пробілі.
/// Не розриває слова.
fn split_by_char_limit(text: &str, limit: usize) -> Vec<TextRange> {
    if limit == 0 {
        return split_full_range(text);
    }

    let mut chunks = Vec::new();
    let mut start = next_non_whitespace_byte(text, 0);

    while start < text.len() {
        let remaining = &text[start..];
        let char_count = remaining.chars().count();
        if char_count <= limit {
            push_trimmed_range(text, start, text.len(), &mut chunks);
            break;
        }

        let limit_byte = remaining
            .char_indices()
            .nth(limit)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(remaining.len());

        let slice = &remaining[..limit_byte];
        let slice_chars: Vec<(usize, char)> = slice.char_indices().collect();

        let split_byte =
            slice_chars
                .iter()
                .enumerate()
                .rev()
                .find_map(|(char_idx, (byte_idx, ch))| {
                    if matches!(ch, '.' | '!' | '?' | ',' | ';' | ':')
                        && !is_numeric_separator(&slice_chars, char_idx)
                    {
                        Some(*byte_idx + ch.len_utf8())
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    slice.char_indices().rev().find_map(|(byte_idx, ch)| {
                        if ch == ' ' { Some(byte_idx) } else { None }
                    })
                })
                .unwrap_or(limit_byte);

        let segment_end = start + split_byte;
        push_trimmed_range(text, start, segment_end, &mut chunks);

        let next_start = next_non_whitespace_byte(text, segment_end);
        if next_start <= start {
            break;
        }
        start = next_start;
    }

    chunks
}

fn push_trimmed_range(text: &str, start: usize, end: usize, ranges: &mut Vec<TextRange>) {
    if start >= end || start >= text.len() {
        return;
    }

    let slice = &text[start..end.min(text.len())];
    let trimmed_start = slice.len() - slice.trim_start().len();
    let trimmed_end = slice.trim_end().len();

    if trimmed_start >= trimmed_end {
        return;
    }

    ranges.push(TextRange {
        byte_start: start + trimmed_start,
        byte_end: start + trimmed_end,
    });
}

fn next_non_whitespace_byte(text: &str, from: usize) -> usize {
    if from >= text.len() {
        return text.len();
    }

    text[from..]
        .char_indices()
        .find_map(|(byte_idx, ch)| {
            if ch.is_whitespace() {
                None
            } else {
                Some(from + byte_idx)
            }
        })
        .unwrap_or(text.len())
}

fn is_numeric_separator(chars: &[(usize, char)], idx: usize) -> bool {
    idx > 0
        && idx + 1 < chars.len()
        && chars[idx - 1].1.is_ascii_digit()
        && chars[idx + 1].1.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::{split_text, split_text_preview_ranges};

    #[test]
    fn preview_ranges_match_sentence_split() {
        let text = "  Перше речення.  Друге речення!\nТретє речення?  ";

        let ranges = split_text_preview_ranges(text, "sentences", 0);
        let preview: Vec<String> = ranges
            .iter()
            .map(|range| range.slice(text).to_string())
            .collect();

        assert_eq!(preview, split_text(text, "sentences", 0));
    }

    #[test]
    fn char_limit_does_not_split_inside_numbers() {
        let text = "Это Андромеда. И примерно через 4,5 миллиарда лет она врежется в Млечный Путь.";

        let chunks = split_text(text, "char_limit", 40);

        assert!(chunks.iter().all(|chunk| !chunk.ends_with("4,")));
        assert!(chunks.iter().all(|chunk| !chunk.starts_with("5 миллиарда")));
        assert!(chunks.iter().any(|chunk| chunk.contains("4,5 миллиарда")));

        let ranges = split_text_preview_ranges(text, "char_limit", 40);
        let preview: Vec<String> = ranges
            .iter()
            .map(|range| range.slice(text).to_string())
            .collect();
        assert_eq!(preview, chunks);
    }
}
