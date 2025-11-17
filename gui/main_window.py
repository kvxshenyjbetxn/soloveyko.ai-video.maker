import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget
from PySide6.QtCore import QCoreApplication
from gui.qt_material import apply_stylesheet

from utils.settings import settings_manager
from utils.translator import translator
from config.version import __version__

from gui.text_tab import TextTab
from gui.settings_tab import SettingsTab

class MainWindow(QMainWindow):
    def __init__(self, app):
        super().__init__()
        self.app = app
        self.settings_manager = settings_manager
        self.translator = translator
        self.init_ui()

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

        # Connect signals
        self.settings_tab.api_tab.openrouter_tab.balance_updated.connect(self.update_balance)

        self.update_balance()

    def update_title(self):
        app_name = self.translator.translate('app_title')
        self.setWindowTitle(f"{app_name} v{__version__}")

    def update_balance(self):
        self.text_tab.update_balance()

    def change_theme(self, theme_name):
        self.settings_manager.set('theme', theme_name)
        
        if theme_name == 'light':
            apply_stylesheet(self.app, theme='light_blue.xml')
        elif theme_name == 'dark':
            apply_stylesheet(self.app, theme='dark_teal.xml')
        elif theme_name == 'black':
            apply_stylesheet(self.app, theme='amoled_black.xml')

    def change_language(self, lang_code):
        self.translator.set_language(lang_code)
        self.retranslate_ui()

    def retranslate_ui(self):
        self.update_title()
        self.tabs.setTabText(0, self.translator.translate('text_processing_tab'))
        self.tabs.setTabText(1, self.translator.translate('settings_tab'))
        
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()

