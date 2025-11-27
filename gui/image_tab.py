from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator
from gui.pollinations_tab import PollinationsTab

class ImageTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)

        self.tabs = QTabWidget()
        main_layout.addWidget(self.tabs)

        # Add Pollinations Tab
        self.pollinations_tab = PollinationsTab()
        self.tabs.addTab(self.pollinations_tab, "Pollinations")

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.translate("pollinations_tab_title"))
        self.pollinations_tab.translate_ui()