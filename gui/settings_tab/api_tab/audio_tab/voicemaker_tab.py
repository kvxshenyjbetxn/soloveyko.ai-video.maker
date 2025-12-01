from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout
from utils.translator import translator
from utils.settings import settings_manager
from api.voicemaker import VoicemakerAPI

class VoicemakerTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.settings = settings_manager
        self.api = VoicemakerAPI()
        self.init_ui()
        self.load_settings()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel("Voicemaker API Key:")
        self.api_key_input = QLineEdit()
        self.api_key_input.setEchoMode(QLineEdit.Password)
        self.api_key_input.textChanged.connect(self.save_api_key)
        api_key_layout.addWidget(self.api_key_label)
        api_key_layout.addWidget(self.api_key_input)
        layout.addLayout(api_key_layout)

        # Balance Label
        balance_layout = QHBoxLayout()
        self.balance_title_label = QLabel("Characters remaining:") 
        self.balance_value_label = QLabel("Unknown")
        balance_layout.addWidget(self.balance_title_label)
        balance_layout.addWidget(self.balance_value_label)
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
        # Використовуємо існуючі ключі перекладу де це можливо, або хардкод для нових елементів поки що
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.balance_title_label.setText(translator.translate("balance_label"))
        self.update_connection_status_label()

    def load_settings(self):
        api_key = self.settings.get("voicemaker_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key

    def save_api_key(self, key):
        self.settings.set("voicemaker_api_key", key)
        self.api.api_key = key

    def check_connection(self):
        self.update_connection_status_label("checking")
        
        status = self.api.check_connection()
        self.update_connection_status_label(status)

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

    def update_balance_label(self, balance):
        if balance is not None:
            self.balance_value_label.setText(str(balance))
        else:
            self.balance_value_label.setText("Unknown")