import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QWidget, QComboBox, QAbstractSpinBox, QAbstractScrollArea, QSlider, QVBoxLayout
from PySide6.QtCore import QCoreApplication, QEvent
from PySide6.QtGui import QWheelEvent
from gui.qt_material import apply_stylesheet

from utils.settings import settings_manager
from utils.translator import translator
from config.version import __version__

from gui.text_tab import TextTab
from gui.settings_tab import SettingsTab
from gui.log_tab import LogTab
from gui.queue_tab import QueueTab
from core.queue_manager import QueueManager
from utils.logger import logger

class MainWindow(QMainWindow):
    def __init__(self, app):
        super().__init__()
        self.app = app
        self.settings_manager = settings_manager
        self.translator = translator
        self.queue_manager = QueueManager()
        self.init_ui()
        logger.log(translator.translate('app_started'))
        self.app.installEventFilter(self)

    def eventFilter(self, obj, event):
        if event.type() == QEvent.Type.Wheel and isinstance(obj, (QComboBox, QAbstractSpinBox, QSlider)):
            
            # Search for a scrollable ancestor.
            parent = obj.parent()
            while parent:
                if isinstance(parent, QAbstractScrollArea):
                    # Found a scroll area. Forward a new event to it to make it scroll.
                    # A new event is created to prevent infinite recursion.
                    parent_pos = parent.mapFromGlobal(event.globalPosition())
                    new_event = QWheelEvent(
                        parent_pos,
                        event.globalPosition(),
                        event.pixelDelta(),
                        event.angleDelta(),
                        event.buttons(),
                        event.modifiers(),
                        event.phase(),
                        event.inverted(),
                        event.source()
                    )
                    QCoreApplication.postEvent(parent, new_event)
                    return True # Swallow the original event.
                parent = parent.parent()

            # No scrollable ancestor found. Just swallow the event to prevent the widget's value from changing.
            return True
        return super().eventFilter(obj, event)

    def init_ui(self):
        self.update_title()
        self.setGeometry(100, 100, 1280, 720)

        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        
        layout = QVBoxLayout(self.central_widget)
        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        self.text_tab = TextTab(main_window=self)
        self.settings_tab = SettingsTab(main_window=self)
        self.log_tab = LogTab()
        self.queue_tab = QueueTab(main_window=self)
        logger.set_log_widget(self.log_tab)

        self.tabs.addTab(self.text_tab, self.translator.translate('text_processing_tab'))
        self.tabs.addTab(self.queue_tab, self.translator.translate('queue_tab'))
        self.tabs.addTab(self.settings_tab, self.translator.translate('settings_tab'))
        self.tabs.addTab(self.log_tab, self.translator.translate('log_tab'))

        # Connect signals
        self.settings_tab.api_tab.openrouter_tab.balance_updated.connect(self.update_balance)
        self.queue_manager.task_added.connect(self.queue_tab.add_task)

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
        self.tabs.setTabText(1, self.translator.translate('queue_tab'))
        self.tabs.setTabText(2, self.translator.translate('settings_tab'))
        self.tabs.setTabText(3, self.translator.translate('log_tab'))
        
        self.text_tab.retranslate_ui()
        self.settings_tab.retranslate_ui()
        self.queue_tab.retranslate_ui()

    def closeEvent(self, event):
        logger.log(translator.translate('app_closing'))
        super().closeEvent(event)

