from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QPushButton, QMessageBox, QWidget
)
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator
from utils.settings import ScopedSettingsManager
import copy

from gui.settings_tab.general_tab import GeneralTab
from gui.settings_tab.languages_tab import LanguagesTab
from gui.settings_tab.prompts_tab import PromptsTab
from gui.settings_tab.api_tab.api_tab import ApiTab
from gui.settings_tab.montage_tab import MontageTab
from gui.settings_tab.subtitles_tab import SubtitlesTab

class TemplateEditorWindow(QDialog):
    def __init__(self, template_data, parent=None, template_name=""):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("template_editor_title", "Template Editor") + f" - {template_name}")
        self.setMinimumSize(900, 700)
        
        # Work on a deep copy to allow cancellation
        self.template_data = copy.deepcopy(template_data)
        self.settings_manager = ScopedSettingsManager(self.template_data)
        
        self.init_ui()
        
    def init_ui(self):
        layout = QVBoxLayout(self)
        
        # Tabs
        self.tabs = AnimatedTabWidget()
        
        # Instantiate tabs with scoped settings manager and template mode flag
        self.general_tab = GeneralTab(main_window=self.parent(), settings_mgr=self.settings_manager, is_template_mode=True)
        self.languages_tab = LanguagesTab(main_window=self.parent(), settings_mgr=self.settings_manager, is_template_mode=True)
        self.prompts_tab = PromptsTab(main_window=self.parent(), settings_mgr=self.settings_manager, is_template_mode=True)
        
        self.tabs.addTab(self.general_tab, translator.translate('general_tab'))
        
        self.api_tab = ApiTab(main_window=self.parent(), settings_mgr=self.settings_manager, is_template_mode=True)
        self.tabs.addTab(self.api_tab, translator.translate('api_tab'))

        self.tabs.addTab(self.languages_tab, translator.translate('languages_tab'))
        self.tabs.addTab(self.prompts_tab, translator.translate('prompts_tab'))
        
        self.montage_tab = MontageTab(settings_mgr=self.settings_manager, is_template_mode=True)
        self.tabs.addTab(self.montage_tab, translator.translate('montage_tab'))
        
        self.subtitles_tab = SubtitlesTab(settings_mgr=self.settings_manager, is_template_mode=True)
        self.tabs.addTab(self.subtitles_tab, translator.translate('subtitles_tab'))
        
        layout.addWidget(self.tabs)
        
        # Buttons
        btn_layout = QHBoxLayout()
        btn_layout.addStretch()
        
        self.save_btn = QPushButton(translator.translate("save_button", "Save"))
        self.save_btn.clicked.connect(self.accept)
        self.save_btn.setProperty("class", "primary") # If styling uses this
        
        self.cancel_btn = QPushButton(translator.translate("cancel_button", "Cancel"))
        self.cancel_btn.clicked.connect(self.reject)
        
        btn_layout.addWidget(self.save_btn)
        btn_layout.addWidget(self.cancel_btn)
        
        layout.addLayout(btn_layout)
        
    def get_template_data(self):
        return self.template_data
