from PySide6.QtWidgets import QWidget, QVBoxLayout, QGroupBox, QFormLayout, QLineEdit, QPushButton, QLabel, QMessageBox, QListWidget, QHBoxLayout, QComboBox
from utils.translator import translator
from api.openrouter import OpenRouterAPI

class OpenRouterTab(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window
        self.settings = self.main_window.settings
        self.all_models = []

        # Layout
        self.layout = QVBoxLayout(self)

        # --- API Key Group ---
        self.api_key_group = QGroupBox(translator.tr("Authentication"))
        self.api_key_layout = QFormLayout(self.api_key_group)
        self.api_key_input = QLineEdit()
        self.api_key_input.setEchoMode(QLineEdit.Password)
        self.api_key_input.setText(self.settings.get_openrouter_api_key())
        self.check_connection_button = QPushButton(translator.tr("Check Connection & Get Balance"))
        self.balance_label = QLabel(translator.tr("Balance: -"))
        self.api_key_layout.addRow(translator.tr("API Key:"), self.api_key_input)
        self.api_key_layout.addRow(self.check_connection_button)
        self.api_key_layout.addRow(self.balance_label)
        self.layout.addWidget(self.api_key_group)

        # --- Model Management Group ---
        self.model_group = QGroupBox(translator.tr("Model Management"))
        self.model_layout = QVBoxLayout(self.model_group)
        self.user_models_list = QListWidget()
        self.add_model_layout = QHBoxLayout()
        self.available_models_combo = QComboBox()
        self.available_models_combo.setEditable(True)
        self.add_model_button = QPushButton(translator.tr("Add Model"))
        self.remove_model_button = QPushButton(translator.tr("Remove Selected Model"))
        self.add_model_layout.addWidget(self.available_models_combo, 1)
        self.add_model_layout.addWidget(self.add_model_button)
        self.model_layout.addWidget(self.user_models_list)
        self.model_layout.addLayout(self.add_model_layout)
        self.info_label = QLabel(translator.tr("Press 'Check Connection' to load available models."))
        self.info_label.setStyleSheet("color: grey;")
        self.model_layout.addWidget(self.info_label)
        self.model_layout.addWidget(self.remove_model_button)
        self.layout.addWidget(self.model_group)

        self.layout.addStretch()

        # --- Connections ---
        self.api_key_input.textChanged.connect(self._save_api_key)
        self.check_connection_button.clicked.connect(self._check_connection)
        self.add_model_button.clicked.connect(self._add_model)
        self.remove_model_button.clicked.connect(self._remove_model)
        translator.language_changed.connect(self.retranslate_ui)

        # --- Initial Load ---
        self._load_user_models()
        self._load_all_models()

    def retranslate_ui(self):
        self.api_key_group.setTitle(translator.tr("Authentication"))
        self.check_connection_button.setText(translator.tr("Check Connection & Get Balance"))
        # api_key_label is part of addRow now
        self.model_group.setTitle(translator.tr("Model Management"))
        self.add_model_button.setText(translator.tr("Add Model"))
        self.remove_model_button.setText(translator.tr("Remove Selected Model"))
        self.info_label.setText(translator.tr("Press 'Check Connection' to load available models."))

    def _save_api_key(self):
        self.settings.set_openrouter_api_key(self.api_key_input.text())

    def _check_connection(self):
        api_key = self.api_key_input.text()
        if not api_key:
            QMessageBox.warning(self, translator.tr("Error"), translator.tr("API key is missing."))
            return

        self.check_connection_button.setEnabled(False)
        api = OpenRouterAPI(api_key)
        success, result = api.check_connection_and_get_balance()
        self.check_connection_button.setEnabled(True)

        if success:
            self.balance_label.setText(f"{translator.tr('Balance:')} ${result:.4f}")
            QMessageBox.information(self, translator.tr("Success"), translator.tr("Connection successful."))
            self._load_all_models() # Refresh models on successful connection
        else:
            self.balance_label.setText(translator.tr("Balance: -"))
            QMessageBox.critical(self, translator.tr("Error"), f"{translator.tr('Connection failed:')}\n{result}")

    def _load_all_models(self):
        api_key = self.settings.get_openrouter_api_key()
        if not api_key:
            return
        api = OpenRouterAPI(api_key)
        success, models = api.get_models()
        if success:
            self.all_models = sorted([model['id'] for model in models])
            self.available_models_combo.clear()
            self.available_models_combo.addItems(self.all_models)
            self.info_label.setVisible(False) # Hide info label on success

    def _load_user_models(self):
        self.user_models_list.clear()
        self.user_models_list.addItems(self.settings.get_openrouter_models())

    def _add_model(self):
        selected_model = self.available_models_combo.currentText()
        if not selected_model:
            return
        
        current_models = self.settings.get_openrouter_models()
        if selected_model in current_models:
            QMessageBox.warning(self, translator.tr("Info"), translator.tr("Model already in the list."))
            return
            
        current_models.append(selected_model)
        self.settings.set_openrouter_models(sorted(current_models))
        self._load_user_models()

    def _remove_model(self):
        current_item = self.user_models_list.currentItem()
        if not current_item:
            return
        
        model_to_remove = current_item.text()
        current_models = self.settings.get_openrouter_models()
        if model_to_remove in current_models:
            current_models.remove(model_to_remove)
            self.settings.set_openrouter_models(current_models)
            self._load_user_models()
