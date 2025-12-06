from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout, QSpinBox
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
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel("Voicemaker API Key:")
        self.api_key_input = QLineEdit()
        self.api_key_input.textChanged.connect(self.save_api_key)
        api_key_layout.addWidget(self.api_key_label)
        api_key_layout.addWidget(self.api_key_input)
        layout.addLayout(api_key_layout)

        # Character Limit
        limit_layout = QHBoxLayout()
        self.limit_label = QLabel(translator.translate("char_limit"))
        self.limit_input = QSpinBox()
        self.limit_input.setRange(100, 100000)
        self.limit_input.setSingleStep(100)
        self.limit_input.setValue(3000) # Default
        self.limit_input.valueChanged.connect(self.save_char_limit)
        limit_layout.addWidget(self.limit_label)
        limit_layout.addWidget(self.limit_input)
        layout.addLayout(limit_layout)

        # Connection Status
        connection_layout = QHBoxLayout()
        self.connection_status_label = QLabel()
        self.check_connection_button = QPushButton()
        self.check_connection_button.clicked.connect(self.check_connection)
        connection_layout.addWidget(self.connection_status_label)
        connection_layout.addWidget(self.check_connection_button)
        layout.addLayout(connection_layout)

        # Balance Label
        balance_layout = QHBoxLayout()
        self.balance_label = QLabel(translator.translate("balance_label"))
        balance_layout.addWidget(self.balance_label)
        balance_layout.addStretch()
        layout.addLayout(balance_layout)

        layout.addStretch()

    def retranslate_ui(self):
        # Використовуємо існуючі ключі перекладу де це можливо, або хардкод для нових елементів поки що
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.limit_label.setText(translator.translate("char_limit"))
        self.update_connection_status_label()
        
        # Preserve current balance text if it has a value, otherwise reset to default
        current_text = self.balance_label.text()
        if ":" in current_text:
             # Just update the prefix "Balance" part if needed, but keeping it simple for now
             pass
        else:
             self.balance_label.setText(translator.translate("balance_label"))


    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.limit_input.blockSignals(True)

        api_key = self.settings.get("voicemaker_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        
        limit = self.settings.get("voicemaker_char_limit", 3000)
        self.limit_input.setValue(int(limit))

        self.api_key_input.blockSignals(False)
        self.limit_input.blockSignals(False)

    def save_api_key(self, key):
        self.settings.set("voicemaker_api_key", key)
        self.api.api_key = key

    def save_char_limit(self, value):
        self.settings.set("voicemaker_char_limit", value)

    def check_connection(self):
        self.update_connection_status_label("checking")
        
        # We can update balance here too if check is successful
        balance, status = self.api.get_balance()
        self.update_connection_status_label(status)
        
        if balance is not None:
             self.update_balance_label(balance)

    def update_balance_label(self, balance):
        if balance is not None:
            self.balance_label.setText(f"{translator.translate('balance_label')} {balance}")
        else:
            self.balance_label.setText(translator.translate("balance_label"))

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