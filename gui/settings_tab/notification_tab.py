from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QFormLayout, QLineEdit, QLabel, 
    QCheckBox, QPushButton, QHBoxLayout, QMessageBox, QApplication
)
from PySide6.QtCore import Qt, QUrl
from PySide6.QtGui import QDesktopServices, QClipboard
from utils.translator import translator
from utils.settings import settings_manager
from core.notification_manager import notification_manager
from gui.api_workers import ApiKeyCheckWorker

class NotificationTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.update_fields()

    def init_ui(self):
        layout = QVBoxLayout(self)
        
        form_layout = QFormLayout()

        # Enable Notifications Checkbox
        self.enable_checkbox = QCheckBox()
        self.enable_checkbox.stateChanged.connect(self.on_enable_changed)
        # Using a default key for translation, but falling back to Ukrainian as requested
        self.enable_label = QLabel(translator.translate('enable_notifications_label', 'Увімкнути сповіщення'))
        form_layout.addRow(self.enable_label, self.enable_checkbox)

        # Telegram User ID Field
        self.user_id_input = QLineEdit()
        self.user_id_input.setPlaceholderText("123456789")
        self.user_id_input.textChanged.connect(self.on_user_id_changed)
        
        self.get_id_button = QPushButton(translator.translate('get_id_button', 'Отримати ID'))
        self.get_id_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.get_id_button.clicked.connect(self.on_get_id_clicked)
        
        user_id_layout = QHBoxLayout()
        user_id_layout.addWidget(self.user_id_input)
        user_id_layout.addWidget(self.get_id_button)
        
        self.user_id_label = QLabel(translator.translate('telegram_user_id_label', 'Telegram ID користувача'))
        form_layout.addRow(self.user_id_label, user_id_layout)

        # Bot Link Display
        self.bot_link_container = QWidget()
        link_layout = QHBoxLayout(self.bot_link_container)
        link_layout.setContentsMargins(0, 0, 0, 0)
        
        self.bot_link_url = notification_manager.get_bot_url()
        self.bot_link_label = QLabel(f"<a href='{self.bot_link_url}'>{self.bot_link_url}</a>")
        self.bot_link_label.setOpenExternalLinks(True)
        self.bot_link_label.setTextInteractionFlags(Qt.TextInteractionFlag.TextBrowserInteraction)
        
        self.copy_button = QPushButton(translator.translate('copy_button', 'Копіювати'))
        self.copy_button.setCursor(Qt.CursorShape.PointingHandCursor)
        # self.copy_button.setFixedSize(80, 25) # Removed to allow dynamic width
        self.copy_button.clicked.connect(self.copy_bot_link)

        link_layout.addWidget(self.bot_link_label)
        link_layout.addWidget(self.copy_button)
        link_layout.addStretch()

        self.bot_info_label = QLabel(translator.translate('bot_link_info', 'Посилання на бота:'))
        form_layout.addRow(self.bot_info_label, self.bot_link_container)

        layout.addLayout(form_layout)

        # Test Notification Button
        self.test_button = QPushButton(translator.translate('test_notification_button', 'Надіслати тестове повідомлення'))
        self.test_button.clicked.connect(self.send_test_notification)
        layout.addWidget(self.test_button)
        
        layout.addStretch()
        self.setLayout(layout)

    def update_fields(self):
        # Block signals
        self.enable_checkbox.blockSignals(True)
        self.user_id_input.blockSignals(True)

        is_enabled = settings_manager.get('notifications_enabled', False)
        self.enable_checkbox.setChecked(is_enabled)
        
        user_id = settings_manager.get('telegram_user_id', '')
        self.user_id_input.setText(user_id)

        # Unblock signals
        self.enable_checkbox.blockSignals(False)
        self.user_id_input.blockSignals(False)

    def on_enable_changed(self, state):
        is_checked = state == Qt.CheckState.Checked.value
        settings_manager.set('notifications_enabled', is_checked)

    def on_user_id_changed(self, text):
        settings_manager.set('telegram_user_id', text)

    def copy_bot_link(self):
        clipboard = QApplication.clipboard()
        clipboard.setText(self.bot_link_url)
        # Optional: Show a small tooltip or status message?
        
    def send_test_notification(self):
        notification_manager.send_test_notification()
        QMessageBox.information(self, "Info", "Test message sent (check log for status)")

    def on_get_id_clicked(self):
        if not self.main_window:
            return
            
        api_key = self.main_window.api_key
        server_url = self.main_window.server_url
        
        if not api_key or not server_url:
            QMessageBox.warning(self, "Error", translator.translate('api_key_error', "API key or server URL is missing."))
            return
            
        self.get_id_button.setEnabled(False)
        self.get_id_button.setText(translator.translate('checking_status', "Checking..."))
        
        worker = ApiKeyCheckWorker(api_key, server_url)
        # Using a lambda to handle the signal with extra arguments if needed, 
        # but the signal matches: bool, str, int, object
        worker.signals.finished.connect(self.on_id_fetched)
        self.main_window.threadpool.start(worker)
        
    def on_id_fetched(self, is_valid, expires_at, subscription_level, telegram_id):
        self.get_id_button.setEnabled(True)
        self.get_id_button.setText(translator.translate('get_id_button', 'Отримати ID'))
        
        if is_valid and telegram_id:
            self.user_id_input.setText(str(telegram_id))
            QMessageBox.information(self, "Success", translator.translate('id_fetched_success', "Telegram ID fetched successfully."))
        elif is_valid and not telegram_id:
             QMessageBox.warning(self, "Warning", translator.translate('id_not_found', "ID not found on server. Please ensure you have started the bot."))
        else:
             QMessageBox.warning(self, "Error", translator.translate('api_validation_failed', "API validation failed."))

    def retranslate_ui(self):
        self.enable_label.setText(translator.translate('enable_notifications_label', 'Увімкнути сповіщення'))
        self.user_id_label.setText(translator.translate('telegram_user_id_label', 'Telegram ID користувача'))
        self.bot_info_label.setText(translator.translate('bot_link_info', 'Посилання на бота:'))
        self.test_button.setText(translator.translate('test_notification_button', 'Надіслати тестове повідомлення'))

        self.copy_button.setText(translator.translate('copy_button', 'Копіювати'))
        self.get_id_button.setText(translator.translate('get_id_button', 'Отримати ID'))
