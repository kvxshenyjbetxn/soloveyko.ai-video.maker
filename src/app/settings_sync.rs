use eframe::egui;

use super::VideoMakerApp;
use crate::gui::settings::storage::{AppSettings, save_settings};
use crate::localization::Language;
use crate::theme::AppTheme;

impl VideoMakerApp {
    /// Автоматично зберігає налаштування після відпускання миші.
    /// Це не змінює логіку збереження, а лише тримає її окремо від головного update().
    pub(super) fn autosave_settings_if_needed(&mut self, ctx: &egui::Context) {
        // Перевіряємо, чи користувач наразі не перетягує панель (миша відпущена).
        // Це запобігає надмірному навантаженню на диск та гарантує запис файлу лише після відпускання миші.
        let is_pointer_down = ctx.input(|i| i.pointer.any_down());

        if !is_pointer_down {
            let current_theme_str = match self.theme {
                AppTheme::Light => "Light".to_string(),
                AppTheme::Dark => "Dark".to_string(),
                AppTheme::Amoled => "Amoled".to_string(),
            };
            let current_color_arr = self.accent_color.to_array();
            let current_language_str = match self.language {
                Language::En => "En".to_string(),
                Language::Ru => "Ru".to_string(),
                _ => "Uk".to_string(),
            };

            // Перевіряємо зміни значень (з дельтою для ширини панелі)
            if current_theme_str != self.last_saved_settings.theme
                || current_color_arr != self.last_saved_settings.accent_color
                || current_language_str != self.last_saved_settings.language
                || self.openrouter_key != self.last_saved_settings.openrouter_key
                || self.voicebot_key != self.last_saved_settings.voicebot_key
                || self.voiceover_provider != self.last_saved_settings.voiceover_provider
                || self.voiceover_template_uuid != self.last_saved_settings.voiceover_template_uuid
                || self.template_name_input != self.last_saved_settings.last_template
                || self.pipeline_translation_enabled
                    != self.last_saved_settings.pipeline_translation_enabled
                || self.pipeline_translation_control_enabled
                    != self
                        .last_saved_settings
                        .pipeline_translation_control_enabled
                || self.pipeline_control_auto_open
                    != self.last_saved_settings.pipeline_control_auto_open
                || self.pipeline_media_control_enabled
                    != self.last_saved_settings.pipeline_media_control_enabled
                || self.pipeline_montage_control_enabled
                    != self.last_saved_settings.pipeline_montage_control_enabled
                || self.pipeline_voiceover_enabled
                    != self.last_saved_settings.pipeline_voiceover_enabled
                || self.pipeline_video_enabled != self.last_saved_settings.pipeline_video_enabled
                || self.pipeline_subtitles_enabled
                    != self.last_saved_settings.pipeline_subtitles_enabled
                || self.pipeline_editing_enabled
                    != self.last_saved_settings.pipeline_editing_enabled
                || self.translation_prompt != self.last_saved_settings.translation_prompt
                || self.translation_model != self.last_saved_settings.translation_model
                || self.translation_model_openrouter
                    != self.last_saved_settings.translation_model_openrouter
                || self.translation_model_claude
                    != self.last_saved_settings.translation_model_claude
                || self.translation_model_gemini
                    != self.last_saved_settings.translation_model_gemini
                || self.translation_model_codex != self.last_saved_settings.translation_model_codex
                || self.translation_model_agy != self.last_saved_settings.translation_model_agy
                || self.translation_model_pi != self.last_saved_settings.translation_model_pi
                || self.googler_key != self.last_saved_settings.googler_key
                || self.assemblyai_key != self.last_saved_settings.assemblyai_key
                || self.pexels_key != self.last_saved_settings.pexels_key
                || self.magnific_key != self.last_saved_settings.magnific_key
                || self.pixabay_key != self.last_saved_settings.pixabay_key
                || self.video_service != self.last_saved_settings.video_service
                || self.video_media_type != self.last_saved_settings.video_media_type
                || self.video_prompt != self.last_saved_settings.video_prompt
                || self.video_context_enabled != self.last_saved_settings.video_context_enabled
                || self.video_context_mode != self.last_saved_settings.video_context_mode
                || self.video_context_chars != self.last_saved_settings.video_context_chars
                || self.video_agent_mode != self.last_saved_settings.video_agent_mode
                || self.video_agent_prompt != self.last_saved_settings.video_agent_prompt
                || self.video_style_enabled != self.last_saved_settings.video_style_enabled
                || self.video_style_prompt != self.last_saved_settings.video_style_prompt
                || self.video_llm_service != self.last_saved_settings.video_llm_service
                || self.video_llm_model != self.last_saved_settings.video_llm_model
                || self.video_llm_model_openrouter
                    != self.last_saved_settings.video_llm_model_openrouter
                || self.video_llm_model_claude != self.last_saved_settings.video_llm_model_claude
                || self.video_llm_model_gemini != self.last_saved_settings.video_llm_model_gemini
                || self.video_llm_model_codex != self.last_saved_settings.video_llm_model_codex
                || self.video_llm_model_agy != self.last_saved_settings.video_llm_model_agy
                || self.video_llm_model_pi != self.last_saved_settings.video_llm_model_pi
                || self.video_llm_temperature != self.last_saved_settings.video_llm_temperature
                || self.text_split_mode != self.last_saved_settings.text_split_mode
                || self.text_split_mode_openrouter
                    != self.last_saved_settings.text_split_mode_openrouter
                || self.text_split_char_limit != self.last_saved_settings.text_split_char_limit
                || self.editor_segment_outlines_enabled
                    != self.last_saved_settings.editor_segment_outlines_enabled
                || self.translation_service != self.last_saved_settings.translation_service
                || self.save_path_macos != self.last_saved_settings.save_path_macos
                || self.save_path_windows != self.last_saved_settings.save_path_windows
                || self.openrouter_max_threads != self.last_saved_settings.openrouter_max_threads
                || self.claude_max_threads != self.last_saved_settings.claude_max_threads
                || self.gemini_max_threads != self.last_saved_settings.gemini_max_threads
                || self.codex_max_threads != self.last_saved_settings.codex_max_threads
                || self.agy_max_threads != self.last_saved_settings.agy_max_threads
                || self.pi_max_threads != self.last_saved_settings.pi_max_threads
                || self.edge_tts_voice != self.last_saved_settings.edge_tts_voice
                || self.edge_tts_rate != self.last_saved_settings.edge_tts_rate
                || self.edge_tts_pitch != self.last_saved_settings.edge_tts_pitch
                || self.edge_tts_volume != self.last_saved_settings.edge_tts_volume
                || self.edge_tts_max_threads != self.last_saved_settings.edge_tts_max_threads
                || self.ffmpeg_max_threads != self.last_saved_settings.ffmpeg_max_threads
                || self.googler_image_max_threads
                    != self.last_saved_settings.googler_image_max_threads
                || self.googler_video_max_threads
                    != self.last_saved_settings.googler_video_max_threads
                || self.voiceover_convert_to_wav
                    != self.last_saved_settings.voiceover_convert_to_wav
                || self.googler_image_priority != self.last_saved_settings.googler_image_priority
                || self.googler_video_priority != self.last_saved_settings.googler_video_priority
                || self.googler_video_disabled != self.last_saved_settings.googler_video_disabled
                || self.translation_temperature != self.last_saved_settings.translation_temperature
                || self.subtitles_service != self.last_saved_settings.subtitles_service
                || self.whisper_language != self.last_saved_settings.whisper_language
                || self.whisper_model != self.last_saved_settings.whisper_model
                || self.whisper_max_line_width != self.last_saved_settings.whisper_max_line_width
                || self.subtitle_font_size != self.last_saved_settings.subtitle_font_size
                || self.subtitle_color != self.last_saved_settings.subtitle_color
                || self.subtitle_margin_v != self.last_saved_settings.subtitle_margin_v
                || self.subtitle_karaoke != self.last_saved_settings.subtitle_karaoke
                || self.subtitle_karaoke_mode != self.last_saved_settings.subtitle_karaoke_mode
                || self.subtitle_karaoke_highlight_color
                    != self.last_saved_settings.subtitle_karaoke_highlight_color
                || self.subtitle_karaoke_outline_color
                    != self.last_saved_settings.subtitle_karaoke_outline_color
                || self.subtitle_karaoke_bold != self.last_saved_settings.subtitle_karaoke_bold
                || self.subtitle_karaoke_scale != self.last_saved_settings.subtitle_karaoke_scale
                || self.subtitle_font != self.last_saved_settings.subtitle_font
                || self.montage_service != self.last_saved_settings.montage_service
                || self.montage_fps != self.last_saved_settings.montage_fps
                || self.montage_preset != self.last_saved_settings.montage_preset
                || self.montage_bitrate != self.last_saved_settings.montage_bitrate
                || self.montage_transition != self.last_saved_settings.montage_transition
                || self.montage_transition_duration
                    != self.last_saved_settings.montage_transition_duration
                || self.montage_image_zoom_enabled
                    != self.last_saved_settings.montage_image_zoom_enabled
                || (self.montage_image_zoom_intensity
                    - self.last_saved_settings.montage_image_zoom_intensity)
                    .abs()
                    > 0.001
                || self.montage_image_zoom_mode != self.last_saved_settings.montage_image_zoom_mode
                || (self.montage_image_zoom_scale
                    - self.last_saved_settings.montage_image_zoom_scale)
                    .abs()
                    > 0.001
                || self.montage_image_shake_enabled
                    != self.last_saved_settings.montage_image_shake_enabled
                || (self.montage_image_shake_intensity
                    - self.last_saved_settings.montage_image_shake_intensity)
                    .abs()
                    > 0.001
                || self.capcut_enabled != self.last_saved_settings.capcut_enabled
                || self.capcut_draft_path != self.last_saved_settings.capcut_draft_path
                || self.overlay_triggers_enabled
                    != self.last_saved_settings.overlay_triggers_enabled
                || self.overlay_triggers != self.last_saved_settings.overlay_triggers
                || self.googler_video_upscale_enabled
                    != self.last_saved_settings.googler_video_upscale_enabled
                || self.googler_video_upscale_resolution
                    != self.last_saved_settings.googler_video_upscale_resolution
                || self.googler_video_upscale_quality
                    != self.last_saved_settings.googler_video_upscale_quality
                || self.preview_quality != self.last_saved_settings.preview_quality
                || (self.preview_fps - self.last_saved_settings.preview_fps).abs() > 0.1
            {
                let new_settings = AppSettings {
                    theme: current_theme_str,
                    accent_color: current_color_arr,
                    pipeline_width: self.pipeline_width,
                    language: current_language_str,
                    openrouter_key: self.openrouter_key.clone(),
                    voicebot_key: self.voicebot_key.clone(),
                    googler_key: self.googler_key.clone(),
                    assemblyai_key: self.assemblyai_key.clone(),
                    pexels_key: self.pexels_key.clone(),
                    magnific_key: self.magnific_key.clone(),
                    pixabay_key: self.pixabay_key.clone(),
                    voiceover_provider: self.voiceover_provider.clone(),
                    voiceover_template_uuid: self.voiceover_template_uuid.clone(),
                    last_template: self.template_name_input.clone(),
                    pipeline_translation_enabled: self.pipeline_translation_enabled,
                    pipeline_translation_control_enabled: self.pipeline_translation_control_enabled,
                    pipeline_control_auto_open: self.pipeline_control_auto_open,
                    pipeline_media_control_enabled: self.pipeline_media_control_enabled,
                    pipeline_montage_control_enabled: self.pipeline_montage_control_enabled,
                    pipeline_voiceover_enabled: self.pipeline_voiceover_enabled,
                    pipeline_video_enabled: self.pipeline_video_enabled,
                    pipeline_subtitles_enabled: self.pipeline_subtitles_enabled,
                    pipeline_editing_enabled: self.pipeline_editing_enabled,
                    translation_prompt: self.translation_prompt.clone(),
                    translation_model: self.translation_model.clone(),
                    translation_model_openrouter: self.translation_model_openrouter.clone(),
                    translation_model_claude: self.translation_model_claude.clone(),
                    translation_model_gemini: self.translation_model_gemini.clone(),
                    translation_model_codex: self.translation_model_codex.clone(),
                    translation_model_agy: self.translation_model_agy.clone(),
                    translation_model_pi: self.translation_model_pi.clone(),
                    video_service: self.video_service.clone(),
                    video_media_type: self.video_media_type.clone(),
                    text_split_mode: self.text_split_mode.clone(),
                    text_split_mode_openrouter: self.text_split_mode_openrouter.clone(),
                    text_split_char_limit: self.text_split_char_limit,
                    editor_segment_outlines_enabled: self.editor_segment_outlines_enabled,
                    video_prompt: self.video_prompt.clone(),
                    video_context_enabled: self.video_context_enabled,
                    video_context_mode: self.video_context_mode.clone(),
                    video_context_chars: self.video_context_chars,
                    video_agent_mode: self.video_agent_mode.clone(),
                    video_agent_prompt: self.video_agent_prompt.clone(),
                    video_style_enabled: self.video_style_enabled,
                    video_style_prompt: self.video_style_prompt.clone(),
                    video_llm_service: self.video_llm_service.clone(),
                    video_llm_model: self.video_llm_model.clone(),
                    video_llm_model_openrouter: self.video_llm_model_openrouter.clone(),
                    video_llm_model_claude: self.video_llm_model_claude.clone(),
                    video_llm_model_gemini: self.video_llm_model_gemini.clone(),
                    video_llm_model_codex: self.video_llm_model_codex.clone(),
                    video_llm_model_agy: self.video_llm_model_agy.clone(),
                    video_llm_model_pi: self.video_llm_model_pi.clone(),
                    video_llm_temperature: self.video_llm_temperature,
                    translation_temperature: self.translation_temperature,
                    translation_service: self.translation_service.clone(),
                    save_path_macos: self.save_path_macos.clone(),
                    save_path_windows: self.save_path_windows.clone(),
                    save_path: String::new(),
                    openrouter_max_threads: self.openrouter_max_threads,
                    claude_max_threads: self.claude_max_threads,
                    gemini_max_threads: self.gemini_max_threads,
                    codex_max_threads: self.codex_max_threads,
                    agy_max_threads: self.agy_max_threads,
                    pi_max_threads: self.pi_max_threads,
                    edge_tts_voice: self.edge_tts_voice.clone(),
                    edge_tts_rate: self.edge_tts_rate.clone(),
                    edge_tts_pitch: self.edge_tts_pitch.clone(),
                    edge_tts_volume: self.edge_tts_volume.clone(),
                    edge_tts_max_threads: self.edge_tts_max_threads,
                    ffmpeg_max_threads: self.ffmpeg_max_threads,
                    googler_image_max_threads: self.googler_image_max_threads,
                    googler_video_max_threads: self.googler_video_max_threads,
                    voiceover_convert_to_wav: self.voiceover_convert_to_wav,
                    googler_image_priority: self.googler_image_priority.clone(),
                    googler_video_priority: self.googler_video_priority.clone(),
                    googler_video_disabled: self.googler_video_disabled.clone(),
                    subtitles_service: self.subtitles_service.clone(),
                    whisper_language: self.whisper_language.clone(),
                    whisper_model: self.whisper_model.clone(),
                    whisper_max_line_width: self.whisper_max_line_width,
                    subtitle_font_size: self.subtitle_font_size,
                    subtitle_color: self.subtitle_color,
                    subtitle_margin_v: self.subtitle_margin_v,
                    subtitle_karaoke: self.subtitle_karaoke,
                    subtitle_karaoke_mode: self.subtitle_karaoke_mode,
                    subtitle_karaoke_highlight_color: self.subtitle_karaoke_highlight_color,
                    subtitle_karaoke_outline_color: self.subtitle_karaoke_outline_color,
                    subtitle_karaoke_bold: self.subtitle_karaoke_bold,
                    subtitle_karaoke_scale: self.subtitle_karaoke_scale,
                    subtitle_font: self.subtitle_font.clone(),
                    capcut_enabled: self.capcut_enabled,
                    capcut_draft_path: self.capcut_draft_path.clone(),
                    montage_service: self.montage_service.clone(),
                    montage_fps: self.montage_fps,
                    montage_preset: self.montage_preset.clone(),
                    montage_bitrate: self.montage_bitrate,
                    montage_transition: self.montage_transition.clone(),
                    montage_transition_duration: self.montage_transition_duration,
                    montage_image_zoom_enabled: self.montage_image_zoom_enabled,
                    montage_image_zoom_intensity: self.montage_image_zoom_intensity,
                    montage_image_zoom_mode: self.montage_image_zoom_mode.clone(),
                    montage_image_zoom_scale: self.montage_image_zoom_scale,
                    montage_image_shake_enabled: self.montage_image_shake_enabled,
                    montage_image_shake_intensity: self.montage_image_shake_intensity,
                    overlay_triggers_enabled: self.overlay_triggers_enabled,
                    overlay_triggers: self.overlay_triggers.clone(),
                    googler_video_upscale_enabled: self.googler_video_upscale_enabled,
                    googler_video_upscale_resolution: self.googler_video_upscale_resolution.clone(),
                    googler_video_upscale_quality: self.googler_video_upscale_quality.clone(),
                    preview_quality: self.preview_quality.clone(),
                    preview_fps: self.preview_fps,
                    show_welcome: self.last_saved_settings.show_welcome,
                };

                // Зберігаємо оновлені налаштування у файл JSON на диску
                save_settings(&new_settings);

                // Оновлюємо копію останніх збережених параметрів у пам'яті
                self.last_saved_settings = new_settings;
            }
        }
    }
}
