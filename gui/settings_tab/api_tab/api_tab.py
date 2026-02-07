from PySide6.QtWidgets import QWidget, QVBoxLayout
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator

from .openrouter_tab import OpenRouterTab
from .audio_tab.audio_tab import AudioTab
from .image_tab.image_tab import ImageTab
from .assemblyai_tab import AssemblyAITab

class ApiTab(QWidget):
    def __init__(self, main_window=None, settings_mgr=None, is_template_mode=False):
        super().__init__()
        self.main_window = main_window # Keep for now for sub-tabs that might need it
        self.settings_manager = settings_mgr
        self.is_template_mode = is_template_mode
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.tabs = AnimatedTabWidget()
        
        self.openrouter_tab = OpenRouterTab(main_window=self.main_window, settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.audio_tab = AudioTab(main_window=self.main_window, settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.image_tab = ImageTab(settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.assemblyai_tab = AssemblyAITab(settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)

        self.tabs.addTab(self.openrouter_tab, translator.translate('openrouter_tab'))
        self.tabs.addTab(self.audio_tab, translator.translate('audio_tab'))
        self.tabs.addTab(self.image_tab, translator.translate('image_tab'))
        self.tabs.addTab(self.assemblyai_tab, translator.translate('assemblyai_tab'))
        
        layout.addWidget(self.tabs)
        self.setLayout(layout)

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.translate('openrouter_tab'))
        self.tabs.setTabText(1, translator.translate('audio_tab'))
        self.tabs.setTabText(2, translator.translate('image_tab'))
        self.tabs.setTabText(3, translator.translate('assemblyai_tab'))
        
        self.openrouter_tab.retranslate_ui()
        self.audio_tab.retranslate_ui()
        self.image_tab.retranslate_ui()
        self.assemblyai_tab.retranslate_ui()

    def update_fields(self):
        self.openrouter_tab.update_fields()
        self.audio_tab.update_fields()
        self.image_tab.update_fields()
        self.assemblyai_tab.update_fields()
