import json
import os
import sys
from utils.settings import settings_manager

class HintManager:
    _instance = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(HintManager, cls).__new__(cls)
            cls._instance.hints = {}
            cls._instance.language = settings_manager.get('language', 'uk')
            cls._instance.load_hints()
        return cls._instance

    def _get_assets_path(self):
        if getattr(sys, 'frozen', False):
            return sys._MEIPASS
        else:
            # Assumes this file is in utils/
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def load_hints(self):
        self.language = settings_manager.get('language', 'uk')
        base_path = self._get_assets_path()
        lang_file = os.path.join(base_path, "assets", "translations", "hints", f"{self.language}.json")
        
        if not os.path.exists(lang_file):
            lang_file = os.path.join(base_path, "assets", "translations", "hints", "uk.json")

        if os.path.exists(lang_file):
            try:
                with open(lang_file, 'r', encoding='utf-8') as f:
                    self.hints = json.load(f)
            except Exception as e:
                print(f"Error loading hints: {e}")
                self.hints = {}
        else:
            self.hints = {}

    def get_hint(self, key):
        return self.hints.get(key, "")

hint_manager = HintManager()
