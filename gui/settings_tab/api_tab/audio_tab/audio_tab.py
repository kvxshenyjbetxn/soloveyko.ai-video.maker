from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator
from .elevenlabs_tab import ElevenLabsTab
from .voicemaker_tab import VoicemakerTab
from .gemini_tts_tab import GeminiTTSTab

class AudioTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.tab_widget = QTabWidget()
        
        self.elevenlabs_tab = ElevenLabsTab(self.main_window)
        self.tab_widget.addTab(self.elevenlabs_tab, "ElevenLabs")
        
        self.voicemaker_tab = VoicemakerTab(self.main_window)
        self.tab_widget.addTab(self.voicemaker_tab, "Voicemaker")

        self.gemini_tts_tab = GeminiTTSTab(self.main_window)
        self.tab_widget.addTab(self.gemini_tts_tab, "GeminiTTS")
        
        layout.addWidget(self.tab_widget)

    def retranslate_ui(self):
        self.tab_widget.setTabText(0, translator.translate("elevenlabs"))
        self.tab_widget.setTabText(1, "Voicemaker")
        self.tab_widget.setTabText(2, "GeminiTTS")

    def update_fields(self):
        self.elevenlabs_tab.update_fields()
        self.voicemaker_tab.update_fields()
        self.gemini_tts_tab.update_fields()
