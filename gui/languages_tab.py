from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QListWidget, QGroupBox, QFormLayout, QLineEdit, QPushButton, QTextEdit, QComboBox, QSplitter, QLabel, QMessageBox, QDoubleSpinBox, QSpinBox
from PySide6.QtCore import Qt
from utils.translator import translator
from api.openrouter import OpenRouterAPI

class LanguagesTab(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window
        self.settings = self.main_window.settings
        self.language_settings = self.settings.get_language_settings()
        self.models = []

        # Layout
        self.layout = QHBoxLayout(self)
        self.splitter = QSplitter(Qt.Horizontal)

        # Left side - Language List
        self.language_list_group = QGroupBox(translator.tr("Languages"))
        self.language_list_layout = QVBoxLayout(self.language_list_group)
        self.language_list = QListWidget()
        self.add_language_layout = QHBoxLayout()
        self.new_language_input = QLineEdit()
        self.new_language_input.setPlaceholderText(translator.tr("Enter language code (e.g., en)"))
        self.add_language_button = QPushButton("+")
        self.remove_language_button = QPushButton("-")
        self.add_language_layout.addWidget(self.new_language_input)
        self.add_language_layout.addWidget(self.add_language_button)
        self.add_language_layout.addWidget(self.remove_language_button)
        self.language_list_layout.addWidget(self.language_list)
        self.language_list_layout.addLayout(self.add_language_layout)

        # Right side - Language Settings
        self.language_settings_group = QGroupBox(translator.tr("Language Settings"))
        self.language_settings_layout = QFormLayout(self.language_settings_group)
        
        self.prompt_input = QTextEdit()
        self.model_combo = QComboBox()
        self.temperature_spinbox = QDoubleSpinBox()
        self.temperature_spinbox.setRange(0.0, 2.0)
        self.temperature_spinbox.setSingleStep(0.1)
        self.temperature_spinbox.setValue(1.0)
        self.max_tokens_spinbox = QSpinBox()
        self.max_tokens_spinbox.setRange(1, 999999)
        self.max_tokens_spinbox.setValue(1024)

        self.language_settings_layout.addRow(translator.tr("Translation Prompt:"), self.prompt_input)
        self.language_settings_layout.addRow(translator.tr("Model:"), self.model_combo)
        self.language_settings_layout.addRow(translator.tr("Temperature:"), self.temperature_spinbox)
        self.language_settings_layout.addRow(translator.tr("Max Tokens:"), self.max_tokens_spinbox)
        self.language_settings_group.setEnabled(False)

        self.splitter.addWidget(self.language_list_group)
        self.splitter.addWidget(self.language_settings_group)
        self.splitter.setSizes([120, 380])

        self.layout.addWidget(self.splitter)

        # Connections
        translator.language_changed.connect(self.retranslate_ui)
        self.add_language_button.clicked.connect(self._add_language)
        self.remove_language_button.clicked.connect(self._remove_language)
        self.language_list.currentItemChanged.connect(self._display_language_settings)
        self.prompt_input.textChanged.connect(self._save_current_language_settings)
        self.model_combo.currentIndexChanged.connect(self._save_current_language_settings)
        self.temperature_spinbox.valueChanged.connect(self._save_current_language_settings)
        self.max_tokens_spinbox.valueChanged.connect(self._save_current_language_settings)

        self._load_languages()
        self._load_models()

    def retranslate_ui(self):
        self.language_list_group.setTitle(translator.tr("Languages"))
        self.new_language_input.setPlaceholderText(translator.tr("Enter language code (e.g., en)"))
        self.language_settings_group.setTitle(translator.tr("Language Settings"))
        # Labels are now in addRow

    def _load_languages(self):
        self.language_list.clear()
        for lang_code in self.language_settings.keys():
            self.language_list.addItem(lang_code)

    def _load_models(self):
        self.models = self.settings.get_openrouter_models()
        self.model_combo.clear()
        self.model_combo.addItems(self.models)

    def _add_language(self):
        lang_code = self.new_language_input.text().strip()
        if not lang_code:
            return
        if lang_code in self.language_settings:
            QMessageBox.warning(self, translator.tr("Error"), translator.tr("Language already exists."))
            return
        
        self.language_settings[lang_code] = {
            "prompt": "", 
            "model": "", 
            "temperature": 1.0, 
            "max_tokens": 1024
        }
        self.settings.set_language_settings(self.language_settings)
        self.language_list.addItem(lang_code)
        self.new_language_input.clear()

    def _remove_language(self):
        current_item = self.language_list.currentItem()
        if not current_item:
            return
        
        lang_code = current_item.text()
        reply = QMessageBox.question(self, translator.tr("Remove Language"), 
                                     translator.tr(f"Are you sure you want to remove '{lang_code}'?"))
        if reply == QMessageBox.Yes:
            del self.language_settings[lang_code]
            self.settings.set_language_settings(self.language_settings)
            self.language_list.takeItem(self.language_list.row(current_item))
            self.language_settings_group.setEnabled(False)

    def _display_language_settings(self, current, previous):
        if not current:
            self.language_settings_group.setEnabled(False)
            self.prompt_input.clear()
            self.model_combo.setCurrentIndex(-1)
            return

        self.language_settings_group.setEnabled(True)
        lang_code = current.text()
        settings = self.language_settings.get(lang_code, {})
        
        # Block signals to prevent saving while populating fields
        for widget in [self.prompt_input, self.model_combo, self.temperature_spinbox, self.max_tokens_spinbox]:
            widget.blockSignals(True)
        
        self.prompt_input.setText(settings.get("prompt", ""))
        model = settings.get("model", "")
        if model in self.models:
            self.model_combo.setCurrentText(model)
        else:
            self.model_combo.setCurrentIndex(-1)
        self.temperature_spinbox.setValue(settings.get("temperature", 1.0))
        self.max_tokens_spinbox.setValue(settings.get("max_tokens", 1024))
            
        # Unblock signals
        for widget in [self.prompt_input, self.model_combo, self.temperature_spinbox, self.max_tokens_spinbox]:
            widget.blockSignals(False)

    def _save_current_language_settings(self):
        current_item = self.language_list.currentItem()
        if not current_item:
            return

        lang_code = current_item.text()
        self.language_settings[lang_code] = {
            "prompt": self.prompt_input.toPlainText(),
            "model": self.model_combo.currentText(),
            "temperature": self.temperature_spinbox.value(),
            "max_tokens": self.max_tokens_spinbox.value()
        }
        self.settings.set_language_settings(self.language_settings)