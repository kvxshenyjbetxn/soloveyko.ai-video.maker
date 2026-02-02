from PySide6.QtWidgets import QDialog, QVBoxLayout, QLabel, QCheckBox, QPushButton, QHBoxLayout, QWidget, QFrame, QComboBox, QScrollArea
from PySide6.QtCore import Qt, QSize
from PySide6.QtGui import QIcon
from utils.translator import translator
from utils.settings import settings_manager

class WelcomeDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate('welcome_title'))
        self.setModal(True)
        self.setMinimumWidth(750)
        self.setFixedHeight(650)  # Set a fixed height as requested
        
        # Remove help button from title bar
        self.setWindowFlags(self.windowFlags() & ~Qt.WindowContextHelpButtonHint)
        
        self.init_ui()
        
    def init_ui(self):
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setSpacing(10)
        self.main_layout.setContentsMargins(20, 20, 20, 20)
        
        # Language selection header (stays fixed at top)
        lang_layout = QHBoxLayout()
        lang_label = QLabel("Мова / Language / Язык:")
        lang_label.setStyleSheet("font-weight: bold; font-size: 13px;")
        
        self.lang_combo = QComboBox()
        self.lang_combo.addItem("Українська", "uk")
        self.lang_combo.addItem("English", "en")
        self.lang_combo.addItem("Русский", "ru")
        
        current_lang = settings_manager.get('language', 'uk')
        index = self.lang_combo.findData(current_lang)
        if index >= 0:
            self.lang_combo.setCurrentIndex(index)
            
        self.lang_combo.currentIndexChanged.connect(self.on_language_changed)
        
        lang_layout.addStretch()
        lang_layout.addWidget(lang_label)
        lang_layout.addWidget(self.lang_combo)
        self.main_layout.addLayout(lang_layout)
        
        # Create Scroll Area
        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")
        
        # Scroll area content container
        self.scroll_content = QWidget()
        self.scroll_layout = QVBoxLayout(self.scroll_content)
        self.scroll_layout.setSpacing(15)
        self.scroll_layout.setContentsMargins(10, 0, 10, 0)
        
        # Welcome Title/Header
        self.title_label = QLabel(translator.translate('welcome_title'))
        self.title_label.setStyleSheet("font-size: 24px; font-weight: bold; margin-bottom: 5px;")
        self.title_label.setAlignment(Qt.AlignCenter)
        self.scroll_layout.addWidget(self.title_label)
        
        # Assistant Notice Card
        self.assistant_notice = QLabel(translator.translate('welcome_assistant_notice'))
        self.assistant_notice.setWordWrap(True)
        self.assistant_notice.setOpenExternalLinks(True)
        self.assistant_notice.setTextFormat(Qt.RichText)
        self.assistant_notice.setStyleSheet("""
            QLabel {
                background-color: qlineargradient(spread:pad, x1:0, y1:0, x2:1, y2:1, stop:0 rgba(0, 123, 255, 30), stop:1 rgba(0, 123, 255, 10));
                border: 1px solid rgba(0, 123, 255, 50);
                border-radius: 8px;
                padding: 15px;
                font-size: 14px;
                line-height: 1.5;
                color: palette(text);
                margin-top: 10px;
                margin-bottom: 15px;
            }
        """)
        self.scroll_layout.addWidget(self.assistant_notice)

        # Content Label
        self.content_label = QLabel(translator.translate('welcome_message_body'))
        self.content_label.setWordWrap(True)
        self.content_label.setOpenExternalLinks(True)
        self.content_label.setStyleSheet("font-size: 14px; line-height: 1.6;")
        self.content_label.setTextFormat(Qt.RichText)
        self.scroll_layout.addWidget(self.content_label)
        
        self.scroll_area.setWidget(self.scroll_content)
        self.main_layout.addWidget(self.scroll_area)
        
        # Separator (stays fixed)
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setFrameShadow(QFrame.Sunken)
        self.main_layout.addWidget(line)
        
        # Bottom controls (stay fixed)
        bottom_layout = QHBoxLayout()
        
        self.cb_dont_show = QCheckBox(translator.translate('do_not_show_again'))
        self.cb_dont_show.setChecked(False)
        self.cb_dont_show.setStyleSheet("font-size: 13px;")
        bottom_layout.addWidget(self.cb_dont_show)
        
        bottom_layout.addStretch()
        
        self.btn_close = QPushButton(translator.translate('start_working'))
        self.btn_close.setCursor(Qt.PointingHandCursor)
        self.btn_close.setMinimumHeight(38)
        self.btn_close.setMinimumWidth(150)
        self.btn_close.setStyleSheet("""
            QPushButton {
                background-color: #007bff;
                color: white;
                border-radius: 6px;
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
        self.assistant_notice.setText(translator.translate('welcome_assistant_notice'))
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
