from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel
from PySide6.QtCore import Qt
from utils.translator import translator

class RewriteTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        self.placeholder_label = QLabel("Rewrite Tab Placeholder")
        self.placeholder_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.placeholder_label)
        self.retranslate_ui()

    def retranslate_ui(self):
        # Even though it's a placeholder, we use translator for consistency if needed later
        # Currently the name of the tab is handled by main_window.tabs.setTabText
        pass
