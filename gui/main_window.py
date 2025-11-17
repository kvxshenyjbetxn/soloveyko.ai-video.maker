import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QVBoxLayout, QWidget
from PySide6.QtCore import QFile, QTextStream, QTimer
from gui.text_tab import TextTab
from gui.settings_tab import SettingsTab
from utils.translator import translator
from api.openrouter import OpenRouterAPI

class MainWindow(QMainWindow):
    def __init__(self, settings):
        super().__init__()

        self.settings = settings
        self.openrouter_api = None

        # Build absolute paths for themes
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)
        self.theme_paths = {
            "Light": os.path.join(project_root, "assets", "styles", "light.qss"),
            "Dark": os.path.join(project_root, "assets", "styles", "dark.qss"),
            "Black (Amoled)": os.path.join(project_root, "assets", "styles", "black.qss"),
        }

        self.setWindowTitle("Modern PySide6 App")
        self.setGeometry(100, 100, 1280, 720)

        # Create a central widget and layout
        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        self.layout = QVBoxLayout(self.central_widget)

        # Create Tab Widget
        self.tabs = QTabWidget()
        self.layout.addWidget(self.tabs)

        # Create Tabs
        self.text_tab = TextTab()
        self.settings_tab = SettingsTab(main_window=self) # Pass reference to main window

        # Add Tabs
        self.tabs.addTab(self.text_tab, "Text Editor")
        self.tabs.addTab(self.settings_tab, "Settings")

        # Load theme
        self.load_theme(self.settings.get_theme())

        # Connect translator
        translator.language_changed.connect(self.retranslate_ui)
        self.retranslate_ui()

        # Initial balance update
        QTimer.singleShot(100, self.refresh_balance)

    def load_theme(self, theme_name):
        if theme_name in self.theme_paths:
            path = self.theme_paths[theme_name]
            if os.path.exists(path):
                qss_file = QFile(path)
                if qss_file.open(QFile.OpenModeFlag.ReadOnly | QFile.OpenModeFlag.Text):
                    stream = QTextStream(qss_file)
                    self.setStyleSheet(stream.readAll())
                    qss_file.close()
                    self.settings.set_theme(theme_name)

    def retranslate_ui(self):
        self.setWindowTitle(translator.tr("Modern PySide6 App"))
        self.tabs.setTabText(0, translator.tr("Text Editor"))
        self.tabs.setTabText(1, translator.tr("Settings"))
        self.update_balance_labels() # Update labels with translated text

    def refresh_balance(self):
        api_key = self.settings.get_openrouter_api_key()
        if not api_key:
            self.update_balance_labels() # Show default text if no key
            return
        
        if not self.openrouter_api or self.openrouter_api.api_key != api_key:
            self.openrouter_api = OpenRouterAPI(api_key)

        # In a real app, this should be in a separate thread
        success, result = self.openrouter_api.check_connection_and_get_balance()
        
        if success:
            self.update_balance_labels(result)
        else:
            self.update_balance_labels() # Show default on failure

    def update_balance_labels(self, usage=None):
        text = "-"
        if usage is not None:
            text = f"${usage:.4f}"
        
        # Update TextTab
        self.text_tab.balance_label.setText(f"{translator.tr('OpenRouter Usage')}: {text}")
        
        # Update OpenRouterTab in Settings
        # This is a bit fragile, but necessary without a signal/slot system for this
        try:
            self.settings_tab.api_tab.openrouter_tab.balance_label.setText(f"{translator.tr('Balance:')} {text}")
        except AttributeError:
            # This might fail if the tab hasn't been fully constructed yet.
            pass
