from PySide6.QtWidgets import (
    QWidget, QHBoxLayout, QVBoxLayout, QListWidget, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QSplitter, QFormLayout, QGroupBox
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager

class LanguagesTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.current_lang_id = None
        self.init_ui()
        self.load_languages()
        self.retranslate_ui()
        if self.lang_list_widget.count() > 0:
            self.lang_list_widget.setCurrentRow(0)

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
        
        model_layout = QHBoxLayout()
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_current_language_settings)
        model_layout.addWidget(self.model_label)
        model_layout.addWidget(self.model_combo)

        right_layout.addWidget(self.prompt_label)
        right_layout.addWidget(self.prompt_edit)
        right_layout.addLayout(model_layout)
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

        self.prompt_edit.setPlainText(config.get("prompt", ""))
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)

        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        
        self.right_panel.setVisible(True)

    def add_language(self):
        display_name = self.lang_name_input.text().strip()
        lang_id = self.lang_id_input.text().strip()

        if not display_name or not lang_id:
            return

        languages = self.settings.get("languages_config", {})
        if lang_id in languages:
            return
            
        languages[lang_id] = {"display_name": display_name, "prompt": "", "model": ""}
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
            self.settings.set("languages_config", languages)

    def retranslate_ui(self):
        self.add_remove_group.setTitle(translator.translate("manage_languages"))
        self.lang_name_label.setText(translator.translate("language_name_label"))
        self.lang_id_label.setText(translator.translate("language_id_label"))
        self.add_lang_button.setText(translator.translate("add_model"))
        self.remove_lang_button.setText(translator.translate("remove_model"))
        self.prompt_label.setText(translator.translate("language_prompt_label"))
        self.model_label.setText(translator.translate("translation_model_label"))
