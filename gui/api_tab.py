from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator

from gui.openrouter_tab import OpenRouterTab
from gui.audio_tab import AudioTab
from gui.image_tab import ImageTab
from gui.dataimpulse_tab import DataImpulseTab

class ApiTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.tabs = QTabWidget()
        
        self.openrouter_tab = OpenRouterTab()
        self.audio_tab = AudioTab()
        self.image_tab = ImageTab()
        self.dataimpulse_tab = DataImpulseTab()

        self.tabs.addTab(self.openrouter_tab, translator.translate('openrouter_tab'))
        self.tabs.addTab(self.audio_tab, translator.translate('audio_tab'))
        self.tabs.addTab(self.image_tab, translator.translate('image_tab'))
        self.tabs.addTab(self.dataimpulse_tab, translator.translate('dataimpulse_tab'))
        
        layout.addWidget(self.tabs)
        self.setLayout(layout)

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.translate('openrouter_tab'))
        self.tabs.setTabText(1, translator.translate('audio_tab'))
        self.tabs.setTabText(2, translator.translate('image_tab'))
        self.tabs.setTabText(3, translator.translate('dataimpulse_tab'))
        
        self.openrouter_tab.retranslate_ui()
        self.audio_tab.retranslate_ui()
        self.image_tab.retranslate_ui()
        self.dataimpulse_tab.retranslate_ui()
