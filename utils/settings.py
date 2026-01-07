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
            'image_prompt_count_check_enabled': True,
            'image_prompt_count': 50,
            'detailed_logging_enabled': True,
            'voicemaker_api_key': '',
            'voicemaker_char_limit': 2900,
            'gemini_tts_api_key': '',
            'gemini_tts_url': 'https://gemini-tts-server-beta-production.up.railway.app',
            'image_generation_provider': 'pollinations',
            'image_generation_prompt': "",
            'image_prompt_settings': {
                'prompt': """You are an expert in creating prompts for generating images on Imagen 4. Your task is to analyze the story text and create EXACTLY 50 high-quality prompts for photorealistic visual accompaniment with consistent characters and unified style.
CRITICAL: YOU MUST CREATE EXACTLY 50 PROMPTS, NOT 30, NOT 40, BUT EXACTLY 50 PROMPTS
STEP 1: IDENTIFY THE HOOK
First, find the HOOK section of the story:

The HOOK is everything BEFORE phrases like "Прежде чем начнем", "Але перш ніж почнемо", "But before we begin", or similar transitional phrases
This hook section is crucial for grabbing attention
The first 5 prompts MUST be based ONLY on this hook section

STEP 2: CHARACTER ANALYSIS
Read the entire story including the hook and identify:

Main characters: their age, gender, physical appearance, hair color and style, eye color, distinctive features, typical clothing
Supporting characters: their key characteristics
Setting details: time period, location, season, atmosphere

Create a CHARACTER REFERENCE LIST with detailed descriptions that will be used consistently across all prompts.
STEP 3: SCENE DISTRIBUTION
The story must be divided as follows:

Prompts 1-5: HOOK section ONLY (the attention-grabbing opening before "Прежде чем начнем")
Prompts 6-50: Main story (45 prompts evenly distributed):

Beginning: prompts 6-15
Early middle: prompts 16-25
Middle: prompts 26-35
Late middle: prompts 36-45
Ending: prompts 46-50



STEP 4: STYLE DEFINITION
Define ONE unified photorealistic style for all images:

Photorealistic style, looks like a real photograph
Consistent color palette throughout the series
Professional cinematography with natural depth of field
High quality realistic rendering

RULES FOR CREATING PROMPTS:

Character Consistency:

Use EXACT same physical descriptions for each character in every prompt they appear
Include: age, gender, hair color and length, eye color, build, distinctive features
Maintain consistent clothing appropriate to the scene


Scene Selection:

First 5 prompts: key visual moments from the HOOK only
Remaining 45 prompts: evenly distributed throughout the main story
Ensure scenes directly reflect the plot


Photorealistic Style Requirements:

Every prompt MUST include: "photorealistic, looks like a real photograph"
Add: "realistic skin textures, natural lighting, real life photography"
Include: "shot on professional camera, 35mm lens"
NEVER use words like: cartoon, animated, illustration, drawing, painting, anime, sketch


Absolutely NO TEXT on Images:

NEVER include dialogue, subtitles, captions, or any text elements
Do NOT mention: speech bubbles, text overlays, subtitles, captions, words, letters
Add explicitly: "no text, no words, no subtitles, no captions"
Focus only on visual storytelling through action and expression


Scene Description Structure:

Main subject with consistent character details
Action or emotion being portrayed
Setting and environment details
Lighting and atmosphere
Camera angle and composition
Photorealistic style markers
Explicit "no text" instruction


Safety:

No blood, violence, or disturbing imagery
Keep all scenes appropriate and tasteful


Format:

Each prompt in English
Use only commas to separate elements
No special characters, brackets, or quotation marks
Number each prompt from 1 to 50



OUTPUT FORMAT:
HOOK SECTION IDENTIFIED:
[Brief description of what the hook covers]
CHARACTER REFERENCE:
[List main characters with full consistent descriptions]
UNIFIED STYLE:
Photorealistic cinematic photography, looks like real photographs, realistic skin textures, natural lighting, shot on professional camera, 35mm lens, high quality, no text, no subtitles, no captions
PROMPTS:
HOOK SECTION (Prompts 1-5):

Photorealistic photograph, [scene from hook], [character details], [setting], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality
Photorealistic photograph, [second scene from hook], [details], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality
Photorealistic photograph, [third scene from hook], [details], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality
Photorealistic photograph, [fourth scene from hook], [details], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality
Photorealistic photograph, [fifth scene from hook], [details], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality

MAIN STORY (Prompts 6-50):

Photorealistic photograph, [beginning of main story], [character details], [setting], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality

7-15. [Continue with beginning section]
16-25. [Early middle section]
26-35. [Middle section]
36-45. [Late middle section]
46-50. [Ending section - final 5 prompts from conclusion]

Photorealistic photograph, [final scene], natural lighting, realistic skin textures, shot on 35mm camera, professional cinematography, no text, no subtitles, no words, high quality

VERIFICATION BEFORE SUBMITTING:

Count prompts 1-5: these MUST be from the hook section only
Count prompts 6-50: these are from the main story (45 prompts)
Total count MUST be exactly 50 prompts

CRITICAL REMINDERS:

First 5 prompts are ONLY from the HOOK (before "Прежде чем начнем" or similar phrase)
Prompts 6-50 cover the main story after the hook
ALWAYS include "photorealistic, looks like a real photograph" in every prompt
ALWAYS include "no text, no subtitles, no words" in every prompt
NEVER use illustration, cartoon, anime, or drawing style words
YOU MUST CREATE EXACTLY 50 PROMPTS - count them before finishing

TASK:
Read the provided story text carefully, identify the HOOK section, create the character reference list and unified photorealistic style definition, then generate EXACTLY 50 PROMPTS following the structure above: 5 prompts for the hook, 45 prompts for the main story.
!!!I don't use names in promts!!!
NO HUMAN NAMES IN PROMTS!

STORY TEXT:
""",
                'model': 'google/gemini-2.5-flash',
                'max_tokens': 10000,
                'temperature': 1.0
            },
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
            'openrouter_models': ['google/gemini-2.5-flash'],
            'subtitles': {
                'whisper_model': 'base.bin',
                'font': 'Arial',
                'fontsize': 60,
                'color': [255, 255, 255],
                'fade_in': 150,
                'fade_out': 150,
                'margin_v': 50,
                'max_words': 10
            },
            'montage': {
                'preset': 'superfast',
                'bitrate_mbps': 5,
                'upscale_factor': 2,
                'transition_duration': 2,
                'enable_sway': True,
                'max_concurrent_montages': 1
            },
            'languages_config': {
                'uk': {
                    'display_name': 'Ukraine',
                    'prompt': """**GENERAL PRINCIPLES:**

* Translate **ALL** text completely, without cuts or omissions.
* Preserve the original structure and narrative style.
* **FULLY** adapt **ALL** cultural elements to be familiar and natural for Ukrainian readers.

**NAMES AND FORMS OF ADDRESS:**

* Adapt **ALL** names to Ukrainian equivalents (e.g., Алёна → Olena, Игорь → Ihor, Дмитрий → Dmytro).
* Handle patronymics appropriately for a Ukrainian context, replacing them with natural forms of address (e.g., first name in conversation, or "pan/pani" + first name in formal settings).
* Use appropriate Ukrainian titles and forms of courtesy (`pan`, `pani`, etc.).

**GEOGRAPHY AND COMPLETE LOCALIZATION:**

* Replace **ALL** geographical references with Ukrainian regions (e.g., taiga → Carpathian forests, Polissia marshlands, or the Kherson steppes).
* Adapt climate and landscape to familiar Ukrainian environments.
* Replace Russian/Siberian settings with equivalent Ukrainian locations (e.g., the Carpathian Mountains, Kyiv, Lviv, the Black Sea coast).
* Use familiar Ukrainian flora and fauna (e.g., bison, bears, wolves, storks).

**LANGUAGE AND STYLE:**

* Use natural Ukrainian idioms and expressions instead of literal translation.
* Adapt dialogues to natural Ukrainian conversational language.
* Preserve emotional weight and atmosphere while making it culturally Ukrainian.
* Use appropriate regional Ukrainian variants where fitting (e.g., Hutsul, Polissian dialects).

**CULTURAL ELEMENTS - COMPLETE ADAPTATION:**

* Replace **ALL** cultural references: food (e.g., *pelmeni* → *varenyky*, *shchi* → *borscht*), clothing (*kosovorotka* → *vyshyvanka*), traditions, and institutions (FSB → SBU).
* Adapt occupations and social structures to Ukrainian equivalents.
* Replace the wildlife conservation context to familiar Ukrainian regions (e.g., Askania-Nova Biosphere Reserve, Carpathian National Nature Park).
* Change all cultural practices to Ukrainian equivalents.
* Adapt government institutions, educational systems, and social norms.

**SETTING ADAPTATION:**

* Transform the Siberian wilderness into a familiar Ukrainian natural environment (e.g., Carpathian forests, Polissia marshlands).
* Adapt the reserve/conservation context to Ukrainian national parks or biosphere reserves.
* Replace all Russian cultural elements with Ukrainian equivalents.

The result should read like an original Ukrainian text set in Ukraine, written for Ukrainian audiences, with **NO** foreign cultural elements remaining, while preserving all plot elements and emotional depth of the original.

Without your comments, nothing superfluous, just text.
Don't write anything unnecessary! Write the translation text right away! Don't write comments like “here's the translation.”

story:
""",
                    'model': 'google/gemini-2.5-flash',
                    'max_tokens': 128000,
                    'temperature': 1.0,
                    'rewrite_prompt': """Role: You are an expert documentary scriptwriter and historian, specializing in Ancient Civilizations (similar to the style of "Kurzgesagt" or high-quality History Channel productions).
Task: Take the provided Russian text, analyze its core meaning and facts, and perform a deep creative rewrite directly into Ukrainian.
Goal: Create an engaging, "edutainment" style narrative about the ancient world that sounds natural, atmospheric, and highly compelling to a modern Ukrainian audience.
Language & Translation Protocols (Crucial):
Input: Russian. Output: Ukrainian.
No Calques/Russisms: Do not translate word-for-word. Completely abandon the sentence structure of the Russian source. Use authentic Ukrainian idioms, syntax, and vocabulary.
Terminology: Use correct Ukrainian historical terminology and transliteration for ancient names and places.
Style Guidelines:
Cinematic Storytelling: Make the reader/viewer see the ancient world. Describe the architecture, the climate, the smells, and the scale of events.
Engagement: Use a "hook" at the beginning. Address the audience occasionally (e.g., "Уявіть собі...", "Мало хто знає, що...").
Flow: The text should read like a fascinating mystery or a journey, not a lecture.
Tone: Awe-inspiring yet accessible. Professional but not dry.
Structure:
Intro: Grab attention immediately.
Main Body: Unfold the history with dramatic flair and sensory details.
Outro: A strong concluding thought or a bridge to modern times.
Original Russian Text: 
""",
                    'rewrite_model': 'google/gemini-2.5-flash',
                    'rewrite_max_tokens': 128000,
                    'rewrite_temperature': 1.0,
                    'tts_provider': 'EdgeTTS',
                    'edgetts_voice': 'uk-UA-OstapNeural',
                    'background_music_volume': 25,
                    'watermark_size': 5
                }
            },
            'custom_stages': [],
            'last_used_template_name': '',
            'notifications_enabled': False,
            'telegram_user_id': '',
            'show_welcome_dialog': True
        }
        self.settings = self.load_settings()

    def _get_base_path(self):
        if platform.system() == "Darwin":
            # На macOS використовуємо Application Support для уникнення PermissionError
            app_support = os.path.expanduser("~/Library/Application Support/Soloveyko.AI-Video.Maker")
            
            try:
                os.makedirs(app_support, exist_ok=True)
            except Exception as e:
                print(f"DEBUG: Error creating directory {app_support}: {e}")
            
            # Створюємо симлінк у Документах для видимості
            docs_link = os.path.expanduser("~/Documents/Soloveyko.AI-Video.Maker")
            if not os.path.exists(docs_link):
                try:
                    # Символьне посилання дозволяє бачити папку в Документах
                    os.symlink(app_support, docs_link)
                    print(f"DEBUG: Created symlink in Documents: {docs_link}")
                except Exception as e:
                    print(f"DEBUG: Could not create symlink in Documents: {e}")
                    # Це не критично, програма продовжить працювати через App Support
            
            print(f"DEBUG: macOS Data Path is: {app_support}")
            return app_support
            
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
            return os.path.expanduser("~/Library/Application Support/Soloveyko.AI-Video.Maker")

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
