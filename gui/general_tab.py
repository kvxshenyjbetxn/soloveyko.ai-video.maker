from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QComboBox, QFormLayout
from utils.translator import translator

class GeneralTab(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window
        self.settings = self.main_window.settings

        self.language_map = {
            "English": "en",
            "Українська": "uk",
            "Русский": "ru"
        }

        # Layout
        self.layout = QVBoxLayout(self)
        self.form_layout = QFormLayout()

        # --- Theme Selection ---
        self.theme_label = QLabel()
        self.theme_combo = QComboBox()
        self.theme_combo.addItems(self.main_window.theme_paths.keys())
        self.form_layout.addRow(self.theme_label, self.theme_combo)

        # --- Language Selection ---
        self.language_label = QLabel()
        self.language_combo = QComboBox()
        self.language_combo.addItems(self.language_map.keys())
        self.form_layout.addRow(self.language_label, self.language_combo)

        self.layout.addLayout(self.form_layout)
        self.layout.addStretch() # Pushes the settings to the top

        # --- Connections ---
        self.theme_combo.currentTextChanged.connect(self._on_theme_changed)
        self.language_combo.currentTextChanged.connect(self._on_language_changed)
        translator.language_changed.connect(self.retranslate_ui)

        # --- Load initial settings ---
        self.load_initial_settings()
        self.retranslate_ui()


    def load_initial_settings(self):
        # Block signals while setting initial values to avoid triggering handlers
        self.theme_combo.blockSignals(True)
        self.language_combo.blockSignals(True)

        current_theme = self.settings.get_theme()
        self.theme_combo.setCurrentText(current_theme)

        current_lang_code = self.settings.get_language()
        for lang_name, lang_code in self.language_map.items():
            if lang_code == current_lang_code:
                self.language_combo.setCurrentText(lang_name)
                break
        
        self.theme_combo.blockSignals(False)
        self.language_combo.blockSignals(False)

    def _on_theme_changed(self, theme_name):
        self.main_window.load_theme(theme_name)

    def _on_language_changed(self, lang_name):
        lang_code = self.language_map.get(lang_name)
        if lang_code:
            self.settings.set_language(lang_code)
            translator.set_language(lang_code)

    def retranslate_ui(self):
        self.theme_label.setText(translator.tr("Theme:"))
        self.language_label.setText(translator.tr("Language:"))
