from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel
from utils.translator import translator

class OpenRouterTab(QWidget):
    def __init__(self):
        super().__init__()
        layout = QVBoxLayout(self)
        self.label = QLabel("OpenRouter Settings")
        layout.addWidget(self.label)
        self.retranslate_ui()

    def retranslate_ui(self):
        # In the future, you would translate the content of this tab
        # For example: self.label.setText(translator.translate('openrouter_settings_title'))
        pass
