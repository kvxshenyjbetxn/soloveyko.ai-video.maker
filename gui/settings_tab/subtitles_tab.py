from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QScrollArea, QFormLayout,
    QPushButton, QSpinBox, QFontComboBox, QColorDialog,
    QGroupBox, QComboBox, QRadioButton, QButtonGroup, QHBoxLayout, QLabel
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor
from utils.settings import settings_manager
from utils.translator import translator

class SubtitlesTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

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
        self.engine_group_btn.addButton(self.rb_amd)
        self.engine_group_btn.addButton(self.rb_standard)
        self.rb_amd.toggled.connect(self.on_engine_changed)
        # self.rb_standard.toggled.connect(self.on_engine_changed) # One connection is enough
        
        engine_layout.addWidget(self.rb_amd)
        engine_layout.addWidget(self.rb_standard)
        self.engine_group.setLayout(engine_layout)
        layout.addWidget(self.engine_group)

        # --- Model Selection ---
        self.whisper_group = QGroupBox()
        whisper_layout = QFormLayout()

        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentTextChanged.connect(self.save_settings)
        whisper_layout.addRow(self.model_label, self.model_combo)

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
        style_layout.addRow(self.margin_v_label, self.margin_v_spin)

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
        logic_layout.addRow(self.fade_in_label, self.fade_in_spin)

        self.fade_out_label = QLabel()
        self.fade_out_spin = QSpinBox()
        self.fade_out_spin.setRange(0, 5000)
        self.fade_out_spin.setSuffix(" ms")
        self.fade_out_spin.valueChanged.connect(self.save_settings)
        logic_layout.addRow(self.fade_out_label, self.fade_out_spin)

        self.max_words_label = QLabel()
        self.max_words_spin = QSpinBox()
        self.max_words_spin.setRange(1, 50)
        self.max_words_spin.valueChanged.connect(self.save_settings)
        logic_layout.addRow(self.max_words_label, self.max_words_spin)

        self.logic_group.setLayout(logic_layout)
        layout.addWidget(self.logic_group)

        layout.addStretch()
        
    def update_fields(self):
        self.settings = settings_manager.get('subtitles', {})
        
        # Block signals
        self.rb_amd.blockSignals(True)
        self.rb_standard.blockSignals(True)
        self.model_combo.blockSignals(True)
        self.font_combo.blockSignals(True)
        self.fontsize_spin.blockSignals(True)
        self.margin_v_spin.blockSignals(True)
        self.fade_in_spin.blockSignals(True)
        self.fade_out_spin.blockSignals(True)
        self.max_words_spin.blockSignals(True)

        saved_type = self.settings.get('whisper_type', 'amd')
        if saved_type == 'standard':
            self.rb_standard.setChecked(True)
        else:
            self.rb_amd.setChecked(True)
        
        self.update_models_list()
        saved_model = self.settings.get('whisper_model', 'base.bin')
        index = self.model_combo.findText(saved_model)
        if index != -1:
            self.model_combo.setCurrentIndex(index)

        current_font = self.settings.get('font', 'Arial')
        self.font_combo.setCurrentFont(current_font)
        self.fontsize_spin.setValue(self.settings.get('fontsize', 60))
        
        self.current_color = self.settings.get('color', [255, 255, 255])
        self.update_color_btn_style()

        self.margin_v_spin.setValue(self.settings.get('margin_v', 50))
        self.fade_in_spin.setValue(self.settings.get('fade_in', 0))
        self.fade_out_spin.setValue(self.settings.get('fade_out', 0))
        self.max_words_spin.setValue(self.settings.get('max_words', 10))

        # Unblock signals
        self.rb_amd.blockSignals(False)
        self.rb_standard.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.font_combo.blockSignals(False)
        self.fontsize_spin.blockSignals(False)
        self.margin_v_spin.blockSignals(False)
        self.fade_in_spin.blockSignals(False)
        self.fade_out_spin.blockSignals(False)
        self.max_words_spin.blockSignals(False)

    def retranslate_ui(self):
        self.engine_group.setTitle(translator.translate("whisper_engine_group"))
        self.rb_amd.setText(translator.translate("amd_gpu_fork_radio"))
        self.rb_standard.setText(translator.translate("standard_python_radio"))
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

    def update_models_list(self):
        self.model_combo.blockSignals(True)
        self.model_combo.clear()

        if self.rb_standard.isChecked():
            models = ["tiny", "base", "small", "medium", "large"]
        else:
            models = ["base.bin", "small.bin", "medium.bin", "large.bin"]

        self.model_combo.addItems(models)
        self.model_combo.blockSignals(False)

    def on_engine_changed(self, *args):
        self.update_models_list()
        current_text = self.model_combo.currentText()
        if self.rb_standard.isChecked() and current_text.endswith(".bin"):
             new_text = current_text.replace(".bin", "")
             index = self.model_combo.findText(new_text)
             if index != -1: self.model_combo.setCurrentIndex(index)
        elif self.rb_amd.isChecked() and not current_text.endswith(".bin"):
             new_text = current_text + ".bin"
             index = self.model_combo.findText(new_text)
             if index != -1: self.model_combo.setCurrentIndex(index)

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
        whisper_type = 'standard' if self.rb_standard.isChecked() else 'amd'

        new_settings = {
            'whisper_type': whisper_type,
            'whisper_model': self.model_combo.currentText(),
            'font': self.font_combo.currentFont().family(),
            'fontsize': self.fontsize_spin.value(),
            'color': self.current_color,
            'margin_v': self.margin_v_spin.value(),
            'fade_in': self.fade_in_spin.value(),
            'fade_out': self.fade_out_spin.value(),
            'max_words': self.max_words_spin.value()
        }
        settings_manager.set('subtitles', new_settings)