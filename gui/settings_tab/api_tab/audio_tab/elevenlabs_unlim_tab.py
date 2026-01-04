from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout, QMessageBox
from PySide6.QtCore import Qt, QTimer, QUrl
from PySide6.QtGui import QDesktopServices
from utils.settings import settings_manager
from utils.translator import translator
from api.elevenlabs_unlim import ElevenLabsUnlimAPI

class ElevenLabsUnlimTab(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Description / Title
        title = QLabel(translator.translate("elevenlabs_unlim_settings_title", "ElevenLabs Unlimited Settings"))
        title.setStyleSheet("font-weight: bold; font-size: 14px; margin-bottom: 10px;")
        self.layout.addWidget(title)



        # API Key Input
        self.api_key_label = QLabel(translator.translate("api_key_label"))
        self.layout.addWidget(self.api_key_label)
        
        self.api_key_input = QLineEdit()
        self.api_key_input.setEchoMode(QLineEdit.EchoMode.Normal)
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key_placeholder"))
        self.api_key_input.setText(settings_manager.get("elevenlabs_unlim_api_key", ""))
        self.api_key_input.textChanged.connect(self.save_api_key)
        self.layout.addWidget(self.api_key_input)

        # Balance Display
        self.balance_label = QLabel("")
        self.balance_label.setStyleSheet("font-weight: bold; margin-top: 10px;")
        self.layout.addWidget(self.balance_label)

        # Check Balance Button
        self.check_btn = QPushButton(translator.translate("check_balance_button"))
        self.check_btn.clicked.connect(self.check_balance)
        self.layout.addWidget(self.check_btn)
        
        # Buy API Key Link
        self.buy_info_layout = QHBoxLayout()
        self.buy_info_label = QLabel(translator.translate("elevenlabs_unlim_buy_info"))
        self.buy_link_label = QLabel('<a href="https://t.me/Elevenlabs_unlimited_bot" style="color: #0078d4;">@Elevenlabs_unlimited_bot</a>')
        self.buy_link_label.setOpenExternalLinks(True)
        self.buy_info_layout.addWidget(self.buy_info_label)
        self.buy_info_layout.addWidget(self.buy_link_label)
        self.buy_info_layout.addStretch()
        self.layout.addLayout(self.buy_info_layout)

        self.layout.addStretch()

        # Initial balance check if key exists
        if self.api_key_input.text():
             QTimer.singleShot(500, lambda: self.check_balance(silent=True))

    def save_api_key(self):
        key = self.api_key_input.text().strip()
        settings_manager.set("elevenlabs_unlim_api_key", key)
        settings_manager.save_settings()



    def update_balance_label(self, balance):
        if balance is not None:
             self.balance_label.setText(f"{translator.translate('balance_label')} {balance} chars")
        else:
             self.balance_label.setText("")

    def check_balance(self, silent=False):
        api_key = settings_manager.get("elevenlabs_unlim_api_key")
        if not api_key:
             self.balance_label.setText(translator.translate("api_key_missing_error"))
             return
             
        self.check_btn.setEnabled(False)
        self.check_btn.setText(translator.translate("checking_status", "Checking..."))
        
        # Let's do a synchronous call for now for simplicity in settings, but handle UI state.
        try:
             api = ElevenLabsUnlimAPI(api_key=api_key)
             balance, status = api.get_balance()
             self.update_balance_label(balance)
             if status == "connected":
                 if not silent:
                    QMessageBox.information(self, translator.translate("success_title"), translator.translate("connection_successful", "Connection Successful!"))
             else:
                 if not silent:
                    QMessageBox.warning(self, translator.translate("error_title"), translator.translate("connection_failed", "Connection Failed."))
        except Exception as e:
             if not silent:
                QMessageBox.critical(self, translator.translate("error_title"), str(e))
        finally:
             self.check_btn.setEnabled(True)
             self.check_btn.setText(translator.translate("check_balance_button"))
