import json
import os
import sys
from utils.settings import settings_manager

class Translator:
    def __init__(self):
        self.language = settings_manager.get('language')
        self.translations = self.load_translations()

    def _get_assets_path(self):
        if getattr(sys, 'frozen', False):
            return sys._MEIPASS
        else:
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def load_translations(self):
        base_path = self._get_assets_path()
        lang_file = os.path.join(base_path, "assets", "translations", f"{self.language}.json")
        
        if not os.path.exists(lang_file):
            # Fallback to a default language if the selected one doesn't exist
            self.language = 'en'
            lang_file = os.path.join(base_path, "assets", "translations", f"{self.language}.json")
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
