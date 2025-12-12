from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QHBoxLayout
from utils.translator import translator
from utils.settings import settings_manager

class AssemblyAITab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.settings = settings_manager
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # API Key
        api_key_layout = QHBoxLayout()
        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.api_key_input.textChanged.connect(self.save_api_key)
        api_key_layout.addWidget(self.api_key_label)
        api_key_layout.addWidget(self.api_key_input)
        layout.addLayout(api_key_layout)

        layout.addStretch()

    def retranslate_ui(self):
        self.api_key_label.setText(translator.translate("assemblyai_api_key"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        api_key = self.settings.get("assemblyai_api_key", "")
        self.api_key_input.setText(api_key)
        self.api_key_input.blockSignals(False)
        
    def save_api_key(self, key):
        self.settings.set("assemblyai_api_key", key)
