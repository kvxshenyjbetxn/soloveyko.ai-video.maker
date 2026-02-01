import json
import os
import sys
import copy
from PySide6.QtWidgets import (
    QWidget, QHBoxLayout, QVBoxLayout, QListWidget, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QSplitter, QFormLayout, QGroupBox, QSpinBox, QDoubleSpinBox,
    QFileDialog, QSlider, QScrollArea, QTableWidget, QTableWidgetItem, QHeaderView, QDialog, QGridLayout,
    QSizePolicy
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager, template_manager
from api.elevenlabs import ElevenLabsAPI
from api.edge_tts_api import EdgeTTSAPI
from gui.widgets.prompt_editor_dialog import PromptEditorDialog
from gui.widgets.help_label import HelpLabel

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(".")

class OverlaySettingsDialog(QDialog):
    def __init__(self, parent=None, x=0, y=0):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("overlay_settings_title", "Position Settings"))
        self.setFixedWidth(250)
        layout = QVBoxLayout(self)

        content_layout = QGridLayout()
        
        content_layout.addWidget(QLabel(translator.translate("trigger_horizontal", "Horizontal (X):")), 0, 0)
        self.x_spin = QSpinBox()
        self.x_spin.setRange(-5000, 5000)
        self.x_spin.setValue(x)
        content_layout.addWidget(self.x_spin, 0, 1)

        content_layout.addWidget(QLabel(translator.translate("trigger_vertical", "Vertical (Y):")), 1, 0)
        self.y_spin = QSpinBox()
        self.y_spin.setRange(-5000, 5000)
        self.y_spin.setValue(y)
        content_layout.addWidget(self.y_spin, 1, 1)

        layout.addLayout(content_layout)

        btn_layout = QHBoxLayout()
        save_btn = QPushButton(translator.translate("save_button", "Save"))
        save_btn.clicked.connect(self.accept)
        cancel_btn = QPushButton(translator.translate("cancel_button", "Cancel"))
        cancel_btn.clicked.connect(self.reject)
        btn_layout.addWidget(save_btn)
        btn_layout.addWidget(cancel_btn)
        layout.addLayout(btn_layout)

    def get_values(self):
        return self.x_spin.value(), self.y_spin.value()

class LanguagesTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.settings = settings_manager
        self.elevenlabs_api = ElevenLabsAPI()
        self.edge_tts_api = EdgeTTSAPI()
        self.elevenlabs_templates = []
        self.voicemaker_voices = []
        self.gemini_voices = []
        self.edge_tts_voices = []
        self.current_lang_id = None
        self._is_loading_lang_settings = False
        self.load_voicemaker_voices()
        self.load_gemini_voices()
        self.load_edge_tts_voices()
        self.init_ui()
        self.load_templates_combo() # Load templates for the new combobox
        self.load_languages()
        self.retranslate_ui()
        if self.lang_list_widget.count() > 0:
            self.lang_list_widget.setCurrentRow(0)

    def load_voicemaker_voices(self):
        try:
            voicemaker_path = os.path.join(BASE_PATH, "assets", "voicemaker_voices.json")
            with open(voicemaker_path, "r", encoding="utf-8") as f:
                self.voicemaker_voices = json.load(f)
        except Exception as e:
            print(f"Error loading voicemaker voices: {e}")
            self.voicemaker_voices = []
            
    def load_gemini_voices(self):
        try:
            gemini_path = os.path.join(BASE_PATH, "assets", "gemini_tts_voices.json")
            with open(gemini_path, "r", encoding="utf-8") as f:
                self.gemini_voices = json.load(f)
        except Exception as e:
            print(f"Error loading gemini voices: {e}")
            self.gemini_voices = []

    def load_edge_tts_voices(self):
        try:
            self.edge_tts_voices = self.edge_tts_api.get_voices()
        except Exception as e:
            print(f"Error loading EdgeTTS voices: {e}")
            self.edge_tts_voices = []

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
        
        self.lang_name_help = HelpLabel("language_name_label")
        self.lang_name_label = QLabel()
        lang_name_container = QWidget()
        lang_name_layout = QHBoxLayout(lang_name_container)
        lang_name_layout.setContentsMargins(0, 0, 0, 0)
        lang_name_layout.setSpacing(5)
        lang_name_layout.addWidget(self.lang_name_help)
        lang_name_layout.addWidget(self.lang_name_label)
        
        self.lang_name_input = QLineEdit()
        
        self.lang_id_help = HelpLabel("language_id_label")
        self.lang_id_label = QLabel()
        lang_id_container = QWidget()
        lang_id_layout = QHBoxLayout(lang_id_container)
        lang_id_layout.setContentsMargins(0, 0, 0, 0)
        lang_id_layout.setSpacing(5)
        lang_id_layout.addWidget(self.lang_id_help)
        lang_id_layout.addWidget(self.lang_id_label)
        
        self.lang_id_input = QLineEdit()
        
        add_form_layout.addRow(lang_name_container, self.lang_name_input)
        add_form_layout.addRow(lang_id_container, self.lang_id_input)
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
        right_layout.setContentsMargins(0, 0, 0, 0) # Zero margins for the panel itself
        splitter.addWidget(self.right_panel)

        # Create Scroll Area
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setFrameShape(QScrollArea.Shape.NoFrame)
        
        # Container for settings
        settings_container = QWidget()
        settings_layout = QFormLayout(settings_container)

        # --- Translation Settings Group ---
        trans_group = QGroupBox(translator.translate("stage_translation"))
        trans_layout = QVBoxLayout(trans_group)

        # Translation Prompt Section
        self.prompt_help = HelpLabel("language_prompt_label")
        self.prompt_label = QLabel(translator.translate("language_prompt_label"))
        prompt_label_layout = QHBoxLayout()
        prompt_label_layout.setContentsMargins(0, 0, 0, 0)
        prompt_label_layout.setSpacing(5)
        prompt_label_layout.addWidget(self.prompt_help)
        prompt_label_layout.addWidget(self.prompt_label)
        prompt_label_layout.addStretch()
        trans_layout.addLayout(prompt_label_layout)

        trans_prompt_area = QHBoxLayout()
        self.prompt_edit = QTextEdit()
        self.prompt_edit.setMinimumHeight(100)
        self.prompt_edit.textChanged.connect(self.save_current_language_settings)
        trans_prompt_area.addWidget(self.prompt_edit)

        trans_edit_btn_layout = QVBoxLayout()
        self.open_editor_button = QPushButton(translator.translate("open_editor_button", "Editor"))
        self.open_editor_button.clicked.connect(self.open_prompt_editor)
        self.open_editor_button.setFixedWidth(105)
        trans_edit_btn_layout.addWidget(self.open_editor_button)
        trans_edit_btn_layout.addStretch()
        trans_prompt_area.addLayout(trans_edit_btn_layout)
        
        trans_layout.addLayout(trans_prompt_area)

        # Translation Parameters (Form Layout)
        trans_params = QFormLayout()
        
        self.model_help = HelpLabel("translation_model_label")
        self.model_label = QLabel(translator.translate("translation_model_label"))
        model_label_container = QWidget()
        model_label_layout = QHBoxLayout(model_label_container)
        model_label_layout.setContentsMargins(0, 0, 0, 0)
        model_label_layout.setSpacing(5)
        model_label_layout.addWidget(self.model_help)
        model_label_layout.addWidget(self.model_label)
        
        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_current_language_settings)
        trans_params.addRow(model_label_container, self.model_combo)

        self.tokens_help = HelpLabel("tokens_label")
        self.tokens_label = QLabel(translator.translate("tokens_label"))
        tokens_label_container = QWidget()
        tokens_label_layout = QHBoxLayout(tokens_label_container)
        tokens_label_layout.setContentsMargins(0, 0, 0, 0)
        tokens_label_layout.setSpacing(5)
        tokens_label_layout.addWidget(self.tokens_help)
        tokens_label_layout.addWidget(self.tokens_label)
        
        self.tokens_spinbox = QSpinBox()
        self.tokens_spinbox.setRange(0, 128000)
        self.tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.tokens_spinbox.setValue(4096)
        self.tokens_spinbox.valueChanged.connect(self.save_current_language_settings)
        trans_params.addRow(tokens_label_container, self.tokens_spinbox)

        self.temperature_help = HelpLabel("temperature_label")
        self.temperature_label = QLabel(translator.translate("temperature_label"))
        temp_label_container = QWidget()
        temp_label_layout = QHBoxLayout(temp_label_container)
        temp_label_layout.setContentsMargins(0, 0, 0, 0)
        temp_label_layout.setSpacing(5)
        temp_label_layout.addWidget(self.temperature_help)
        temp_label_layout.addWidget(self.temperature_label)
        
        self.temperature_spinbox = QDoubleSpinBox()
        self.temperature_spinbox.setRange(0.0, 2.0)
        self.temperature_spinbox.setSingleStep(0.1)
        self.temperature_spinbox.setValue(0.7)
        self.temperature_spinbox.valueChanged.connect(self.save_current_language_settings)
        trans_params.addRow(temp_label_container, self.temperature_spinbox)

        trans_layout.addLayout(trans_params)
        
        settings_layout.addRow(trans_group)


        # --- Rewrite Settings Group ---
        self.rewrite_group = QGroupBox(translator.translate("stage_rewrite"))
        rewrite_layout = QVBoxLayout(self.rewrite_group)

        # Rewrite Prompt Section
        self.rewrite_prompt_help = HelpLabel("rewrite_prompt_label")
        self.rewrite_prompt_label = QLabel(translator.translate("rewrite_prompt_label"))
        rewrite_prompt_label_layout = QHBoxLayout()
        rewrite_prompt_label_layout.setContentsMargins(0, 0, 0, 0)
        rewrite_prompt_label_layout.setSpacing(5)
        rewrite_prompt_label_layout.addWidget(self.rewrite_prompt_help)
        rewrite_prompt_label_layout.addWidget(self.rewrite_prompt_label)
        rewrite_prompt_label_layout.addStretch()
        rewrite_layout.addLayout(rewrite_prompt_label_layout)

        rewrite_prompt_area = QHBoxLayout()
        self.rewrite_prompt_edit = QTextEdit()
        self.rewrite_prompt_edit.setMinimumHeight(100)
        self.rewrite_prompt_edit.textChanged.connect(self.save_current_language_settings)
        rewrite_prompt_area.addWidget(self.rewrite_prompt_edit)

        rewrite_edit_btn_layout = QVBoxLayout()
        self.open_rewrite_editor_button = QPushButton(translator.translate("open_editor_button", "Editor"))
        self.open_rewrite_editor_button.clicked.connect(self.open_rewrite_prompt_editor)
        self.open_rewrite_editor_button.setFixedWidth(105)
        rewrite_edit_btn_layout.addWidget(self.open_rewrite_editor_button)
        rewrite_edit_btn_layout.addStretch()
        rewrite_prompt_area.addLayout(rewrite_edit_btn_layout)

        rewrite_layout.addLayout(rewrite_prompt_area)

        # Rewrite Parameters (Form Layout)
        rewrite_params = QFormLayout()
        
        self.rewrite_model_help = HelpLabel("rewrite_model_label")
        self.rewrite_model_label = QLabel(translator.translate("rewrite_model_label"))
        rewrite_model_label_container = QWidget()
        rewrite_model_label_layout = QHBoxLayout(rewrite_model_label_container)
        rewrite_model_label_layout.setContentsMargins(0, 0, 0, 0)
        rewrite_model_label_layout.setSpacing(5)
        rewrite_model_label_layout.addWidget(self.rewrite_model_help)
        rewrite_model_label_layout.addWidget(self.rewrite_model_label)
        
        self.rewrite_model_combo = QComboBox()
        self.rewrite_model_combo.currentIndexChanged.connect(self.save_current_language_settings)
        rewrite_params.addRow(rewrite_model_label_container, self.rewrite_model_combo)

        self.rewrite_tokens_help = HelpLabel("rewrite_tokens_label")
        self.rewrite_tokens_label = QLabel(translator.translate("tokens_label"))
        rewrite_tokens_label_container = QWidget()
        rewrite_tokens_label_layout = QHBoxLayout(rewrite_tokens_label_container)
        rewrite_tokens_label_layout.setContentsMargins(0, 0, 0, 0)
        rewrite_tokens_label_layout.setSpacing(5)
        rewrite_tokens_label_layout.addWidget(self.rewrite_tokens_help)
        rewrite_tokens_label_layout.addWidget(self.rewrite_tokens_label)
        
        self.rewrite_tokens_spinbox = QSpinBox()
        self.rewrite_tokens_spinbox.setRange(0, 128000)
        self.rewrite_tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.rewrite_tokens_spinbox.setValue(4096)
        self.rewrite_tokens_spinbox.valueChanged.connect(self.save_current_language_settings)
        rewrite_params.addRow(rewrite_tokens_label_container, self.rewrite_tokens_spinbox)

        self.rewrite_temperature_help = HelpLabel("rewrite_temperature_label")
        self.rewrite_temperature_label = QLabel(translator.translate("temperature_label"))
        rewrite_temp_label_container = QWidget()
        rewrite_temp_label_layout = QHBoxLayout(rewrite_temp_label_container)
        rewrite_temp_label_layout.setContentsMargins(0, 0, 0, 0)
        rewrite_temp_label_layout.setSpacing(5)
        rewrite_temp_label_layout.addWidget(self.rewrite_temperature_help)
        rewrite_temp_label_layout.addWidget(self.rewrite_temperature_label)
        
        self.rewrite_temperature_spinbox = QDoubleSpinBox()
        self.rewrite_temperature_spinbox.setRange(0.0, 2.0)
        self.rewrite_temperature_spinbox.setSingleStep(0.1)
        self.rewrite_temperature_spinbox.setValue(0.7)
        self.rewrite_temperature_spinbox.valueChanged.connect(self.save_current_language_settings)
        rewrite_params.addRow(rewrite_temp_label_container, self.rewrite_temperature_spinbox)
        
        rewrite_layout.addLayout(rewrite_params)

        settings_layout.addRow(self.rewrite_group)

        # Default Template
        self.default_template_help = HelpLabel("default_template_label")
        self.default_template_label = QLabel()
        template_label_container = QWidget()
        template_label_layout = QHBoxLayout(template_label_container)
        template_label_layout.setContentsMargins(0, 0, 0, 0)
        template_label_layout.setSpacing(5)
        template_label_layout.addWidget(self.default_template_help)
        template_label_layout.addWidget(self.default_template_label)
        
        self.default_template_combo = QComboBox()
        self.default_template_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(template_label_container, self.default_template_combo)

        # TTS Provider
        self.tts_provider_help = HelpLabel("tts_provider_label")
        self.tts_provider_label = QLabel("TTS Provider:")
        tts_label_container = QWidget()
        tts_label_layout = QHBoxLayout(tts_label_container)
        tts_label_layout.setContentsMargins(0, 0, 0, 0)
        tts_label_layout.setSpacing(5)
        tts_label_layout.addWidget(self.tts_provider_help)
        tts_label_layout.addWidget(self.tts_provider_label)
        
        self.tts_provider_combo = QComboBox()
        self.tts_provider_combo.addItems(["ElevenLabs", "ElevenLabsUnlim", "VoiceMaker", "GeminiTTS", "EdgeTTS"])
        self.tts_provider_combo.currentIndexChanged.connect(self.on_tts_provider_changed)
        self.tts_provider_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(tts_label_container, self.tts_provider_combo)

        # ElevenLabs Settings
        self.elevenlabs_template_help = HelpLabel("elevenlabs_template_label")
        self.elevenlabs_template_label = QLabel()
        self.elevenlabs_label_container = QWidget()
        elevenlabs_label_layout = QHBoxLayout(self.elevenlabs_label_container)
        elevenlabs_label_layout.setContentsMargins(0, 0, 0, 0)
        elevenlabs_label_layout.setSpacing(5)
        elevenlabs_label_layout.addWidget(self.elevenlabs_template_help)
        elevenlabs_label_layout.addWidget(self.elevenlabs_template_label)
        
        self.elevenlabs_template_combo = QComboBox()
        self.elevenlabs_template_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.elevenlabs_label_container, self.elevenlabs_template_combo)

        # ElevenLabs Unlim Settings
        self.eleven_unlim_group = QGroupBox("ElevenLabs Unlim Settings")
        eleven_unlim_layout = QFormLayout(self.eleven_unlim_group)

        self.eleven_unlim_voice_id_help = HelpLabel("eleven_unlim_voice_id_label")
        self.eleven_unlim_voice_id_label = QLabel("Voice ID:")
        self.voice_id_label_container = QWidget()
        voice_id_label_layout = QHBoxLayout(self.voice_id_label_container)
        voice_id_label_layout.setContentsMargins(0, 0, 0, 0)
        voice_id_label_layout.setSpacing(5)
        voice_id_label_layout.addWidget(self.eleven_unlim_voice_id_help)
        voice_id_label_layout.addWidget(self.eleven_unlim_voice_id_label)
        
        self.eleven_unlim_voice_id_input = QLineEdit()
        self.eleven_unlim_voice_id_input.textChanged.connect(self.save_current_language_settings)
        eleven_unlim_layout.addRow(self.voice_id_label_container, self.eleven_unlim_voice_id_input)

        self.eleven_unlim_stability_label = QLabel("Stability:")
        self.eleven_unlim_stability_spin = QDoubleSpinBox()
        self.eleven_unlim_stability_spin.setRange(0.0, 1.0)
        self.eleven_unlim_stability_spin.setSingleStep(0.1)
        self.eleven_unlim_stability_spin.setValue(0.5)
        self.eleven_unlim_stability_spin.valueChanged.connect(self.save_current_language_settings)
        eleven_unlim_layout.addRow(self.eleven_unlim_stability_label, self.eleven_unlim_stability_spin)

        self.eleven_unlim_similarity_label = QLabel("Similarity Boost:")
        self.eleven_unlim_similarity_spin = QDoubleSpinBox()
        self.eleven_unlim_similarity_spin.setRange(0.0, 1.0)
        self.eleven_unlim_similarity_spin.setSingleStep(0.1)
        self.eleven_unlim_similarity_spin.setValue(0.75)
        self.eleven_unlim_similarity_spin.valueChanged.connect(self.save_current_language_settings)
        eleven_unlim_layout.addRow(self.eleven_unlim_similarity_label, self.eleven_unlim_similarity_spin)

        self.eleven_unlim_style_label = QLabel("Style:")
        self.eleven_unlim_style_spin = QDoubleSpinBox()
        self.eleven_unlim_style_spin.setRange(0.0, 1.0)
        self.eleven_unlim_style_spin.setSingleStep(0.1)
        self.eleven_unlim_style_spin.setValue(0.0)
        self.eleven_unlim_style_spin.valueChanged.connect(self.save_current_language_settings)
        eleven_unlim_layout.addRow(self.eleven_unlim_style_label, self.eleven_unlim_style_spin)

        self.eleven_unlim_boost_label = QLabel("Speaker Boost:")
        self.eleven_unlim_boost_check = QComboBox()
        self.eleven_unlim_boost_check.addItems(["True", "False"])
        self.eleven_unlim_boost_check.currentIndexChanged.connect(self.save_current_language_settings)
        eleven_unlim_layout.addRow(self.eleven_unlim_boost_label, self.eleven_unlim_boost_check)

        settings_layout.addRow(self.eleven_unlim_group)

        # VoiceMaker Settings
        self.voicemaker_voice_help = HelpLabel("voicemaker_voice_label")
        self.voicemaker_voice_label = QLabel("VoiceMaker Voice:")
        self.voicemaker_label_container = QWidget()
        voicemaker_label_layout = QHBoxLayout(self.voicemaker_label_container)
        voicemaker_label_layout.setContentsMargins(0, 0, 0, 0)
        voicemaker_label_layout.setSpacing(5)
        voicemaker_label_layout.addWidget(self.voicemaker_voice_help)
        voicemaker_label_layout.addWidget(self.voicemaker_voice_label)
        
        self.voicemaker_voice_combo = QComboBox()
        self.voicemaker_voice_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.voicemaker_label_container, self.voicemaker_voice_combo)

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

        # EdgeTTS Settings
        self.edgetts_voice_help = HelpLabel("edgetts_voice_label")
        self.edgetts_voice_label = QLabel(translator.translate("edgetts_voice_label", "EdgeTTS Voice:"))
        self.edgetts_label_container = QWidget()
        edgetts_label_layout = QHBoxLayout(self.edgetts_label_container)
        edgetts_label_layout.setContentsMargins(0, 0, 0, 0)
        edgetts_label_layout.setSpacing(5)
        edgetts_label_layout.addWidget(self.edgetts_voice_help)
        edgetts_label_layout.addWidget(self.edgetts_voice_label)
        
        self.edgetts_voice_combo = QComboBox()
        self.edgetts_voice_combo.currentIndexChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.edgetts_label_container, self.edgetts_voice_combo)

        self.edgetts_rate_label = QLabel(translator.translate("edgetts_rate_label", "EdgeTTS Rate (%):"))
        self.edgetts_rate_spinbox = QSpinBox()
        self.edgetts_rate_spinbox.setRange(-100, 100)
        self.edgetts_rate_spinbox.setValue(0)
        self.edgetts_rate_spinbox.setSuffix("%")
        self.edgetts_rate_spinbox.valueChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.edgetts_rate_label, self.edgetts_rate_spinbox)

        self.edgetts_pitch_label = QLabel(translator.translate("edgetts_pitch_label", "EdgeTTS Pitch (Hz):"))
        self.edgetts_pitch_spinbox = QSpinBox()
        self.edgetts_pitch_spinbox.setRange(-100, 100)
        self.edgetts_pitch_spinbox.setValue(0)
        self.edgetts_pitch_spinbox.setSuffix("Hz")
        self.edgetts_pitch_spinbox.valueChanged.connect(self.save_current_language_settings)
        settings_layout.addRow(self.edgetts_pitch_label, self.edgetts_pitch_spinbox)

        # Background Music Settings
        self.bg_music_help = HelpLabel("background_music_label")
        self.bg_music_label = QLabel()
        bg_music_label_container = QWidget()
        bg_music_label_layout = QHBoxLayout(bg_music_label_container)
        bg_music_label_layout.setContentsMargins(0, 0, 0, 0)
        bg_music_label_layout.setSpacing(5)
        bg_music_label_layout.addWidget(self.bg_music_help)
        bg_music_label_layout.addWidget(self.bg_music_label)
        
        bg_music_layout = QHBoxLayout()
        self.bg_music_path_input = QLineEdit()
        self.bg_music_path_input.setReadOnly(True)
        self.bg_music_path_input.textChanged.connect(self.save_current_language_settings)
        
        self.browse_bg_music_button = QPushButton()
        self.browse_bg_music_button.clicked.connect(self.browse_for_music_file)
        
        self.clear_bg_music_button = QPushButton()
        self.clear_bg_music_button.clicked.connect(self.clear_background_music)
        
        bg_music_layout.addWidget(self.bg_music_path_input)
        bg_music_layout.addWidget(self.browse_bg_music_button)
        bg_music_layout.addWidget(self.clear_bg_music_button)

        settings_layout.addRow(bg_music_label_container, bg_music_layout)

        self.bg_music_volume_help = HelpLabel("music_volume_label")
        self.bg_music_volume_label = QLabel()
        volume_label_container = QWidget()
        volume_label_layout = QHBoxLayout(volume_label_container)
        volume_label_layout.setContentsMargins(0, 0, 0, 0)
        volume_label_layout.setSpacing(5)
        volume_label_layout.addWidget(self.bg_music_volume_help)
        volume_label_layout.addWidget(self.bg_music_volume_label)
        
        volume_layout = QHBoxLayout()
        self.bg_music_volume_slider = QSlider(Qt.Orientation.Horizontal)
        self.bg_music_volume_slider.setRange(0, 100)
        self.bg_music_volume_slider.valueChanged.connect(self.on_volume_slider_changed)
        
        self.bg_music_volume_value_label = QLabel("100")
        self.bg_music_volume_slider.setValue(100)

        volume_layout.addWidget(self.bg_music_volume_slider)
        volume_layout.addWidget(self.bg_music_volume_value_label)
        
        settings_layout.addRow(volume_label_container, volume_layout)

        # --- Initial Video Settings ---
        self.initial_video_help = HelpLabel("initial_video_label")
        self.initial_video_label = QLabel()
        initial_video_label_container = QWidget()
        initial_video_label_layout = QHBoxLayout(initial_video_label_container)
        initial_video_label_layout.setContentsMargins(0, 0, 0, 0)
        initial_video_label_layout.setSpacing(5)
        initial_video_label_layout.addWidget(self.initial_video_help)
        initial_video_label_layout.addWidget(self.initial_video_label)
        
        initial_video_layout = QHBoxLayout()
        self.initial_video_path_input = QLineEdit()
        self.initial_video_path_input.setReadOnly(True)
        self.initial_video_browse_button = QPushButton()
        self.initial_video_browse_button.clicked.connect(self.browse_for_initial_video)
        self.initial_video_clear_button = QPushButton()
        self.initial_video_clear_button.clicked.connect(self.clear_initial_video)
        
        initial_video_layout.addWidget(self.initial_video_path_input)
        initial_video_layout.addWidget(self.initial_video_browse_button)
        initial_video_layout.addWidget(self.initial_video_clear_button)
        
        settings_layout.addRow(initial_video_label_container, initial_video_layout)
        
        # --- Effects & Watermark Settings (New) ---
        effects_group = QGroupBox(translator.translate("overlay_effects_group", "Overlay Effects"))
        effects_layout = QFormLayout(effects_group)
        
        # Effect Selection
        self.overlay_effect_help = HelpLabel("overlay_effect_label")
        self.overlay_effect_label = QLabel(translator.translate("effect_selection_title", "Overlay Effect:"))
        overlay_label_container = QWidget()
        overlay_label_layout = QHBoxLayout(overlay_label_container)
        overlay_label_layout.setContentsMargins(0, 0, 0, 0)
        overlay_label_layout.setSpacing(5)
        overlay_label_layout.addWidget(self.overlay_effect_help)
        overlay_label_layout.addWidget(self.overlay_effect_label)
        
        self.overlay_effect_path_input = QLineEdit()
        self.overlay_effect_path_input.setReadOnly(True)
        # self.overlay_effect_path_input.textChanged.connect(self.save_current_language_settings) # ReadOnly, so no direct text change
        
        effect_buttons_layout = QHBoxLayout()
        self.select_effect_button = QPushButton(translator.translate("select_effect_button", "Select Effect"))
        self.select_effect_button.clicked.connect(self.open_effect_dialog)
        self.clear_effect_button = QPushButton(translator.translate("clear_button", "Clear"))
        self.clear_effect_button.clicked.connect(self.clear_overlay_effect)
        
        effect_buttons_layout.addWidget(self.overlay_effect_path_input)
        effect_buttons_layout.addWidget(self.select_effect_button)
        effect_buttons_layout.addWidget(self.clear_effect_button)
        
        effects_layout.addRow(overlay_label_container, effect_buttons_layout)

        # Watermark Selection
        self.watermark_help = HelpLabel("watermark_label")
        self.watermark_label = QLabel(translator.translate("watermark_group", "Watermark:"))
        watermark_label_container = QWidget()
        watermark_label_layout = QHBoxLayout(watermark_label_container)
        watermark_label_layout.setContentsMargins(0, 0, 0, 0)
        watermark_label_layout.setSpacing(5)
        watermark_label_layout.addWidget(self.watermark_help)
        watermark_label_layout.addWidget(self.watermark_label)
        
        self.watermark_path_input = QLineEdit()
        self.watermark_path_input.setReadOnly(True)
        
        watermark_buttons_layout = QHBoxLayout()
        self.select_watermark_button = QPushButton(translator.translate("select_watermark_button", "Select Watermark"))
        self.select_watermark_button.clicked.connect(self.select_watermark)
        self.clear_watermark_button = QPushButton(translator.translate("clear_watermark_button", "Clear"))
        self.clear_watermark_button.clicked.connect(self.clear_watermark)
        
        watermark_buttons_layout.addWidget(self.watermark_path_input)
        watermark_buttons_layout.addWidget(self.select_watermark_button)
        watermark_buttons_layout.addWidget(self.clear_watermark_button)
        
        effects_layout.addRow(watermark_label_container, watermark_buttons_layout)
        
        # Watermark Size
        self.watermark_size_label = QLabel()
        watermark_size_layout = QHBoxLayout()
        self.watermark_size_slider = QSlider(Qt.Orientation.Horizontal)
        self.watermark_size_slider.setRange(5, 50)  # 5% до 50% від ширини
        self.watermark_size_slider.setValue(20)  # Дефолт 20%
        self.watermark_size_slider.valueChanged.connect(self.on_watermark_size_changed)
        self.watermark_size_value_label = QLabel("20%")
        watermark_size_layout.addWidget(self.watermark_size_slider)
        watermark_size_layout.addWidget(self.watermark_size_value_label)
        effects_layout.addRow(self.watermark_size_label, watermark_size_layout)
        
        # Watermark Position
        self.watermark_position_label = QLabel()
        self.watermark_position_combo = QComboBox()
        # Додати 9 позицій
        self.watermark_position_combo.addItem(translator.translate("position_top_left", "Top Left"), 0)
        self.watermark_position_combo.addItem(translator.translate("position_top_center", "Top Center"), 1)
        self.watermark_position_combo.addItem(translator.translate("position_top_right", "Top Right"), 2)
        self.watermark_position_combo.addItem(translator.translate("position_center_left", "Center Left"), 3)
        self.watermark_position_combo.addItem(translator.translate("position_center", "Center"), 4)
        self.watermark_position_combo.addItem(translator.translate("position_center_right", "Center Right"), 5)
        self.watermark_position_combo.addItem(translator.translate("position_bottom_left", "Bottom Left"), 6)
        self.watermark_position_combo.addItem(translator.translate("position_bottom_center", "Bottom Center"), 7)
        self.watermark_position_combo.addItem(translator.translate("position_bottom_right", "Bottom Right"), 8)
        self.watermark_position_combo.setCurrentIndex(8)  # Дефолт: Bottom Right
        self.watermark_position_combo.currentIndexChanged.connect(self.save_current_language_settings)
        effects_layout.addRow(self.watermark_position_label, self.watermark_position_combo)
        
        settings_layout.addRow(effects_group)

        # --- Dynamic Overlays (Triggers) ---
        self.triggers_group = QGroupBox()
        self.triggers_help = HelpLabel("overlay_triggers_group")
        self.triggers_group_title = QLabel(translator.translate("overlay_triggers_group", "Dynamic Overlays (Triggers)"))
        self.triggers_group_title.setStyleSheet("font-weight: bold;")
        
        triggers_title_container = QWidget()
        triggers_title_layout = QHBoxLayout(triggers_title_container)
        triggers_title_layout.setContentsMargins(0, 0, 0, 0)
        triggers_title_layout.setSpacing(5)
        triggers_title_layout.addWidget(self.triggers_help)
        triggers_title_layout.addWidget(self.triggers_group_title)
        
        triggers_main_layout = QVBoxLayout(self.triggers_group)
        triggers_main_layout.addWidget(triggers_title_container)

        self.triggers_table = QTableWidget()
        self.triggers_table.setColumnCount(4)
        self.triggers_table.setHorizontalHeaderLabels([
            translator.translate("trigger_column", "Trigger"),
            translator.translate("type_column", "Type"),
            translator.translate("file_column", "Effect File"),
            translator.translate("actions_column", "Actions")
        ])
        self.triggers_table.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.Interactive)
        self.triggers_table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Interactive) # Trigger (Interactive)
        self.triggers_table.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents) # Type (Auto-fit)
        self.triggers_table.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)     # File (Flexible)
        self.triggers_table.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents) # Actions (Auto-fit)
        
        self.triggers_table.setColumnWidth(0, 200)
        self.triggers_table.setMinimumHeight(200)
        self.triggers_table.verticalHeader().setDefaultSectionSize(45)
        
        triggers_main_layout.addWidget(self.triggers_table)

        self.add_trigger_button = QPushButton(translator.translate("add_trigger_button", "Add Trigger"))
        self.add_trigger_button.clicked.connect(self.add_empty_trigger_row)
        triggers_main_layout.addWidget(self.add_trigger_button)

        settings_layout.addRow(self.triggers_group)


        scroll_area.setWidget(settings_container)
        right_layout.addWidget(scroll_area)

        # Set initial state
        self.right_panel.setVisible(False)
        splitter.setSizes([200, 600])

    def open_prompt_editor(self):
        dialog = PromptEditorDialog(self, self.prompt_edit.toPlainText())
        if dialog.exec():
            self.prompt_edit.setPlainText(dialog.get_text())

    def open_rewrite_prompt_editor(self):
        current_text = self.rewrite_prompt_edit.toPlainText()
        dialog = PromptEditorDialog(self, current_text)
        if dialog.exec():
            self.rewrite_prompt_edit.setPlainText(dialog.get_text())

    def set_rewrite_visible(self, visible: bool):
        if hasattr(self, 'rewrite_group'):
            self.rewrite_group.setVisible(visible)
            
    def browse_for_music_file(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            translator.translate("select_background_music_file", "Select Background Music File"),
            "",
            translator.translate("audio_files_filter", "Audio Files (*.mp3 *.wav)")
        )
        if file_path:
            self.bg_music_path_input.setText(file_path)
            self.save_current_language_settings()
            
    def clear_background_music(self):
        self.bg_music_path_input.clear()

    def browse_for_initial_video(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            translator.translate("select_initial_video_file", "Select Initial Video File"),
            "",
            translator.translate("video_files_filter", "Video Files (*.mp4 *.mkv *.mov *.avi *.webm)")
        )
        if file_path:
            self.initial_video_path_input.setText(file_path)
            self.save_current_language_settings()

    def clear_initial_video(self):
        self.initial_video_path_input.clear()
        self.save_current_language_settings()

    def on_volume_slider_changed(self, value):
        self.bg_music_volume_value_label.setText(str(value))
        self.save_current_language_settings()

    def on_watermark_size_changed(self, value):
        self.watermark_size_value_label.setText(f"{value}%")
        self.save_current_language_settings()
    

    def open_effect_dialog(self):
        from gui.widgets.effect_selection_dialog import EffectSelectionDialog
        current_path = self.overlay_effect_path_input.text()
        dialog = EffectSelectionDialog(self, initial_selection=current_path)
        if dialog.exec():
            selected = dialog.get_selected_effect()
            if selected:
                self.overlay_effect_path_input.setText(selected)
                self.save_current_language_settings()

    def clear_overlay_effect(self):
        self.overlay_effect_path_input.clear()
        self.save_current_language_settings()

    def select_watermark(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            translator.translate("select_watermark_button", "Select Watermark"),
            "",
            translator.translate("watermark_filter", "Image Files (*.png)")
        )
        if file_path:
            self.watermark_path_input.setText(file_path)
            self.save_current_language_settings()

    def clear_watermark(self):
        self.watermark_path_input.clear()
        self.save_current_language_settings()


    def load_languages(self):
        self.lang_list_widget.clear()
        languages = self.settings.get("languages_config", {})
        for lang_id, config in languages.items():
            display_name = config.get("display_name", lang_id)
            self.lang_list_widget.addItem(f"{display_name} [{lang_id}]")
        self.load_models()

    def load_models(self):
        self.model_combo.blockSignals(True)
        self.rewrite_model_combo.blockSignals(True)
        
        current_model = self.model_combo.currentText()
        current_rewrite = self.rewrite_model_combo.currentText()
        
        self.model_combo.clear()
        self.rewrite_model_combo.clear()
        
        models = self.settings.get("openrouter_models", [])
        self.model_combo.addItems(models)
        self.rewrite_model_combo.addItems(models)
        
        idx = self.model_combo.findText(current_model)
        if idx >= 0: self.model_combo.setCurrentIndex(idx)
        
        ridx = self.rewrite_model_combo.findText(current_rewrite)
        if ridx >= 0: self.rewrite_model_combo.setCurrentIndex(ridx)
        
        self.model_combo.blockSignals(False)
        self.rewrite_model_combo.blockSignals(False)

    def load_templates_combo(self):
        self.default_template_combo.blockSignals(True)
        self.default_template_combo.clear()
        self.default_template_combo.addItem(translator.translate("none_template", "None"), None)
        templates = template_manager.get_templates()
        for template_name in templates:
            self.default_template_combo.addItem(template_name, template_name)
        self.default_template_combo.blockSignals(False)

    def load_elevenlabs_templates(self, force=False):
        # Avoid reloading from API if we already have templates, unless forced
        if self.elevenlabs_templates and not force:
             # Just refresh the combo from cached templates
             self.elevenlabs_template_combo.blockSignals(True)
             self.elevenlabs_template_combo.clear()
             for template in self.elevenlabs_templates:
                 self.elevenlabs_template_combo.addItem(template["name"], template["uuid"])
             self.elevenlabs_template_combo.blockSignals(False)
             return

        self.elevenlabs_template_combo.blockSignals(True)
        self.elevenlabs_template_combo.clear()
        
        # This makes the API request
        templates, status = self.elevenlabs_api.get_templates()
        
        if status == "connected" and templates:
            self.elevenlabs_templates = templates
            for template in self.elevenlabs_templates:
                self.elevenlabs_template_combo.addItem(template["name"], template["uuid"])
        else:
            self.elevenlabs_template_combo.addItem(translator.translate("no_templates_found"), "")
        self.elevenlabs_template_combo.blockSignals(False)
        
        # Restore selection for current language if applicable
        if self.lang_list_widget.currentItem():
            self.on_language_selected(self.lang_list_widget.currentItem(), None)

    def populate_voicemaker_voices(self, lang_id):
        self.voicemaker_voice_combo.blockSignals(True)
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
        self.voicemaker_voice_combo.blockSignals(False)

    def populate_edgetts_voices(self, lang_id):
        self.edgetts_voice_combo.blockSignals(True)
        self.edgetts_voice_combo.clear()
        
        # Filter voices by lang_id (e.g. "en-US", "uk-UA")
        # EdgeTTS voice "ShortName" usually looks like "uk-UA-OstapNeural"
        
        normalized_lang_id = lang_id.lower().replace("_", "-")
        # Try to handle 'uk' -> 'uk-UA'
        if normalized_lang_id == 'uk': normalized_lang_id = 'uk-ua'
        if normalized_lang_id == 'en': normalized_lang_id = 'en-us'
        if normalized_lang_id == 'ru': normalized_lang_id = 'ru-ru'
        
        matched_voices = []
        other_voices = []
        
        for voice in self.edge_tts_voices:
            short_name = voice.get("ShortName", "")
            friendly_name = voice.get("FriendlyName", short_name)
            locale = voice.get("Locale", "").lower()
            
            item_text = f"{friendly_name}"
            
            if normalized_lang_id in locale:
                matched_voices.append((item_text, short_name))
            else:
                other_voices.append((item_text, short_name))
        
        if matched_voices:
             for text, data in matched_voices:
                 self.edgetts_voice_combo.addItem(text, data)
             self.edgetts_voice_combo.insertSeparator(self.edgetts_voice_combo.count())
             
        for text, data in other_voices:
            self.edgetts_voice_combo.addItem(text, data)
            
        self.edgetts_voice_combo.blockSignals(False)

    def on_tts_provider_changed(self, index):
        provider = self.tts_provider_combo.currentText()
        
        # ElevenLabs
        is_eleven = (provider == "ElevenLabs")
        self.elevenlabs_label_container.setVisible(is_eleven)
        self.elevenlabs_template_combo.setVisible(is_eleven)
        
        # VoiceMaker
        is_vm = (provider == "VoiceMaker")
        self.voicemaker_label_container.setVisible(is_vm)
        self.voicemaker_voice_combo.setVisible(is_vm)
        
        # GeminiTTS
        is_gemini = (provider == "GeminiTTS")
        self.gemini_voice_label.setVisible(is_gemini)
        self.gemini_voice_combo.setVisible(is_gemini)
        self.gemini_tone_label.setVisible(is_gemini)
        self.gemini_tone_input.setVisible(is_gemini)

        # EdgeTTS
        is_edge = (provider == "EdgeTTS")
        self.edgetts_label_container.setVisible(is_edge)
        self.edgetts_voice_combo.setVisible(is_edge)
        self.edgetts_rate_label.setVisible(is_edge)
        self.edgetts_rate_spinbox.setVisible(is_edge)
        self.edgetts_pitch_label.setVisible(is_edge)
        self.edgetts_pitch_spinbox.setVisible(is_edge)

        # ElevenLabs Unlim
        is_eleven_unlim = (provider == "ElevenLabsUnlim")
        self.eleven_unlim_group.setVisible(is_eleven_unlim)

    def on_language_selected(self, current, previous):
        self._is_loading_lang_settings = True
        try:
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
            self.rewrite_prompt_edit.blockSignals(True)
            self.rewrite_model_combo.blockSignals(True)
            self.rewrite_tokens_spinbox.blockSignals(True)
            self.rewrite_temperature_spinbox.blockSignals(True)
            self.elevenlabs_template_combo.blockSignals(True)
            self.tts_provider_combo.blockSignals(True)
            self.voicemaker_voice_combo.blockSignals(True)
            self.gemini_voice_combo.blockSignals(True)
            self.gemini_tone_input.blockSignals(True)
            self.edgetts_voice_combo.blockSignals(True)
            self.edgetts_rate_spinbox.blockSignals(True)
            self.edgetts_pitch_spinbox.blockSignals(True)
            self.bg_music_path_input.blockSignals(True)
            self.bg_music_volume_slider.blockSignals(True)
            self.gemini_voice_combo.blockSignals(True)
            self.gemini_tone_input.blockSignals(True)
            self.bg_music_path_input.blockSignals(True)
            self.bg_music_path_input.blockSignals(True)
            self.bg_music_volume_slider.blockSignals(True)
            self.initial_video_path_input.blockSignals(True)
            self.watermark_size_slider.blockSignals(True)
            self.watermark_position_combo.blockSignals(True)
            self.watermark_size_slider.blockSignals(True)
            self.watermark_position_combo.blockSignals(True)
            self.eleven_unlim_voice_id_input.blockSignals(True)
            self.eleven_unlim_stability_spin.blockSignals(True)
            self.eleven_unlim_similarity_spin.blockSignals(True)
            self.eleven_unlim_style_spin.blockSignals(True)
            self.eleven_unlim_boost_check.blockSignals(True)
            self.default_template_combo.blockSignals(True)
            # No block signals needed for inputs that are read-only and updated by buttons,
            # but usually good practice if we were using textChanged on them.


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
            template_index = self.elevenlabs_template_combo.findData(current_template_uuid)
            self.elevenlabs_template_combo.setCurrentIndex(template_index if template_index >= 0 else 0)

            # ElevenLabs Unlim Settings
            unlim_settings = config.get("eleven_unlim_settings", {})
            self.eleven_unlim_voice_id_input.setText(unlim_settings.get("voice_id", ""))
            self.eleven_unlim_stability_spin.setValue(unlim_settings.get("stability", 0.5))
            self.eleven_unlim_similarity_spin.setValue(unlim_settings.get("similarity_boost", 0.75))
            self.eleven_unlim_style_spin.setValue(unlim_settings.get("style", 0.0))
            use_boost = unlim_settings.get("use_speaker_boost", True)
            self.eleven_unlim_boost_check.setCurrentIndex(0 if use_boost else 1)

            # VoiceMaker Voice
            self.populate_voicemaker_voices(self.current_lang_id)
            current_voicemaker_voice = config.get("voicemaker_voice_id", "")
            voice_index = self.voicemaker_voice_combo.findData(current_voicemaker_voice)
            self.voicemaker_voice_combo.setCurrentIndex(voice_index if voice_index >= 0 else 0)

            # GeminiTTS Settings
            current_gemini_voice = config.get("gemini_voice", "Puck")
            gemini_index = self.gemini_voice_combo.findData(current_gemini_voice)
            self.gemini_voice_combo.setCurrentIndex(gemini_index if gemini_index >= 0 else 0)
            self.gemini_voice_combo.setCurrentIndex(gemini_index if gemini_index >= 0 else 0)
            self.gemini_tone_input.setText(config.get("gemini_tone", ""))

            # EdgeTTS Settings
            self.populate_edgetts_voices(self.current_lang_id)
            current_edgetts_voice = config.get("edgetts_voice", "uk-UA-OstapNeural") # Default for uk-UA test
            edge_index = self.edgetts_voice_combo.findData(current_edgetts_voice)
            self.edgetts_voice_combo.setCurrentIndex(edge_index if edge_index >= 0 else 0)
            self.edgetts_rate_spinbox.setValue(config.get("edgetts_rate", 0))
            self.edgetts_pitch_spinbox.setValue(config.get("edgetts_pitch", 0))

            self.bg_music_path_input.setText(config.get("background_music_path", ""))
            volume = config.get("background_music_volume", 100)
            self.bg_music_volume_slider.setValue(volume)
            self.bg_music_volume_value_label.setText(str(volume))

            self.initial_video_path_input.setText(config.get("initial_video_path", ""))

            self.tokens_spinbox.setValue(config.get("max_tokens", 4096))
            self.temperature_spinbox.setValue(config.get("temperature", 0.7))

            # Rewrite Settings
            self.rewrite_prompt_edit.setPlainText(config.get("rewrite_prompt", ""))
            
            current_rewrite_model = config.get("rewrite_model", "")
            idx = self.rewrite_model_combo.findText(current_rewrite_model)
            self.rewrite_model_combo.setCurrentIndex(idx if idx >= 0 else 0)

            self.rewrite_tokens_spinbox.setValue(config.get("rewrite_max_tokens", 4096))
            self.rewrite_temperature_spinbox.setValue(config.get("rewrite_temperature", 0.7))

            # Default Template
            current_default_template = config.get("default_template")
            if current_default_template is None:
                self.default_template_combo.setCurrentIndex(0)
            else:
                index = self.default_template_combo.findData(current_default_template)
                self.default_template_combo.setCurrentIndex(index if index >= 0 else 0)

            # Effects & Watermark
            self.overlay_effect_path_input.setText(config.get("overlay_effect_path", ""))
            self.watermark_path_input.setText(config.get("watermark_path", ""))
            
            # Watermark Size
            watermark_size = config.get("watermark_size", 20)
            self.watermark_size_slider.setValue(watermark_size)
            self.watermark_size_value_label.setText(f"{watermark_size}%")
            
            # Watermark Position
            watermark_position = config.get("watermark_position", 8)
            # Знайти індекс за data
            pos_index = self.watermark_position_combo.findData(watermark_position)
            if pos_index >= 0:
                self.watermark_position_combo.setCurrentIndex(pos_index)
            else:
                self.watermark_position_combo.setCurrentIndex(8)  # Дефолт
            
            # Load Overlays Triggers
            self.load_overlay_triggers(config.get("overlay_triggers", []))


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
            self.edgetts_voice_combo.blockSignals(False)
            self.edgetts_rate_spinbox.blockSignals(False)
            self.edgetts_pitch_spinbox.blockSignals(False)
            self.bg_music_path_input.blockSignals(False)
            self.bg_music_volume_slider.blockSignals(False)
            self.initial_video_path_input.blockSignals(False)
            self.default_template_combo.blockSignals(False)
            self.watermark_size_slider.blockSignals(False)
            self.watermark_position_combo.blockSignals(False)
            self.rewrite_prompt_edit.blockSignals(False)
            self.rewrite_model_combo.blockSignals(False)
            self.rewrite_tokens_spinbox.blockSignals(False)
            self.rewrite_temperature_spinbox.blockSignals(False)
            self.eleven_unlim_voice_id_input.blockSignals(False)
            self.eleven_unlim_stability_spin.blockSignals(False)
            self.eleven_unlim_similarity_spin.blockSignals(False)
            self.eleven_unlim_style_spin.blockSignals(False)
            self.eleven_unlim_boost_check.blockSignals(False)

            
            self.right_panel.setVisible(True)
        finally:
            self._is_loading_lang_settings = False

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
            "model": "google/gemini-2.5-flash",
            "max_tokens": 128000,
            "temperature": 1.0,
            "rewrite_prompt": "",
            "rewrite_model": "google/gemini-2.5-flash",
            "rewrite_max_tokens": 128000,
            "rewrite_temperature": 1.0,
            "tts_provider": "EdgeTTS",
            "elevenlabs_template_uuid": "",
            "voicemaker_voice_id": "",
            "gemini_voice": "Puck",
            "gemini_tone": "",
            "edgetts_voice": "",
            "edgetts_rate": 0,
            "edgetts_pitch": 0,
            "background_music_path": "",
            "background_music_volume": 25,
            "initial_video_path": "",
            "default_template": "",
            "overlay_effect_path": "",
            "watermark_path": "",
            "watermark_size": 5,
            "watermark_position": 8
        }

        self.settings.set("languages_config", languages)
        
        self.lang_name_input.clear()
        self.lang_id_input.clear()
        self.load_languages()
        
        if self.main_window:
            self.main_window.refresh_language_menus()

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
            
            if self.main_window:
                self.main_window.refresh_language_menus()

    def save_current_language_settings(self):
        if self._is_loading_lang_settings:
            return
            
        if not self.current_lang_id:
            return

        languages_config = self.settings.get("languages_config", {})
        lang_settings = languages_config.get(self.current_lang_id)

        if not lang_settings:
            return

        # Update the dictionary in-place
        lang_settings["prompt"] = self.prompt_edit.toPlainText()
        lang_settings["model"] = self.model_combo.currentText()
        lang_settings["max_tokens"] = self.tokens_spinbox.value()
        lang_settings["temperature"] = self.temperature_spinbox.value()
        
        lang_settings["rewrite_prompt"] = self.rewrite_prompt_edit.toPlainText()
        lang_settings["rewrite_model"] = self.rewrite_model_combo.currentText()
        lang_settings["rewrite_max_tokens"] = self.rewrite_tokens_spinbox.value()
        lang_settings["rewrite_temperature"] = self.rewrite_temperature_spinbox.value()

        lang_settings["tts_provider"] = self.tts_provider_combo.currentText()
        lang_settings["elevenlabs_template_uuid"] = self.elevenlabs_template_combo.currentData()
        lang_settings["voicemaker_voice_id"] = self.voicemaker_voice_combo.currentData()
        lang_settings["gemini_voice"] = self.gemini_voice_combo.currentData()
        lang_settings["gemini_tone"] = self.gemini_tone_input.text()
        lang_settings["edgetts_voice"] = self.edgetts_voice_combo.currentData()
        lang_settings["edgetts_rate"] = self.edgetts_rate_spinbox.value()
        lang_settings["edgetts_pitch"] = self.edgetts_pitch_spinbox.value()
        lang_settings["background_music_path"] = self.bg_music_path_input.text()
        lang_settings["background_music_volume"] = self.bg_music_volume_slider.value()
        lang_settings["initial_video_path"] = self.initial_video_path_input.text()
        lang_settings["default_template"] = self.default_template_combo.currentData()
        lang_settings["overlay_effect_path"] = self.overlay_effect_path_input.text()
        lang_settings["watermark_path"] = self.watermark_path_input.text()
        lang_settings["watermark_size"] = self.watermark_size_slider.value()
        lang_settings["watermark_position"] = self.watermark_position_combo.currentData()
        lang_settings["overlay_triggers"] = self.get_overlay_triggers_from_table()


        # Explicitly save the entire settings file
        lang_settings["eleven_unlim_settings"] = {
            "voice_id": self.eleven_unlim_voice_id_input.text(),
            "stability": self.eleven_unlim_stability_spin.value(),
            "similarity_boost": self.eleven_unlim_similarity_spin.value(),
            "style": self.eleven_unlim_style_spin.value(),
            "use_speaker_boost": (self.eleven_unlim_boost_check.currentText() == "True")
        }

        self.settings.save_settings()

    def update_fields(self):
        # Store current selection
        current_item = self.lang_list_widget.currentItem()
        current_lang_text = current_item.text() if current_item else None

        # Reload languages and models from settings
        self.load_languages()
        self.load_models()
        self.load_elevenlabs_templates()
        self.load_templates_combo()
        
        # Triggers column headers
        self.triggers_table.setHorizontalHeaderLabels([
            translator.translate("trigger_column", "Trigger"),
            translator.translate("type_column", "Type"),
            translator.translate("file_column", "Effect File"),
            translator.translate("actions_column", "Actions")
        ])

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
        self.lang_name_help.update_tooltip()
        self.lang_id_help.update_tooltip()
        self.add_lang_button.setText(translator.translate("add_model"))
        self.remove_lang_button.setText(translator.translate("remove_model"))
        self.prompt_label.setText(translator.translate("language_prompt_label"))
        self.prompt_help.update_tooltip()
        self.model_label.setText(translator.translate("translation_model_label"))
        self.elevenlabs_template_label.setText(translator.translate("elevenlabs_template_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))
        
        # Update hints
        self.model_help.update_tooltip()
        self.tokens_help.update_tooltip()
        self.temperature_help.update_tooltip()
        self.tts_provider_label.setText(translator.translate("tts_provider_label"))
        self.voicemaker_voice_label.setText(translator.translate("voicemaker_voice_label"))
        self.gemini_voice_label.setText(translator.translate("gemini_voice_label"))
        self.default_template_label.setText(translator.translate("default_template_label", "Default Template:"))
        self.temperature_label.setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")
        self.rewrite_prompt_label.setText(translator.translate("rewrite_prompt_label"))
        self.rewrite_model_label.setText(translator.translate("rewrite_model_label"))
        self.rewrite_tokens_label.setText(translator.translate("tokens_label"))
        self.rewrite_temperature_label.setText(translator.translate("temperature_label"))
        
        # Update hints
        self.rewrite_prompt_help.update_tooltip()
        self.rewrite_model_help.update_tooltip()
        self.rewrite_tokens_help.update_tooltip()
        self.rewrite_temperature_help.update_tooltip()
        
        self.bg_music_label.setText(translator.translate("background_music_label", "Background Music:"))
        self.browse_bg_music_button.setText(translator.translate("browse_button", "Browse..."))
        self.clear_bg_music_button.setText(translator.translate("clear_button", "Clear"))
        self.bg_music_volume_label.setText(translator.translate("music_volume_label", "Music Volume:"))
        
        self.initial_video_label.setText(translator.translate("initial_video_label", "Initial Video:"))
        self.initial_video_browse_button.setText(translator.translate("browse_button", "Browse..."))
        self.initial_video_clear_button.setText(translator.translate("clear_button", "Clear"))
        self.initial_video_help.update_tooltip()
        
        # Update hints
        self.default_template_help.update_tooltip()
        self.tts_provider_help.update_tooltip()
        self.bg_music_help.update_tooltip()
        self.bg_music_volume_help.update_tooltip()
        self.elevenlabs_template_help.update_tooltip()
        self.eleven_unlim_voice_id_help.update_tooltip()
        self.voicemaker_voice_help.update_tooltip()
        self.edgetts_voice_help.update_tooltip()
        self.overlay_effect_help.update_tooltip()
        self.watermark_help.update_tooltip()
        
        self.overlay_effect_label.setText(translator.translate("effect_selection_title", "Overlay Effect:"))
        self.select_effect_button.setText(translator.translate("select_effect_button", "Select Effect"))
        self.clear_effect_button.setText(translator.translate("clear_button", "Clear"))
        self.watermark_label.setText(translator.translate("watermark_group", "Watermark:"))
        self.select_watermark_button.setText(translator.translate("select_watermark_button", "Select Watermark"))
        self.clear_watermark_button.setText(translator.translate("clear_watermark_button", "Clear"))
        
        self.watermark_size_label.setText(translator.translate("watermark_size_label", "Watermark Size:"))
        self.watermark_position_label.setText(translator.translate("watermark_position_label", "Watermark Position:"))

        self.tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.rewrite_tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))

        self.triggers_group_title.setText(translator.translate("overlay_triggers_group", "Dynamic Overlays (Triggers)"))
        self.add_trigger_button.setText(translator.translate("add_trigger_button", "Add Trigger"))
        
        # Force table to redraw header translations
        self.triggers_table.setHorizontalHeaderLabels([
            translator.translate("trigger_column", "Trigger"),
            translator.translate("type_column", "Type"),
            translator.translate("file_column", "Effect File"),
            translator.translate("actions_column", "Actions")
        ])

    def add_empty_trigger_row(self):
        self.add_trigger_to_table({
            "type": "text",
            "value": "",
            "path": ""
        })
        self.save_current_language_settings()

    def add_trigger_to_table(self, trigger_data):
        row = self.triggers_table.rowCount()
        self.triggers_table.insertRow(row)

        # Trigger Value (Phrase or Time)
        val_input = QLineEdit(trigger_data.get("value", ""))
        val_input.textChanged.connect(self.save_current_language_settings)
        self.triggers_table.setCellWidget(row, 0, val_input)

        # Type ComboBox
        type_combo = QComboBox()
        type_combo.addItem(translator.translate("trigger_type_text", "Text Phrase"), "text")
        type_combo.addItem(translator.translate("trigger_type_time", "Time (SS or MM:SS)"), "time")
        
        current_type = trigger_data.get("type", "text")
        idx = type_combo.findData(current_type)
        if idx >= 0: type_combo.setCurrentIndex(idx)
        
        type_combo.currentIndexChanged.connect(self.save_current_language_settings)
        self.triggers_table.setCellWidget(row, 1, type_combo)

        # Path Selection
        path_container = QWidget()
        path_layout = QHBoxLayout(path_container)
        path_layout.setContentsMargins(2, 2, 2, 2)
        
        full_path = trigger_data.get("path", "")
        path_input = QLineEdit()
        path_input.setProperty("full_path", full_path)
        if full_path:
            path_input.setText(os.path.basename(full_path))
        
        path_input.setReadOnly(True)
        path_input.setPlaceholderText(translator.translate("no_file_selected", "No file selected"))
        path_input.setToolTip(full_path)
        path_input.setFixedHeight(30)
        path_input.setStyleSheet("color: #ffffff; background: #222222; border: 1px solid #555555; padding: 5px; font-size: 12px;")
        
        browse_btn = QPushButton(translator.translate("trigger_browse", "Browse"))
        browse_btn.setFixedHeight(30)
        browse_btn.setStyleSheet("padding-left: 10px; padding-right: 10px;")
        browse_btn.clicked.connect(lambda: self.browse_trigger_file(path_input))
        
        path_layout.addWidget(path_input)
        path_layout.addWidget(browse_btn)
        self.triggers_table.setCellWidget(row, 2, path_container)

        # Actions Layout (Delete + Settings)
        actions_container = QWidget()
        actions_layout = QHBoxLayout(actions_container)
        actions_layout.setContentsMargins(5, 0, 5, 0)
        actions_layout.setSpacing(10)
        actions_layout.setAlignment(Qt.AlignmentFlag.AlignRight)

        # Settings Button
        settings_btn = QPushButton(translator.translate("trigger_settings", "Position"))
        settings_btn.setFixedHeight(30)
        settings_btn.setStyleSheet("padding-left: 12px; padding-right: 12px;")
        settings_btn.setToolTip(translator.translate("trigger_settings_tooltip", "Adjust Position"))
        settings_btn.setProperty("x_offset", trigger_data.get("x", 0))
        settings_btn.setProperty("y_offset", trigger_data.get("y", 0))
        settings_btn.clicked.connect(self.open_trigger_settings)
        
        # Remove Button
        remove_btn = QPushButton(translator.translate("trigger_delete", "Delete"))
        remove_btn.setFixedHeight(30)
        remove_btn.setStyleSheet("color: #ff4444; font-weight: bold; border: 1px solid #ff4444; background: transparent; padding-left: 12px; padding-right: 12px;")
        remove_btn.clicked.connect(lambda: self.remove_trigger_row(row))
        
        actions_layout.addWidget(settings_btn)
        actions_layout.addWidget(remove_btn)
        actions_layout.addStretch() # Stability
        self.triggers_table.setCellWidget(row, 3, actions_container)
        
        # Set trigger row height
        self.triggers_table.setRowHeight(row, 45)

    def open_trigger_settings(self):
        button = self.sender()
        if not button: return
        
        x = button.property("x_offset")
        y = button.property("y_offset")
        
        dialog = OverlaySettingsDialog(self, x, y)
        if dialog.exec():
            new_x, new_y = dialog.get_values()
            button.setProperty("x_offset", new_x)
            button.setProperty("y_offset", new_y)
            self.save_current_language_settings()

    def browse_trigger_file(self, line_edit):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            translator.translate("effect_selection_title", "Select Overlay Effect"),
            "",
            "Media Files (*.mp4 *.mov *.webm *.png *.jpg *.jpeg)"
        )
        if file_path:
            line_edit.setProperty("full_path", file_path)
            line_edit.setText(os.path.basename(file_path))
            line_edit.setToolTip(file_path)
            self.save_current_language_settings()

    def remove_trigger_row(self, row_idx):
        # Since indexes change after removal, we need a safer way if multiple are deleted,
        # but for single clicks it's fine.
        # Actually sending row index to lambda is risky. Better to find row by sender.
        button = self.sender()
        if button:
            # Find row of the button
            for r in range(self.triggers_table.rowCount()):
                if self.triggers_table.cellWidget(r, 3) == button:
                    self.triggers_table.removeRow(r)
                    self.save_current_language_settings()
                    break

    def load_overlay_triggers(self, triggers):
        self.triggers_table.setRowCount(0)
        for trigger in triggers:
            self.add_trigger_to_table(trigger)

    def get_overlay_triggers_from_table(self):
        triggers = []
        for row in range(self.triggers_table.rowCount()):
            val_widget = self.triggers_table.cellWidget(row, 0)
            type_widget = self.triggers_table.cellWidget(row, 1)
            path_widget_container = self.triggers_table.cellWidget(row, 2)
            
                # Actions container is for offsets
            actions_widget = self.triggers_table.cellWidget(row, 3)
            settings_btn = actions_widget.findChild(QPushButton) if actions_widget else None
            x_off = settings_btn.property("x_offset") if settings_btn else 0
            y_off = settings_btn.property("y_offset") if settings_btn else 0
            
            if val_widget and type_widget and path_widget_container:
                val = val_widget.text().strip()
                trigger_type = type_widget.currentData()
                
                # Path input is first child of layout
                path_input = path_widget_container.findChild(QLineEdit)
                path = path_input.property("full_path") if path_input else ""
                if not path and path_input:
                    path = path_input.text()
                
                if val or path:
                    triggers.append({
                        "type": trigger_type,
                        "value": val,
                        "path": path,
                        "x": x_off,
                        "y": y_off
                    })
        return triggers
