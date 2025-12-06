import json
import os
from PySide6.QtWidgets import (
    QWidget, QHBoxLayout, QVBoxLayout, QListWidget, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QSplitter, QFormLayout, QGroupBox, QSpinBox, QDoubleSpinBox
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from api.elevenlabs import ElevenLabsAPI

class LanguagesTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.elevenlabs_api = ElevenLabsAPI()
        self.elevenlabs_templates = []
        self.voicemaker_voices = []
        self.gemini_voices = []
        self.current_lang_id = None
        self.load_voicemaker_voices()
        self.load_gemini_voices()
        self.init_ui()
        self.load_languages()
        self.load_elevenlabs_templates()
        self.retranslate_ui()
        if self.lang_list_widget.count() > 0:
            self.lang_list_widget.setCurrentRow(0)

    def load_voicemaker_voices(self):
        try:
            with open("assets/voicemaker_voices.json", "r", encoding="utf-8") as f:
                self.voicemaker_voices = json.load(f)
        except Exception as e:
            print(f"Error loading voicemaker voices: {e}")
            self.voicemaker_voices = []
            
    def load_gemini_voices(self):
        try:
            with open("assets/gemini_tts_voices.json", "r", encoding="utf-8") as f:
                self.gemini_voices = json.load(f)
        except Exception as e:
            print(f"Error loading gemini voices: {e}")
            self.gemini_voices = []

    def init_ui(self):
        main_layout = QHBoxLayout(self)
        splitter = QSplitter(Qt.Orientation.Horizontal)
        main_layout.addWidget(splitter)

        # --- Left Panel ---
        left_panel = QWidget()
        left_layout = QVBoxLayout(left_panel)
        splitter.addWidget(left_panel)

        self.lang_list_widget = QListWidget()
        self.lang_list_widget.currentItemChanged.connect(self.on_language_selected)
        left_layout.addWidget(self.lang_list_widget)

        self.add_remove_group = QGroupBox()
        add_remove_layout = QVBoxLayout(self.add_remove_group)
        left_layout.addWidget(self.add_remove_group)
        
        add_form_layout = QFormLayout()
        self.lang_name_label = QLabel()
        self.lang_name_input = QLineEdit()
        self.lang_id_label = QLabel()
        self.lang_id_input = QLineEdit()
        add_form_layout.addRow(self.lang_name_label, self.lang_name_input)
        add_form_layout.addRow(self.lang_id_label, self.lang_id_input)
        add_remove_layout.addLayout(add_form_layout)

        add_remove_buttons_layout = QHBoxLayout()
        self.add_lang_button = QPushButton()
        self.add_lang_button.clicked.connect(self.add_language)
        self.remove_lang_button = QPushButton()
        self.remove_lang_button.clicked.connect(self.remove_language)
        add_remove_buttons_layout.addWidget(self.add_lang_button)
        add_remove_buttons_layout.addWidget(self.remove_lang_button)
        add_remove_layout.addLayout(add_remove_buttons_layout)
        
        # --- Right Panel ---
        self.right_panel = QWidget()
        right_layout = QVBoxLayout(self.right_panel)
        splitter.addWidget(self.right_panel)

        self.prompt_label = QLabel()
        self.prompt_edit = QTextEdit()
        self.prompt_edit.textChanged.connect(self.save_current_language_settings)
        
        settings_layout = QFormLayout()
        
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.model_label, self.model_combo)

        self.tokens_label = QLabel()
        self.tokens_spinbox = QSpinBox()
        self.tokens_spinbox.setRange(0, 128000)
        self.tokens_spinbox.setValue(4096)
        self.tokens_spinbox.valueChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.tokens_label, self.tokens_spinbox)

        self.temperature_label = QLabel(translator.translate("temperature_label"))
        self.temperature_spinbox = QDoubleSpinBox()
        self.temperature_spinbox.setRange(0.0, 2.0)
        self.temperature_spinbox.setSingleStep(0.1)
        self.temperature_spinbox.setValue(0.7)
        self.temperature_spinbox.valueChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.temperature_label, self.temperature_spinbox)

        # TTS Provider
        self.tts_provider_label = QLabel("TTS Provider:")
        self.tts_provider_combo = QComboBox()
        self.tts_provider_combo.addItems(["ElevenLabs", "VoiceMaker", "GeminiTTS"])
        self.tts_provider_combo.currentIndexChanged.connect(self.on_tts_provider_changed)
        self.tts_provider_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.tts_provider_label, self.tts_provider_combo)

        # ElevenLabs Settings
        self.elevenlabs_template_label = QLabel()
        self.elevenlabs_template_combo = QComboBox()
        self.elevenlabs_template_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.elevenlabs_template_label, self.elevenlabs_template_combo)

        # VoiceMaker Settings
        self.voicemaker_voice_label = QLabel("VoiceMaker Voice:")
        self.voicemaker_voice_combo = QComboBox()
        self.voicemaker_voice_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.voicemaker_voice_label, self.voicemaker_voice_combo)

        # GeminiTTS Settings
        self.gemini_voice_label = QLabel("Gemini Voice:")
        self.gemini_voice_combo = QComboBox()
        for voice in self.gemini_voices:
            self.gemini_voice_combo.addItem(f"{voice['name']} ({voice['description']})", voice['value'])
        self.gemini_voice_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.gemini_voice_label, self.gemini_voice_combo)

        self.gemini_tone_label = QLabel("Gemini Tone:")
        self.gemini_tone_input = QLineEdit()
        self.gemini_tone_input.setPlaceholderText("sad, excited, whispering... or a full instruction")
        self.gemini_tone_input.textChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.gemini_tone_label, self.gemini_tone_input)



        right_layout.addWidget(self.prompt_label)
        right_layout.addWidget(self.prompt_edit)
        right_layout.addLayout(settings_layout)
        right_layout.addStretch()

        # Set initial state
        self.right_panel.setVisible(False)
        splitter.setSizes([200, 600])

    def load_languages(self):
        self.lang_list_widget.clear()
        languages = self.settings.get("languages_config", {})
        for lang_id, config in languages.items():
            display_name = config.get("display_name", lang_id)
            self.lang_list_widget.addItem(f"{display_name} [{lang_id}]")
        self.load_models()

    def load_models(self):
        self.model_combo.clear()
        models = self.settings.get("openrouter_models", [])
        self.model_combo.addItems(models)

    def load_elevenlabs_templates(self):
        self.elevenlabs_template_combo.clear()
        self.elevenlabs_templates, status = self.elevenlabs_api.get_templates()
        if status == "connected" and self.elevenlabs_templates:
            for template in self.elevenlabs_templates:
                self.elevenlabs_template_combo.addItem(template["name"], template["uuid"])
        else:
            self.elevenlabs_template_combo.addItem(translator.translate("no_templates_found"), "")

    def populate_voicemaker_voices(self, lang_id):
        self.voicemaker_voice_combo.clear()
        
        # Try to find voices matching the lang_id (e.g. "en-US", "uk-UA")
        matched_voices = []
        other_voices = []
        
        # Normalize lang_id for comparison
        normalized_lang_id = lang_id.lower().replace("_", "-")
        
        for lang_data in self.voicemaker_voices:
            code = lang_data.get("LanguageCode", "").lower()
            # Check for exact match or prefix match (e.g. 'en' matches 'en-US', 'en-GB')
            # But be careful: 'en' should match 'en-*'
            is_match = False
            if code == normalized_lang_id:
                is_match = True
            elif normalized_lang_id in code: # Loose matching
                is_match = True
            elif code.startswith(normalized_lang_id + "-"):
                is_match = True
                
            for voice_id in lang_data.get("Voices", []):
                item_text = f"{lang_data['Language']}: {voice_id}"
                if is_match:
                    matched_voices.append((item_text, voice_id))
                else:
                    other_voices.append((item_text, voice_id))
        
        # Add matched voices first
        if matched_voices:
             for text, data in matched_voices:
                 self.voicemaker_voice_combo.addItem(text, data)
             self.voicemaker_voice_combo.insertSeparator(self.voicemaker_voice_combo.count())
        
        # Add other voices
        for text, data in other_voices:
            self.voicemaker_voice_combo.addItem(text, data)

    def on_tts_provider_changed(self, index):
        provider = self.tts_provider_combo.currentText()
        if provider == "ElevenLabs":
            self.elevenlabs_template_label.setVisible(True)
            self.elevenlabs_template_combo.setVisible(True)
            self.voicemaker_voice_label.setVisible(False)
            self.voicemaker_voice_combo.setVisible(False)
            self.gemini_voice_label.setVisible(False)
            self.gemini_voice_combo.setVisible(False)
            self.gemini_tone_label.setVisible(False)
            self.gemini_tone_input.setVisible(False)
        elif provider == "VoiceMaker":
            self.elevenlabs_template_label.setVisible(False)
            self.elevenlabs_template_combo.setVisible(False)
            self.voicemaker_voice_label.setVisible(True)
            self.voicemaker_voice_combo.setVisible(True)
            self.gemini_voice_label.setVisible(False)
            self.gemini_voice_combo.setVisible(False)
            self.gemini_tone_label.setVisible(False)
            self.gemini_tone_input.setVisible(False)
        elif provider == "GeminiTTS":
            self.elevenlabs_template_label.setVisible(False)
            self.elevenlabs_template_combo.setVisible(False)
            self.voicemaker_voice_label.setVisible(False)
            self.voicemaker_voice_combo.setVisible(False)
            self.gemini_voice_label.setVisible(True)
            self.gemini_voice_combo.setVisible(True)
            self.gemini_tone_label.setVisible(True)
            self.gemini_tone_input.setVisible(True)

    def on_language_selected(self, current, previous):
        if not current:
            self.right_panel.setVisible(False)
            self.current_lang_id = None
            return

        lang_text = current.text()
        try:
            self.current_lang_id = lang_text.split('[')[-1][:-1]
        except IndexError:
            self.current_lang_id = None
            self.right_panel.setVisible(False)
            return

        languages = self.settings.get("languages_config", {})
        config = languages.get(self.current_lang_id)

        if not config:
            self.right_panel.setVisible(False)
            return

        self.prompt_edit.blockSignals(True)
        self.model_combo.blockSignals(True)
        self.tokens_spinbox.blockSignals(True)
        self.temperature_spinbox.blockSignals(True)
        self.elevenlabs_template_combo.blockSignals(True)
        self.tts_provider_combo.blockSignals(True)
        self.voicemaker_voice_combo.blockSignals(True)
        self.gemini_voice_combo.blockSignals(True)
        self.gemini_tone_input.blockSignals(True)

        self.prompt_edit.setPlainText(config.get("prompt", ""))
        
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)

        # TTS Provider
        tts_provider = config.get("tts_provider", "ElevenLabs")
        provider_index = self.tts_provider_combo.findText(tts_provider)
        self.tts_provider_combo.setCurrentIndex(provider_index if provider_index >= 0 else 0)

        # ElevenLabs Template
        current_template_uuid = config.get("elevenlabs_template_uuid", "")
        template_index = self.elevenlabs_template_combo.findData(current_template_uuid)
        self.elevenlabs_template_combo.setCurrentIndex(template_index if template_index >= 0 else 0)

        # VoiceMaker Voice
        self.populate_voicemaker_voices(self.current_lang_id)
        current_voicemaker_voice = config.get("voicemaker_voice_id", "")
        voice_index = self.voicemaker_voice_combo.findData(current_voicemaker_voice)
        self.voicemaker_voice_combo.setCurrentIndex(voice_index if voice_index >= 0 else 0)

        # GeminiTTS Settings
        current_gemini_voice = config.get("gemini_voice", "Puck")
        gemini_index = self.gemini_voice_combo.findData(current_gemini_voice)
        self.gemini_voice_combo.setCurrentIndex(gemini_index if gemini_index >= 0 else 0)
        self.gemini_tone_input.setText(config.get("gemini_tone", ""))

        self.tokens_spinbox.setValue(config.get("max_tokens", 4096))
        self.temperature_spinbox.setValue(config.get("temperature", 0.7))

        self.on_tts_provider_changed(self.tts_provider_combo.currentIndex())

        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.tokens_spinbox.blockSignals(False)
        self.temperature_spinbox.blockSignals(False)
        self.elevenlabs_template_combo.blockSignals(False)
        self.tts_provider_combo.blockSignals(False)
        self.voicemaker_voice_combo.blockSignals(False)
        self.gemini_voice_combo.blockSignals(False)
        self.gemini_tone_input.blockSignals(False)
        
        self.right_panel.setVisible(True)

    def add_language(self):
        display_name = self.lang_name_input.text().strip()
        lang_id = self.lang_id_input.text().strip()

        if not display_name or not lang_id:
            return

        languages = self.settings.get("languages_config", {})
        if lang_id in languages:
            return
            
        languages[lang_id] = {
            "display_name": display_name, 
            "prompt": "", 
            "model": "",
            "max_tokens": 4096,
            "temperature": 0.7,
            "tts_provider": "ElevenLabs",
            "elevenlabs_template_uuid": "",
            "voicemaker_voice_id": "",
            "gemini_voice": "Puck",
            "gemini_tone": ""
        }
        self.settings.set("languages_config", languages)
        
        self.lang_name_input.clear()
        self.lang_id_input.clear()
        self.load_languages()

    def remove_language(self):
        current_item = self.lang_list_widget.currentItem()
        if not current_item:
            return

        lang_text = current_item.text()
        try:
            lang_id_to_remove = lang_text.split('[')[-1][:-1]
        except IndexError:
            return

        languages = self.settings.get("languages_config", {})
        if lang_id_to_remove in languages:
            del languages[lang_id_to_remove]
            self.settings.set("languages_config", languages)
            self.load_languages()
            self.right_panel.setVisible(False)

    def save_current_language_settings(self):
        if not self.current_lang_id:
            return

        languages = self.settings.get("languages_config", {})
        if self.current_lang_id in languages:
            languages[self.current_lang_id]["prompt"] = self.prompt_edit.toPlainText()
            languages[self.current_lang_id]["model"] = self.model_combo.currentText()
            languages[self.current_lang_id]["max_tokens"] = self.tokens_spinbox.value()
            languages[self.current_lang_id]["temperature"] = self.temperature_spinbox.value()
            
            languages[self.current_lang_id]["tts_provider"] = self.tts_provider_combo.currentText()
            
            selected_template_index = self.elevenlabs_template_combo.currentIndex()
            if selected_template_index >= 0:
                template_uuid = self.elevenlabs_template_combo.itemData(selected_template_index)
                languages[self.current_lang_id]["elevenlabs_template_uuid"] = template_uuid
            
            selected_voice_index = self.voicemaker_voice_combo.currentIndex()
            if selected_voice_index >= 0:
                voice_id = self.voicemaker_voice_combo.itemData(selected_voice_index)
                languages[self.current_lang_id]["voicemaker_voice_id"] = voice_id

            selected_gemini_index = self.gemini_voice_combo.currentIndex()
            if selected_gemini_index >= 0:
                gemini_voice = self.gemini_voice_combo.itemData(selected_gemini_index)
                languages[self.current_lang_id]["gemini_voice"] = gemini_voice

            languages[self.current_lang_id]["gemini_tone"] = self.gemini_tone_input.text()

            self.settings.set("languages_config", languages)

    def update_fields(self):
        # Store current selection
        current_item = self.lang_list_widget.currentItem()
        current_lang_text = current_item.text() if current_item else None

        # Reload languages and models from settings
        self.load_languages()
        self.load_models()
        self.load_elevenlabs_templates()

        # Try to restore selection
        if current_lang_text:
            items = self.lang_list_widget.findItems(current_lang_text, Qt.MatchExactly)
            if items:
                self.lang_list_widget.setCurrentItem(items[0])
            elif self.lang_list_widget.count() > 0:
                self.lang_list_widget.setCurrentRow(0)
        elif self.lang_list_widget.count() > 0:
            self.lang_list_widget.setCurrentRow(0)

        # If no item is selected and list is not empty, select first
        if self.lang_list_widget.currentItem() is None and self.lang_list_widget.count() > 0:
            self.lang_list_widget.setCurrentRow(0)

        # Manually trigger update if the selection didn't change but content did
        if self.lang_list_widget.currentItem():
            self.on_language_selected(self.lang_list_widget.currentItem(), None)

    def retranslate_ui(self):
        self.add_remove_group.setTitle(translator.translate("manage_languages"))
        self.lang_name_label.setText(translator.translate("language_name_label"))
        self.lang_id_label.setText(translator.translate("language_id_label"))
        self.add_lang_button.setText(translator.translate("add_model"))
        self.remove_lang_button.setText(translator.translate("remove_model"))
        self.prompt_label.setText(translator.translate("language_prompt_label"))
        self.model_label.setText(translator.translate("translation_model_label"))
        self.elevenlabs_template_label.setText(translator.translate("elevenlabs_template_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))
        self.tts_provider_label.setText(translator.translate("tts_provider_label"))
        self.voicemaker_voice_label.setText(translator.translate("voicemaker_voice_label"))
        self.gemini_voice_label.setText(translator.translate("gemini_voice_label"))
        self.temperature_label.setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")

