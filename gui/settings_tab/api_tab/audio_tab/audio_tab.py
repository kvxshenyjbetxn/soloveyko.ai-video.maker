from PySide6.QtWidgets import QWidget, QVBoxLayout
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator
from .elevenlabs_tab import ElevenLabsTab
from .voicemaker_tab import VoicemakerTab
from .gemini_tts_tab import GeminiTTSTab
from .elevenlabs_unlim_tab import ElevenLabsUnlimTab
from .edgetts_tab import EdgeTTSTab

class AudioTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 10, 0, 0)
        self.tab_widget = AnimatedTabWidget()
        
        self.elevenlabs_tab = ElevenLabsTab(self.main_window)
        self.tab_widget.addTab(self.elevenlabs_tab, "ElevenLabs")
        
        self.elevenlabs_unlim_tab = ElevenLabsUnlimTab(main_window=self.main_window)
        self.tab_widget.addTab(self.elevenlabs_unlim_tab, "ElevenLabs Unlim")
        
        self.voicemaker_tab = VoicemakerTab(self.main_window)
        self.tab_widget.addTab(self.voicemaker_tab, "Voicemaker")

        self.gemini_tts_tab = GeminiTTSTab(self.main_window)
        self.tab_widget.addTab(self.gemini_tts_tab, "GeminiTTS")
        
        self.edgetts_tab = EdgeTTSTab(self.main_window)
        self.tab_widget.addTab(self.edgetts_tab, "EdgeTTS")
        
        layout.addWidget(self.tab_widget)

    def retranslate_ui(self):
        self.tab_widget.setTabText(0, translator.translate("elevenlabs"))
        self.tab_widget.setTabText(1, "ElevenLabs Unlim")
        self.tab_widget.setTabText(2, "Voicemaker")
        self.tab_widget.setTabText(3, "GeminiTTS")
        self.tab_widget.setTabText(4, "EdgeTTS")

    def update_fields(self):
        self.elevenlabs_tab.update_fields()
        # ElevenLabsUnlimTab handles its own updates via textChanged/clicked if needed, 
        # but if we add update_fields there we should call it. 
        # Currently it reads directly from settings on init and updates on change.
        # But if we want to refresh it (e.g. template applied):
        # self.elevenlabs_unlim_tab.update_fields() 
        self.voicemaker_tab.update_fields()
        self.gemini_tts_tab.update_fields()
        self.edgetts_tab.update_fields()

