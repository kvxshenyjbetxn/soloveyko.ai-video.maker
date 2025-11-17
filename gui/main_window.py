import os
from PySide6.QtWidgets import QMainWindow, QTabWidget, QVBoxLayout, QWidget
from PySide6.QtCore import QFile, QTextStream
from gui.text_tab import TextTab
from gui.settings_tab import SettingsTab
from utils.settings import AppSettings
from utils.translator import translator

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()

        self.settings = AppSettings()

        # Build absolute paths for themes
        current_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(current_dir)
        self.theme_paths = {
            "Light": os.path.join(project_root, "assets", "styles", "light.qss"),
            "Dark": os.path.join(project_root, "assets", "styles", "dark.qss"),
            "Black (Amoled)": os.path.join(project_root, "assets", "styles", "black.qss"),
        }

        self.setWindowTitle("Modern PySide6 App")
        self.setGeometry(100, 100, 800, 600)

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

    def load_theme(self, theme_name):
        if theme_name in self.theme_paths:
            path = self.theme_paths[theme_name]
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
