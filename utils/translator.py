import json
import os
from PySide6.QtCore import QObject, Signal

class Translator(QObject):
    language_changed = Signal()

    def __init__(self):
        super().__init__()
        self._translations = {}
        self._current_lang = "en"
        self._load_translations()

    def _load_translations(self):
        # Build an absolute path to the translations directory
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)
        translations_dir = os.path.join(project_root, "assets", "translations")

        if not os.path.isdir(translations_dir):
            print(f"Error: Translations directory not found at {translations_dir}")
            return

        for filename in os.listdir(translations_dir):
            if filename.endswith(".json"):
                lang_code = filename.split(".")[0]
                filepath = os.path.join(translations_dir, filename)
                with open(filepath, "r", encoding="utf-8") as f:
                    self._translations[lang_code] = json.load(f)

    def set_language(self, lang_code):
        if lang_code in self._translations:
            self._current_lang = lang_code
            self.language_changed.emit()

    def tr(self, key):
        return self._translations.get(self._current_lang, {}).get(key, key)

# Singleton instance
translator = Translator()
