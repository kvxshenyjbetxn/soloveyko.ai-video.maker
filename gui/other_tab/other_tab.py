from PySide6.QtWidgets import QWidget, QVBoxLayout
from gui.widgets.animated_tab_widget import AnimatedTabWidget
from utils.translator import translator

from .statistics_tab import StatisticsTab
from .history_tab import HistoryTab

class OtherTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.tabs = AnimatedTabWidget()

        self.statistics_tab = StatisticsTab()
        self.history_tab = HistoryTab(self.main_window)

        self.tabs.addTab(self.statistics_tab, translator.translate('statistics_tab'))
        self.tabs.addTab(self.history_tab, translator.translate('history_tab_title', 'Історія'))

        layout.addWidget(self.tabs)
        self.setLayout(layout)

    def retranslate_ui(self):
        self.tabs.setTabText(0, translator.translate('statistics_tab'))
        self.tabs.setTabText(1, translator.translate('history_tab_title', 'Історія'))

        self.statistics_tab.retranslate_ui()
        self.history_tab.retranslate_ui()
