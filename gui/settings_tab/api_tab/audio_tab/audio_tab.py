from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator
from .elevenlabs_tab import ElevenLabsTab

class AudioTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.tab_widget = QTabWidget()
        
        self.elevenlabs_tab = ElevenLabsTab(self.main_window)
        self.tab_widget.addTab(self.elevenlabs_tab, "ElevenLabs")
        
        layout.addWidget(self.tab_widget)

    def retranslate_ui(self):
        self.tab_widget.setTabText(0, translator.translate("elevenlabs"))
