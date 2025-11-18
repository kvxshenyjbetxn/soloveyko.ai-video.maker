from PySide6.QtWidgets import QWidget, QVBoxLayout, QTextEdit
from utils.translator import translator

class LogTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.log_output = QTextEdit()
        self.log_output.setReadOnly(True)
        layout.addWidget(self.log_output)

    def retranslate_ui(self):
        # The title of the tab itself is set in main_window.py
        pass

    def add_log_message(self, message):
        self.log_output.append(message)
