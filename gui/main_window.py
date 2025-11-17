import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QApplication
from PySide6.QtGui import QPalette, QColor
from PySide6.QtCore import QCoreApplication

from utils.settings import settings_manager
from utils.translator import translator
from config.version import __version__

from gui.text_tab import TextTab
from gui.settings_tab import SettingsTab

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.settings_manager = settings_manager
        self.translator = translator
        self.init_ui()
        self.apply_theme(self.settings_manager.get('theme', 'light'))

    def init_ui(self):
        self.update_title()
        self.setGeometry(100, 100, 1280, 720)

        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        
        self.tabs = QTabWidget(self.central_widget)
        self.tabs.setGeometry(0, 0, 1280, 720)

        self.text_tab = TextTab()
        self.settings_tab = SettingsTab(main_window=self)

        self.tabs.addTab(self.text_tab, self.translator.translate('text_processing_tab'))
        self.tabs.addTab(self.settings_tab, self.translator.translate('settings_tab'))

    def update_title(self):
        app_name = self.translator.translate('app_title')
        self.setWindowTitle(f"{app_name} v{__version__}")

    def apply_theme(self, theme_name):
        app = QApplication.instance()
        
        # Start with a fresh palette
        palette = QPalette()

        if theme_name == 'dark':
            palette.setColor(QPalette.ColorRole.Window, QColor(53, 53, 53))
            palette.setColor(QPalette.ColorRole.WindowText, QColor(255, 255, 255))
            palette.setColor(QPalette.ColorRole.Base, QColor(25, 25, 25))
            palette.setColor(QPalette.ColorRole.AlternateBase, QColor(53, 53, 53))
            palette.setColor(QPalette.ColorRole.ToolTipBase, QColor(255, 255, 220))
            palette.setColor(QPalette.ColorRole.ToolTipText, QColor(0, 0, 0))
            palette.setColor(QPalette.ColorRole.Text, QColor(255, 255, 255))
            palette.setColor(QPalette.ColorRole.Button, QColor(53, 53, 53))
            palette.setColor(QPalette.ColorRole.ButtonText, QColor(255, 255, 255))
            palette.setColor(QPalette.ColorRole.BrightText, QColor(255, 0, 0))
            palette.setColor(QPalette.ColorRole.Link, QColor(42, 130, 218))
            palette.setColor(QPalette.ColorRole.Highlight, QColor(42, 130, 218))
            palette.setColor(QPalette.ColorRole.HighlightedText, QColor(0, 0, 0))
        elif theme_name == 'black':
            palette.setColor(QPalette.ColorRole.Window, QColor(0, 0, 0))
            palette.setColor(QPalette.ColorRole.WindowText, QColor(225, 225, 225))
            palette.setColor(QPalette.ColorRole.Base, QColor(18, 18, 18))
            palette.setColor(QPalette.ColorRole.AlternateBase, QColor(28, 28, 28))
            palette.setColor(QPalette.ColorRole.ToolTipBase, QColor(255, 255, 220))
            palette.setColor(QPalette.ColorRole.ToolTipText, QColor(0, 0, 0))
            palette.setColor(QPalette.ColorRole.Text, QColor(225, 225, 225))
            palette.setColor(QPalette.ColorRole.Button, QColor(28, 28, 28))
            palette.setColor(QPalette.ColorRole.ButtonText, QColor(225, 225, 225))
            palette.setColor(QPalette.ColorRole.BrightText, QColor(255, 0, 0))
            palette.setColor(QPalette.ColorRole.Link, QColor(42, 130, 218))
            palette.setColor(QPalette.ColorRole.Highlight, QColor(42, 130, 218))
            palette.setColor(QPalette.ColorRole.HighlightedText, QColor(255, 255, 255))
        else: # 'light' theme
            # Use the default system palette for the light theme
            app.setPalette(app.style().standardPalette())
            return

        app.setPalette(palette)

    def change_theme(self, theme_name):
        self.settings_manager.set('theme', theme_name)
        self.apply_theme(theme_name)

    def change_language(self, lang_code):
        self.translator.set_language(lang_code)
        self.retranslate_ui()

    def retranslate_ui(self):
        self.update_title()
        self.tabs.setTabText(0, self.translator.translate('text_processing_tab'))
        self.tabs.setTabText(1, self.translator.translate('settings_tab'))
        
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()

