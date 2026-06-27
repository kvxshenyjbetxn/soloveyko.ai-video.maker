use eframe::egui;

use super::{Tab, VideoMakerApp};
use crate::gui::settings::storage::{load_settings, AppSettings};
use crate::localization::Language;
use crate::theme::AppTheme;

impl Default for VideoMakerApp {
    fn default() -> Self {
        let default_settings = AppSettings::default();
        let app = Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme: AppTheme::Dark, // Сучасна темна тема за замовчуванням
            accent_color: egui::Color32::from_rgb(0, 122, 255), // Синій колір за замовчуванням
            pipeline_width: 450.0,
            language: Language::Uk,
            openrouter_key: String::new(),
            openrouter_status: None,
            template_name_input: String::new(),
            saved_templates: crate::gui::settings::storage::load_saved_templates(),
            template_status: None,
            voicebot_key: String::new(),
            voicebot_status: None,
            googler_key: String::new(),
            googler_status: None,
            googler_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            googler_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            assemblyai_key: String::new(),
            assemblyai_status: None,
            assemblyai_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pexels_key: String::new(),
            pexels_status: None,
            pexels_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pixabay_key: String::new(),
            pixabay_status: None,
            pixabay_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voiceover_provider: "Voice Bot".to_string(),
            voiceover_template_uuid: String::new(),
            voicebot_templates: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            voicebot_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pipeline_translation_enabled: true,
            pipeline_translation_control_enabled: false,
            pipeline_control_auto_open: false,
            pipeline_media_control_enabled: false,

            pipeline_montage_control_enabled: false,
            montage_editor_open_job: None,
            montage_editor_state: None,
            gallery_textures: std::collections::HashMap::new(),
            gallery_image_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            gallery_image_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            gallery_preview: None,
            gallery_anim_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            media_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            media_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            media_regen_target: None,
            media_regen_paths: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            media_regen_results_queue: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            media_regen_window_open: false,
            media_regen_base_settings: None,
            media_regen_media_type: "image".to_string(),
            media_regen_image_priority: vec![
                "flow_nano_banana_pro".to_string(),
                "flow_nano_banana_2".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "openai".to_string(),
            ],
            media_regen_video_priority: vec![
                "flow_fast".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "flow_omni_flash".to_string(),
                "flow_light".to_string(),
                "flow_ultra_light".to_string(),
                "flow_quality".to_string(),
            ],
            media_regen_prompt: String::new(),
            media_regen_error: None,
            media_regen_job_id: 0,
            media_regen_job_name: String::new(),
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
            translation_model_agy: "gemini-3.5-flash".to_string(),
            translation_model_pi: "gemini-2.5-flash".to_string(),
            translation_model_search: String::new(),
            openrouter_models: std::sync::Arc::new(std::sync::Mutex::new(None)),
            openrouter_models_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            openrouter_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_balance: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_service: "Googler".to_string(),
            video_media_type: "image".to_string(),
            text_split_mode: "paragraphs".to_string(),
            text_split_mode_openrouter: "paragraphs".to_string(),
            text_split_char_limit: 500,
            video_prompt: String::new(),
            video_context_enabled: false,
            video_context_mode: "around".to_string(),
            video_context_chars: 500,
            video_agent_mode: "full".to_string(),
            googler_video_upscale_enabled: default_settings.googler_video_upscale_enabled,
            googler_video_upscale_resolution: default_settings
                .googler_video_upscale_resolution
                .clone(),
            googler_video_upscale_quality: default_settings.googler_video_upscale_quality.clone(),
            preview_quality: default_settings.preview_quality.clone(),
            preview_fps: default_settings.preview_fps,
            video_agent_prompt: String::new(),
            video_style_enabled: false,
            video_style_prompt: String::new(),
            video_llm_service: "None".to_string(),
            video_llm_model: String::new(),
            video_llm_model_openrouter: String::new(),
            video_llm_model_claude: "sonnet".to_string(),
            video_llm_model_gemini: "gemini-2.5-flash".to_string(),
            video_llm_model_codex: "gpt-5.4-mini".to_string(),
            video_llm_model_agy: "gemini-3.5-flash".to_string(),
            video_llm_model_pi: "gemini-2.5-flash".to_string(),
            video_llm_temperature: 0.7,
            video_llm_model_search: String::new(),
            googler_image_priority: vec![
                "flow_nano_banana_pro".to_string(),
                "flow_nano_banana_2".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "openai".to_string(),
            ],
            googler_video_priority: vec![
                "flow_fast".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "flow_omni_flash".to_string(),
                "flow_light".to_string(),
                "flow_ultra_light".to_string(),
                "flow_quality".to_string(),
            ],
            googler_video_disabled: vec![],
            translation_temperature: 0.7,
            translation_service: "OpenRouter".to_string(),
            balance_window_open: false,
            threads_window_open: false,
            save_path_macos: String::new(),
            save_path_windows: String::new(),
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            retry_request: None,
            open_job_logs: std::collections::HashMap::new(),
            open_job_controls: std::collections::HashMap::new(),
            open_agent_chats: std::collections::HashMap::new(),
            control_dismissed: std::collections::HashSet::new(),
            job_name_dialog_open: false,
            job_name_input: String::new(),
            resume_dialog_open: false,
            resume_pending: None,
            openrouter_max_threads: 5,
            claude_max_threads: 5,
            gemini_max_threads: 5,
            codex_max_threads: 5,
            agy_max_threads: 5,
            pi_max_threads: 5,
            welcome_open: false,
            welcome_dont_show: false,
            tool_checks: crate::gui::welcome::ToolChecks::new(),
            pending_tool_check: None,
            edge_tts_voice: default_settings.edge_tts_voice.clone(),
            edge_tts_rate: default_settings.edge_tts_rate.clone(),
            edge_tts_pitch: default_settings.edge_tts_pitch.clone(),
            edge_tts_volume: default_settings.edge_tts_volume.clone(),
            edge_tts_max_threads: default_settings.edge_tts_max_threads,
            ffmpeg_max_threads: default_settings.ffmpeg_max_threads,
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: default_settings.googler_image_max_threads,
            googler_video_max_threads: default_settings.googler_video_max_threads,
            voiceover_convert_to_wav: false,
            subtitles_service: "Whisper".to_string(),
            whisper_language: "auto".to_string(),
            whisper_model: "base".to_string(),
            whisper_max_line_width: 42,
            whisper_model_download: std::sync::Arc::new(std::sync::Mutex::new(
                crate::gui::welcome::BinaryDownload::Idle,
            )),
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
            available_subtitle_fonts: Vec::new(),
            capcut_enabled: false,
            capcut_draft_path: String::new(),
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
            overlay_triggers_enabled: false,
            overlay_triggers: vec![],
            copied_toast: None,
            auto_scroll_logs: true,
            last_saved_settings: default_settings,
            video_hover_frames: std::collections::HashMap::new(),
            video_hover_state: std::collections::HashMap::new(),
            video_hover_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            video_hover_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            video_thumbnails: std::collections::HashMap::new(),
            video_thumb_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            video_thumb_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            video_player: None,
            gallery_prompt_popup: None,
            editor_stats: crate::gui::editor::EditorStats::default(),
            task_history: Vec::new(),
            update_info: std::sync::Arc::new(std::sync::Mutex::new(None)),
            update_dialog_open: false,
            queue_panel_collapsed: false,
            queue_panel_fullscreen: false,
            media_control_notified: std::collections::HashSet::new(),
            stock_picker_state: None,
        };

        crate::api::googler::GooglerImageLimiter::get()
            .set_max_threads(app.googler_image_max_threads);
        crate::api::googler::GooglerVideoLimiter::get()
            .set_max_threads(app.googler_video_max_threads);

        app
    }
}

impl VideoMakerApp {
    /// Створює новий екземпляр додатку, завантажуючи збережені налаштування з диска.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Завантажуємо збережені налаштування користувача з файлу settings.json
        let saved = load_settings();

        // Конвертуємо назву теми (String) у тип AppTheme
        let theme = match saved.theme.as_str() {
            "Light" => AppTheme::Light,
            "Amoled" => AppTheme::Amoled,
            _ => AppTheme::Dark,
        };

        // Відновлюємо колір акценту з масиву [r, g, b, a]
        let accent_color = egui::Color32::from_rgba_unmultiplied(
            saved.accent_color[0],
            saved.accent_color[1],
            saved.accent_color[2],
            saved.accent_color[3],
        );

        let pipeline_width = saved.pipeline_width;

        // Конвертуємо назву мови (String) у тип Language
        let language = match saved.language.as_str() {
            "En" => Language::En,
            "Ru" => Language::Ru,
            _ => Language::Uk,
        };

        let openrouter_key = saved.openrouter_key.clone();
        let voicebot_key = saved.voicebot_key.clone();
        let googler_key = saved.googler_key.clone();
        let assemblyai_key = saved.assemblyai_key.clone();
        let pexels_key = saved.pexels_key.clone();
        let pixabay_key = saved.pixabay_key.clone();
        let voiceover_provider = saved.voiceover_provider.clone();
        let voiceover_template_uuid = saved.voiceover_template_uuid.clone();
        let pipeline_translation_enabled = saved.pipeline_translation_enabled;
        let pipeline_translation_control_enabled = saved.pipeline_translation_control_enabled;
        let pipeline_control_auto_open = saved.pipeline_control_auto_open;
        let pipeline_media_control_enabled = saved.pipeline_media_control_enabled;
        let pipeline_montage_control_enabled = saved.pipeline_montage_control_enabled;
        let pipeline_voiceover_enabled = saved.pipeline_voiceover_enabled;
        let pipeline_video_enabled = saved.pipeline_video_enabled;
        let pipeline_subtitles_enabled = saved.pipeline_subtitles_enabled;
        let pipeline_editing_enabled = saved.pipeline_editing_enabled;
        let translation_prompt = saved.translation_prompt.clone();
        let translation_model = saved.translation_model.clone();
        let translation_service = saved.translation_service.clone();
        let mut translation_model_openrouter = saved.translation_model_openrouter.clone();
        let mut translation_model_claude = saved.translation_model_claude.clone();
        let mut translation_model_gemini = saved.translation_model_gemini.clone();
        let mut translation_model_codex = saved.translation_model_codex.clone();
        let mut translation_model_agy = saved.translation_model_agy.clone();
        let mut translation_model_pi = saved.translation_model_pi.clone();

        // Зворотна сумісність: якщо завантажені окремі слоти порожні, але є загальне поле translation_model
        if translation_model_openrouter.is_empty() && translation_service == "OpenRouter" {
            translation_model_openrouter = translation_model.clone();
        }
        if translation_model_claude.is_empty() {
            translation_model_claude = if translation_service == "Claude Code" {
                translation_model.clone()
            } else {
                "sonnet".to_string()
            };
        }
        if translation_model_gemini.is_empty() {
            translation_model_gemini = if translation_service == "Gemini CLI" {
                translation_model.clone()
            } else {
                "gemini-2.5-flash".to_string()
            };
        }
        if translation_model_codex.is_empty() {
            translation_model_codex = if translation_service == "Codex CLI" {
                translation_model.clone()
            } else {
                "gpt-5.4-mini".to_string()
            };
        }
        if translation_model_agy.is_empty() {
            translation_model_agy = if translation_service == "AGY CLI" {
                translation_model.clone()
            } else {
                "default".to_string()
            };
        }
        if translation_model_pi.is_empty() {
            translation_model_pi = if translation_service == "Pi CLI" {
                translation_model.clone()
            } else {
                "gemini-2.5-flash".to_string()
            };
        }

        let video_service = saved.video_service.clone();
        let video_media_type = saved.video_media_type.clone();
        let text_split_mode = saved.text_split_mode.clone();
        let text_split_mode_openrouter = saved.text_split_mode_openrouter.clone();
        let text_split_char_limit = saved.text_split_char_limit;
        let video_prompt = saved.video_prompt.clone();
        let video_context_enabled = saved.video_context_enabled;
        let video_context_mode = saved.video_context_mode.clone();
        let video_context_chars = saved.video_context_chars;
        let video_agent_mode = saved.video_agent_mode.clone();
        let video_agent_prompt = saved.video_agent_prompt.clone();
        let video_style_enabled = saved.video_style_enabled;
        let video_style_prompt = saved.video_style_prompt.clone();
        let video_llm_service = saved.video_llm_service.clone();
        let mut video_llm_model_openrouter = saved.video_llm_model_openrouter.clone();
        let video_llm_model_claude = saved.video_llm_model_claude.clone();
        let video_llm_model_gemini = saved.video_llm_model_gemini.clone();
        let video_llm_model_codex = saved.video_llm_model_codex.clone();
        let video_llm_model_agy = saved.video_llm_model_agy.clone();
        let video_llm_model_pi = saved.video_llm_model_pi.clone();
        let video_llm_temperature = saved.video_llm_temperature;
        // Відновлюємо активну модель залежно від збереженого сервісу
        let video_llm_model = match video_llm_service.as_str() {
            "OpenRouter" => {
                if saved.video_llm_model.is_empty() {
                    video_llm_model_openrouter = saved.video_llm_model.clone();
                }
                saved.video_llm_model.clone()
            }
            "Claude Code" => video_llm_model_claude.clone(),
            "Gemini CLI" => video_llm_model_gemini.clone(),
            "Codex CLI" => video_llm_model_codex.clone(),
            "AGY CLI" => video_llm_model_agy.clone(),
            "Pi CLI" => video_llm_model_pi.clone(),
            _ => saved.video_llm_model.clone(),
        };
        let googler_image_priority = saved.googler_image_priority.clone();
        let mut googler_video_priority = saved.googler_video_priority.clone();
        for p in &[
            "flow_fast",
            "flower",
            "grok",
            "flow_omni_flash",
            "flow_light",
            "flow_ultra_light",
            "flow_quality",
        ] {
            if !googler_video_priority.contains(&p.to_string()) {
                googler_video_priority.push(p.to_string());
            }
        }
        let googler_video_upscale_enabled = saved.googler_video_upscale_enabled;
        let googler_video_upscale_resolution = saved.googler_video_upscale_resolution.clone();
        let googler_video_upscale_quality = saved.googler_video_upscale_quality.clone();
        let preview_quality = saved.preview_quality.clone();
        let preview_fps = saved.preview_fps;
        let translation_temperature = saved.translation_temperature;
        let save_path_macos = saved.save_path_macos.clone();
        let save_path_windows = saved.save_path_windows.clone();

        let openrouter_max_threads = saved.openrouter_max_threads;
        let claude_max_threads = saved.claude_max_threads;
        let gemini_max_threads = saved.gemini_max_threads;
        let codex_max_threads = saved.codex_max_threads;
        let agy_max_threads = saved.agy_max_threads;
        let pi_max_threads = saved.pi_max_threads;
        let show_welcome = saved.show_welcome;

        // Налаштовуємо глобальний лімітер одночасних запитів OpenRouter
        crate::api::openrouter::OpenRouterLimiter::get().set_max_threads(openrouter_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Claude Code
        crate::api::claude::ClaudeLimiter::get().set_max_threads(claude_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Gemini CLI
        crate::api::gemini::GeminiLimiter::get().set_max_threads(gemini_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Codex CLI
        crate::api::codex::CodexLimiter::get().set_max_threads(codex_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів AGY CLI
        crate::api::agy::AgyLimiter::get().set_max_threads(agy_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Pi CLI
        crate::api::pi::PiLimiter::get().set_max_threads(pi_max_threads);
        // Налаштовуємо глобальний лімітер одночасних запитів Edge TTS
        crate::api::edgetts::EdgeTTSLimiter::get().set_max_threads(saved.edge_tts_max_threads);
        // Налаштовуємо глобальний лімітер одночасних процесів FFmpeg
        crate::api::ffmpeg::FfmpegLimiter::get().set_max_threads(saved.ffmpeg_max_threads);

        let saved_templates = crate::gui::settings::storage::load_saved_templates();

        // Ініціалізуємо вікно привітання та одразу запускаємо фонові перевірки CLI
        let tool_checks = crate::gui::welcome::ToolChecks::new();
        if show_welcome {
            tool_checks.start(cc.egui_ctx.clone());
        }

        let openrouter_balance = std::sync::Arc::new(std::sync::Mutex::new(None));
        let voicebot_balance = std::sync::Arc::new(std::sync::Mutex::new(None));
        let googler_balance = std::sync::Arc::new(std::sync::Mutex::new(None));

        // Завантажуємо баланси у фоні при старті, якщо ключі вже збережені
        if !openrouter_key.is_empty() && openrouter_key.starts_with("sk-or-") {
            crate::api::openrouter::fetch_balance(
                openrouter_key.clone(),
                std::sync::Arc::clone(&openrouter_balance),
                cc.egui_ctx.clone(),
            );
        }
        if !voicebot_key.is_empty() {
            crate::api::voicebot::fetch_balance(
                voicebot_key.clone(),
                std::sync::Arc::clone(&voicebot_balance),
                cc.egui_ctx.clone(),
            );
        }
        if !googler_key.is_empty() {
            crate::api::googler::fetch_balance(
                googler_key.clone(),
                std::sync::Arc::clone(&googler_balance),
                cc.egui_ctx.clone(),
            );
        }

        let app = Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme,
            accent_color,
            pipeline_width,
            language,
            openrouter_key,
            openrouter_status: None,
            template_name_input: saved.last_template.clone(),
            saved_templates,
            template_status: None,
            voicebot_key,
            voicebot_status: None,
            googler_key,
            googler_status: None,
            googler_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            googler_balance,
            assemblyai_key,
            assemblyai_status: None,
            assemblyai_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pexels_key,
            pexels_status: None,
            pexels_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pixabay_key,
            pixabay_status: None,
            pixabay_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voiceover_provider,
            voiceover_template_uuid,
            voicebot_templates: std::sync::Arc::new(std::sync::Mutex::new(None)),
            voicebot_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            voicebot_test_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            pipeline_translation_enabled,
            pipeline_translation_control_enabled,
            pipeline_control_auto_open,
            pipeline_media_control_enabled,
            pipeline_montage_control_enabled,
            montage_editor_open_job: None,
            montage_editor_state: None,
            gallery_textures: std::collections::HashMap::new(),
            gallery_image_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            gallery_image_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            gallery_preview: None,
            gallery_anim_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            media_regen_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            media_regen_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            media_regen_target: None,
            media_regen_paths: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            media_regen_results_queue: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            media_regen_window_open: false,
            media_regen_base_settings: None,
            media_regen_media_type: "image".to_string(),
            media_regen_image_priority: vec![
                "flow_nano_banana_pro".to_string(),
                "flow_nano_banana_2".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "openai".to_string(),
            ],
            media_regen_video_priority: vec![
                "flow_fast".to_string(),
                "flower".to_string(),
                "grok".to_string(),
                "flow_omni_flash".to_string(),
                "flow_light".to_string(),
                "flow_ultra_light".to_string(),
                "flow_quality".to_string(),
            ],
            media_regen_prompt: String::new(),
            media_regen_error: None,
            media_regen_job_id: 0,
            media_regen_job_name: String::new(),
            pipeline_voiceover_enabled,
            pipeline_video_enabled,
            pipeline_subtitles_enabled,
            pipeline_editing_enabled,
            translation_prompt,
            translation_model,
            translation_model_openrouter,
            translation_model_claude,
            translation_model_gemini,
            translation_model_codex,
            translation_model_agy,
            translation_model_pi,
            translation_model_search: String::new(),
            openrouter_models: std::sync::Arc::new(std::sync::Mutex::new(None)),
            openrouter_models_loading: std::sync::Arc::new(std::sync::Mutex::new(false)),
            openrouter_balance,
            voicebot_balance,
            video_service,
            video_media_type,
            text_split_mode,
            text_split_mode_openrouter,
            text_split_char_limit,
            video_prompt,
            video_context_enabled,
            video_context_mode,
            video_context_chars,
            video_agent_mode,
            googler_video_upscale_enabled,
            googler_video_upscale_resolution,
            googler_video_upscale_quality,
            preview_quality,
            preview_fps,
            video_agent_prompt,
            video_style_enabled,
            video_style_prompt,
            video_llm_service,

            video_llm_model,
            video_llm_model_openrouter,
            video_llm_model_claude,
            video_llm_model_gemini,
            video_llm_model_codex,
            video_llm_model_agy,
            video_llm_model_pi,
            video_llm_temperature,
            video_llm_model_search: String::new(),
            googler_image_priority,
            googler_video_priority,
            googler_video_disabled: saved.googler_video_disabled.clone(),
            translation_temperature,
            translation_service,
            balance_window_open: false,
            threads_window_open: false,
            save_path_macos,
            save_path_windows,
            jobs: Vec::new(),
            job_counter: 0,
            queue_error: None,
            retry_request: None,
            open_job_logs: std::collections::HashMap::new(),
            open_job_controls: std::collections::HashMap::new(),
            open_agent_chats: std::collections::HashMap::new(),
            control_dismissed: std::collections::HashSet::new(),
            job_name_dialog_open: false,
            job_name_input: String::new(),
            resume_dialog_open: false,
            resume_pending: None,
            openrouter_max_threads,
            claude_max_threads,
            gemini_max_threads,
            codex_max_threads,
            agy_max_threads,
            pi_max_threads,
            welcome_open: show_welcome,
            welcome_dont_show: false,
            tool_checks,
            pending_tool_check: None,
            edge_tts_voice: saved.edge_tts_voice.clone(),
            edge_tts_rate: saved.edge_tts_rate.clone(),
            edge_tts_pitch: saved.edge_tts_pitch.clone(),
            edge_tts_volume: saved.edge_tts_volume.clone(),
            edge_tts_max_threads: saved.edge_tts_max_threads,
            ffmpeg_max_threads: saved.ffmpeg_max_threads,
            edge_tts_voices: std::sync::Arc::new(std::sync::Mutex::new(None)),
            edge_tts_loading_voices: std::sync::Arc::new(std::sync::Mutex::new(false)),
            edge_tts_show_all_languages: false,
            googler_image_max_threads: saved.googler_image_max_threads,
            googler_video_max_threads: saved.googler_video_max_threads,
            voiceover_convert_to_wav: saved.voiceover_convert_to_wav,
            subtitles_service: saved.subtitles_service.clone(),
            whisper_language: saved.whisper_language.clone(),
            whisper_model: saved.whisper_model.clone(),
            whisper_max_line_width: saved.whisper_max_line_width,
            whisper_model_download: std::sync::Arc::new(std::sync::Mutex::new(
                crate::gui::welcome::BinaryDownload::Idle,
            )),
            subtitle_font_size: saved.subtitle_font_size,
            subtitle_color: saved.subtitle_color,
            subtitle_margin_v: saved.subtitle_margin_v,
            subtitle_karaoke: saved.subtitle_karaoke,
            subtitle_karaoke_mode: saved.subtitle_karaoke_mode,
            subtitle_karaoke_highlight_color: saved.subtitle_karaoke_highlight_color,
            subtitle_karaoke_outline_color: saved.subtitle_karaoke_outline_color,
            subtitle_karaoke_bold: saved.subtitle_karaoke_bold,
            subtitle_karaoke_scale: saved.subtitle_karaoke_scale,
            subtitle_font: saved.subtitle_font.clone(),
            available_subtitle_fonts: crate::gui::subtitle_fonts::load_subtitle_fonts(&cc.egui_ctx),
            capcut_enabled: saved.capcut_enabled,
            capcut_draft_path: saved.capcut_draft_path.clone(),
            montage_service: saved.montage_service.clone(),
            montage_fps: saved.montage_fps,
            montage_preset: saved.montage_preset.clone(),
            montage_bitrate: saved.montage_bitrate,
            montage_transition: saved.montage_transition.clone(),
            montage_transition_duration: saved.montage_transition_duration,
            montage_image_zoom_enabled: saved.montage_image_zoom_enabled,
            montage_image_zoom_intensity: saved.montage_image_zoom_intensity,
            montage_image_zoom_mode: saved.montage_image_zoom_mode.clone(),
            montage_image_zoom_scale: saved.montage_image_zoom_scale,
            montage_image_shake_enabled: saved.montage_image_shake_enabled,
            montage_image_shake_intensity: saved.montage_image_shake_intensity,
            overlay_triggers_enabled: saved.overlay_triggers_enabled,
            overlay_triggers: saved.overlay_triggers.clone(),
            copied_toast: None,
            auto_scroll_logs: true,
            last_saved_settings: saved,
            video_hover_frames: std::collections::HashMap::new(),
            video_hover_state: std::collections::HashMap::new(),
            video_hover_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            video_hover_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            video_thumbnails: std::collections::HashMap::new(),
            video_thumb_loading: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            video_thumb_result: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            video_player: None,
            gallery_prompt_popup: None,
            editor_stats: crate::gui::editor::EditorStats::default(),
            task_history: crate::gui::settings::storage::load_task_history(),
            update_info: std::sync::Arc::new(std::sync::Mutex::new(None)),
            update_dialog_open: false,
            queue_panel_collapsed: false,
            queue_panel_fullscreen: false,
            media_control_notified: std::collections::HashSet::new(),
            stock_picker_state: None,
        };

        // Синхронізуємо лімітери потоків зі збереженими налаштуваннями
        crate::api::googler::GooglerImageLimiter::get()
            .set_max_threads(app.googler_image_max_threads);
        crate::api::googler::GooglerVideoLimiter::get()
            .set_max_threads(app.googler_video_max_threads);

        // Прогрів tiktoken encoder у фоновому потоці, щоб уникнути freeze при першому відкритті редактора
        std::thread::spawn(|| {
            crate::gui::editor::count_tokens("");
        });

        // Фонова перевірка оновлень при старті
        crate::api::updater::check_for_updates(
            std::sync::Arc::clone(&app.update_info),
            cc.egui_ctx.clone(),
        );

        app
    }
}
