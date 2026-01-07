from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QCheckBox, QPushButton, QHBoxLayout, QWidget, QFrame
from PySide6.QtCore import Qt, QSize
from PySide6.QtGui import QIcon
from utils.translator import translator
from utils.settings import settings_manager

class WelcomeDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate('welcome_title'))
        self.setModal(True)
        self.setMinimumWidth(500)
        
        # Remove help button from title bar
        self.setWindowFlags(self.windowFlags() & ~Qt.WindowContextHelpButtonHint)
        
        self.init_ui()
        
    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(20)
        layout.setContentsMargins(30, 30, 30, 30)
        
        # Welcome Title/Header
        title_label = QLabel(translator.translate('welcome_title'))
        title_label.setStyleSheet("font-size: 24px; font-weight: bold; margin-bottom: 10px;")
        title_label.setAlignment(Qt.AlignCenter)
        layout.addWidget(title_label)
        
        # Content
        content_label = QLabel(translator.translate('welcome_message_body'))
        content_label.setWordWrap(True)
        content_label.setOpenExternalLinks(True)
        content_label.setStyleSheet("font-size: 14px; line-height: 1.4;")
        content_label.setTextFormat(Qt.RichText)
        layout.addWidget(content_label)
        
        # Separator
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setFrameShadow(QFrame.Sunken)
        layout.addWidget(line)
        
        # Bottom controls
        bottom_layout = QHBoxLayout()
        
        self.cb_dont_show = QCheckBox(translator.translate('do_not_show_again'))
        self.cb_dont_show.setChecked(False) # Default unchecked, let user decide
        self.cb_dont_show.setStyleSheet("font-size: 13px;")
        bottom_layout.addWidget(self.cb_dont_show)
        
        bottom_layout.addStretch()
        
        btn_close = QPushButton(translator.translate('start_working'))
        btn_close.setCursor(Qt.PointingHandCursor)
        btn_close.setMinimumHeight(36)
        btn_close.setMinimumWidth(120)
        # Style the button to look "primary"
        btn_close.setStyleSheet("""
            QPushButton {
                background-color: #007bff;
                color: white;
                border-radius: 4px;
                font-weight: bold;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #0069d9;
            }
            QPushButton:pressed {
                background-color: #0056b3;
            }
        """)
        btn_close.clicked.connect(self.on_close)
        bottom_layout.addWidget(btn_close)
        
        layout.addLayout(bottom_layout)
        
    def on_close(self):
        if self.cb_dont_show.isChecked():
            settings_manager.set('show_welcome_dialog', False)
            settings_manager.save_settings()
        self.accept()
