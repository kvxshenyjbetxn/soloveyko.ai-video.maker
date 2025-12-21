import json
import os
import platform
import sys
import copy
from PySide6.QtCore import QStandardPaths

class SettingsManager:
    def __init__(self, settings_file='config/settings.json'):
        self.base_path = self._get_base_path()
        self.settings_file = os.path.join(self.base_path, settings_file)
        self.defaults = {
            'language': 'uk',
            'theme': 'dark',
            'results_path': '',
            'image_review_enabled': False,
            'image_prompt_count_check_enabled': False,
            'image_prompt_count': 50,
            'detailed_logging_enabled': False,
            'voicemaker_api_key': '',
            'voicemaker_char_limit': 2900,
            'gemini_tts_api_key': '',
            'gemini_tts_url': 'https://gemini-tts-server-beta-production.up.railway.app',
            'googler': {
                'api_key': '',
                'aspect_ratio': 'IMAGE_ASPECT_RATIO_LANDSCAPE',
                'max_threads': 25,
                'max_video_threads': 10,
                'video_prompt': 'Animate this scene, cinematic movement, 4k',
                'seed': '',
                'negative_prompt': 'blood'
            },
            'pollinations': {
                'model': 'flux',
                'token': '',
                'width': 1280,
                'height': 720,
                'nologo': True,
                'enhance': False
            },
            'subtitles': {
                'whisper_model': 'base.bin', # Замість whisper_exe тепер зберігаємо тільки модель
                'font': 'Arial',
                'fontsize': 60,
                'color': [255, 255, 255],
                'fade_in': 0,
                'fade_out': 0,
                'margin_v': 50,
                'max_words': 10
            },
            'custom_stages': [],
            'last_used_template_name': ''
        }
        self.settings = self.load_settings()

    def _get_base_path(self):
        if platform.system() == "Darwin":
            # На macOS використовуємо стандартний шлях для даних додатку
            # Намагаємось отримати через Qt, але з перевіркою
            try:
                from PySide6.QtCore import QCoreApplication
                if QCoreApplication.instance():
                    base = QStandardPaths.writableLocation(QStandardPaths.AppDataLocation)
                else:
                    # Фоллбек якщо QApplication ще не створено
                    base = os.path.expanduser("~/Library/Application Support/CombainAI")
            except Exception as e:
                print(f"DEBUG: Error getting macOS path via Qt: {e}")
                base = os.path.expanduser("~/Library/Application Support/CombainAI")

            # Якщо назва додатку не додалась автоматично
            if not base.endswith("CombainAI"):
                base = os.path.join(base, "CombainAI")
            
            try:
                os.makedirs(base, exist_ok=True)
            except Exception as e:
                print(f"DEBUG: Error creating directory {base}: {e}")
            
            print(f"DEBUG: macOS Data Path is: {base}")
            return base
            
        if getattr(sys, 'frozen', False):
            # Поруч з EXE (Windows)
            return os.path.dirname(sys.executable)
        else:
            # Корень проекту (батьківська папка utils)
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def load_settings(self):
        print(f"DEBUG: Loading settings from {self.settings_file}")
        
        current_settings = {}
        if os.path.exists(self.settings_file):
            # Список кодувань для спроби
            encodings = ['utf-8-sig', 'utf-8', 'utf-16', 'cp1251']
            
            for enc in encodings:
                try:
                    with open(self.settings_file, 'r', encoding=enc) as f:
                        loaded = json.load(f)
                        if isinstance(loaded, dict):
                            current_settings = loaded
                            print(f"DEBUG: Successfully loaded settings with {enc}")
                            break
                        else:
                            print(f"DEBUG: Loaded settings is not a dictionary ({type(loaded)})")
                except (json.JSONDecodeError, UnicodeDecodeError):
                    continue 
                except Exception as e:
                    print(f"DEBUG: Error loading settings with {enc}: {e}")
                    break
        else:
            print("DEBUG: Settings file does not exist, using defaults.")

        # РЕКУРСИВНО заповнюємо відсутні ключі з дефолтів
        self._deep_merge(self.defaults, current_settings)
        return current_settings

    def _deep_merge(self, source, destination):
        """
        Рекурсивно копіює ключі з source в destination, якщо їх там немає.
        """
        for key, value in source.items():
            if isinstance(value, dict):
                # Якщо ключа немає або він має Не-словниковий тип (стара версія конфігу)
                if key not in destination or not isinstance(destination[key], dict):
                    destination[key] = copy.deepcopy(value)
                else:
                    self._deep_merge(value, destination[key])
            else:
                if key not in destination:
                    destination[key] = value
        return destination

    def get(self, key, default=None):
        if default is None:
            default = self.defaults.get(key)
        return self.settings.get(key, default)

    def set(self, key, value):
        self.settings[key] = value
        self.save_settings()

    def save_settings(self):
        try:
            os.makedirs(os.path.dirname(self.settings_file), exist_ok=True)
            with open(self.settings_file, 'w', encoding='utf-8') as f:
                json.dump(self.settings, f, indent=4, ensure_ascii=False)
                f.flush()
                os.fsync(f.fileno())
        except Exception as e:
            print(f"Error saving settings: {e}")

class TemplateManager:
    def __init__(self, template_dir='config/templates'):
        self.base_path = self._get_base_path()
        self.template_dir = os.path.join(self.base_path, template_dir)
        os.makedirs(self.template_dir, exist_ok=True)

    def _get_base_path(self):
        if platform.system() == "Darwin":
            try:
                from PySide6.QtCore import QCoreApplication
                if QCoreApplication.instance():
                    base = QStandardPaths.writableLocation(QStandardPaths.AppDataLocation)
                else:
                    base = os.path.expanduser("~/Library/Application Support/CombainAI")
            except:
                base = os.path.expanduser("~/Library/Application Support/CombainAI")
                
            if not base.endswith("CombainAI"):
                base = os.path.join(base, "CombainAI")
            return base

        if getattr(sys, 'frozen', False):
            return os.path.dirname(sys.executable)
        else:
            return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def get_templates(self):
        templates = [f.split('.')[0] for f in os.listdir(self.template_dir) if f.endswith('.json')]
        return sorted(templates)

    def load_template(self, name):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        if os.path.exists(file_path):
            with open(file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        return {}

    def save_template(self, name, data):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=4, ensure_ascii=False)

    def delete_template(self, name):
        file_path = os.path.join(self.template_dir, f"{name}.json")
        if os.path.exists(file_path):
            os.remove(file_path)

    def rename_template(self, old_name, new_name):
        old_path = os.path.join(self.template_dir, f"{old_name}.json")
        new_path = os.path.join(self.template_dir, f"{new_name}.json")
        if os.path.exists(old_path) and not os.path.exists(new_path):
            os.rename(old_path, new_path)

settings_manager = SettingsManager()
template_manager = TemplateManager()
