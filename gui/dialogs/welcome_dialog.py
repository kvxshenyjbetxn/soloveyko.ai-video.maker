from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QCheckBox, QPushButton, QHBoxLayout, QWidget, QFrame, QComboBox
from PySide6.QtCore import Qt, QSize
from PySide6.QtGui import QIcon
from utils.translator import translator
from utils.settings import settings_manager

class WelcomeDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate('welcome_title'))
        self.setModal(True)
        self.setMinimumWidth(700)
        
        # Remove help button from title bar
        self.setWindowFlags(self.windowFlags() & ~Qt.WindowContextHelpButtonHint)
        
        self.init_ui()
        
    def init_ui(self):
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setSpacing(15)
        self.main_layout.setContentsMargins(30, 20, 30, 30)
        
        # Language selection header
        lang_layout = QHBoxLayout()
        lang_label = QLabel("Мова / Language / Язык:")
        lang_label.setStyleSheet("font-weight: bold; font-size: 13px;")
        
        self.lang_combo = QComboBox()
        self.lang_combo.addItem("Українська", "uk")
        self.lang_combo.addItem("English", "en")
        self.lang_combo.addItem("Русский", "ru")
        
        # Set current language
        current_lang = settings_manager.get('language', 'uk')
        index = self.lang_combo.findData(current_lang)
        if index >= 0:
            self.lang_combo.setCurrentIndex(index)
            
        self.lang_combo.currentIndexChanged.connect(self.on_language_changed)
        
        lang_layout.addStretch()
        lang_layout.addWidget(lang_label)
        lang_layout.addWidget(self.lang_combo)
        self.main_layout.addLayout(lang_layout)
        
        # Welcome Title/Header
        self.title_label = QLabel(translator.translate('welcome_title'))
        self.title_label.setStyleSheet("font-size: 24px; font-weight: bold; margin-bottom: 5px;")
        self.title_label.setAlignment(Qt.AlignCenter)
        self.main_layout.addWidget(self.title_label)
        
        # Content
        self.content_label = QLabel(translator.translate('welcome_message_body'))
        self.content_label.setWordWrap(True)
        self.content_label.setOpenExternalLinks(True)
        self.content_label.setStyleSheet("font-size: 14px; line-height: 1.4;")
        self.content_label.setTextFormat(Qt.RichText)
        self.main_layout.addWidget(self.content_label)
        
        # Separator
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setFrameShadow(QFrame.Sunken)
        self.main_layout.addWidget(line)
        
        # Bottom controls
        bottom_layout = QHBoxLayout()
        
        self.cb_dont_show = QCheckBox(translator.translate('do_not_show_again'))
        self.cb_dont_show.setChecked(False)
        self.cb_dont_show.setStyleSheet("font-size: 13px;")
        bottom_layout.addWidget(self.cb_dont_show)
        
        bottom_layout.addStretch()
        
        self.btn_close = QPushButton(translator.translate('start_working'))
        self.btn_close.setCursor(Qt.PointingHandCursor)
        self.btn_close.setMinimumHeight(36)
        self.btn_close.setMinimumWidth(140)
        self.btn_close.setStyleSheet("""
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
        """)
        self.btn_close.clicked.connect(self.on_close)
        bottom_layout.addWidget(self.btn_close)
        
        self.main_layout.addLayout(bottom_layout)
        
    def on_language_changed(self, index):
        lang_code = self.lang_combo.itemData(index)
        if lang_code:
            translator.set_language(lang_code)
            self.retranslate_ui()
            
    def retranslate_ui(self):
        self.setWindowTitle(translator.translate('welcome_title'))
        self.title_label.setText(translator.translate('welcome_title'))
        self.content_label.setText(translator.translate('welcome_message_body'))
        self.cb_dont_show.setText(translator.translate('do_not_show_again'))
        self.btn_close.setText(translator.translate('start_working'))
        
        # Also notify parent if it exists and has refresh method
        if self.parent() and hasattr(self.parent(), 'refresh_ui_from_settings'):
            self.parent().refresh_ui_from_settings()

    def on_close(self):
        if self.cb_dont_show.isChecked():
            settings_manager.set('show_welcome_dialog', False)
            settings_manager.save_settings()
        self.accept()
