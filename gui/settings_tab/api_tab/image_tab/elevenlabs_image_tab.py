
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QHBoxLayout, QCheckBox, QVBoxLayout
from utils.settings import settings_manager
from utils.translator import translator
from gui.widgets.help_label import HelpLabel

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
        self.aspect_ratio_help = HelpLabel("elevenlabs_image_aspect_ratio")
        self.aspect_ratio_label = QLabel()
        aspect_label_container = QWidget()
        aspect_label_layout = QHBoxLayout(aspect_label_container)
        aspect_label_layout.setContentsMargins(0, 0, 0, 0)
        aspect_label_layout.setSpacing(5)
        aspect_label_layout.addWidget(self.aspect_ratio_help)
        aspect_label_layout.addWidget(self.aspect_ratio_label)
        
        self.aspect_ratio_combo = QComboBox()
        self.aspect_ratio_combo.addItems(self.aspect_ratios)
        layout.addRow(aspect_label_container, self.aspect_ratio_combo)

        # Max Threads
        self.max_threads_help = HelpLabel("elevenlabs_image_max_threads")
        self.max_threads_label = QLabel()
        max_threads_label_container = QWidget()
        max_threads_label_layout = QHBoxLayout(max_threads_label_container)
        max_threads_label_layout.setContentsMargins(0, 0, 0, 0)
        max_threads_label_layout.setSpacing(5)
        max_threads_label_layout.addWidget(self.max_threads_help)
        max_threads_label_layout.addWidget(self.max_threads_label)
        
        self.max_threads_spinbox = QSpinBox()
        self.max_threads_spinbox.setRange(1, 25)
        layout.addRow(max_threads_label_container, self.max_threads_spinbox)

        # layout.addRow(self.max_threads_label, self.max_threads_spinbox) # Already added above

        self.buy_info_layout = QHBoxLayout()
        self.buy_info_label = QLabel()
        self.buy_link_label = QLabel('<a href="https://t.me/elevenLabsVoicerBot" style="color: #0078d4;">@elevenLabsVoicerBot</a>')
        self.buy_link_label.setOpenExternalLinks(True)
        self.buy_info_layout.addWidget(self.buy_info_label)
        self.buy_info_layout.addWidget(self.buy_link_label)
        self.buy_info_layout.addStretch()
        layout.addRow("", self.buy_info_layout)

        # Proxy Settings
        proxy_layout = QVBoxLayout()
        proxy_header_layout = QHBoxLayout()
        
        # Hint Label (Left of the setting)
        self.proxy_hint_label = HelpLabel("elevenlabs_proxy_hint")
        
        self.proxy_checkbox = QCheckBox()
        self.proxy_checkbox.toggled.connect(self.toggle_proxy_input)
        self.proxy_checkbox.toggled.connect(self.save_settings)
        
        proxy_header_layout.addWidget(self.proxy_hint_label)
        proxy_header_layout.addWidget(self.proxy_checkbox)
        proxy_header_layout.addStretch()
        
        self.proxy_input = QLineEdit()
        self.proxy_input.setPlaceholderText("http://user:pass@host:port")
        self.proxy_input.textChanged.connect(self.save_settings)
        
        # Proxy Recommendation
        self.proxy_recommendation_layout = QHBoxLayout()
        self.proxy_recommendation_label = QLabel()
        self.proxy_recommendation_link = QLabel('<a href="https://stableproxy.com/" style="color: #0078d4;">StableProxy</a>')
        self.proxy_recommendation_link.setOpenExternalLinks(True)
        self.proxy_recommendation_layout.addWidget(self.proxy_recommendation_label)
        self.proxy_recommendation_layout.addWidget(self.proxy_recommendation_link)
        self.proxy_recommendation_layout.addStretch()
        
        # Container for proxy input and recommendation
        self.proxy_details_widget = QWidget()
        proxy_details_layout = QVBoxLayout(self.proxy_details_widget)
        proxy_details_layout.setContentsMargins(0, 0, 0, 0)
        proxy_details_layout.addWidget(self.proxy_input)
        proxy_details_layout.addLayout(self.proxy_recommendation_layout)
        
        proxy_layout.addLayout(proxy_header_layout)
        proxy_layout.addWidget(self.proxy_details_widget)
        layout.addRow("", proxy_layout)

        self.setLayout(layout)

    def connect_signals(self):
        self.api_key_input.textChanged.connect(self.save_settings)
        self.aspect_ratio_combo.currentIndexChanged.connect(self.save_settings)
        self.max_threads_spinbox.valueChanged.connect(self.save_settings)

    def toggle_proxy_input(self, checked):
        self.proxy_details_widget.setVisible(checked)

    def translate_ui(self):
        self.api_key_label.setText("ElevenLabsImage API")
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.aspect_ratio_label.setText(translator.translate("aspect_ratio"))
        self.max_threads_label.setText(translator.translate("max_threads"))
        self.buy_info_label.setText(translator.translate("elevenlabs_buy_info"))
        self.proxy_checkbox.setText(translator.translate("enable_proxy", "Enable Proxy"))
        self.proxy_input.setPlaceholderText(translator.translate("proxy_placeholder", "http://user:pass@127.0.0.1:8080"))
        self.proxy_recommendation_label.setText(translator.translate("proxy_recommendation"))
        
        # Update hints
        self.aspect_ratio_help.update_tooltip()
        self.max_threads_help.update_tooltip()
        self.proxy_hint_label.update_tooltip()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.aspect_ratio_combo.blockSignals(True)
        self.max_threads_spinbox.blockSignals(True)
        self.proxy_checkbox.blockSignals(True)
        self.proxy_input.blockSignals(True)

        elevenlabs_image_settings = settings_manager.get("elevenlabs_image", {})
        self.api_key_input.setText(elevenlabs_image_settings.get("api_key", ""))
        self.aspect_ratio_combo.setCurrentText(elevenlabs_image_settings.get("aspect_ratio", "16:9"))
        self.max_threads_spinbox.setValue(elevenlabs_image_settings.get("max_threads", 5))

        proxy_enabled = elevenlabs_image_settings.get("proxy_enabled", False)
        proxy_url = elevenlabs_image_settings.get("proxy_url", "")
        
        self.proxy_checkbox.setChecked(proxy_enabled)
        self.proxy_input.setText(proxy_url)
        self.toggle_proxy_input(proxy_enabled)

        self.api_key_input.blockSignals(False)
        self.aspect_ratio_combo.blockSignals(False)
        self.max_threads_spinbox.blockSignals(False)
        self.proxy_checkbox.blockSignals(False)
        self.proxy_input.blockSignals(False)

    def save_settings(self):
        elevenlabs_image_settings = {
            "api_key": self.api_key_input.text(),
            "aspect_ratio": self.aspect_ratio_combo.currentText(),
            "max_threads": self.max_threads_spinbox.value(),
            "proxy_enabled": self.proxy_checkbox.isChecked(),
            "proxy_url": self.proxy_input.text().strip(),
        }
        settings_manager.set("elevenlabs_image", elevenlabs_image_settings)
