from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator
from gui.openrouter_tab import OpenRouterTab

class ApiTab(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window

        # Layout
        self.layout = QVBoxLayout(self)
        self.tabs = QTabWidget()

        # --- OpenRouter Tab ---
        self.openrouter_tab = OpenRouterTab(self.main_window)
        self.tabs.addTab(self.openrouter_tab, "OpenRouter")

        self.layout.addWidget(self.tabs)

        # Connections
        translator.language_changed.connect(self.retranslate_ui)

    def retranslate_ui(self):
        self.tabs.setTabText(0, "OpenRouter")
