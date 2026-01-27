from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QHBoxLayout, QMessageBox
from PySide6.QtCore import Qt, QTimer, QUrl, QThread, Signal
from PySide6.QtGui import QDesktopServices
from utils.settings import settings_manager
from utils.translator import translator
from api.elevenlabs_unlim import ElevenLabsUnlimAPI
from gui.widgets.setting_row import add_setting_row

class BalanceCheckWorker(QThread):
    finished = Signal(int, str) # balance, status
    error = Signal(str)

    def __init__(self, api_key):
        super().__init__()
        self.api_key = api_key

    def run(self):
        try:
            api = ElevenLabsUnlimAPI(api_key=self.api_key)
            balance, status = api.get_balance()
            self.finished.emit(balance, status)
        except Exception as e:
            self.error.emit(str(e))

class ElevenLabsUnlimTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.layout = QVBoxLayout(self)
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        # API Key Input
        self.api_key_label = QLabel(translator.translate("api_key_label"))
        
        self.api_key_input = QLineEdit()
        self.api_key_input.setEchoMode(QLineEdit.EchoMode.Normal)
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key_placeholder"))
        self.api_key_input.textChanged.connect(self.save_api_key)

        def refresh_quick_panel():
            if self.main_window:
                self.main_window.refresh_quick_settings_panels()

        add_setting_row(self.layout, self.api_key_label, self.api_key_input, "elevenlabs_unlim_api_key", refresh_quick_panel)

        # Balance Display
        balance_layout = QHBoxLayout()
        self.balance_label = QLabel("")
        balance_layout.addWidget(self.balance_label)
        balance_layout.addStretch()
        self.layout.addLayout(balance_layout)

        # Connection Status & Check Button
        connection_layout = QHBoxLayout()
        self.connection_status_label = QLabel()
        self.check_btn = QPushButton(translator.translate("check_balance_button"))
        self.check_btn.clicked.connect(self.check_balance)
        connection_layout.addWidget(self.connection_status_label)
        connection_layout.addWidget(self.check_btn)
        self.layout.addLayout(connection_layout)
        
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

    def retranslate_ui(self):
        self.api_key_label.setText(translator.translate("api_key_label"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key_placeholder"))
        self.check_btn.setText(translator.translate("check_balance_button"))
        self.buy_info_label.setText(translator.translate("elevenlabs_unlim_buy_info"))
        self.update_connection_status_label()
        # Initial balance label update is handled by update_fields/check_balance

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.api_key_input.setText(settings_manager.get("elevenlabs_unlim_api_key", ""))
        self.api_key_input.blockSignals(False)
        self.update_balance_label(None)
        self.update_connection_status_label(None)

    def save_api_key(self):
        key = self.api_key_input.text().strip()
        settings_manager.set("elevenlabs_unlim_api_key", key)
        settings_manager.save_settings()

    def update_balance_label(self, balance):
        if balance is not None:
             self.balance_label.setText(f"{translator.translate('balance')}: {balance}")
        else:
             self.balance_label.setText(translator.translate("balance_not_loaded"))

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

    def check_balance(self, silent=False):
        api_key = settings_manager.get("elevenlabs_unlim_api_key")
        if not api_key:
             self.balance_label.setText(translator.translate("api_key_missing_error"))
             return
             
        self.check_btn.setEnabled(False)
        self.check_btn.setText(translator.translate("checking_status", "Checking..."))
        self.update_connection_status_label("checking")
        
        self.worker = BalanceCheckWorker(api_key)
        self.worker.finished.connect(lambda b, s: self._on_check_finished(b, s, silent))
        self.worker.error.connect(lambda e: self._on_check_error(e, silent))
        self.worker.finished.connect(self.worker.deleteLater)
        self.worker.error.connect(self.worker.deleteLater)
        self.worker.start()

    def _on_check_finished(self, balance, status, silent):
        self.check_btn.setEnabled(True)
        self.check_btn.setText(translator.translate("check_balance_button"))
        self.update_balance_label(balance)
        self.update_connection_status_label(status)
        
        if status == "connected":
             if self.main_window:
                 self.main_window.update_balance()
             if not silent:
                QMessageBox.information(self, translator.translate("success_title"), translator.translate("connection_successful", "Connection Successful!"))
        else:
             if not silent:
                QMessageBox.warning(self, translator.translate("error_title"), translator.translate("connection_failed", "Connection Failed."))

    def _on_check_error(self, error_msg, silent):
        self.check_btn.setEnabled(True)
        self.check_btn.setText(translator.translate("check_balance_button"))
        self.update_connection_status_label("error")
        if not silent:
            QMessageBox.critical(self, translator.translate("error_title"), error_msg)
