import ast
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QScrollArea, QFormLayout,
    QPushButton, QSpinBox, QFontComboBox, QColorDialog,
    QGroupBox, QComboBox, QRadioButton, QButtonGroup, QHBoxLayout, QLabel
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor
from utils.settings import settings_manager
from utils.translator import translator
from gui.widgets.help_label import HelpLabel

class SubtitlesTab(QWidget):
    def __init__(self):
        super().__init__()
        self.is_loading = True
        self.current_color = [255, 255, 255] # Default to prevent AttributeError
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()
        self.is_loading = False

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        scroll_content = QWidget()
        scroll_area.setWidget(scroll_content)

        layout = QVBoxLayout(scroll_content)

        # --- Engine Selection ---
        self.engine_group = QGroupBox()
        engine_layout = QHBoxLayout()

        self.engine_group_btn = QButtonGroup(self)
        self.rb_amd = QRadioButton()
        self.rb_standard = QRadioButton()
        self.rb_assemblyai = QRadioButton()
        self.engine_group_btn.addButton(self.rb_amd)
        self.engine_group_btn.addButton(self.rb_standard)
        self.engine_group_btn.addButton(self.rb_assemblyai)
        self.rb_amd.toggled.connect(self.on_engine_changed)
        self.rb_standard.toggled.connect(self.on_engine_changed)
        self.rb_assemblyai.toggled.connect(self.on_engine_changed)
        
        # Add labels with help icons
        self.standard_help = HelpLabel("standard_python_hint")
        self.amd_help = HelpLabel("amd_gpu_fork_hint")
        self.assemblyai_help = HelpLabel("assemblyai_hint")

        def create_radio_container(rb, help_label):
            container = QWidget()
            cont_layout = QHBoxLayout(container)
            cont_layout.setContentsMargins(0, 0, 5, 0)
            cont_layout.setSpacing(5)
            cont_layout.addWidget(help_label)
            cont_layout.addWidget(rb)
            return container

        engine_layout.addWidget(create_radio_container(self.rb_standard, self.standard_help))
        engine_layout.addWidget(create_radio_container(self.rb_amd, self.amd_help))
        engine_layout.addWidget(create_radio_container(self.rb_assemblyai, self.assemblyai_help))
        engine_layout.addStretch()

        self.engine_group.setLayout(engine_layout)
        layout.addWidget(self.engine_group)

        # --- Model Selection ---
        self.whisper_group = QGroupBox()
        whisper_layout = QFormLayout()

        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentTextChanged.connect(self.save_settings)
        
        self.model_help = HelpLabel("whisper_model_hint")
        model_label_container = QWidget()
        model_label_layout = QHBoxLayout(model_label_container)
        model_label_layout.setContentsMargins(0, 0, 0, 0)
        model_label_layout.setSpacing(5)
        model_label_layout.addWidget(self.model_help)
        model_label_layout.addWidget(self.model_label)

        whisper_layout.addRow(model_label_container, self.model_combo)

        self.whisper_group.setLayout(whisper_layout)
        layout.addWidget(self.whisper_group)

        # --- Style Configuration ---
        self.style_group = QGroupBox()
        style_layout = QFormLayout()

        self.font_label = QLabel()
        self.font_combo = QFontComboBox()
        self.font_combo.currentFontChanged.connect(self.save_settings)
        style_layout.addRow(self.font_label, self.font_combo)

        self.fontsize_label = QLabel()
        self.fontsize_spin = QSpinBox()
        self.fontsize_spin.setRange(10, 200)
        self.fontsize_spin.valueChanged.connect(self.save_settings)
        style_layout.addRow(self.fontsize_label, self.fontsize_spin)

        self.color_label = QLabel()
        self.color_btn = QPushButton()
        self.color_btn.clicked.connect(self.choose_color)
        style_layout.addRow(self.color_label, self.color_btn)

        self.margin_v_label = QLabel()
        self.margin_v_spin = QSpinBox()
        self.margin_v_spin.setRange(0, 500)
        self.margin_v_spin.valueChanged.connect(self.save_settings)
        
        self.margin_v_help = HelpLabel("vertical_margin_hint")
        margin_v_container = QWidget()
        margin_v_layout = QHBoxLayout(margin_v_container)
        margin_v_layout.setContentsMargins(0, 0, 0, 0)
        margin_v_layout.setSpacing(5)
        margin_v_layout.addWidget(self.margin_v_help)
        margin_v_layout.addWidget(self.margin_v_label)

        style_layout.addRow(margin_v_container, self.margin_v_spin)

        self.style_group.setLayout(style_layout)
        layout.addWidget(self.style_group)

        # --- Animation & Logic ---
        self.logic_group = QGroupBox()
        logic_layout = QFormLayout()

        self.fade_in_label = QLabel()
        self.fade_in_spin = QSpinBox()
        self.fade_in_spin.setRange(0, 5000)
        self.fade_in_spin.setSuffix(" ms")
        self.fade_in_spin.valueChanged.connect(self.save_settings)
        
        self.fade_in_help = HelpLabel("fade_hint")
        fade_in_container = QWidget()
        fade_in_layout = QHBoxLayout(fade_in_container)
        fade_in_layout.setContentsMargins(0, 0, 0, 0)
        fade_in_layout.setSpacing(5)
        fade_in_layout.addWidget(self.fade_in_help)
        fade_in_layout.addWidget(self.fade_in_label)
        logic_layout.addRow(fade_in_container, self.fade_in_spin)

        self.fade_out_label = QLabel()
        self.fade_out_spin = QSpinBox()
        self.fade_out_spin.setRange(0, 5000)
        self.fade_out_spin.setSuffix(" ms")
        self.fade_out_spin.valueChanged.connect(self.save_settings)

        self.fade_out_help = HelpLabel("fade_hint")
        fade_out_container = QWidget()
        fade_out_layout = QHBoxLayout(fade_out_container)
        fade_out_layout.setContentsMargins(0, 0, 0, 0)
        fade_out_layout.setSpacing(5)
        fade_out_layout.addWidget(self.fade_out_help)
        fade_out_layout.addWidget(self.fade_out_label)
        logic_layout.addRow(fade_out_container, self.fade_out_spin)

        self.max_words_label = QLabel()
        self.max_words_spin = QSpinBox()
        self.max_words_spin.setRange(1, 50)
        self.max_words_spin.valueChanged.connect(self.save_settings)

        self.max_words_help = HelpLabel("max_words_hint")
        max_words_container = QWidget()
        max_words_layout = QHBoxLayout(max_words_container)
        max_words_layout.setContentsMargins(0, 0, 0, 0)
        max_words_layout.setSpacing(5)
        max_words_layout.addWidget(self.max_words_help)
        max_words_layout.addWidget(self.max_words_label)
        logic_layout.addRow(max_words_container, self.max_words_spin)

        self.logic_group.setLayout(logic_layout)
        layout.addWidget(self.logic_group)

        layout.addStretch()
        
    def update_fields(self):
        self.is_loading = True
        self.settings = settings_manager.get('subtitles', {})
        
        # --- Populate fields from settings ---
        saved_type = self.settings.get('whisper_type', 'standard')
        if saved_type == 'standard':
            self.rb_standard.setChecked(True)
        elif saved_type == 'amd':
            self.rb_amd.setChecked(True)
        else: # assemblyai
            self.rb_assemblyai.setChecked(True)
        
        self._update_engine_ui(update_model_list=True)

        saved_model = self.settings.get('whisper_model', 'base')
        index = self.model_combo.findText(saved_model)
        if index != -1:
            self.model_combo.setCurrentIndex(index)
        else:
            default_model = "base"
            if self.rb_amd.isChecked():
                default_model = "base.bin"
            index = self.model_combo.findText(default_model)
            if index != -1:
                self.model_combo.setCurrentIndex(index)

        current_font = self.settings.get('font', 'Arial')
        self.font_combo.setCurrentFont(current_font)
        self.fontsize_spin.setValue(self.settings.get('fontsize', 60))
        
        # Get color, with defensive parsing for corrupted settings
        color_val = self.settings.get('color', [255, 255, 255])
        if isinstance(color_val, str):
            try:
                parsed_val = ast.literal_eval(color_val)
                if isinstance(parsed_val, list) and len(parsed_val) == 3:
                    self.current_color = [int(c) for c in parsed_val] # Ensure all are integers
                else:
                    self.current_color = [255, 255, 255]
            except (ValueError, SyntaxError, TypeError):
                self.current_color = [255, 255, 255] # Fallback on parsing error
        elif isinstance(color_val, list):
             self.current_color = [int(c) for c in color_val] # Ensure all are integers
        else:
            self.current_color = [255, 255, 255]

        self.update_color_btn_style()

        self.margin_v_spin.setValue(self.settings.get('margin_v', 50))
        self.fade_in_spin.setValue(self.settings.get('fade_in', 0))
        self.fade_out_spin.setValue(self.settings.get('fade_out', 0))
        self.max_words_spin.setValue(self.settings.get('max_words', 10))
        
        self.is_loading = False


    def retranslate_ui(self):
        self.engine_group.setTitle(translator.translate("whisper_engine_group"))
        self.rb_amd.setText(translator.translate("amd_gpu_fork_radio"))
        self.rb_standard.setText(translator.translate("standard_python_radio"))
        self.rb_assemblyai.setText(translator.translate("assemblyai_radio"))
        self.whisper_group.setTitle(translator.translate("model_selection_group"))
        self.model_label.setText(translator.translate("model_label"))
        self.style_group.setTitle(translator.translate("subtitle_style_group"))
        self.font_label.setText(translator.translate("font_label"))
        self.fontsize_label.setText(translator.translate("font_size_label"))
        self.color_label.setText(translator.translate("color_label"))
        self.margin_v_label.setText(translator.translate("vertical_margin_label"))
        self.logic_group.setTitle(translator.translate("animation_logic_group"))
        self.fade_in_label.setText(translator.translate("fade_in_label"))
        self.fade_out_label.setText(translator.translate("fade_out_label"))
        self.max_words_label.setText(translator.translate("max_words_per_line_label"))

        self.standard_help.update_tooltip()
        self.amd_help.update_tooltip()
        self.assemblyai_help.update_tooltip()
        self.model_help.update_tooltip()
        self.margin_v_help.update_tooltip()
        self.fade_in_help.update_tooltip()
        self.fade_out_help.update_tooltip()
        self.max_words_help.update_tooltip()

    def update_models_list(self):
        self.model_combo.blockSignals(True)
        self.model_combo.clear()

        if self.rb_standard.isChecked():
            models = ["tiny", "base", "small", "medium", "large"]
        else: # amd
            models = ["base.bin", "small.bin", "medium.bin", "large.bin"]

        self.model_combo.addItems(models)
        self.model_combo.blockSignals(False)

    def _update_engine_ui(self, update_model_list=False):
        """Updates the UI based on the selected engine, without saving."""
        is_whisper = self.rb_standard.isChecked() or self.rb_amd.isChecked()
        self.whisper_group.setVisible(is_whisper)

        if is_whisper:
            current_text = self.model_combo.currentText()

            if update_model_list:
                self.update_models_list()
            
            if self.rb_standard.isChecked() and current_text.endswith(".bin"):
                new_text = current_text.replace(".bin", "")
                index = self.model_combo.findText(new_text)
                if index != -1: self.model_combo.setCurrentIndex(index)
            elif self.rb_amd.isChecked() and not current_text.endswith(".bin") and current_text:
                new_text = current_text + ".bin"
                index = self.model_combo.findText(new_text)
                if index != -1: self.model_combo.setCurrentIndex(index)

    def on_engine_changed(self, checked=False):
        """Connected to the radio button toggle signals."""
        if getattr(self, 'is_loading', False) or not checked:
            return

        self._update_engine_ui(update_model_list=True)
        self.save_settings()

    def update_color_btn_style(self):
        color = QColor(self.current_color[0], self.current_color[1], self.current_color[2])
        self.color_btn.setStyleSheet(f"background-color: {color.name()};")

    def choose_color(self):
        initial = QColor(self.current_color[0], self.current_color[1], self.current_color[2])
        color = QColorDialog.getColor(initial, self)
        if color.isValid():
            self.current_color = [color.red(), color.green(), color.blue()]
            self.update_color_btn_style()
            self.save_settings()

    def save_settings(self, *args):
        if getattr(self, 'is_loading', False):
            return

        # Start with the existing settings to avoid overwriting keys
        # for invisible fields or having defaults from UI overwrite good settings.
        new_settings = settings_manager.get('subtitles', {})

        if self.rb_standard.isChecked():
            new_settings['whisper_type'] = 'standard'
        elif self.rb_amd.isChecked():
            new_settings['whisper_type'] = 'amd'
        else:
            new_settings['whisper_type'] = 'assemblyai'

        if self.whisper_group.isVisible():
            new_settings['whisper_model'] = self.model_combo.currentText()
        
        new_settings['font'] = self.font_combo.currentFont().family()
        new_settings['fontsize'] = self.fontsize_spin.value()
        new_settings['color'] = self.current_color
        new_settings['margin_v'] = self.margin_v_spin.value()
        new_settings['fade_in'] = self.fade_in_spin.value()
        new_settings['fade_out'] = self.fade_out_spin.value()
        new_settings['max_words'] = self.max_words_spin.value()
        
        settings_manager.set('subtitles', new_settings)