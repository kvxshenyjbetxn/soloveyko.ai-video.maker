import os
import sys
import json
from utils.translator import translator
from utils.settings import settings_manager

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    # Assuming this file is in gui/settings_metadata.py, so up one level is gui/..
    # d:\vs-code\soloveykoai\soloveyko.ai-video.maker\gui\settings_metadata.py
    # BASE_PATH should be root of project.
    BASE_PATH = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def load_json_assets(filename):
    try:
        path = os.path.join(BASE_PATH, "assets", filename)
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading {filename}: {e}")
        return []

VOICEMAKER_VOICES = load_json_assets("voicemaker_voices.json")
GEMINI_VOICES = load_json_assets("gemini_tts_voices.json")

# --- Settings Metadata ---
# This dictionary will define the editor type and data for each setting.
# 'type' can be 'bool', 'int', 'float', 'choice', 'str'
# 'options' is for 'choice' type
SETTINGS_METADATA = {
    'openrouter_models': {'type': 'string_list', 'label': 'openrouter_models'},
    'image_generation_provider': {'type': 'choice', 'options': ["pollinations", "googler"], 'label': 'image_generation_provider'},
    'image_review_enabled': {'type': 'bool', 'label': 'image_review_enabled'},
    'results_path': {'type': 'folder_path', 'label': 'results_path'},
    'montage': {
        # Render
        'codec': {'type': 'choice', 'options': ["libx264", "libx265", "h264_nvenc", "h264_amf"]},
        'preset': {'type': 'choice', 'options': ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"]},
        'bitrate_mbps': {'type': 'int', 'min': 1, 'max': 100, 'suffix': ' Mbps'},
        'upscale_factor': {'type': 'float', 'min': 1.0, 'max': 5.0, 'step': 0.1, 'suffix': 'x'},
        # Transitions
        'enable_transitions': {'type': 'bool'},
        'transition_duration': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1, 'suffix': ' s'},
        # Zoom
        'enable_zoom': {'type': 'bool'},
        'zoom_speed_factor': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1},
        'zoom_intensity': {'type': 'float', 'min': 0.01, 'max': 1.0, 'step': 0.05},
        # Sway
        'enable_sway': {'type': 'bool'},
        'sway_speed_factor': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1},
        # Special Processing
        'special_processing_mode': {'type': 'choice', 'options': ["Disabled", "Quick show", "Video at the beginning"]},
        'special_processing_image_count': {'type': 'int', 'min': 1, 'max': 100},
        'special_processing_duration_per_image': {'type': 'float', 'min': 0.1, 'max': 10.0, 'step': 0.1, 'suffix': ' s'},
        'special_processing_video_count': {'type': 'int', 'min': 1, 'max': 100},
        'special_processing_check_sequence': {'type': 'bool'},
        'max_concurrent_montages': {'type': 'int', 'min': 1, 'max': 10}
    },
    'subtitles': {
        'whisper_type': {'type': 'choice', 'options': ['standard', 'amd', 'assemblyai']},
        'whisper_model': {'type': 'choice', 'options': ["tiny", "base", "small", "medium", "large", "base.bin", "small.bin", "medium.bin", "large.bin"]},
        'font': {'type': 'font'},
        'fontsize': {'type': 'int', 'min': 10, 'max': 200},
        'margin_v': {'type': 'int', 'min': 0, 'max': 500},
        'fade_in': {'type': 'int', 'min': 0, 'max': 5000, 'suffix': ' ms'},
        'fade_out': {'type': 'int', 'min': 0, 'max': 5000, 'suffix': ' ms'},
        'max_words': {'type': 'int', 'min': 1, 'max': 50},
        'color': {'type': 'color'},
    },
    'googler': {
        'api_key': {'type': 'str'}, # Assuming basic string
        'aspect_ratio': {'type': 'choice', 'options': ["IMAGE_ASPECT_RATIO_LANDSCAPE", "IMAGE_ASPECT_RATIO_PORTRAIT", "IMAGE_ASPECT_RATIO_SQUARE"]},
        'video_prompt': {'type': 'text_edit_button'},
        'negative_prompt': {'type': 'str'},
        'seed': {'type': 'str'},
    },
    'pollinations': {
        'model': {'type': 'choice', 'options': ["flux", "flux-realism", "flux-3d", "flux-cablyai", "dall-e-3", "midjourney", "boreal"]},
        'token': {'type': 'str'},
        'width': {'type': 'int', 'min': 64, 'max': 4096},
        'height': {'type': 'int', 'min': 64, 'max': 4096},
        'nologo': {'type': 'bool'},
        'enhance': {'type': 'bool'},
    },
     'image_prompt_settings': {
        'prompt': {'type': 'text_edit_button'},
        'model': {'type': 'model_selection'},
        'max_tokens': {'type': 'int', 'min': 1, 'max': 128000},
        'temperature': {'type': 'float', 'min': 0.0, 'max': 2.0, 'step': 0.1},
    },
    'languages_config': {
        'prompt': {'type': 'text_edit_button'},
        'model': {'type': 'model_selection'},
        'max_tokens': {'type': 'int', 'min': 1, 'max': 128000},
        'temperature': {'type': 'float', 'min': 0.0, 'max': 2.0, 'step': 0.1},
        'background_music_path': {'type': 'file_path'},
        'background_music_volume': {'type': 'int', 'min': 0, 'max': 100},
        'tts_provider': {'type': 'choice', 'options': ["ElevenLabs", "VoiceMaker", "GeminiTTS", "EdgeTTS", "ElevenLabsUnlim"]},
        'voicemaker_voice_id': {'type': 'voicemaker_voice'},
        'gemini_voice': {'type': 'gemini_voice'},
        'gemini_tone': {'type': 'str'},
        'edgetts_voice': {'type': 'str'}, 
        'edgetts_rate': {'type': 'int', 'min': -100, 'max': 100, 'suffix': '%'},
        'edgetts_pitch': {'type': 'int', 'min': -100, 'max': 100, 'suffix': ' Hz'},
        'elevenlabs_template_uuid': {'type': 'str'},
        'default_template': {'type': 'str'}, 
        'rewrite_prompt': {'type': 'text_edit_button'}, # Added missing keys compared to TAB
        'rewrite_model': {'type': 'model_selection'},
        'rewrite_max_tokens': {'type': 'int', 'min': 1, 'max': 128000},
        'rewrite_temperature': {'type': 'float', 'min': 0.0, 'max': 2.0, 'step': 0.1},
        'watermark_size': {'type': 'int', 'min': 1, 'max': 100},
        'watermark_position': {'type': 'choice', 'options': ["Top Left", "Top Center", "Top Right", "Center Left", "Center", "Center Right", "Bottom Left", "Bottom Center", "Bottom Right"]},
        'eleven_unlim_settings': {
             'voice_id': {'type': 'str'},
             'stability': {'type': 'float', 'min': 0.0, 'max': 1.0, 'step': 0.1},
             'similarity_boost': {'type': 'float', 'min': 0.0, 'max': 1.0, 'step': 0.1},
             'style': {'type': 'float', 'min': 0.0, 'max': 1.0, 'step': 0.1},
             'use_speaker_boost': {'type': 'bool'}
        }
    }
}

KEY_TO_TRANSLATION_MAP = {
    # General
    'results_path': 'results_path_label',
    'image_review_enabled': 'image_review_label',
    'prompt_count_control_enabled': 'prompt_count_control_label',
    'prompt_count': 'prompt_count_label',
    'image_generation_provider': 'image_generation_provider_label',
    
    # API Tab
    'openrouter_api_key': 'openrouter_api_key_label', 
    'openrouter_models': 'openrouter_models_label',
    'elevenlabs_api_key': 'elevenlabs_api_key_label',
    'voicemaker_api_key': 'voicemaker_api_key_label',
    'voicemaker_char_limit': 'voicemaker_char_limit_label',
    'gemini_tts_api_key': 'gemini_tts_api_key_label',

    # Montage Tab
    'codec': 'codec_label',
    'preset': 'preset_label',
    'bitrate_mbps': 'bitrate_label',
    'upscale_factor': 'upscale_factor_label',
    'enable_transitions': 'enable_transitions_label',
    'transition_duration': 'duration_label',
    'enable_zoom': 'enable_zoom_label',
    'zoom_speed_factor': 'zoom_speed_factor_label',
    'zoom_intensity': 'zoom_intensity_label',
    'enable_sway': 'enable_sway_label',
    'sway_speed_factor': 'sway_speed_factor_label',
    'special_processing_mode': 'special_proc_mode_label',
    'special_processing_image_count': 'image_count_label',
    'special_processing_duration_per_image': 'duration_per_image_label',
    'special_processing_video_count': 'special_proc_video_count_label',
    'special_processing_check_sequence': 'special_proc_check_sequence_label',
    'max_concurrent_montages': 'max_concurrent_montages_label',

    # Subtitles Tab
    'whisper_model': 'model_label',
    'font': 'font_label',
    'fontsize': 'font_size_label',
    'margin_v': 'vertical_margin_label',
    'fade_in': 'fade_in_label',
    'fade_out': 'fade_out_label',
    'max_words': 'max_words_per_line_label',
    'color': 'color_label',
    
    # Googler
    'api_key': 'googler_api_key_label',
    'aspect_ratio': 'googler_aspect_ratio_label',
    'max_threads': 'googler_max_threads_label',
    'max_video_threads': 'googler_max_video_threads_label',
    'video_prompt': 'googler_video_prompt_label',
    'negative_prompt': 'googler_negative_prompt_label',
    'seed': 'googler_seed_label',

    # Pollinations
    'model': 'pollinations_model_label',
    'token': 'pollinations_token_label',
    'width': 'image_width_label', 
    'height': 'image_height_label', 
    'nologo': 'nologo_label',
    'enhance': 'enhance_prompt_label',

    # Image Prompt Settings
    'max_tokens': 'max_tokens_label',
    'temperature': 'temperature_label',

    # Languages Config keys
    'prompt': 'language_prompt_label',
    
    'background_music_path': 'background_music_path_label',
    'background_music_volume': 'background_music_volume_label',
    'tts_provider': 'tts_provider_label',
    'voicemaker_voice_id': 'voicemaker_voice_label',
    'gemini_voice': 'gemini_voice_label',
    'gemini_tone': 'gemini_tone_label',
    'edgetts_voice': 'edgetts_voice_label', 
    'edgetts_rate': 'edgetts_rate_label',
    'edgetts_pitch': 'edgetts_pitch_label',
    'elevenlabs_template_uuid': 'elevenlabs_template_label',
    'default_template': 'default_template_label',
    'rewrite_prompt': 'rewrite_prompt_label',
    'rewrite_model': 'rewrite_model_label',
    'rewrite_max_tokens': 'rewrite_max_tokens_label',
    'rewrite_temperature': 'rewrite_temperature_label',
    'overlay_effect_path': 'effect_selection_title',
    'watermark_path': 'watermark_group',
    'watermark_size': 'watermark_size_label',
    'watermark_position': 'watermark_position_label'
}
