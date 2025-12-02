import json
import os

class SettingsManager:
    def __init__(self, settings_file='config/settings.json'):
        self.settings_file = settings_file
        self.defaults = {
            'language': 'uk',
            'theme': 'dark',
            'results_path': '',
            'voicemaker_api_key': '',
            'gemini_tts_api_key': '',
            'gemini_tts_url': 'http://127.0.0.1:8000'
        }
        self.settings = self.load_settings()

    def load_settings(self):
        if os.path.exists(self.settings_file):
            try:
                with open(self.settings_file, 'r', encoding='utf-8') as f:
                    settings = json.load(f)
                    # Ensure all keys are present
                    for key, value in self.defaults.items():
                        if key not in settings:
                            settings[key] = value
                    return settings
            except (json.JSONDecodeError, TypeError):
                return self.defaults
        return self.defaults

    def save_settings(self):
        os.makedirs(os.path.dirname(self.settings_file), exist_ok=True)
        with open(self.settings_file, 'w', encoding='utf-8') as f:
            json.dump(self.settings, f, indent=4, ensure_ascii=False)

    def get(self, key, default=None):
        if default is None:
            default = self.defaults.get(key)
        return self.settings.get(key, default)

    def set(self, key, value):
        self.settings[key] = value
        self.save_settings()

settings_manager = SettingsManager()
