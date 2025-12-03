from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QScrollArea, QFormLayout, 
    QPushButton, QSpinBox, QFontComboBox, QColorDialog, 
    QGroupBox, QComboBox
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor
from utils.settings import settings_manager

class SubtitlesTab(QWidget):
    def __init__(self):
        super().__init__()
        # Завантажуємо існуючі налаштування
        self.settings = settings_manager.get('subtitles', {})
        
        # Ініціалізуємо колір
        self.current_color = self.settings.get('color', [255, 255, 255]) 
        
        self.init_ui()

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
        
        # --- Whisper Configuration ---
        whisper_group = QGroupBox("Whisper Model Selection")
        whisper_layout = QFormLayout()
        
        # Model Selection (Dropdown)
        self.model_combo = QComboBox()
        self.model_combo.addItems(["base.bin", "medium.bin", "small.bin"])
        
        # Встановлюємо збережену модель або дефолтну
        saved_model = self.settings.get('whisper_model', 'base.bin')
        index = self.model_combo.findText(saved_model)
        if index != -1:
            self.model_combo.setCurrentIndex(index)
        
        self.model_combo.currentTextChanged.connect(self.save_settings)
        whisper_layout.addRow("Model:", self.model_combo)
        
        whisper_group.setLayout(whisper_layout)
        layout.addWidget(whisper_group)

        # --- Style Configuration ---
        style_group = QGroupBox("Subtitle Style")
        style_layout = QFormLayout()

        # Font
        self.font_combo = QFontComboBox()
        current_font = self.settings.get('font', 'Arial')
        self.font_combo.setCurrentFont(current_font)
        self.font_combo.currentFontChanged.connect(self.save_settings)
        style_layout.addRow("Font:", self.font_combo)

        # Font Size
        self.fontsize_spin = QSpinBox()
        self.fontsize_spin.setRange(10, 200)
        self.fontsize_spin.setValue(self.settings.get('fontsize', 60))
        self.fontsize_spin.valueChanged.connect(self.save_settings)
        style_layout.addRow("Font Size:", self.fontsize_spin)

        # Color
        self.color_btn = QPushButton()
        self.update_color_btn_style()
        self.color_btn.clicked.connect(self.choose_color)
        style_layout.addRow("Color:", self.color_btn)

        # Margin Vertical
        self.margin_v_spin = QSpinBox()
        self.margin_v_spin.setRange(0, 500)
        self.margin_v_spin.setValue(self.settings.get('margin_v', 50))
        self.margin_v_spin.valueChanged.connect(self.save_settings)
        style_layout.addRow("Vertical Margin:", self.margin_v_spin)

        style_group.setLayout(style_layout)
        layout.addWidget(style_group)

        # --- Animation & Logic ---
        logic_group = QGroupBox("Animation & Logic")
        logic_layout = QFormLayout()

        # Fade In
        self.fade_in_spin = QSpinBox()
        self.fade_in_spin.setRange(0, 5000)
        self.fade_in_spin.setSuffix(" ms")
        self.fade_in_spin.setValue(self.settings.get('fade_in', 0))
        self.fade_in_spin.valueChanged.connect(self.save_settings)
        logic_layout.addRow("Fade In:", self.fade_in_spin)

        # Fade Out
        self.fade_out_spin = QSpinBox()
        self.fade_out_spin.setRange(0, 5000)
        self.fade_out_spin.setSuffix(" ms")
        self.fade_out_spin.setValue(self.settings.get('fade_out', 0))
        self.fade_out_spin.valueChanged.connect(self.save_settings)
        logic_layout.addRow("Fade Out:", self.fade_out_spin)
        
        # Max Words
        self.max_words_spin = QSpinBox()
        self.max_words_spin.setRange(1, 50)
        self.max_words_spin.setValue(self.settings.get('max_words', 10))
        self.max_words_spin.valueChanged.connect(self.save_settings)
        logic_layout.addRow("Max Words per Line:", self.max_words_spin)

        logic_group.setLayout(logic_layout)
        layout.addWidget(logic_group)

        layout.addStretch()

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

    def save_settings(self):
        new_settings = {
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