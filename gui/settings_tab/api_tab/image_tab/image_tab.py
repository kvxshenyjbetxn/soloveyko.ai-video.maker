from PySide6.QtWidgets import QWidget, QVBoxLayout
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator
from .pollinations_tab import PollinationsTab
from .googler_tab import GooglerTab

class ImageTab(QWidget):

    def __init__(self):

        super().__init__()

        self.init_ui()

        self.retranslate_ui()



    def init_ui(self):

        main_layout = QVBoxLayout(self)

        main_layout.setContentsMargins(0, 0, 0, 0)



        self.tabs = AnimatedTabWidget()

        main_layout.addWidget(self.tabs)



        # Add Pollinations Tab

        self.pollinations_tab = PollinationsTab()

        self.tabs.addTab(self.pollinations_tab, "Pollinations")



        # Add Googler Tab

        self.googler_tab = GooglerTab()

        self.tabs.addTab(self.googler_tab, "Googler")



    def retranslate_ui(self):

        self.tabs.setTabText(0, translator.translate("pollinations_tab_title"))

        self.tabs.setTabText(1, translator.translate("googler_tab_title"))

        self.pollinations_tab.translate_ui()

        self.googler_tab.translate_ui()



    def update_fields(self):

        self.pollinations_tab.update_fields()

        self.googler_tab.update_fields()
