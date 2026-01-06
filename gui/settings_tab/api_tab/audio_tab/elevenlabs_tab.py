from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout, QCheckBox
from utils.translator import translator
from utils.settings import settings_manager
from utils.hint_manager import hint_manager
from api.elevenlabs import ElevenLabsAPI
from gui.widgets.help_label import HelpLabel

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

        # Buy API Key Link
        self.buy_info_layout = QHBoxLayout()
        self.buy_info_label = QLabel()
        self.buy_link_label = QLabel('<a href="https://t.me/elevenLabsVoicerBot" style="color: #0078d4;">@elevenLabsVoicerBot</a>')
        self.buy_link_label.setOpenExternalLinks(True)
        self.buy_info_layout.addWidget(self.buy_info_label)
        self.buy_info_layout.addWidget(self.buy_link_label)
        self.buy_info_layout.addStretch()
        layout.addLayout(self.buy_info_layout)

        # Proxy Settings
        proxy_layout = QVBoxLayout()
        
        proxy_header_layout = QHBoxLayout()
        
        # Hint Label (Left of the setting)
        self.proxy_hint_label = HelpLabel("elevenlabs_proxy_hint")
        
        self.proxy_checkbox = QCheckBox()
        self.proxy_checkbox.toggled.connect(self.toggle_proxy_input)
        self.proxy_checkbox.toggled.connect(self.save_proxy_settings)
        
        proxy_header_layout.addWidget(self.proxy_hint_label)
        proxy_header_layout.addWidget(self.proxy_checkbox)
        proxy_header_layout.addStretch()
        
        self.proxy_input = QLineEdit()
        self.proxy_input.setPlaceholderText("http://user:pass@host:port")
        self.proxy_input.textChanged.connect(self.save_proxy_settings)
        
        # Proxy Recommendation
        self.proxy_recommendation_layout = QHBoxLayout()
        self.proxy_recommendation_label = QLabel()
        self.proxy_recommendation_link = QLabel('<a href="https://stableproxy.com/" style="color: #0078d4;">StableProxy</a>')
        self.proxy_recommendation_link.setOpenExternalLinks(True)
        self.proxy_recommendation_layout.addWidget(self.proxy_recommendation_label)
        self.proxy_recommendation_layout.addWidget(self.proxy_recommendation_link)
        self.proxy_recommendation_layout.addStretch()
        
        # Container for proxy input and recommendation to easily toggle visibility
        self.proxy_details_widget = QWidget()
        proxy_details_layout = QVBoxLayout(self.proxy_details_widget)
        proxy_details_layout.setContentsMargins(0, 0, 0, 0)
        proxy_details_layout.addWidget(self.proxy_input)
        proxy_details_layout.addLayout(self.proxy_recommendation_layout)
        
        proxy_layout.addLayout(proxy_header_layout)
        proxy_layout.addWidget(self.proxy_details_widget)
        layout.addLayout(proxy_layout)

        layout.addStretch()

    def retranslate_ui(self):
        self.api_key_label.setText(translator.translate("elevenlabs_api_key"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.check_connection_button.setText(translator.translate("check_connection"))
        self.buy_info_label.setText(translator.translate("elevenlabs_buy_info"))
        self.proxy_checkbox.setText(translator.translate("enable_proxy", "Enable Proxy"))
        self.proxy_input.setPlaceholderText(translator.translate("proxy_placeholder", "http://user:pass@127.0.0.1:8080"))
        self.proxy_recommendation_label.setText(translator.translate("proxy_recommendation"))
        self.proxy_hint_label.update_tooltip()
        self.update_connection_status_label()
        self.update_balance_label()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("elevenlabs_api_key", "")
        self.api_key_input.setText(api_key)
        self.api.api_key = api_key
        self.api_key_input.blockSignals(False)

        self.proxy_checkbox.blockSignals(True)
        self.proxy_input.blockSignals(True)
        
        proxy_enabled = self.settings.get("elevenlabs_proxy_enabled", False)
        proxy_url = self.settings.get("elevenlabs_proxy_url", "")
        
        self.proxy_checkbox.setChecked(proxy_enabled)
        self.proxy_input.setText(proxy_url)
        self.toggle_proxy_input(proxy_enabled)
        
        self.proxy_checkbox.blockSignals(False)
        self.proxy_input.blockSignals(False)

    def toggle_proxy_input(self, checked):
        self.proxy_details_widget.setVisible(checked)

    def save_proxy_settings(self):
        self.settings.set("elevenlabs_proxy_enabled", self.proxy_checkbox.isChecked())
        self.settings.set("elevenlabs_proxy_url", self.proxy_input.text().strip())

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

