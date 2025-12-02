from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout
from utils.translator import translator
from utils.settings import settings_manager
from api.gemini_tts import GeminiTTSAPI

class GeminiTTSTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.settings = settings_manager
        self.api = GeminiTTSAPI()
        self.init_ui()
        self.load_settings()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # API URL
        url_layout = QHBoxLayout()
        self.url_label = QLabel("GeminiTTS URL:")
        self.url_input = QLineEdit()
        self.url_input.textChanged.connect(self.save_settings)
        url_layout.addWidget(self.url_label)
        url_layout.addWidget(self.url_input)
        layout.addLayout(url_layout)

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel("GeminiTTS API Key:")
        self.api_key_input = QLineEdit()
        self.api_key_input.textChanged.connect(self.save_settings)
        api_key_layout.addWidget(self.api_key_label)
        api_key_layout.addWidget(self.api_key_input)
        layout.addLayout(api_key_layout)

        # Balance
        balance_layout = QHBoxLayout()
        self.balance_label = QLabel()
        balance_layout.addWidget(self.balance_label)
        balance_layout.addStretch()
        layout.addLayout(balance_layout)

        # Connection Status
        connection_layout = QHBoxLayout()
        self.connection_status_label = QLabel()
        self.check_connection_button = QPushButton()
        self.check_connection_button.clicked.connect(self.check_connection)
        connection_layout.addWidget(self.connection_status_label)
        connection_layout.addWidget(self.check_connection_button)
        layout.addLayout(connection_layout)

        layout.addStretch()

    def retranslate_ui(self):
        self.url_label.setText(translator.translate("gemini_tts_url") if translator.translate("gemini_tts_url") != "gemini_tts_url" else "GeminiTTS URL:")
        self.api_key_label.setText(translator.translate("gemini_tts_api_key") if translator.translate("gemini_tts_api_key") != "gemini_tts_api_key" else "GeminiTTS API Key:")
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.update_connection_status_label()
        self.update_balance_label()

    def load_settings(self):
        url = self.settings.get("gemini_tts_url", "http://127.0.0.1:8000")
        api_key = self.settings.get("gemini_tts_api_key", "")
        self.url_input.setText(url)
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        self.api.base_url = url

    def save_settings(self):
        url = self.url_input.text()
        key = self.api_key_input.text()
        self.settings.set("gemini_tts_url", url)
        self.settings.set("gemini_tts_api_key", key)
        self.api.base_url = url
        self.api.api_key = key

    def check_connection(self):
        self.update_connection_status_label("checking")
        self.balance_label.setText(translator.translate("balance_loading"))
        
        # Ensure API instance has latest settings
        self.api.base_url = self.url_input.text()
        self.api.api_key = self.api_key_input.text()

        balance, status = self.api.get_balance()
        self.update_connection_status_label(status)
        
        if status == "connected":
            if self.main_window and hasattr(self.main_window, 'update_gemini_tts_balance'):
                self.main_window.update_gemini_tts_balance()
            self.update_balance_label(balance)

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

    def update_balance_label(self, balance=None):
        if balance is not None:
            self.balance_label.setText(f"{translator.translate('balance')}: {balance}")
        else:
            self.balance_label.setText(translator.translate("balance_not_loaded"))
