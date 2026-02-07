from PySide6.QtWidgets import QWidget, QVBoxLayout
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator
from .pollinations_tab import PollinationsTab
from .googler_tab import GooglerTab
from .elevenlabs_image_tab import ElevenLabsImageTab

class ImageTab(QWidget):

    def __init__(self, settings_mgr=None, is_template_mode=False):
        super().__init__()
        self.settings_manager = settings_mgr
        self.is_template_mode = is_template_mode
        self.init_ui()

        self.retranslate_ui()



    def init_ui(self):

        main_layout = QVBoxLayout(self)

        main_layout.setContentsMargins(0, 10, 0, 0)



        self.tabs = AnimatedTabWidget()

        main_layout.addWidget(self.tabs)



        # Add Pollinations Tab

        # Add Pollinations Tab
        # Add Pollinations Tab
        self.pollinations_tab = PollinationsTab(settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.tabs.addTab(self.pollinations_tab, "Pollinations")

        # Add Googler Tab
        self.googler_tab = GooglerTab(settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.tabs.addTab(self.googler_tab, "Googler")
        
        # Add ElevenLabsImage Tab
        self.elevenlabs_image_tab = ElevenLabsImageTab(settings_mgr=self.settings_manager, is_template_mode=self.is_template_mode)
        self.tabs.addTab(self.elevenlabs_image_tab, "ElevenLabsImage")



    def retranslate_ui(self):

        self.tabs.setTabText(0, translator.translate("pollinations_tab_title"))

        self.tabs.setTabText(1, translator.translate("googler_tab_title"))
        
        self.tabs.setTabText(2, translator.translate("elevenlabs_image_tab_title"))

        self.pollinations_tab.translate_ui()

        self.googler_tab.translate_ui()
        
        self.elevenlabs_image_tab.translate_ui()



    def update_fields(self):

        self.pollinations_tab.update_fields()

        self.googler_tab.update_fields()
        
        self.elevenlabs_image_tab.update_fields()
