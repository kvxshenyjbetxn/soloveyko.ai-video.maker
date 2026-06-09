/// Дефолтна системна інструкція для CLI-агента відеоряду.
/// Пояснює агенту формат вихідного JSON (timeline.json) та правила роботи.
/// До неї через два переноси рядка додається користувацький промт з UI.
///
/// ## Плейсхолдери (замінюються автоматично перед відправкою агенту)
///
/// - `{{srt}}`  — абсолютний шлях до файлу субтитрів (`<save_path>/subtitle.srt`)
/// - `{{path}}` — абсолютний шлях до вихідного файлу (`<save_path>/timeline.json`)
///
/// Заміна відбувається в `src/core/pipeline/mod.rs` (функція запуску агента відеоряду).
pub const VIDEO_AGENT_SYSTEM_PROMPT: &str = r#"На основі субтитрів нижче створи файл timeline.json.

Використай Bash або Write tool щоб записати файл за шляхом:
{{path}}

Структура JSON:
{
  "total_duration_secs": <end_secs останнього сегменту>,
  "segments": [
    {
      "index": 0,
      "text": "short visual scene description for image generation",
      "start_secs": 0.0,
      "end_secs": 4.5,
      "duration_secs": 4.5,
      "confidence": 1.0,
      "media": null
    }
  ]
}

Правила:
- text — короткий англійський опис сцени для зображення (не транскрипція)
- start_secs / end_secs з таймінгу субтитрів
- media завжди null
- ОБОВ'ЯЗКОВО запиши JSON у файл {{path}}, не виводь у чат
"#;
