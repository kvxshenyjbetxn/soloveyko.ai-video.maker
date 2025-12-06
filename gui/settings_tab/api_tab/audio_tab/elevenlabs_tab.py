from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout
from utils.translator import translator
from utils.settings import settings_manager
from api.elevenlabs import ElevenLabsAPI

class ElevenLabsTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.settings = settings_manager
        self.api = ElevenLabsAPI()
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel("ElevenLabs API Key:")
        self.api_key_input = QLineEdit()
        self.api_key_input.textChanged.connect(self.save_api_key)
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
        self.api_key_label.setText(translator.translate("elevenlabs_api_key"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.update_connection_status_label()
        self.update_balance_label()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("elevenlabs_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        self.api_key_input.blockSignals(False)

    def save_api_key(self, key):
        self.settings.set("elevenlabs_api_key", key)
        self.api.api_key = key

    def check_connection(self):
        self.update_connection_status_label("checking")
        self.balance_label.setText(translator.translate("balance_loading"))
        
        balance, status = self.api.get_balance()
        self.update_connection_status_label(status)
        
        if status == "connected":
            self.main_window.update_balance()
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

