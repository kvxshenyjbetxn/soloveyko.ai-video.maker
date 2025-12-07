import json
import os
from utils.settings import settings_manager

class Translator:
    def __init__(self):
        self.language = settings_manager.get('language')
        self.translations = self.load_translations()

    def load_translations(self):
        lang_file = f"assets/translations/{self.language}.json"
        if not os.path.exists(lang_file):
            # Fallback to a default language if the selected one doesn't exist
            self.language = 'en'
            lang_file = f"assets/translations/{self.language}.json"
            settings_manager.set('language', self.language)

        if os.path.exists(lang_file):
            with open(lang_file, 'r', encoding='utf-8') as f:
                return json.load(f)
        return {}

    def translate(self, key, default=None):
        return self.translations.get(key, default if default is not None else key)

    def set_language(self, language_code):
        self.language = language_code
        settings_manager.set('language', self.language)
        self.translations = self.load_translations()

translator = Translator()
