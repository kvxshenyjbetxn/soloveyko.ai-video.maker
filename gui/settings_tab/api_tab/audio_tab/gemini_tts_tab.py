from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout
from utils.translator import translator
from utils.settings import settings_manager
from api.gemini_tts import GeminiTTSAPI
from gui.widgets.setting_row import add_setting_row

class GeminiTTSTab(QWidget):
    def __init__(self, main_window=None, settings_mgr=None, is_template_mode=False):
        super().__init__()
        self.main_window = main_window
        self.settings = settings_mgr or settings_manager
        self.is_template_mode = is_template_mode
        self.api = GeminiTTSAPI()
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # API Key
        # API Key
        self.api_key_label = QLabel("GeminiTTS API Key:")
        self.api_key_input = QLineEdit()
        self.api_key_input.textChanged.connect(self.save_settings)
        
        def refresh_quick_panel():
            if self.main_window:
                self.main_window.refresh_quick_settings_panels()

        add_setting_row(layout, self.api_key_label, self.api_key_input, "gemini_tts_api_key", refresh_quick_panel, show_star=not self.is_template_mode)

        if not self.is_template_mode:
            # Balance
            balance_layout = QHBoxLayout()
            self.balance_label = QLabel()
            balance_layout.addWidget(self.balance_label)
            balance_layout.addStretch()
            layout.addLayout(balance_layout)

        if not self.is_template_mode:
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
        self.api_key_label.setText(translator.translate("gemini_tts_api_key") if translator.translate("gemini_tts_api_key") != "gemini_tts_api_key" else "GeminiTTS API Key:")
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        if hasattr(self, 'check_connection_button'):
             self.check_connection_button.setText(translator.translate("check_connection"))
        if hasattr(self, 'connection_status_label'): self.update_connection_status_label()
        if hasattr(self, 'balance_label'): self.update_balance_label()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("gemini_tts_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        self.api_key_input.blockSignals(False)

    def save_settings(self, key):
        self.settings.set("gemini_tts_api_key", key)
        self.api.api_key = key

    def check_connection(self):
        self.update_connection_status_label("checking")
        self.balance_label.setText(translator.translate("balance_loading"))
        
        # Ensure API instance has latest settings
        self.api.api_key = self.api_key_input.text()

        balance, status = self.api.get_balance()
        self.update_connection_status_label(status)
        
        if status == "connected":
            if self.main_window and hasattr(self.main_window, 'update_gemini_tts_balance'):
                self.main_window.update_gemini_tts_balance()
            if hasattr(self, 'balance_label'): self.update_balance_label(balance)

    def update_connection_status_label(self, status=None):
        if not hasattr(self, 'connection_status_label'): return
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
        if not hasattr(self, 'balance_label'): return
        if balance is not None:
            self.balance_label.setText(f"{translator.translate('balance')}: {balance}")
        else:
            self.balance_label.setText(translator.translate("balance_not_loaded"))