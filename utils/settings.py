import os
import json

class AppSettings:
    def __init__(self):
        # Define the path for the config file
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)
        self.config_dir = os.path.join(project_root, "config")
        self.config_path = os.path.join(self.config_dir, "settings.json")
        
        self.data = {}
        self._load()

    def _load(self):
        """Loads settings from the JSON file, or creates it with defaults."""
        if not os.path.exists(self.config_dir):
            os.makedirs(self.config_dir)
            
        if os.path.exists(self.config_path):
            with open(self.config_path, 'r', encoding='utf-8') as f:
                self.data = json.load(f)
        else:
            # Default settings
            self.data = {
                "theme": "Light",
                "language": "en"
            }
            self._save()

    def _save(self):
        """Saves the current settings to the JSON file."""
        with open(self.config_path, 'w', encoding='utf-8') as f:
            json.dump(self.data, f, indent=4)

    def get_theme(self):
        return self.data.get("theme", "Light")

    def set_theme(self, theme_name):
        self.data["theme"] = theme_name
        self._save()

    def get_language(self):
        return self.data.get("language", "en")

    def set_language(self, lang_code):
        self.data["language"] = lang_code
        self._save()
