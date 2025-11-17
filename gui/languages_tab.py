from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel
from utils.translator import translator

class LanguagesTab(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        self.label = QLabel("Language Settings")
        layout.addWidget(self.label)
        self.retranslate_ui()

    def retranslate_ui(self):
        pass
