from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QFormLayout, QLineEdit, QLabel, 
    QCheckBox, QPushButton, QHBoxLayout, QMessageBox, QApplication
)
from PySide6.QtCore import Qt, QUrl
from PySide6.QtGui import QDesktopServices, QClipboard
from utils.translator import translator
from utils.settings import settings_manager
from core.notification_manager import notification_manager

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
        self.user_id_label = QLabel(translator.translate('telegram_user_id_label', 'Telegram ID користувача'))
        form_layout.addRow(self.user_id_label, self.user_id_input)

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

    def retranslate_ui(self):
        self.enable_label.setText(translator.translate('enable_notifications_label', 'Увімкнути сповіщення'))
        self.user_id_label.setText(translator.translate('telegram_user_id_label', 'Telegram ID користувача'))
        self.bot_info_label.setText(translator.translate('bot_link_info', 'Посилання на бота:'))
        self.test_button.setText(translator.translate('test_notification_button', 'Надіслати тестове повідомлення'))
        self.copy_button.setText(translator.translate('copy_button', 'Копіювати'))
