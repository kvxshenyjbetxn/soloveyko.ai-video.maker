from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QScrollArea
from PySide6.QtCore import Qt
from utils.translator import translator

class DataImpulseTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0,0,0,0)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        scroll_content = QWidget()
        scroll_area.setWidget(scroll_content)

        layout = QVBoxLayout(scroll_content)
        self.label = QLabel("DataImpulse Settings")
        layout.addWidget(self.label)
        layout.addStretch()

    def retranslate_ui(self):
        self.label.setText(translator.translate("dataimpulse_settings_label"))