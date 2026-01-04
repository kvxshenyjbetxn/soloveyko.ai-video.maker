
from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QListWidget, QHBoxLayout, QScrollArea, QSpinBox
from PySide6.QtCore import Signal, Qt
from utils.translator import translator
from utils.settings import settings_manager
from api.openrouter import OpenRouterAPI
from utils.logger import logger, LogLevel
from gui.widgets.help_label import HelpLabel

class OpenRouterTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window # Still needed for balance update
        self.settings = settings_manager
        self.api = OpenRouterAPI()
        self.init_ui()
        self.update_fields()
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

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel(translator.translate("openrouter_api_key"))
        self.api_key_input = QLineEdit()
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.api_key_input.textChanged.connect(self.save_api_key)
        api_key_layout.addWidget(self.api_key_label)
        api_key_layout.addWidget(self.api_key_input)
        layout.addLayout(api_key_layout)

        # Connection Status
        connection_layout = QHBoxLayout()
        self.connection_status_label = QLabel(translator.translate("connection_status_not_checked"))
        self.check_connection_button = QPushButton(translator.translate("check_connection"))
        self.check_connection_button.clicked.connect(self.check_connection)
        connection_layout.addWidget(self.connection_status_label)
        connection_layout.addWidget(self.check_connection_button)
        layout.addLayout(connection_layout)

        # Max Threads
        max_threads_layout = QHBoxLayout()
        self.max_threads_help = HelpLabel("max_concurrent_requests")
        self.max_threads_label = QLabel(translator.translate("max_concurrent_requests"))
        
        threads_label_container = QWidget()
        threads_label_layout = QHBoxLayout(threads_label_container)
        threads_label_layout.setContentsMargins(0, 0, 0, 0)
        threads_label_layout.setSpacing(5)
        threads_label_layout.addWidget(self.max_threads_help)
        threads_label_layout.addWidget(self.max_threads_label)
        
        self.max_threads_input = QSpinBox()
        self.max_threads_input.setRange(1, 50)
        self.max_threads_input.setValue(settings_manager.get("openrouter_max_threads", 5))
        self.max_threads_input.valueChanged.connect(self.save_max_threads)
        max_threads_layout.addWidget(threads_label_container)
        max_threads_layout.addWidget(self.max_threads_input)
        layout.addLayout(max_threads_layout)

        # Balance
        self.balance_label = QLabel(translator.translate("balance_loading"))
        layout.addWidget(self.balance_label)

        # Model Management
        self.models_label = QLabel(translator.translate("models"))
        layout.addWidget(self.models_label)
        
        self.models_list = QListWidget()
        layout.addWidget(self.models_list)

        model_management_layout = QHBoxLayout()
        self.add_model_help = HelpLabel("openrouter_add_model_hint")
        self.add_model_input = QLineEdit()
        self.add_model_input.setPlaceholderText(translator.translate("enter_model_name"))
        self.add_model_button = QPushButton(translator.translate("add_model"))
        self.add_model_button.clicked.connect(self.add_model)
        self.remove_model_button = QPushButton(translator.translate("remove_model"))
        self.remove_model_button.clicked.connect(self.remove_model)
        
        model_management_layout.addWidget(self.add_model_help)
        model_management_layout.addWidget(self.add_model_input)
        model_management_layout.addWidget(self.add_model_button)
        model_management_layout.addWidget(self.remove_model_button)
        layout.addLayout(model_management_layout)

        # Info Link
        self.info_layout = QHBoxLayout()
        self.info_label = QLabel()
        self.link_label = QLabel('<a href="https://openrouter.ai/models" style="color: #0078d4;">https://openrouter.ai/models</a>')
        self.link_label.setOpenExternalLinks(True)
        self.info_layout.addWidget(self.info_label)
        self.info_layout.addWidget(self.link_label)
        self.info_layout.addStretch()
        layout.addLayout(self.info_layout)

    def retranslate_ui(self):
        self.api_key_label.setText(translator.translate("openrouter_api_key"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.max_threads_label.setText(translator.translate("max_concurrent_requests"))
        self.max_threads_help.update_tooltip()
        self.update_connection_status_label()
        self.models_label.setText(translator.translate("models"))
        self.add_model_input.setPlaceholderText(translator.translate("enter_model_name"))
        self.add_model_button.setText(translator.translate("add_model"))
        self.remove_model_button.setText(translator.translate("remove_model"))
        self.add_model_help.update_tooltip()
        self.info_label.setText(translator.translate("openrouter_info"))

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("openrouter_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        self.api_key_input.blockSignals(False)
        
        self.max_threads_input.blockSignals(True)
        self.max_threads_input.setValue(self.settings.get("openrouter_max_threads", 5))
        self.max_threads_input.blockSignals(False)

        self.update_models_list()
        
    def save_api_key(self, key):
        self.settings.set("openrouter_api_key", key)
        self.api.api_key = key

    def save_max_threads(self, value):
        self.settings.set("openrouter_max_threads", value)

    def check_connection(self):
        self.update_connection_status_label("checking")
        self.balance_label.setText(translator.translate("balance_loading"))
        
        status = self.api.check_connection()
        self.update_connection_status_label(status)
        
        if status == "connected":
            self.main_window.update_balance()

    def update_connection_status_label(self, status=None):
        if status == "checking":
            self.connection_status_label.setText(translator.translate("connection_status_checking"))
        elif status == "connected":
            self.connection_status_label.setText(translator.translate("connection_status_connected"))
        elif status == "error":
            self.connection_status_label.setText(translator.translate("connection_status_error"))
        elif status == "not_configured":
            self.connection_status_label.setText(translator.translate("connection_status_not_configured"))
        else:
            self.connection_status_label.setText(translator.translate("connection_status_not_checked"))

    def update_balance_label(self, balance_text):
        self.balance_label.setText(balance_text)

    def add_model(self):
        model_name = self.add_model_input.text().strip()
        if model_name:
            models = self.settings.get("openrouter_models", [])
            if model_name not in models:
                models.append(model_name)
                self.settings.set("openrouter_models", models)
                self.update_models_list()
            self.add_model_input.clear()

    def remove_model(self):
        selected_items = self.models_list.selectedItems()
        if not selected_items:
            return
        
        models = self.settings.get("openrouter_models", [])
        for item in selected_items:
            if item.text() in models:
                models.remove(item.text())
        
        self.settings.set("openrouter_models", models)
        self.update_models_list()

    def update_models_list(self):
        self.models_list.clear()
        models = self.settings.get("openrouter_models", [])
        self.models_list.addItems(models)
