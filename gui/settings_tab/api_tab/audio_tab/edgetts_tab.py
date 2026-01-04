from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel
from PySide6.QtCore import Qt
from utils.translator import translator

class EdgeTTSTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setAlignment(Qt.AlignTop)

        self.info_label = QLabel()
        self.info_label.setWordWrap(True)
        self.info_label.setStyleSheet("font-size: 14px; margin-bottom: 10px;")
        layout.addWidget(self.info_label)

        self.link_info_label = QLabel()
        self.link_info_label.setWordWrap(True)
        self.link_info_label.setStyleSheet("font-size: 14px;")
        layout.addWidget(self.link_info_label)

        self.link_label = QLabel('<a href="https://huggingface.co/spaces/innoai/Edge-TTS-Text-to-Speech" style="color: #0078d4;">https://huggingface.co/spaces/innoai/Edge-TTS-Text-to-Speech</a>')
        self.link_label.setOpenExternalLinks(True)
        self.link_label.setStyleSheet("font-size: 14px; margin-top: 5px;")
        layout.addWidget(self.link_label)

        layout.addStretch()

    def retranslate_ui(self):
        self.info_label.setText(translator.translate("edgetts_info"))
        self.link_info_label.setText(translator.translate("edgetts_test_link"))

    def update_fields(self):
        # EdgeTTS doesn't have settings to update in this tab
        pass
