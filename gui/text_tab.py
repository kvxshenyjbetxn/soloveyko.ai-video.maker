from PySide6.QtWidgets import QWidget, QVBoxLayout, QTextEdit
from utils.translator import translator

class TextTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)

        self.text_edit = QTextEdit()
        self.text_edit.setObjectName("textEdit") # For styling
        layout.addWidget(self.text_edit)

    def retranslate_ui(self):
        # This tab has no text that needs re-translation on language change,
        # but the method is here for consistency.
        pass
