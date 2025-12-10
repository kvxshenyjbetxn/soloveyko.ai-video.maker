from PySide6.QtWidgets import QWidget, QVBoxLayout, QTabWidget
from utils.translator import translator

from .general_tab import GeneralTab
from .api_tab.api_tab import ApiTab
from .languages_tab import LanguagesTab
from .prompts_tab import PromptsTab
from .montage_tab import MontageTab
from .subtitles_tab import SubtitlesTab
from .templates_tab import TemplatesTab
from .statistics_tab import StatisticsTab

class SettingsTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.templates_tab.template_applied.connect(self._update_all_tabs)

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.tabs = QTabWidget()

        self.general_tab = GeneralTab(self.main_window)
        self.api_tab = ApiTab(main_window=self.main_window)
        self.languages_tab = LanguagesTab()
        self.prompts_tab = PromptsTab()
        self.montage_tab = MontageTab()
        self.subtitles_tab = SubtitlesTab()
        self.templates_tab = TemplatesTab()
        self.statistics_tab = StatisticsTab()

        self.tabs.addTab(self.general_tab, translator.translate('general_tab'))
        self.tabs.addTab(self.api_tab, translator.translate('api_tab'))
        self.tabs.addTab(self.languages_tab, translator.translate('languages_tab'))
        self.tabs.addTab(self.prompts_tab, translator.translate('prompts_tab'))
        self.tabs.addTab(self.montage_tab, translator.translate('montage_tab'))
        self.tabs.addTab(self.subtitles_tab, translator.translate('subtitles_tab'))
        self.tabs.addTab(self.templates_tab, translator.translate('templates_tab'))
        self.tabs.addTab(self.statistics_tab, translator.translate('statistics_tab'))

        layout.addWidget(self.tabs)
        self.setLayout(layout)

    def _update_all_tabs(self):
        self.general_tab.update_fields()
        self.api_tab.update_fields()
        self.languages_tab.update_fields()
        self.prompts_tab.update_fields()
        self.montage_tab.update_fields()
        self.subtitles_tab.update_fields()

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.translate('general_tab'))
        self.tabs.setTabText(1, translator.translate('api_tab'))
        self.tabs.setTabText(2, translator.translate('languages_tab'))
        self.tabs.setTabText(3, translator.translate('prompts_tab'))
        self.tabs.setTabText(4, translator.translate('montage_tab'))
        self.tabs.setTabText(5, translator.translate('subtitles_tab'))
        self.tabs.setTabText(6, translator.translate('templates_tab'))
        self.tabs.setTabText(7, translator.translate('statistics_tab'))

        # Retranslate all sub-tabs
        self.general_tab.retranslate_ui()
        self.api_tab.retranslate_ui()
        self.languages_tab.retranslate_ui()
        self.prompts_tab.retranslate_ui()
        self.montage_tab.retranslate_ui()
        self.subtitles_tab.retranslate_ui()
        self.templates_tab.retranslate_ui()
        self.statistics_tab.retranslate_ui()
