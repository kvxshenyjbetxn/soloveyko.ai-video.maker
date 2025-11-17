from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator
from gui.general_tab import GeneralTab

class SettingsTab(QWidget):
    def __init__(self, main_window):
        super().__init__()
        self.main_window = main_window

        # Layout
        self.layout = QVBoxLayout(self)
        self.tabs = QTabWidget()

        # --- General Tab ---
        self.general_tab = GeneralTab(self.main_window)
        self.tabs.addTab(self.general_tab, "")

        self.layout.addWidget(self.tabs)

        # --- Connections ---
        translator.language_changed.connect(self.retranslate_ui)

        # --- Load initial settings ---
        self.retranslate_ui()

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.tr("General"))
