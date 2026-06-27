use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Тайминг одного медіафайлу — виходить у segments.json.
#[derive(Serialize, Deserialize, Clone)]
pub struct SegmentTiming {
    pub index: usize,
    pub text: String,
    pub start_secs: f64,
    pub end_secs: f64,
    pub duration_secs: f64,
    /// Впевненість нечіткого збігу [0.0..1.0]; 0 — таймінг оцінений (EST).
    pub confidence: f64,
    /// Відносний шлях до медіафайлу (наприклад "media/0001.jpg").
    pub media: Option<String>,
    /// Початок обрізки відео (секунди); встановлюється в редакторі монтажу.
    #[serde(default)]
    pub trim_start: f64,
}

/// Маніфест синхронізації — зберігається як segments.json.
#[derive(Serialize, Deserialize)]
pub struct Timeline {
    pub total_duration_secs: f64,
    #[serde(default)]
    pub audio_start_secs: f64,
    pub segments: Vec<SegmentTiming>,
}

// ─── Внутрішні структури ────────────────────────────────────────────────────

struct SrtEntry {
    start: f64,
    end: f64,
    text: String,
}

/// Зв'язок між діапазоном символів оригінального потоку та часом SRT-запису.
struct CharToTime {
    char_start: usize,
    char_end: usize,
    time_start: f64,
    time_end: f64,
}

struct WordPos {
    text: String,
    start: usize, // абсолютний rune-індекс у потоці
    end: usize,
}

// ─── Парсинг SRT ─────────────────────────────────────────────────────────────

fn parse_srt_time(s: &str) -> Option<f64> {
    let (time_part, millis_str) = s.trim().split_once(',')?;
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let sec: f64 = parts[2].parse().ok()?;
    let ms: f64 = millis_str.parse::<f64>().ok()? / 1000.0;
    Some(h * 3600.0 + m * 60.0 + sec + ms)
}

fn parse_srt(content: &str) -> Vec<SrtEntry> {
    let content = content.replace("\r\n", "\n").replace('\r', "\n");
    let mut entries = Vec::new();

    for block in content.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            continue;
        }
        let time_line = lines[1];
        let Some(arrow) = time_line.find("-->") else {
            continue;
        };
        let start_str = time_line[..arrow].trim();
        let end_str = time_line[arrow + 3..].trim();
        let (Some(start), Some(end)) = (parse_srt_time(start_str), parse_srt_time(end_str)) else {
            continue;
        };
        let text = lines[2..].join(" ").trim().to_string();
        if text.is_empty() {
            continue;
        }
        entries.push(SrtEntry { start, end, text });
    }

    entries
}

// ─── Нормалізація тексту ─────────────────────────────────────────────────────

fn is_punctuation(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '{'
            | '|'
            | '}'
            | '~'
            | '。'
            | '！'
            | '？'
            | '、'
            | '，'
            | '；'
            | '：'
            | '\u{201C}'
            | '\u{201D}'
            | '\u{2018}'
            | '\u{2019}'
            | '【'
            | '】'
            | '（'
            | '）'
            | '…'
            | '·'
            | '\u{061F}'
            | '\u{060C}'
            | '\u{061B}'
    )
}

/// Нормалізує текст (нижній регістр, без пунктуації, без зайвих пробілів).
/// Повертає нормалізований рядок та маппінг: normalized_char_idx → original_char_idx.
fn normalize_text_with_mapping(text: &str) -> (String, Vec<usize>) {
    let runes: Vec<char> = text.chars().collect();
    let mut normalized = Vec::<char>::new();
    let mut mapping = Vec::<usize>::new();

    for (i, &r) in runes.iter().enumerate() {
        // Заміна ё/Ё
        let c = match r {
            'ё' => 'е',
            'Ё' => 'Е',
            other => other,
        };

        let normalized_c: char = if c == '-' || c == '—' || is_punctuation(c) || c.is_whitespace()
        {
            ' '
        } else {
            c.to_lowercase().next().unwrap_or(c)
        };

        if normalized_c == ' ' {
            // Пропускаємо початкові та подвійні пробіли
            if normalized.is_empty() || normalized.last() == Some(&' ') {
                continue;
            }
        }

        normalized.push(normalized_c);
        mapping.push(i);
    }

    // Обрізаємо кінцевий пробіл
    if normalized.last() == Some(&' ') {
        normalized.pop();
        mapping.pop();
    }

    (normalized.iter().collect(), mapping)
}

// ─── Побудова текстового потоку з SRT ────────────────────────────────────────

/// Конкатенує тексти SRT-записів у єдиний потік.
/// Повертає об'єднаний рядок та карту: діапазон символів → часовий діапазон.
fn build_text_stream(blocks: &[SrtEntry]) -> (String, Vec<CharToTime>) {
    let mut stream_chars = Vec::<char>::new();
    let mut time_map = Vec::<CharToTime>::new();
    let mut current_char = 0usize;

    for b in blocks {
        let text = b.text.trim();
        if text.is_empty() {
            continue;
        }

        if !stream_chars.is_empty() {
            stream_chars.push(' ');
            current_char += 1;
        }

        let b_chars: Vec<char> = text.chars().collect();
        let start_char = current_char;
        stream_chars.extend_from_slice(&b_chars);
        current_char += b_chars.len();

        time_map.push(CharToTime {
            char_start: start_char,
            char_end: current_char,
            time_start: b.start,
            time_end: b.end,
        });
    }

    (stream_chars.iter().collect(), time_map)
}

/// Перетворює позицію символу в оригінальному потоці на час (з інтерполяцією).
fn char_to_time_at(pos: usize, time_map: &[CharToTime]) -> f64 {
    if time_map.is_empty() {
        return 0.0;
    }
    for entry in time_map {
        if pos >= entry.char_start && pos < entry.char_end {
            let segment_len = entry.char_end - entry.char_start;
            let segment_dur = entry.time_end - entry.time_start;
            if segment_len > 0 {
                let ratio = (pos - entry.char_start) as f64 / segment_len as f64;
                return entry.time_start + ratio * segment_dur;
            }
            return entry.time_start;
        }
    }
    if pos >= time_map.last().map(|e| e.char_end).unwrap_or(0) {
        return time_map.last().map(|e| e.time_end).unwrap_or(0.0);
    }
    time_map.first().map(|e| e.time_start).unwrap_or(0.0)
}

// ─── Нечітке порівняння слів (Левенштейн) ────────────────────────────────────

fn levenshtein_distance(s1: &[char], s2: &[char]) -> usize {
    let n = s1.len();
    let m = s2.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..=n {
        d[i][0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
        }
    }
    d[n][m]
}

fn is_word_similar(s1: &str, s2: &str, threshold: f64) -> bool {
    if s1 == s2 {
        return true;
    }
    let c1: Vec<char> = s1.chars().collect();
    let c2: Vec<char> = s2.chars().collect();
    let dist = levenshtein_distance(&c1, &c2);
    let max_len = c1.len().max(c2.len());
    if max_len == 0 {
        return true;
    }
    dist as f64 / max_len as f64 <= threshold
}

// ─── Пошук сегменту в потоці ─────────────────────────────────────────────────

/// Нечіткий пошук сегменту в текстовому потоці.
/// Повертає (start_char, end_char, confidence) у char-індексах normalized-потоку.
/// start_from — з якої позиції починати пошук (не шукати раніше вже знайдених).
fn find_segment_in_stream(
    segment: &str,
    stream: &str,
    start_from: usize,
) -> (Option<usize>, Option<usize>, f64) {
    if segment.is_empty() {
        return (None, None, 0.0);
    }

    let stream_chars: Vec<char> = stream.chars().collect();
    if start_from >= stream_chars.len() {
        return (None, None, 0.0);
    }

    let target_words: Vec<&str> = segment.split_whitespace().collect();
    if target_words.is_empty() {
        return (None, None, 0.0);
    }

    // Будуємо список слів зі стріму (починаючи з start_from, з абсолютними індексами)
    let mut stream_words: Vec<WordPos> = Vec::new();
    let mut current_word = Vec::<char>::new();
    let mut word_start: Option<usize> = None;

    for i in start_from..stream_chars.len() {
        let c = stream_chars[i];
        if !c.is_whitespace() {
            if word_start.is_none() {
                word_start = Some(i);
            }
            current_word.push(c);
        } else if let Some(ws) = word_start {
            stream_words.push(WordPos {
                text: current_word.iter().collect(),
                start: ws,
                end: i,
            });
            current_word.clear();
            word_start = None;
        }
    }
    if let Some(ws) = word_start {
        stream_words.push(WordPos {
            text: current_word.iter().collect(),
            start: ws,
            end: stream_chars.len(),
        });
    }

    // Fallback: якщо слів у стрімі менше ніж у сегменті — пряме порівняння
    if stream_words.len() < target_words.len() {
        let stream_slice: String = stream_chars[start_from..].iter().collect();
        if let Some(idx) = stream_slice.find(segment) {
            let start = start_from + stream_slice[..idx].chars().count();
            let end = start + segment.chars().count();
            return (Some(start), Some(end), 1.0);
        }
        return (None, None, 0.0);
    }

    // Порогова впевненість: для дуже коротких сегментів вимагаємо точного співпадіння
    let threshold = if target_words.len() <= 2 { 1.0 } else { 0.60 };

    let mut best_start: Option<usize> = None;
    let mut best_end: Option<usize> = None;
    let mut max_confidence = 0.0f64;

    'outer: for i in 0..=(stream_words.len().saturating_sub(target_words.len())) {
        let mut match_count = 0usize;
        let mut last_word_idx: Option<usize> = None;
        let mut current_idx = i;

        for tw in &target_words {
            let lookahead = 6;
            let limit = (current_idx + lookahead).min(stream_words.len());
            let mut found = false;

            for j in current_idx..limit {
                if is_word_similar(&stream_words[j].text, tw, 0.4) {
                    match_count += 1;
                    last_word_idx = Some(j);
                    current_idx = j + 1;
                    found = true;
                    break;
                }
            }

            if current_idx >= stream_words.len() && !found {
                break;
            }
        }

        let confidence = match_count as f64 / target_words.len() as f64;
        if confidence >= threshold && confidence > max_confidence {
            max_confidence = confidence;
            best_start = Some(stream_words[i].start);
            best_end = last_word_idx
                .map(|li| stream_words[li].end)
                .or_else(|| Some(stream_words[i].end));

            if confidence >= 0.9 {
                break 'outer;
            }
        }
    }

    if max_confidence >= threshold {
        (best_start, best_end, max_confidence)
    } else {
        (None, None, 0.0)
    }
}

// ─── Збір медіафайлів ────────────────────────────────────────────────────────

/// Повертає відсортований список імен медіафайлів із папки.
fn collect_media_files(media_dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(media_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            let ext = s.rsplit('.').next().unwrap_or("").to_lowercase();
            matches!(
                ext.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "mp4" | "mov" | "avi" | "mkv" | "webm"
            )
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    files
}

// ─── Допоміжні структури ─────────────────────────────────────────────────────

struct SegmentMatch {
    start: f64,
    end: f64,
    confidence: f64,
}

struct ImageResult {
    start: f64,
    end: f64,
    duration: f64,
    seg_idx: usize,
    filename: String,
}

// ─── Головна функція синхронізації ──────────────────────────────────────────

/// Будує segments.json для завершеної задачі.
///
/// Алгоритм (портований з Go-програми-аналога):
/// 1. Парсить subtitle.srt у текстовий потік з часовою картою.
/// 2. Нормалізує текст сегментів і потоку для нечіткого порівняння.
/// 3. Нечітким пошуком (Левенштейн) знаходить позицію кожного сегменту в потоці.
/// 4. Отримує реальний часовий діапазон через інтерполяцію всередині SRT-записів.
/// 5. Для незнайдених сегментів — заповнення пропуску пропорційно до довжини тексту.
/// 6. Зберігає результат як segments.json і звіт sync_debug.txt.
pub fn build_timeline(
    save_dir: &Path,
    segments: &[String],
    audio_duration_secs: Option<f64>,
    task_label: &str,
) -> Result<(), String> {
    if segments.is_empty() {
        return Err("No segments to build timeline for".to_string());
    }

    let srt_path = save_dir.join("subtitle.srt");
    let srt_data = std::fs::read_to_string(&srt_path)
        .map_err(|_| "subtitle.srt not found — run subtitles stage first".to_string())?;

    let blocks = parse_srt(&srt_data);
    if blocks.is_empty() {
        return Err("SRT file parsed but has no entries".to_string());
    }

    // Визначаємо загальну тривалість
    let srt_end = blocks.last().map(|b| b.end).unwrap_or(0.0);
    let total_duration = audio_duration_secs
        .map(|d| d.max(srt_end))
        .unwrap_or(srt_end);

    if total_duration <= 0.0 {
        return Err("Cannot build timeline: audio duration is zero".to_string());
    }

    let (stream, time_map) = build_text_stream(&blocks);
    let (stream_norm, stream_mapping) = normalize_text_with_mapping(&stream);

    let anchor_threshold = 0.65f64;

    // Шукаємо кожен сегмент у нормалізованому потоці
    let mut matches: Vec<Option<SegmentMatch>> = (0..segments.len()).map(|_| None).collect();
    let mut last_search_start = 0usize;

    for (i, segment) in segments.iter().enumerate() {
        let (seg_norm, _) = normalize_text_with_mapping(segment);
        if seg_norm.is_empty() {
            continue;
        }

        let (sc_opt, ec_opt, confidence) =
            find_segment_in_stream(&seg_norm, &stream_norm, last_search_start);

        if let (Some(sc), Some(ec)) = (sc_opt, ec_opt) {
            if confidence >= anchor_threshold
                && sc < stream_mapping.len()
                && ec > 0
                && ec - 1 < stream_mapping.len()
            {
                let orig_start = stream_mapping[sc];
                let orig_end = stream_mapping[ec - 1] + 1;

                let start_time = char_to_time_at(orig_start, &time_map);
                let end_time = char_to_time_at(orig_end, &time_map);
                let end_time = if end_time <= start_time {
                    start_time + 0.5
                } else {
                    end_time
                };

                matches[i] = Some(SegmentMatch {
                    start: start_time,
                    end: end_time,
                    confidence,
                });
                last_search_start = ec;
            }
        }
    }

    // Заповнення пропусків (незнайдені сегменти)
    let mut final_timings: Vec<(f64, f64)> = Vec::new(); // (start, end) per segment
    let mut prev_valid_end = 0.0f64;
    let mut i = 0;

    while i < segments.len() {
        if let Some(ref m) = matches[i] {
            let start = m.start.max(prev_valid_end);
            let end = if m.end <= start { start + 0.5 } else { m.end };
            prev_valid_end = end;
            final_timings.push((start, end));
            i += 1;
        } else {
            // Знаходимо весь блок незнайдених сегментів
            let gap_start = i;
            let mut gap_end = i;
            while gap_end < segments.len() && matches[gap_end].is_none() {
                gap_end += 1;
            }

            let next_valid_start = if gap_end < segments.len() {
                matches[gap_end]
                    .as_ref()
                    .map(|m| m.start)
                    .unwrap_or(total_duration)
            } else {
                total_duration
            };

            let time_budget = (next_valid_start - prev_valid_end).max(0.0);
            let num_missing = gap_end - gap_start;

            // Розподіл пропорційно до довжини нормалізованого тексту
            let total_gap_chars: usize = (gap_start..gap_end)
                .map(|k| normalize_text_with_mapping(&segments[k]).0.chars().count())
                .sum::<usize>()
                .max(1);

            let mut cursor = prev_valid_end;
            for k in gap_start..gap_end {
                let seg_norm_len = normalize_text_with_mapping(&segments[k]).0.chars().count();
                let weight = if seg_norm_len == 0 {
                    1.0 / num_missing as f64
                } else {
                    seg_norm_len as f64 / total_gap_chars as f64
                };
                let dur = time_budget * weight;
                final_timings.push((cursor, cursor + dur));
                cursor += dur;
            }

            prev_valid_end = cursor;
            i = gap_end;
        }
    }

    // Перший тайминг починається з 0 (поглинаємо початкову тишу)
    if let Some(first) = final_timings.first_mut() {
        first.0 = 0.0;
    }
    // Усуваємо прогалини між сегментами: end[i] = start[i+1].
    // SRT-записи мають паузи між собою; без цього в FFmpeg-concat накопичена
    // прогалина зміщує відеоряд відносно аудіо на кожному кліпі.
    for i in 0..final_timings.len().saturating_sub(1) {
        let next_start = final_timings[i + 1].0;
        if final_timings[i].1 < next_start {
            final_timings[i].1 = next_start;
        }
    }
    // Останній тайминг закінчується точно на total_duration
    if let Some(last) = final_timings.last_mut() {
        last.1 = total_duration;
    }

    // ─── STRETCH / SPLIT / NORMAL ────────────────────────────────────────────
    // Зіставляємо таймінги сегментів з реальними медіафайлами.
    // STRETCH  — картинок менше ніж сегментів: об'єднуємо сегменти.
    // SPLIT    — картинок більше ніж сегментів: ділимо кожен сегмент.
    // NORMAL   — 1 картинка на 1 сегмент.

    let media_dir = save_dir.join("media");
    let visual_files = collect_media_files(&media_dir);
    let total_images = visual_files.len();
    let n_segs = final_timings.len();

    let mut results: Vec<ImageResult> = Vec::new();

    if n_segs == 0 {
        // нічого
    } else if total_images == 0 {
        // медіафайлів нема — один запис на сегмент
        for (i, &(s, e)) in final_timings.iter().enumerate() {
            results.push(ImageResult {
                start: s,
                end: e,
                duration: e - s,
                seg_idx: i,
                filename: String::new(),
            });
        }
    } else if total_images < n_segs {
        // STRETCH: менше картинок ніж сегментів
        let group_size = n_segs as f64 / total_images as f64;
        for i in 0..total_images {
            let start_idx = (i as f64 * group_size).floor() as usize;
            let end_idx = if i == total_images - 1 {
                n_segs
            } else {
                ((i + 1) as f64 * group_size).floor() as usize
            };
            if start_idx < n_segs {
                let s_time = final_timings[start_idx].0;
                let e_time = final_timings[(end_idx - 1).min(n_segs - 1)].1;
                results.push(ImageResult {
                    start: s_time,
                    end: e_time,
                    duration: e_time - s_time,
                    seg_idx: start_idx,
                    filename: visual_files.get(i).cloned().unwrap_or_default(),
                });
            }
        }
    } else {
        // SPLIT/NORMAL: 1 або більше картинок на сегмент
        let image_per_seg = (total_images / n_segs).max(1);
        for (seg_i, &(s, e)) in final_timings.iter().enumerate() {
            let sub_dur = (e - s) / image_per_seg as f64;
            for j in 0..image_per_seg {
                let img_idx = seg_i * image_per_seg + j;
                results.push(ImageResult {
                    start: s + j as f64 * sub_dur,
                    end: s + (j + 1) as f64 * sub_dur,
                    duration: sub_dur,
                    seg_idx: seg_i,
                    filename: visual_files.get(img_idx).cloned().unwrap_or_default(),
                });
            }
        }
    }

    // Вирівнюємо точну кількість до total_images (обрізаємо або доповнюємо)
    if total_images > 0 {
        if results.len() > total_images {
            results.truncate(total_images);
        } else if results.len() < total_images {
            let last_end = results.last().map(|r| r.end).unwrap_or(0.0);
            let rem = total_images - results.len();
            let dur = ((total_duration - last_end) / rem as f64).max(0.1);
            let base = results.len();
            for k in 0..rem {
                results.push(ImageResult {
                    start: last_end + k as f64 * dur,
                    end: last_end + (k + 1) as f64 * dur,
                    duration: dur,
                    seg_idx: n_segs.saturating_sub(1),
                    filename: visual_files.get(base + k).cloned().unwrap_or_default(),
                });
            }
        }
    }

    // ─── Будуємо SegmentTiming на кожен медіафайл ─────────────────────────
    let segment_timings: Vec<SegmentTiming> = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let text = segments.get(r.seg_idx).cloned().unwrap_or_default();
            let confidence = matches
                .get(r.seg_idx)
                .and_then(|m| m.as_ref())
                .map(|m| m.confidence)
                .unwrap_or(0.0);
            SegmentTiming {
                index: i + 1,
                text,
                start_secs: r.start,
                end_secs: r.end,
                duration_secs: r.duration,
                confidence,
                media: if r.filename.is_empty() {
                    None
                } else {
                    Some(format!("media/{}", r.filename))
                },
                trim_start: 0.0,
            }
        })
        .collect();

    let timeline = Timeline {
        total_duration_secs: total_duration,
        audio_start_secs: 0.0,
        segments: segment_timings.clone(),
    };

    let json = serde_json::to_string_pretty(&timeline).map_err(|e| format!("JSON error: {}", e))?;
    std::fs::write(save_dir.join("segments.json"), json)
        .map_err(|e| format!("Write error: {}", e))?;

    // ─── Debug-звіт ───────────────────────────────────────────────────────
    let _ = write_sync_debug(
        save_dir,
        segments,
        &matches,
        &results,
        &segment_timings,
        total_duration,
        task_label,
    );

    Ok(())
}

fn write_sync_debug(
    save_dir: &Path,
    segments: &[String],
    seg_matches: &[Option<SegmentMatch>],
    results: &[ImageResult],
    segment_timings: &[SegmentTiming],
    total_duration: f64,
    task_label: &str,
) -> Result<(), std::io::Error> {
    let fmt_time = |s: f64| -> String {
        let m = (s / 60.0) as u32;
        let sec = s % 60.0;
        let cs = ((sec.fract()) * 100.0).round() as u32;
        let sec = sec.floor() as u32;
        format!("{:02}:{:02}.{:02}", m, sec, cs)
    };

    let matched_count = seg_matches.iter().filter(|m| m.is_some()).count();
    let total_conf: f64 = seg_matches
        .iter()
        .filter_map(|m| m.as_ref())
        .map(|m| m.confidence)
        .sum();
    let avg_conf = if matched_count > 0 {
        (total_conf / matched_count as f64 * 100.0).round() as usize
    } else {
        0
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut report = String::new();
    let _ = writeln!(
        report,
        "===================================================================================================="
    );
    let _ = writeln!(report, "SYNCHRONIZATION DEBUG REPORT");
    let _ = writeln!(report, "Generated: {} (unix)", now);
    let _ = writeln!(report, "Task: {}", task_label);
    let _ = writeln!(
        report,
        "====================================================================================================\n"
    );
    let _ = writeln!(report, "SUMMARY");
    let _ = writeln!(report, "--------------------------------------------------");
    let _ = writeln!(report, "Total Segments: {}", segments.len());
    let _ = writeln!(report, "Final Visuals:  {}", results.len());
    let _ = writeln!(
        report,
        "Total Duration: {} ({:.2}s)",
        fmt_time(total_duration),
        total_duration
    );
    let _ = writeln!(report, "Avg Confidence: {}%\n", avg_conf);
    let _ = writeln!(report, "DETAILED SYNCHRONIZATION TABLE");
    let _ = writeln!(
        report,
        "===================================================================================================="
    );
    let _ = writeln!(
        report,
        "{:<5}{:<21}{:<21}{:<21}{:<9}{}",
        "#", "Image", "Display Time", "Subtitle Match", "Conf", "Text Segment"
    );
    let _ = writeln!(
        report,
        "----------------------------------------------------------------------------------------------------"
    );

    for (i, (r, st)) in results.iter().zip(segment_timings.iter()).enumerate() {
        let img_name = if r.filename.is_empty() {
            "n/a"
        } else {
            &r.filename
        };
        let display_time = format!("{} - {}", fmt_time(st.start_secs), fmt_time(st.end_secs));
        let (sub_match, conf_str) = seg_matches
            .get(r.seg_idx)
            .and_then(|m| m.as_ref())
            .map(|m| {
                (
                    format!("{} - {}", fmt_time(m.start), fmt_time(m.end)),
                    format!("{}%", (m.confidence * 100.0).round() as usize),
                )
            })
            .unwrap_or_else(|| ("EST".to_string(), "EST".to_string()));
        let text_preview: String = st.text.chars().take(60).collect();
        let _ = writeln!(
            report,
            "{:<5}{:<21}{:<21}{:<21}{:<9}{}",
            i + 1,
            img_name,
            display_time,
            sub_match,
            conf_str,
            text_preview
        );
    }

    std::fs::write(save_dir.join("sync_debug.txt"), report)
}
