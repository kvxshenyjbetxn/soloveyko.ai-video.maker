import os
from PyQt6.QtWidgets import QMainWindow, QTabWidget, QWidget
from PyQt6.QtCore import QCoreApplication

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
        self.apply_theme()

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

    def apply_theme(self):
        theme = self.settings_manager.get('theme')
        style_sheet = self.load_style_sheet(theme)
        self.setStyleSheet(style_sheet)

    def load_style_sheet(self, theme_name):
        path = f"assets/styles/{theme_name}.qss"
        if os.path.exists(path):
            with open(path, "r") as f:
                return f.read()
        return ""

    def change_theme(self, theme_name):
        self.settings_manager.set('theme', theme_name)
        self.apply_theme()

    def change_language(self, lang_code):
        self.translator.set_language(lang_code)
        self.retranslate_ui()

    def retranslate_ui(self):
        self.update_title()
        self.tabs.setTabText(0, self.translator.translate('text_processing_tab'))
        self.tabs.setTabText(1, self.translator.translate('settings_tab'))
        
        # Retranslate all tabs
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()

