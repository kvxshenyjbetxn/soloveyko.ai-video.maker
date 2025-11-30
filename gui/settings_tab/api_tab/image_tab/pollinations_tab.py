import logging
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QCheckBox, QHBoxLayout
from utils.settings import settings_manager
from utils.translator import translator

class PollinationsTab(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.models = ["flux", "flux-realism", "flux-3d", "flux-cablyai", "dall-e-3", "midjourney", "boreal"]
        self.initUI()
        self.load_settings()

    def initUI(self):
        layout = QFormLayout(self)

        # Model
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.addItems(self.models)
        layout.addRow(self.model_label, self.model_combo)

        # Token
        self.token_label = QLabel()
        self.token_input = QLineEdit()
        layout.addRow(self.token_label, self.token_input)

        # Width and Height
        self.size_label = QLabel()
        size_layout = QHBoxLayout()
        self.width_spinbox = QSpinBox()
        self.width_spinbox.setRange(1, 4096)
        self.width_spinbox.setValue(1024)
        self.height_spinbox = QSpinBox()
        self.height_spinbox.setRange(1, 4096)
        self.height_spinbox.setValue(1024)
        size_layout.addWidget(self.width_spinbox)
        size_layout.addWidget(QLabel("x"))
        size_layout.addWidget(self.height_spinbox)
        layout.addRow(self.size_label, size_layout)

        # NoLogo
        self.nologo_checkbox = QCheckBox()
        layout.addRow(self.nologo_checkbox)

        # Enhance
        self.enhance_checkbox = QCheckBox()
        layout.addRow(self.enhance_checkbox)

        self.setLayout(layout)

    def translate_ui(self):
        self.model_label.setText(translator.translate("pollinations_model_label"))
        self.token_label.setText(translator.translate("pollinations_token_label"))
        self.token_input.setPlaceholderText(translator.translate("pollinations_token_placeholder"))
        self.size_label.setText(translator.translate("image_size_label"))
        self.nologo_checkbox.setText(translator.translate("nologo_label"))
        self.enhance_checkbox.setText(translator.translate("enhance_prompt_label"))

    def load_settings(self):
        pollinations_settings = settings_manager.get("pollinations", {})
        self.model_combo.setCurrentText(pollinations_settings.get("model", "flux"))
        self.token_input.setText(pollinations_settings.get("token", ""))
        self.width_spinbox.setValue(pollinations_settings.get("width", 1024))
        self.height_spinbox.setValue(pollinations_settings.get("height", 1024))
        self.nologo_checkbox.setChecked(pollinations_settings.get("nologo", False))
        self.enhance_checkbox.setChecked(pollinations_settings.get("enhance", False))

    def save_settings(self):
        pollinations_settings = {
            "model": self.model_combo.currentText(),
            "token": self.token_input.text(),
            "width": self.width_spinbox.value(),
            "height": self.height_spinbox.value(),
            "nologo": self.nologo_checkbox.isChecked(),
            "enhance": self.enhance_checkbox.isChecked(),
        }
        settings_manager.set("pollinations", pollinations_settings)