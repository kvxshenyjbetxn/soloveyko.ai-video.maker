from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QFormLayout, QSpinBox, QScrollArea
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager

class PromptsTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.init_ui()
        self.load_settings()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0,0,0,0)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        scroll_content = QWidget()
        scroll_area.setWidget(scroll_content)

        layout = QVBoxLayout(scroll_content)
        
        self.prompt_content_label = QLabel()
        self.prompt_edit = QTextEdit()
        self.prompt_edit.textChanged.connect(self.save_settings)
        
        settings_form_layout = QFormLayout()
        
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.model_label, self.model_combo)

        self.tokens_label = QLabel()
        self.tokens_spinbox = QSpinBox()
        self.tokens_spinbox.setRange(0, 128000)
        self.tokens_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.tokens_label, self.tokens_spinbox)

        layout.addWidget(self.prompt_content_label)
        layout.addWidget(self.prompt_edit)
        layout.addLayout(settings_form_layout)
        layout.addStretch()

    def load_settings(self):
        config = self.settings.get("image_prompt_settings", {})
        
        self.prompt_edit.blockSignals(True)
        self.model_combo.blockSignals(True)
        self.tokens_spinbox.blockSignals(True)

        self.prompt_edit.setPlainText(config.get("prompt", ""))
        self.load_models()
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)
        self.tokens_spinbox.setValue(config.get("max_tokens", 4096))

        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.tokens_spinbox.blockSignals(False)

    def load_models(self):
        self.model_combo.clear()
        models = self.settings.get("openrouter_models", [])
        self.model_combo.addItems(models)

    def save_settings(self):
        config = {
            "prompt": self.prompt_edit.toPlainText(),
            "model": self.model_combo.currentText(),
            "max_tokens": self.tokens_spinbox.value()
        }
        self.settings.set("image_prompt_settings", config)

    def retranslate_ui(self):
        self.prompt_content_label.setText(translator.translate("prompt_content_label"))
        self.model_label.setText(translator.translate("image_model_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))