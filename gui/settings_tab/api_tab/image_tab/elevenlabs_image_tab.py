
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QHBoxLayout
from utils.settings import settings_manager
from utils.translator import translator

class ElevenLabsImageTab(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.aspect_ratios = ["3:2", "16:9", "1:1", "9:16", "2:3", "4:5", "5:4"] # Common aspects
        self.initUI()
        self.update_fields()
        self.connect_signals()

    def initUI(self):
        layout = QFormLayout(self)

        # API Key
        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        layout.addRow(self.api_key_label, self.api_key_input)

        # Aspect Ratio
        self.aspect_ratio_label = QLabel()
        self.aspect_ratio_combo = QComboBox()
        self.aspect_ratio_combo.addItems(self.aspect_ratios)
        layout.addRow(self.aspect_ratio_label, self.aspect_ratio_combo)

        # Max Threads
        self.max_threads_label = QLabel()
        self.max_threads_spinbox = QSpinBox()
        self.max_threads_spinbox.setRange(1, 25)
        layout.addRow(self.max_threads_label, self.max_threads_spinbox)

        # layout.addRow(self.max_threads_label, self.max_threads_spinbox) # Already added above

        self.buy_info_layout = QHBoxLayout()
        self.buy_info_label = QLabel()
        self.buy_link_label = QLabel('<a href="https://t.me/elevenLabsVoicerBot" style="color: #0078d4;">@elevenLabsVoicerBot</a>')
        self.buy_link_label.setOpenExternalLinks(True)
        self.buy_info_layout.addWidget(self.buy_info_label)
        self.buy_info_layout.addWidget(self.buy_link_label)
        self.buy_info_layout.addStretch()
        layout.addRow("", self.buy_info_layout)

        self.setLayout(layout)

    def connect_signals(self):
        self.api_key_input.textChanged.connect(self.save_settings)
        self.aspect_ratio_combo.currentIndexChanged.connect(self.save_settings)
        self.max_threads_spinbox.valueChanged.connect(self.save_settings)

    def translate_ui(self):
        self.api_key_label.setText("ElevenLabsImage API")
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.aspect_ratio_label.setText(translator.translate("aspect_ratio"))
        self.max_threads_label.setText(translator.translate("max_threads"))
        self.buy_info_label.setText(translator.translate("elevenlabs_buy_info"))

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.aspect_ratio_combo.blockSignals(True)
        self.max_threads_spinbox.blockSignals(True)

        elevenlabs_image_settings = settings_manager.get("elevenlabs_image", {})
        self.api_key_input.setText(elevenlabs_image_settings.get("api_key", ""))
        self.aspect_ratio_combo.setCurrentText(elevenlabs_image_settings.get("aspect_ratio", "16:9"))
        self.max_threads_spinbox.setValue(elevenlabs_image_settings.get("max_threads", 5))

        self.api_key_input.blockSignals(False)
        self.aspect_ratio_combo.blockSignals(False)
        self.max_threads_spinbox.blockSignals(False)

    def save_settings(self):
        elevenlabs_image_settings = {
            "api_key": self.api_key_input.text(),
            "aspect_ratio": self.aspect_ratio_combo.currentText(),
            "max_threads": self.max_threads_spinbox.value(),
        }
        settings_manager.set("elevenlabs_image", elevenlabs_image_settings)
