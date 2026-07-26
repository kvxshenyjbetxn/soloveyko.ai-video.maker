use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn default_true() -> bool {
    true
}
fn default_upscale_quality() -> String {
    "balanced".to_string()
}
fn default_upscale_resolution() -> String {
    "1080p".to_string()
}

fn default_image_priority() -> Vec<String> {
    vec![
        "flow_nano_banana_pro".to_string(),
        "flow_nano_banana_2".to_string(),
        "flower".to_string(),
        "grok".to_string(),
        "openai".to_string(),
    ]
}
fn default_video_priority() -> Vec<String> {
    vec![
        "flow_fast".to_string(),
        "flower".to_string(),
        "grok".to_string(),
        "flow_omni_flash".to_string(),
        "flow_light".to_string(),
        "flow_ultra_light".to_string(),
        "flow_quality".to_string(),
    ]
}

/// Мапить старі та нові ключі провайдерів зображень до канонічних v6 назв.
fn canonical_image_provider_key(key: &str) -> Option<&'static str> {
    match key {
        // Старі v4/v5 ключі Flow більше не існують у v6,
        // тому мігруємо їх на найближчі підтримувані моделі.
        "flow_IMAGEN_3_5" | "flow_GEM_PIX_2" | "flow_nano_banana_pro" => {
            Some("flow_nano_banana_pro")
        }
        "flow_NARWHAL" | "flow_nano_banana_2" => Some("flow_nano_banana_2"),
        "flower" => Some("flower"),
        "grok" => Some("grok"),
        "openai" => Some("openai"),
        _ => None,
    }
}

/// Мапить старі та нові ключі провайдерів відео до канонічних v6 назв.
fn canonical_video_provider_key(key: &str) -> Option<&'static str> {
    match key {
        "flow" | "flow_fast" => Some("flow_fast"),
        "flower" => Some("flower"),
        "grok" => Some("grok"),
        "flow_omni_flash" => Some("flow_omni_flash"),
        "flow_light" => Some("flow_light"),
        "flow_ultra_light" => Some("flow_ultra_light"),
        "flow_quality" => Some("flow_quality"),
        _ => None,
    }
}

/// Додає ключ у список лише один раз, зберігаючи початковий порядок.
fn push_unique(list: &mut Vec<String>, key: &str) {
    if !list.iter().any(|item| item == key) {
        list.push(key.to_string());
    }
}

/// Мігрує збережені пріоритети провайдерів Googler на канонічні v6 ключі.
fn migrate_googler_provider_settings(
    image_priority: &mut Vec<String>,
    video_priority: &mut Vec<String>,
    video_disabled: &mut Vec<String>,
) {
    let mut migrated_image_priority = Vec::new();
    for key in image_priority.iter() {
        if let Some(canonical) = canonical_image_provider_key(key) {
            push_unique(&mut migrated_image_priority, canonical);
        }
    }
    for key in default_image_priority() {
        push_unique(&mut migrated_image_priority, &key);
    }
    *image_priority = migrated_image_priority;

    let mut migrated_video_priority = Vec::new();
    for key in video_priority.iter() {
        if let Some(canonical) = canonical_video_provider_key(key) {
            push_unique(&mut migrated_video_priority, canonical);
        }
    }
    for key in default_video_priority() {
        push_unique(&mut migrated_video_priority, &key);
    }
    *video_priority = migrated_video_priority;

    let mut migrated_video_disabled = Vec::new();
    for key in video_disabled.iter() {
        if let Some(canonical) = canonical_video_provider_key(key) {
            push_unique(&mut migrated_video_disabled, canonical);
        }
    }
    *video_disabled = migrated_video_disabled;
}
fn default_video_service() -> String {
    "Googler".to_string()
}
fn default_video_agent_mode() -> String {
    "full".to_string()
}
fn default_video_llm_service() -> String {
    "None".to_string()
}
fn default_video_llm_model_claude() -> String {
    "sonnet".to_string()
}
fn default_video_llm_model_gemini() -> String {
    "gemini-2.5-flash".to_string()
}
fn default_text_split_mode() -> String {
    "paragraphs".to_string()
}
fn default_text_split_char_limit() -> usize {
    500
}
fn default_editor_segment_outlines_enabled() -> bool {
    true
}
fn default_temperature() -> f32 {
    0.7
}
fn default_openrouter_max_threads() -> usize {
    5
}
fn default_claude_max_threads() -> usize {
    5
}
fn default_gemini_max_threads() -> usize {
    5
}
fn default_translation_service() -> String {
    "OpenRouter".to_string()
}
fn default_show_welcome() -> bool {
    true
}
fn default_model_claude() -> String {
    "sonnet".to_string()
}
fn default_model_gemini() -> String {
    "gemini-2.5-flash".to_string()
}
fn default_model_codex() -> String {
    "gpt-5.4-mini".to_string()
}
fn default_video_llm_model_codex() -> String {
    "gpt-5.4-mini".to_string()
}
fn default_codex_max_threads() -> usize {
    5
}
fn default_model_agy() -> String {
    "gemini-3.5-flash".to_string()
}
fn default_agy_max_threads() -> usize {
    5
}
fn default_model_pi() -> String {
    "gemini-2.5-flash".to_string()
}
fn default_pi_max_threads() -> usize {
    5
}
fn default_edge_tts_voice() -> String {
    "uk-UA-PolinaNeural".to_string()
}
fn default_edge_tts_rate() -> String {
    "0".to_string()
}
fn default_edge_tts_pitch() -> String {
    "0".to_string()
}
fn default_edge_tts_volume() -> String {
    "0".to_string()
}
fn default_edge_tts_max_threads() -> usize {
    5
}
fn default_ffmpeg_max_threads() -> usize {
    2
}
fn default_googler_threads() -> usize {
    5
}
fn default_video_media_type() -> String {
    "image".to_string()
}
fn default_video_context_mode() -> String {
    "around".to_string()
}
fn default_video_context_chars() -> usize {
    500
}
fn default_subtitles_service() -> String {
    "Whisper".to_string()
}
fn default_assemblyai_key() -> String {
    String::new()
}
fn default_pexels_key() -> String {
    String::new()
}
fn default_magnific_key() -> String {
    String::new()
}
fn default_pixabay_key() -> String {
    String::new()
}
fn default_whisper_language() -> String {
    "auto".to_string()
}
fn default_whisper_model() -> String {
    "base".to_string()
}
fn default_whisper_max_line_width() -> usize {
    42
}
fn default_montage_service() -> String {
    "FFmpeg".to_string()
}
fn default_capcut_draft_path() -> String {
    String::new()
}
fn default_montage_fps() -> u32 {
    30
}
fn default_montage_preset() -> String {
    "medium".to_string()
}
fn default_montage_bitrate() -> u32 {
    8
}
fn default_montage_transition() -> String {
    "none".to_string()
}
fn default_montage_transition_duration() -> f32 {
    0.5
}
fn default_montage_image_zoom_intensity() -> f32 {
    0.5
}
fn default_montage_image_zoom_mode() -> String {
    "alternate".to_string()
}
fn default_montage_image_zoom_scale() -> f32 {
    1.3
}
fn default_montage_image_shake_intensity() -> f32 {
    0.5
}
fn default_subtitle_font_size() -> u32 {
    24
}
fn default_subtitle_color() -> [u8; 3] {
    [255, 255, 255]
}
fn default_subtitle_margin_v() -> u32 {
    30
}
fn default_subtitle_font() -> String {
    "Arial".to_string()
}
fn default_subtitle_karaoke_mode() -> u8 {
    0
}
fn default_subtitle_karaoke_highlight_color() -> [u8; 3] {
    [255, 255, 0]
}
fn default_subtitle_karaoke_outline_color() -> [u8; 3] {
    [0, 0, 0]
}
fn default_subtitle_karaoke_scale() -> u32 {
    120
}
fn default_preview_quality() -> String {
    "balanced".to_string()
}
fn default_preview_fps() -> f32 {
    30.0
}

/// Очищає текстові параметри (темп, тональність, гучність), прибираючи відсотки, герци та інші букви
fn clean_numeric_param(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
        .collect();
    if cleaned.is_empty() {
        return "0".to_string();
    }
    if cleaned == "+0" || cleaned == "-0" || cleaned == "0" {
        return "0".to_string();
    }
    cleaned
}

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
    /// Ключ API для Lumean.
    #[serde(alias = "voicebot_key")]
    pub lumean_key: String,
    /// Чи спрямовувати трафік Lumean через проксі.
    #[serde(default)]
    pub lumean_proxy_enabled: bool,
    /// URL HTTP-проксі для Lumean.
    #[serde(default)]
    pub lumean_proxy_url: String,
    /// Ключ API для Googler
    pub googler_key: String,
    /// Ключ API для AssemblyAI
    #[serde(default = "default_assemblyai_key")]
    pub assemblyai_key: String,
    /// Ключ API для Pexels Stock
    #[serde(default = "default_pexels_key")]
    pub pexels_key: String,
    /// Ключ API для Magnific Stock
    #[serde(default = "default_magnific_key")]
    pub magnific_key: String,
    /// Ключ API для Pixabay Stock
    #[serde(default = "default_pixabay_key")]
    pub pixabay_key: String,
    /// Поточний вибраний сервіс озвучки ("Lumean")
    pub voiceover_provider: String,
    /// UUID обраного шаблону озвучки
    pub voiceover_template_uuid: String,
    /// Назва останнього завантаженого шаблону пайплайну
    pub last_template: String,
    /// Чи увімкнено етап "Переклад" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_translation_enabled: bool,
    /// Чи увімкнено контроль перекладу у пайплайні
    #[serde(default)]
    pub pipeline_translation_control_enabled: bool,
    /// Чи відкривати вікно контролю автоматично при переході задачі в AwaitingControl
    #[serde(default)]
    pub pipeline_control_auto_open: bool,
    /// Чи увімкнено контроль зображень (пауза після відеоряду для перегляду)
    #[serde(default)]
    pub pipeline_media_control_enabled: bool,
    /// Чи увімкнено контроль монтажу (показує кнопку редактора монтажу в карточці)
    #[serde(default)]
    pub pipeline_montage_control_enabled: bool,
    /// Чи увімкнено етап "Озвучка" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_voiceover_enabled: bool,
    /// Чи увімкнено етап "Відеоряд" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_video_enabled: bool,
    /// Чи увімкнено етап "Субтитри" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_subtitles_enabled: bool,
    /// Чи увімкнено етап "Монтаж" у пайплайні
    #[serde(default = "default_true")]
    pub pipeline_editing_enabled: bool,
    /// Промт для моделі перекладу
    pub translation_prompt: String,
    /// ID обраної активної моделі для перекладу (загальне поле)
    pub translation_model: String,
    /// Обрана модель OpenRouter
    #[serde(default)]
    pub translation_model_openrouter: String,
    /// Обрана модель Claude
    #[serde(default = "default_model_claude")]
    pub translation_model_claude: String,
    /// Обрана модель Gemini
    #[serde(default = "default_model_gemini")]
    pub translation_model_gemini: String,
    /// Обрана модель Codex
    #[serde(default = "default_model_codex")]
    pub translation_model_codex: String,
    /// Обрана модель AGY
    #[serde(default = "default_model_agy")]
    pub translation_model_agy: String,
    /// Обрана модель Pi
    #[serde(default = "default_model_pi")]
    pub translation_model_pi: String,
    /// Обраний сервіс для генерації відеоряду ("Googler")
    #[serde(default = "default_video_service")]
    pub video_service: String,
    /// Режим нарізання тексту: "paragraphs" | "sentences" | "char_limit" | "full"
    #[serde(default = "default_text_split_mode")]
    pub text_split_mode: String,
    /// Збережений режим нарізання для не-CLI сервісів (відновлюється після переключення з агентів)
    #[serde(default = "default_text_split_mode")]
    pub text_split_mode_openrouter: String,
    /// Ліміт символів для режиму нарізання "char_limit"
    #[serde(default = "default_text_split_char_limit")]
    pub text_split_char_limit: usize,
    /// Чи показувати контури сегментів у редакторі сценарію
    #[serde(default = "default_editor_segment_outlines_enabled")]
    pub editor_segment_outlines_enabled: bool,
    /// Температура моделі для перекладу (0.0 — 2.0)
    #[serde(default = "default_temperature")]
    pub translation_temperature: f32,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code")
    #[serde(default = "default_translation_service")]
    pub translation_service: String,
    /// Шлях збереження для macOS
    #[serde(default)]
    pub save_path_macos: String,
    /// Шлях збереження для Windows
    #[serde(default)]
    pub save_path_windows: String,
    /// Чи увімкнено shared cache для завантажених stock-медіа
    #[serde(default)]
    pub shared_stock_cache_enabled: bool,
    /// Спільна папка кешу для завантажених stock-медіа
    #[serde(default)]
    pub shared_stock_cache_dir: String,
    /// Застаріле поле — читається лише для міграції зі старих конфігів, не записується
    #[serde(default, skip_serializing)]
    pub save_path: String,
    /// Максимальна кількість потоків для OpenRouter
    #[serde(default = "default_openrouter_max_threads")]
    pub openrouter_max_threads: usize,
    /// Максимальна кількість потоків для Claude Code
    #[serde(default = "default_claude_max_threads")]
    pub claude_max_threads: usize,
    /// Максимальна кількість потоків для Gemini CLI
    #[serde(default = "default_gemini_max_threads")]
    pub gemini_max_threads: usize,
    /// Максимальна кількість потоків для Codex CLI
    #[serde(default = "default_codex_max_threads")]
    pub codex_max_threads: usize,
    /// Максимальна кількість потоків для AGY CLI
    #[serde(default = "default_agy_max_threads")]
    pub agy_max_threads: usize,
    /// Максимальна кількість потоків для Pi CLI
    #[serde(default = "default_pi_max_threads")]
    pub pi_max_threads: usize,
    /// Чи показувати вікно привітання при наступному запуску
    #[serde(default = "default_show_welcome")]
    pub show_welcome: bool,
    /// Обраний голос для Edge TTS
    #[serde(default = "default_edge_tts_voice")]
    pub edge_tts_voice: String,
    /// Темп для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_rate")]
    pub edge_tts_rate: String,
    /// Тональність для Edge TTS (наприклад, "+0Hz")
    #[serde(default = "default_edge_tts_pitch")]
    pub edge_tts_pitch: String,
    /// Гучність для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_volume")]
    pub edge_tts_volume: String,
    /// Максимальна кількість потоків для Edge TTS
    #[serde(default = "default_edge_tts_max_threads")]
    pub edge_tts_max_threads: usize,
    /// Максимальна кількість одночасних процесів FFmpeg
    #[serde(default = "default_ffmpeg_max_threads")]
    pub ffmpeg_max_threads: usize,
    /// Максимальна кількість потоків зображень Googler
    #[serde(default = "default_googler_threads")]
    pub googler_image_max_threads: usize,
    /// Максимальна кількість потоків відео Googler
    #[serde(default = "default_googler_threads")]
    pub googler_video_max_threads: usize,
    /// Конвертувати аудіо в WAV після озвучки
    #[serde(default)]
    pub voiceover_convert_to_wav: bool,
    /// Промт для генерації зображень відеоряду
    #[serde(default)]
    pub video_prompt: String,
    /// Системна інструкція агенту для створення segments.json (лише для Claude Code / Gemini CLI)
    #[serde(default)]
    pub video_agent_prompt: String,
    /// Чи увімкнено поле стилю для генерації медіа
    #[serde(default)]
    pub video_style_enabled: bool,
    /// Промт стилю для генерації медіа (з плейсхолдером {{text}})
    #[serde(default)]
    pub video_style_prompt: String,
    /// Чи додавати контекст сценарію в API-промт відеоряду.
    #[serde(default)]
    pub video_context_enabled: bool,
    /// Режим контексту: "full" або "around".
    #[serde(default = "default_video_context_mode")]
    pub video_context_mode: String,
    /// Кількість символів контексту навколо сегмента для режиму "around".
    #[serde(default = "default_video_context_chars")]
    pub video_context_chars: usize,
    /// Підрежим агента відеоряду: "full" або "prompt_only"
    #[serde(default = "default_video_agent_mode")]
    pub video_agent_mode: String,
    /// Пріоритетний список провайдерів зображень
    #[serde(default = "default_image_priority")]
    pub googler_image_priority: Vec<String>,
    /// Пріоритетний список провайдерів відео
    #[serde(default = "default_video_priority")]
    pub googler_video_priority: Vec<String>,
    /// Вимкнені провайдери відео
    #[serde(default)]
    pub googler_video_disabled: Vec<String>,
    /// Тип медіа для генерації: "image" або "video"
    #[serde(default = "default_video_media_type")]
    pub video_media_type: String,
    /// Обраний сервіс для генерації субтитрів ("Whisper")
    #[serde(default = "default_subtitles_service")]
    pub subtitles_service: String,
    /// Мова розпізнавання для Whisper ("auto", "uk", "en", ...)
    #[serde(default = "default_whisper_language")]
    pub whisper_language: String,
    /// Модель Whisper ("tiny", "base", "small", "medium", "large-v3")
    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,
    /// Максимальна кількість символів у сегменті субтитрів (0 = без обмеження)
    #[serde(default = "default_whisper_max_line_width")]
    pub whisper_max_line_width: usize,
    /// Розмір шрифту субтитрів (пунктів)
    #[serde(default = "default_subtitle_font_size")]
    pub subtitle_font_size: u32,
    /// RGB колір тексту субтитрів [r, g, b]
    #[serde(default = "default_subtitle_color")]
    pub subtitle_color: [u8; 3],
    /// Вертикальний відступ субтитрів від нижнього краю (пікселів)
    #[serde(default = "default_subtitle_margin_v")]
    pub subtitle_margin_v: u32,
    /// Ефект karaoke: підсвічує слово яке проговорюється (тільки WhisperX/AssemblyAI)
    #[serde(default)]
    pub subtitle_karaoke: bool,
    /// Режим karaoke: 0 = fill (\kf), 1 = switch (\k), 2 = follow (тільки поточне слово)
    #[serde(default = "default_subtitle_karaoke_mode")]
    pub subtitle_karaoke_mode: u8,
    /// RGB колір слова що проговорюється
    #[serde(default = "default_subtitle_karaoke_highlight_color")]
    pub subtitle_karaoke_highlight_color: [u8; 3],
    /// RGB колір обводки субтитрів
    #[serde(default = "default_subtitle_karaoke_outline_color")]
    pub subtitle_karaoke_outline_color: [u8; 3],
    /// Жирний текст для karaoke субтитрів
    #[serde(default)]
    pub subtitle_karaoke_bold: bool,
    /// Масштаб поточного слова у % (режим follow, 100 = без змін)
    #[serde(default = "default_subtitle_karaoke_scale")]
    pub subtitle_karaoke_scale: u32,
    /// Назва шрифту для субтитрів (наприклад "Arial", "Impact")
    #[serde(default = "default_subtitle_font")]
    pub subtitle_font: String,
    /// Сервіс монтажу ("FFmpeg")
    #[serde(default = "default_montage_service")]
    pub montage_service: String,
    /// FPS для монтажу
    #[serde(default = "default_montage_fps")]
    pub montage_fps: u32,
    /// Пресет кодування FFmpeg (ultrafast, medium, slow, ...)
    #[serde(default = "default_montage_preset")]
    pub montage_preset: String,
    /// Бітрейт відео у МБ/с
    #[serde(default = "default_montage_bitrate")]
    pub montage_bitrate: u32,
    /// Тип переходу між кліпами ("none", "random", або конкретна назва xfade)
    #[serde(default = "default_montage_transition")]
    pub montage_transition: String,
    /// Тривалість переходу в секундах
    #[serde(default = "default_montage_transition_duration")]
    pub montage_transition_duration: f32,
    /// Чи увімкнено ефект зуму для зображень
    #[serde(default)]
    pub montage_image_zoom_enabled: bool,
    /// Інтенсивність зуму для зображень (0.1..1.0)
    #[serde(default = "default_montage_image_zoom_intensity")]
    pub montage_image_zoom_intensity: f32,
    /// Режим зуму: "alternate" (чергування) або "oscillate" (туди-сюди)
    #[serde(default = "default_montage_image_zoom_mode")]
    pub montage_image_zoom_mode: String,
    /// Кратність зуму (1.1..2.0)
    #[serde(default = "default_montage_image_zoom_scale")]
    pub montage_image_zoom_scale: f32,
    /// Чи увімкнено ефект покачування для зображень
    #[serde(default)]
    pub montage_image_shake_enabled: bool,
    /// Інтенсивність покачування для зображень (0.1..1.0)
    #[serde(default = "default_montage_image_shake_intensity")]
    pub montage_image_shake_intensity: f32,
    /// Генерувати CapCut-проект замість локального FFmpeg-монтажу
    #[serde(default)]
    pub capcut_enabled: bool,
    /// Шлях до кореневого каталогу чернеток CapCut
    #[serde(default = "default_capcut_draft_path")]
    pub capcut_draft_path: String,
    /// Сервіс ЛЛМ для генерації промтів відеоряду ("None", "OpenRouter", "Claude Code", "Gemini CLI")
    #[serde(default = "default_video_llm_service")]
    pub video_llm_service: String,
    /// Активна модель для генерації відео-промтів (залежить від сервісу)
    #[serde(default)]
    pub video_llm_model: String,
    /// Модель OpenRouter для відео-промтів
    #[serde(default)]
    pub video_llm_model_openrouter: String,
    /// Модель Claude для відео-промтів
    #[serde(default = "default_video_llm_model_claude")]
    pub video_llm_model_claude: String,
    /// Модель Gemini для відео-промтів
    #[serde(default = "default_video_llm_model_gemini")]
    pub video_llm_model_gemini: String,
    /// Модель Codex для відео-промтів
    #[serde(default = "default_video_llm_model_codex")]
    pub video_llm_model_codex: String,
    /// Модель AGY для відео-промтів
    #[serde(default = "default_model_agy")]
    pub video_llm_model_agy: String,
    /// Модель Pi для відео-промтів
    #[serde(default = "default_model_pi")]
    pub video_llm_model_pi: String,
    /// Температура ЛЛМ для відео-промтів (0.0 — 2.0)
    #[serde(default = "default_temperature")]
    pub video_llm_temperature: f32,
    /// Чи увімкнено тригери накладення медіа за ключовими фразами
    #[serde(default)]
    pub overlay_triggers_enabled: bool,
    /// Список тригерів накладення медіа
    #[serde(default)]
    pub overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
    /// Чи увімкнено автоматичний апскейл Googler відео
    #[serde(default)]
    pub googler_video_upscale_enabled: bool,
    /// Роздільна здатність апскейлу ("1080p", "2K", "4K")
    #[serde(default = "default_upscale_resolution")]
    pub googler_video_upscale_resolution: String,
    /// Якість апскейлу ("fast", "balanced", "max")
    #[serde(default = "default_upscale_quality")]
    pub googler_video_upscale_quality: String,
    /// Якість превʼю редактора монтажу — не впливає на фінальний рендер
    #[serde(default = "default_preview_quality")]
    pub preview_quality: String,
    /// FPS превʼю редактора монтажу — не впливає на фінальний рендер
    #[serde(default = "default_preview_fps")]
    pub preview_fps: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            accent_color: [0, 122, 255, 255],
            pipeline_width: 450.0,
            language: "Uk".to_string(),
            openrouter_key: String::new(),
            lumean_key: String::new(),
            lumean_proxy_enabled: false,
            lumean_proxy_url: String::new(),
            googler_key: String::new(),
            assemblyai_key: String::new(),
            pexels_key: String::new(),
            magnific_key: String::new(),
            pixabay_key: String::new(),
            voiceover_provider: "Lumean".to_string(),
            voiceover_template_uuid: String::new(),
            last_template: String::new(),
            pipeline_translation_enabled: true,
            pipeline_translation_control_enabled: false,
            pipeline_control_auto_open: false,
            pipeline_media_control_enabled: false,
            pipeline_montage_control_enabled: false,
            pipeline_voiceover_enabled: true,
            pipeline_video_enabled: true,
            pipeline_subtitles_enabled: true,
            pipeline_editing_enabled: true,
            translation_prompt: String::new(),
            translation_model: String::new(),
            translation_model_openrouter: String::new(),
            translation_model_claude: "sonnet".to_string(),
            translation_model_gemini: "gemini-2.5-flash".to_string(),
            translation_model_codex: "gpt-5.4-mini".to_string(),
            translation_model_agy: "default".to_string(),
            translation_model_pi: "gemini-2.5-flash".to_string(),
            video_service: "Googler".to_string(),
            text_split_mode: "paragraphs".to_string(),
            text_split_mode_openrouter: "paragraphs".to_string(),
            text_split_char_limit: 500,
            editor_segment_outlines_enabled: true,
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            save_path_macos: String::new(),
            save_path_windows: String::new(),
            shared_stock_cache_enabled: false,
            shared_stock_cache_dir: String::new(),
            save_path: String::new(),
            openrouter_max_threads: 5,
            claude_max_threads: 5,
            gemini_max_threads: 5,
            codex_max_threads: 5,
            agy_max_threads: 5,
            pi_max_threads: 5,
            show_welcome: true,
            edge_tts_voice: "uk-UA-PolinaNeural".to_string(),
            edge_tts_rate: "0".to_string(),
            edge_tts_pitch: "0".to_string(),
            edge_tts_volume: "0".to_string(),
            edge_tts_max_threads: 5,
            ffmpeg_max_threads: 2,
            googler_image_max_threads: 5,
            googler_video_max_threads: 5,
            voiceover_convert_to_wav: false,
            video_prompt: String::new(),
            video_agent_prompt: String::new(),
            video_style_enabled: false,
            video_style_prompt: String::new(),
            video_context_enabled: false,
            video_context_mode: default_video_context_mode(),
            video_context_chars: default_video_context_chars(),
            video_agent_mode: default_video_agent_mode(),
            googler_image_priority: default_image_priority(),
            googler_video_priority: default_video_priority(),
            googler_video_disabled: vec![],
            video_media_type: "image".to_string(),
            subtitles_service: "Whisper".to_string(),
            whisper_language: "auto".to_string(),
            whisper_model: "base".to_string(),
            whisper_max_line_width: 42,
            subtitle_font_size: 24,
            subtitle_color: [255, 255, 255],
            subtitle_margin_v: 30,
            subtitle_karaoke: false,
            subtitle_karaoke_mode: 0,
            subtitle_karaoke_highlight_color: [255, 255, 0],
            subtitle_karaoke_outline_color: [0, 0, 0],
            subtitle_karaoke_bold: false,
            subtitle_karaoke_scale: 120,
            subtitle_font: "Arial".to_string(),
            montage_service: "FFmpeg".to_string(),
            montage_fps: 30,
            montage_preset: "medium".to_string(),
            montage_bitrate: 8,
            montage_transition: "none".to_string(),
            montage_transition_duration: 0.5,
            montage_image_zoom_enabled: false,
            montage_image_zoom_intensity: 0.5,
            montage_image_zoom_mode: "alternate".to_string(),
            montage_image_zoom_scale: 1.3,
            montage_image_shake_enabled: false,
            montage_image_shake_intensity: 0.5,
            capcut_enabled: false,
            capcut_draft_path: String::new(),
            video_llm_service: "None".to_string(),
            video_llm_model: String::new(),
            video_llm_model_openrouter: String::new(),
            video_llm_model_claude: "sonnet".to_string(),
            video_llm_model_gemini: "gemini-2.5-flash".to_string(),
            video_llm_model_codex: "gpt-5.4-mini".to_string(),
            video_llm_model_agy: "default".to_string(),
            video_llm_model_pi: "gemini-2.5-flash".to_string(),
            video_llm_temperature: 0.7,
            overlay_triggers_enabled: false,
            overlay_triggers: vec![],
            googler_video_upscale_enabled: false,
            googler_video_upscale_resolution: "1080p".to_string(),
            googler_video_upscale_quality: "balanced".to_string(),
            preview_quality: default_preview_quality(),
            preview_fps: default_preview_fps(),
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
                if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&content) {
                    settings.edge_tts_rate = clean_numeric_param(&settings.edge_tts_rate);
                    settings.edge_tts_pitch = clean_numeric_param(&settings.edge_tts_pitch);
                    settings.edge_tts_volume = clean_numeric_param(&settings.edge_tts_volume);
                    migrate_lumean_provider(&mut settings.voiceover_provider);
                    // Міграція з єдиного save_path у платформо-специфічні поля
                    if settings.save_path_macos.is_empty()
                        && settings.save_path_windows.is_empty()
                        && !settings.save_path.is_empty()
                    {
                        if cfg!(target_os = "macos") {
                            settings.save_path_macos = settings.save_path.clone();
                        } else {
                            settings.save_path_windows = settings.save_path.clone();
                        }
                    }

                    migrate_googler_provider_settings(
                        &mut settings.googler_image_priority,
                        &mut settings.googler_video_priority,
                        &mut settings.googler_video_disabled,
                    );

                    return settings;
                }
            }
        }
    }
    AppSettings::default()
}

/// Замінює застарілу назву провайдера в існуючих налаштуваннях.
fn migrate_lumean_provider(provider: &mut String) {
    if provider == "Voice Bot" {
        *provider = "Lumean".to_string();
    }
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

/// Відкриває довільну папку у системному файловому менеджері (Explorer / Finder / xdg-open).
pub fn open_folder(path: &std::path::Path) {
    let _ = fs::create_dir_all(path);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}

/// Відкриває папку налаштувань у системному файловому менеджері (Explorer / Finder / xdg-open).
pub fn open_settings_folder() {
    if let Some(dir) = get_settings_dir() {
        open_folder(&dir);
    }
}

/// Структура, яка описує шаблон налаштувань пайплайну.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PipelineTemplate {
    /// Збережений API ключ для OpenRouter
    pub openrouter_key: String,
    /// Збережений API ключ для AssemblyAI
    #[serde(default = "default_assemblyai_key")]
    pub assemblyai_key: String,
    /// Збережений API ключ для Pexels Stock
    #[serde(default = "default_pexels_key")]
    pub pexels_key: String,
    /// Збережений API ключ для Magnific Stock
    #[serde(default = "default_magnific_key")]
    pub magnific_key: String,
    /// Збережений API ключ для Pixabay Stock
    #[serde(default = "default_pixabay_key")]
    pub pixabay_key: String,
    /// Обраний провайдер озвучки
    pub voiceover_provider: String,
    /// UUID обраного шаблону озвучки
    pub voiceover_template_uuid: String,
    /// Чи увімкнено етап "Переклад"
    #[serde(default = "default_true")]
    pub pipeline_translation_enabled: bool,
    /// Чи увімкнено контроль перекладу
    #[serde(default)]
    pub pipeline_translation_control_enabled: bool,
    /// Чи відкривати вікно контролю автоматично
    #[serde(default)]
    pub pipeline_control_auto_open: bool,
    /// Чи увімкнено контроль зображень
    #[serde(default)]
    pub pipeline_media_control_enabled: bool,
    /// Чи увімкнено контроль монтажу
    #[serde(default)]
    pub pipeline_montage_control_enabled: bool,
    /// Чи увімкнено етап "Озвучка"
    #[serde(default = "default_true")]
    pub pipeline_voiceover_enabled: bool,
    /// Чи увімкнено етап "Відеоряд"
    #[serde(default = "default_true")]
    pub pipeline_video_enabled: bool,
    /// Чи увімкнено етап "Субтитри"
    #[serde(default = "default_true")]
    pub pipeline_subtitles_enabled: bool,
    /// Чи увімкнено етап "Монтаж"
    #[serde(default = "default_true")]
    pub pipeline_editing_enabled: bool,
    /// Промт для моделі перекладу
    pub translation_prompt: String,
    /// ID обраної моделі OpenRouter для перекладу
    pub translation_model: String,
    /// Обрана модель OpenRouter
    #[serde(default)]
    pub translation_model_openrouter: String,
    /// Обрана модель Claude
    #[serde(default = "default_model_claude")]
    pub translation_model_claude: String,
    /// Обрана модель Gemini
    #[serde(default = "default_model_gemini")]
    pub translation_model_gemini: String,
    /// Обрана модель Codex
    #[serde(default = "default_model_codex")]
    pub translation_model_codex: String,
    /// Обрана модель AGY
    #[serde(default = "default_model_agy")]
    pub translation_model_agy: String,
    /// Обрана модель Pi
    #[serde(default = "default_model_pi")]
    pub translation_model_pi: String,
    /// Обраний сервіс для генерації відеоряду
    #[serde(default = "default_video_service")]
    pub video_service: String,
    /// Режим нарізання тексту
    #[serde(default = "default_text_split_mode")]
    pub text_split_mode: String,
    /// Збережений режим нарізання для не-CLI сервісів
    #[serde(default = "default_text_split_mode")]
    pub text_split_mode_openrouter: String,
    /// Ліміт символів для режиму нарізання "char_limit"
    #[serde(default = "default_text_split_char_limit")]
    pub text_split_char_limit: usize,
    /// Температура моделі для перекладу (0.0 — 2.0)
    #[serde(default = "default_temperature")]
    pub translation_temperature: f32,
    /// Чи увімкнено тригери накладення медіа за ключовими фразами
    #[serde(default)]
    pub overlay_triggers_enabled: bool,
    /// Список тригерів накладення медіа
    #[serde(default)]
    pub overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
    /// Обраний сервіс для перекладу ("OpenRouter" або "Claude Code")
    #[serde(default = "default_translation_service")]
    pub translation_service: String,
    /// Обраний голос для Edge TTS
    #[serde(default = "default_edge_tts_voice")]
    pub edge_tts_voice: String,
    /// Темп для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_rate")]
    pub edge_tts_rate: String,
    /// Тональність для Edge TTS (наприклад, "+0Hz")
    #[serde(default = "default_edge_tts_pitch")]
    pub edge_tts_pitch: String,
    /// Гучність для Edge TTS (наприклад, "+0%")
    #[serde(default = "default_edge_tts_volume")]
    pub edge_tts_volume: String,
    /// Максимальна кількість потоків зображень Googler
    #[serde(default = "default_googler_threads")]
    pub googler_image_max_threads: usize,
    /// Максимальна кількість потоків відео Googler
    #[serde(default = "default_googler_threads")]
    pub googler_video_max_threads: usize,
    /// Конвертувати аудіо в WAV після озвучки
    #[serde(default)]
    pub voiceover_convert_to_wav: bool,
    /// Промт для генерації зображень відеоряду
    #[serde(default)]
    pub video_prompt: String,
    /// Системна інструкція агенту для створення segments.json (лише для Claude Code / Gemini CLI)
    #[serde(default)]
    pub video_agent_prompt: String,
    /// Чи увімкнено поле стилю для генерації медіа
    #[serde(default)]
    pub video_style_enabled: bool,
    /// Промт стилю для генерації медіа (з плейсхолдером {{text}})
    #[serde(default)]
    pub video_style_prompt: String,
    /// Чи додавати контекст сценарію в API-промт відеоряду.
    #[serde(default)]
    pub video_context_enabled: bool,
    /// Режим контексту: "full" або "around".
    #[serde(default = "default_video_context_mode")]
    pub video_context_mode: String,
    /// Кількість символів контексту навколо сегмента для режиму "around".
    #[serde(default = "default_video_context_chars")]
    pub video_context_chars: usize,
    /// Підрежим агента відеоряду: "full" або "prompt_only"
    #[serde(default = "default_video_agent_mode")]
    pub video_agent_mode: String,
    /// Пріоритетний список провайдерів зображень
    #[serde(default = "default_image_priority")]
    pub googler_image_priority: Vec<String>,
    /// Пріоритетний список провайдерів відео
    #[serde(default = "default_video_priority")]
    pub googler_video_priority: Vec<String>,
    /// Вимкнені провайдери відео
    #[serde(default)]
    pub googler_video_disabled: Vec<String>,
    /// Тип медіа для генерації: "image" або "video"
    #[serde(default = "default_video_media_type")]
    pub video_media_type: String,
    /// Обраний сервіс для генерації субтитрів ("Whisper")
    #[serde(default = "default_subtitles_service")]
    pub subtitles_service: String,
    /// Мова розпізнавання для Whisper ("auto", "uk", "en", ...)
    #[serde(default = "default_whisper_language")]
    pub whisper_language: String,
    /// Модель Whisper ("tiny", "base", "small", "medium", "large-v3")
    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,
    /// Максимальна кількість символів у сегменті субтитрів (0 = без обмеження)
    #[serde(default = "default_whisper_max_line_width")]
    pub whisper_max_line_width: usize,
    /// Розмір шрифту субтитрів (пунктів)
    #[serde(default = "default_subtitle_font_size")]
    pub subtitle_font_size: u32,
    /// RGB колір тексту субтитрів [r, g, b]
    #[serde(default = "default_subtitle_color")]
    pub subtitle_color: [u8; 3],
    /// Вертикальний відступ субтитрів від нижнього краю (пікселів)
    #[serde(default = "default_subtitle_margin_v")]
    pub subtitle_margin_v: u32,
    /// Ефект karaoke: підсвічує слово яке проговорюється (тільки WhisperX/AssemblyAI)
    #[serde(default)]
    pub subtitle_karaoke: bool,
    /// Режим karaoke: 0 = fill (\kf), 1 = switch (\k), 2 = follow (тільки поточне слово)
    #[serde(default = "default_subtitle_karaoke_mode")]
    pub subtitle_karaoke_mode: u8,
    /// RGB колір слова що проговорюється
    #[serde(default = "default_subtitle_karaoke_highlight_color")]
    pub subtitle_karaoke_highlight_color: [u8; 3],
    /// RGB колір обводки субтитрів
    #[serde(default = "default_subtitle_karaoke_outline_color")]
    pub subtitle_karaoke_outline_color: [u8; 3],
    /// Жирний текст для karaoke субтитрів
    #[serde(default)]
    pub subtitle_karaoke_bold: bool,
    /// Масштаб поточного слова у % (режим follow, 100 = без змін)
    #[serde(default = "default_subtitle_karaoke_scale")]
    pub subtitle_karaoke_scale: u32,
    /// Назва шрифту для субтитрів (наприклад "Arial", "Impact")
    #[serde(default = "default_subtitle_font")]
    pub subtitle_font: String,
    /// Сервіс монтажу ("FFmpeg")
    #[serde(default = "default_montage_service")]
    pub montage_service: String,
    /// FPS для монтажу
    #[serde(default = "default_montage_fps")]
    pub montage_fps: u32,
    /// Пресет кодування FFmpeg
    #[serde(default = "default_montage_preset")]
    pub montage_preset: String,
    /// Бітрейт відео у МБ/с
    #[serde(default = "default_montage_bitrate")]
    pub montage_bitrate: u32,
    /// Тип переходу між кліпами ("none", "random", або конкретна назва xfade)
    #[serde(default = "default_montage_transition")]
    pub montage_transition: String,
    /// Тривалість переходу в секундах
    #[serde(default = "default_montage_transition_duration")]
    pub montage_transition_duration: f32,
    /// Чи увімкнено ефект зуму для зображень
    #[serde(default)]
    pub montage_image_zoom_enabled: bool,
    /// Інтенсивність зуму для зображень (0.1..1.0)
    #[serde(default = "default_montage_image_zoom_intensity")]
    pub montage_image_zoom_intensity: f32,
    /// Режим зуму: "alternate" (чергування) або "oscillate" (туди-сюди)
    #[serde(default = "default_montage_image_zoom_mode")]
    pub montage_image_zoom_mode: String,
    /// Кратність зуму (1.1..2.0)
    #[serde(default = "default_montage_image_zoom_scale")]
    pub montage_image_zoom_scale: f32,
    /// Чи увімкнено ефект покачування для зображень
    #[serde(default)]
    pub montage_image_shake_enabled: bool,
    /// Інтенсивність покачування для зображень (0.1..1.0)
    #[serde(default = "default_montage_image_shake_intensity")]
    pub montage_image_shake_intensity: f32,
    /// Генерувати CapCut-проект замість локального FFmpeg-монтажу
    #[serde(default)]
    pub capcut_enabled: bool,
    /// Шлях до кореневого каталогу чернеток CapCut
    #[serde(default = "default_capcut_draft_path")]
    pub capcut_draft_path: String,
    /// Сервіс ЛЛМ для генерації промтів відеоряду
    #[serde(default = "default_video_llm_service")]
    pub video_llm_service: String,
    /// Активна модель для відео-промтів
    #[serde(default)]
    pub video_llm_model: String,
    /// Модель OpenRouter для відео-промтів
    #[serde(default)]
    pub video_llm_model_openrouter: String,
    /// Модель Claude для відео-промтів
    #[serde(default = "default_video_llm_model_claude")]
    pub video_llm_model_claude: String,
    /// Модель Gemini для відево-промтів
    #[serde(default = "default_video_llm_model_gemini")]
    pub video_llm_model_gemini: String,
    /// Модель Codex для відео-промтів
    #[serde(default = "default_video_llm_model_codex")]
    pub video_llm_model_codex: String,
    /// Модель AGY для відео-промтів
    #[serde(default = "default_model_agy")]
    pub video_llm_model_agy: String,
    /// Модель Pi для відео-промтів
    #[serde(default = "default_model_pi")]
    pub video_llm_model_pi: String,
    /// Температура ЛЛМ для відео-промтів
    #[serde(default = "default_temperature")]
    pub video_llm_temperature: f32,
    /// Чи увімкнено автоматичний апскейл Googler відео
    #[serde(default)]
    pub googler_video_upscale_enabled: bool,
    /// Роздільна здатність апскейлу ("1080p", "2K", "4K")
    #[serde(default = "default_upscale_resolution")]
    pub googler_video_upscale_resolution: String,
    /// Якість апскейлу ("fast", "balanced", "max")
    #[serde(default = "default_upscale_quality")]
    pub googler_video_upscale_quality: String,
}

/// Повертає шлях до підпапки templates всередині директорії налаштувань додатку.
pub fn get_templates_dir() -> Option<PathBuf> {
    get_settings_dir().map(|mut path| {
        path.push("templates");
        path
    })
}

/// Зберігає поточні налаштування пайплайну як шаблон у файл <name>.json.
#[allow(clippy::too_many_arguments)]
pub fn save_template(
    name: &str,
    openrouter_key: &str,
    assemblyai_key: &str,
    pexels_key: &str,
    magnific_key: &str,
    pixabay_key: &str,
    voiceover_provider: &str,
    voiceover_template_uuid: &str,
    pipeline_translation_enabled: bool,
    pipeline_translation_control_enabled: bool,
    pipeline_control_auto_open: bool,
    pipeline_media_control_enabled: bool,
    pipeline_montage_control_enabled: bool,
    pipeline_voiceover_enabled: bool,
    pipeline_video_enabled: bool,
    pipeline_subtitles_enabled: bool,
    pipeline_editing_enabled: bool,
    translation_prompt: &str,
    translation_model: &str,
    translation_model_openrouter: &str,
    translation_model_claude: &str,
    translation_model_gemini: &str,
    translation_model_codex: &str,
    translation_model_agy: &str,
    translation_model_pi: &str,
    video_service: &str,
    text_split_mode: &str,
    text_split_mode_openrouter: &str,
    text_split_char_limit: usize,
    translation_temperature: f32,
    translation_service: &str,
    edge_tts_voice: &str,
    edge_tts_rate: &str,
    edge_tts_pitch: &str,
    edge_tts_volume: &str,
    googler_image_max_threads: usize,
    googler_video_max_threads: usize,
    voiceover_convert_to_wav: bool,
    video_prompt: &str,
    video_agent_mode: &str,
    video_agent_prompt: &str,
    video_style_enabled: bool,
    video_style_prompt: &str,
    video_context_enabled: bool,
    video_context_mode: &str,
    video_context_chars: usize,
    googler_image_priority: Vec<String>,
    googler_video_priority: Vec<String>,
    googler_video_disabled: Vec<String>,
    video_media_type: &str,
    subtitles_service: &str,
    whisper_language: &str,
    whisper_model: &str,
    whisper_max_line_width: usize,
    subtitle_font_size: u32,
    subtitle_color: [u8; 3],
    subtitle_margin_v: u32,
    subtitle_karaoke: bool,
    subtitle_karaoke_mode: u8,
    subtitle_karaoke_highlight_color: [u8; 3],
    subtitle_karaoke_outline_color: [u8; 3],
    subtitle_karaoke_bold: bool,
    subtitle_karaoke_scale: u32,
    subtitle_font: &str,
    montage_service: &str,
    montage_fps: u32,
    montage_preset: &str,
    montage_bitrate: u32,
    montage_transition: &str,
    montage_transition_duration: f32,
    montage_image_zoom_enabled: bool,
    montage_image_zoom_intensity: f32,
    montage_image_zoom_mode: &str,
    montage_image_zoom_scale: f32,
    montage_image_shake_enabled: bool,
    montage_image_shake_intensity: f32,
    capcut_enabled: bool,
    capcut_draft_path: &str,
    video_llm_service: &str,
    video_llm_model: &str,
    video_llm_model_openrouter: &str,
    video_llm_model_claude: &str,
    video_llm_model_gemini: &str,
    video_llm_model_codex: &str,
    video_llm_model_agy: &str,
    video_llm_model_pi: &str,
    video_llm_temperature: f32,
    overlay_triggers_enabled: bool,
    overlay_triggers: Vec<crate::core::pipeline::montage::OverlayTrigger>,
    googler_video_upscale_enabled: bool,
    googler_video_upscale_resolution: &str,
    googler_video_upscale_quality: &str,
) -> Result<(), std::io::Error> {
    if let Some(dir) = get_templates_dir() {
        fs::create_dir_all(&dir)?;

        let mut path = dir;
        path.push(format!("{}.json", name));

        let template = PipelineTemplate {
            openrouter_key: openrouter_key.to_string(),
            assemblyai_key: assemblyai_key.to_string(),
            pexels_key: pexels_key.to_string(),
            magnific_key: magnific_key.to_string(),
            pixabay_key: pixabay_key.to_string(),
            voiceover_provider: voiceover_provider.to_string(),
            voiceover_template_uuid: voiceover_template_uuid.to_string(),
            pipeline_translation_enabled,
            pipeline_translation_control_enabled,
            pipeline_control_auto_open,
            pipeline_media_control_enabled,
            pipeline_montage_control_enabled,
            pipeline_voiceover_enabled,
            pipeline_video_enabled,
            pipeline_subtitles_enabled,
            pipeline_editing_enabled,
            translation_prompt: translation_prompt.to_string(),
            translation_model: translation_model.to_string(),
            translation_model_openrouter: translation_model_openrouter.to_string(),
            translation_model_claude: translation_model_claude.to_string(),
            translation_model_gemini: translation_model_gemini.to_string(),
            translation_model_codex: translation_model_codex.to_string(),
            translation_model_agy: translation_model_agy.to_string(),
            translation_model_pi: translation_model_pi.to_string(),
            video_service: video_service.to_string(),
            text_split_mode: text_split_mode.to_string(),
            text_split_mode_openrouter: text_split_mode_openrouter.to_string(),
            text_split_char_limit,
            translation_temperature,
            translation_service: translation_service.to_string(),
            edge_tts_voice: edge_tts_voice.to_string(),
            edge_tts_rate: edge_tts_rate.to_string(),
            edge_tts_pitch: edge_tts_pitch.to_string(),
            edge_tts_volume: edge_tts_volume.to_string(),
            googler_image_max_threads,
            googler_video_max_threads,
            voiceover_convert_to_wav,
            video_prompt: video_prompt.to_string(),
            video_agent_mode: video_agent_mode.to_string(),
            video_agent_prompt: video_agent_prompt.to_string(),
            video_style_enabled,
            video_style_prompt: video_style_prompt.to_string(),
            video_context_enabled,
            video_context_mode: video_context_mode.to_string(),
            video_context_chars,
            googler_image_priority,
            googler_video_priority,
            googler_video_disabled,
            video_media_type: video_media_type.to_string(),
            subtitles_service: subtitles_service.to_string(),
            whisper_language: whisper_language.to_string(),
            whisper_model: whisper_model.to_string(),
            whisper_max_line_width,
            subtitle_font_size,
            subtitle_color,
            subtitle_margin_v,
            subtitle_karaoke,
            subtitle_karaoke_mode,
            subtitle_karaoke_highlight_color,
            subtitle_karaoke_outline_color,
            subtitle_karaoke_bold,
            subtitle_karaoke_scale,
            subtitle_font: subtitle_font.to_string(),
            montage_service: montage_service.to_string(),
            montage_fps,
            montage_preset: montage_preset.to_string(),
            montage_bitrate,
            montage_transition: montage_transition.to_string(),
            montage_transition_duration,
            montage_image_zoom_enabled,
            montage_image_zoom_intensity,
            montage_image_zoom_mode: montage_image_zoom_mode.to_string(),
            montage_image_zoom_scale,
            montage_image_shake_enabled,
            montage_image_shake_intensity,
            capcut_enabled,
            capcut_draft_path: capcut_draft_path.to_string(),
            video_llm_service: video_llm_service.to_string(),
            video_llm_model: video_llm_model.to_string(),
            video_llm_model_openrouter: video_llm_model_openrouter.to_string(),
            video_llm_model_claude: video_llm_model_claude.to_string(),
            video_llm_model_gemini: video_llm_model_gemini.to_string(),
            video_llm_model_codex: video_llm_model_codex.to_string(),
            video_llm_model_agy: video_llm_model_agy.to_string(),
            video_llm_model_pi: video_llm_model_pi.to_string(),
            video_llm_temperature,
            overlay_triggers_enabled,
            overlay_triggers,
            googler_video_upscale_enabled,
            googler_video_upscale_resolution: googler_video_upscale_resolution.to_string(),
            googler_video_upscale_quality: googler_video_upscale_quality.to_string(),
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
                if let Ok(mut template) = serde_json::from_str::<PipelineTemplate>(&content) {
                    template.edge_tts_rate = clean_numeric_param(&template.edge_tts_rate);
                    template.edge_tts_pitch = clean_numeric_param(&template.edge_tts_pitch);
                    template.edge_tts_volume = clean_numeric_param(&template.edge_tts_volume);
                    migrate_lumean_provider(&mut template.voiceover_provider);
                    migrate_googler_provider_settings(
                        &mut template.googler_image_priority,
                        &mut template.googler_video_priority,
                        &mut template.googler_video_disabled,
                    );
                    return Some(template);
                }
            }
        }
    }
    None
}

/// Запис в історії задач, що були додані в чергу.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(default)]
pub struct TaskHistoryEntry {
    /// Унікальний ідентифікатор задачі
    pub id: u64,
    /// Назва задачі
    pub name: String,
    /// Unix timestamp створення
    pub created_at: i64,
    /// Назва шаблону (якщо завантажувався)
    pub template_name: Option<String>,
    /// Текст сценарію на момент додавання задачі
    pub text: String,
    /// Які етапи були увімкнені (для відображення крапок в UI)
    pub stage_translation: bool,
    pub stage_voiceover: bool,
    pub stage_video: bool,
    pub stage_subtitles: bool,
    pub stage_editing: bool,
}

impl Default for TaskHistoryEntry {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            created_at: 0,
            template_name: None,
            text: String::new(),
            stage_translation: false,
            stage_voiceover: false,
            stage_video: false,
            stage_subtitles: false,
            stage_editing: false,
        }
    }
}

/// Повертає шлях до файлу history задач.
pub fn get_history_path() -> Option<PathBuf> {
    get_settings_dir().map(|mut p| {
        p.push("task_history.json");
        p
    })
}

/// Завантажує список задач з файлу task_history.json.
pub fn load_task_history() -> Vec<TaskHistoryEntry> {
    if let Some(path) = get_history_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(entries) = serde_json::from_str::<Vec<TaskHistoryEntry>>(&content) {
                    return entries;
                }
            }
        }
    }
    Vec::new()
}

/// Зберігає список задач у файл task_history.json.
pub fn save_task_history(entries: &[TaskHistoryEntry]) {
    if let Some(dir) = get_settings_dir() {
        let _ = fs::create_dir_all(&dir);
        if let Some(path) = get_history_path() {
            if let Ok(json) = serde_json::to_string_pretty(entries) {
                let _ = fs::write(path, json);
            }
        }
    }
}

/// Додає новий запис в кінець history, обрізає до 100 записів, зберігає на диск.
pub fn append_to_task_history(entries: &mut Vec<TaskHistoryEntry>, new_entry: TaskHistoryEntry) {
    entries.push(new_entry);
    if entries.len() > 100 {
        let drain = entries.len() - 100;
        entries.drain(0..drain);
    }
    save_task_history(entries);
}

/// Видаляє запис за індексом з history та зберігає.
pub fn remove_from_task_history(entries: &mut Vec<TaskHistoryEntry>, idx: usize) {
    if idx < entries.len() {
        entries.remove(idx);
        save_task_history(entries);
    }
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
