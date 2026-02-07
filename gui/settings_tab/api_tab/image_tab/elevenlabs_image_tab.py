
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QHBoxLayout, QCheckBox, QVBoxLayout
from utils.settings import settings_manager

from utils.translator import translator
from gui.widgets.help_label import HelpLabel
from gui.widgets.setting_row import add_setting_row

class ElevenLabsImageTab(QWidget):
    def __init__(self, parent=None, settings_mgr=None, is_template_mode=False):
        super().__init__(parent)
        self.settings = settings_mgr or settings_manager
        self.is_template_mode = is_template_mode
        self.aspect_ratios = ["16:9", "9:16"] # Common aspects
        self.initUI()
        self.update_fields()
        self.connect_signals()

    def initUI(self):
        layout = QFormLayout(self)

        # API Key
        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        
        # Determine if we show stars
        show_stars = not self.is_template_mode

        def refresh_quick_panel():
            if self.window():
                 if hasattr(self.window(), 'refresh_quick_settings_panels'):
                      self.window().refresh_quick_settings_panels()

        add_setting_row(layout, self.api_key_label, self.api_key_input, "elevenlabs_image.api_key", refresh_quick_panel, show_star=show_stars)

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
        add_setting_row(layout, aspect_label_container, self.aspect_ratio_combo, "elevenlabs_image.aspect_ratio", refresh_quick_panel, show_star=show_stars)

        # Max Threads
        if not self.is_template_mode:
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
            add_setting_row(layout, max_threads_label_container, self.max_threads_spinbox, "elevenlabs_image.max_threads", refresh_quick_panel, show_star=show_stars)

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
        if hasattr(self, 'max_threads_spinbox'): self.max_threads_spinbox.valueChanged.connect(self.save_settings)

    def toggle_proxy_input(self, checked):
        self.proxy_details_widget.setVisible(checked)

    def translate_ui(self):
        self.api_key_label.setText(translator.translate("elevenlabs_image_api_key"))
        self.api_key_input.setPlaceholderText(translator.translate("enter_api_key"))
        self.aspect_ratio_label.setText(translator.translate("aspect_ratio"))
        if hasattr(self, 'max_threads_label'):
             self.max_threads_label.setText(translator.translate("max_threads"))
        self.buy_info_label.setText(translator.translate("elevenlabs_buy_info"))
        self.proxy_checkbox.setText(translator.translate("enable_proxy", "Enable Proxy"))
        self.proxy_input.setPlaceholderText(translator.translate("proxy_placeholder", "http://user:pass@127.0.0.1:8080"))
        self.proxy_recommendation_label.setText(translator.translate("proxy_recommendation"))
        
        # Update hints
        self.aspect_ratio_help.update_tooltip()
        if hasattr(self, 'max_threads_help'): self.max_threads_help.update_tooltip()
        self.proxy_hint_label.update_tooltip()

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.aspect_ratio_combo.blockSignals(True)
        if hasattr(self, 'max_threads_spinbox'): self.max_threads_spinbox.blockSignals(True)
        self.proxy_checkbox.blockSignals(True)
        self.proxy_input.blockSignals(True)

        elevenlabs_image_settings = self.settings.get("elevenlabs_image", {})
        self.api_key_input.setText(elevenlabs_image_settings.get("api_key", ""))
        self.aspect_ratio_combo.setCurrentText(elevenlabs_image_settings.get("aspect_ratio", "16:9"))
        if hasattr(self, 'max_threads_spinbox'):
            self.max_threads_spinbox.setValue(elevenlabs_image_settings.get("max_threads", 5))

        proxy_enabled = elevenlabs_image_settings.get("proxy_enabled", False)
        proxy_url = elevenlabs_image_settings.get("proxy_url", "")
        
        self.proxy_checkbox.setChecked(bool(proxy_enabled))
        self.proxy_input.setText(proxy_url)
        self.toggle_proxy_input(bool(proxy_enabled))

        self.api_key_input.blockSignals(False)
        self.aspect_ratio_combo.blockSignals(False)
        if hasattr(self, 'max_threads_spinbox'): self.max_threads_spinbox.blockSignals(False)
        self.proxy_checkbox.blockSignals(False)
        self.proxy_input.blockSignals(False)

    def save_settings(self):
        current_settings = self.settings.get("elevenlabs_image", {})
        max_threads = self.max_threads_spinbox.value() if hasattr(self, 'max_threads_spinbox') else current_settings.get("max_threads", 5)

        elevenlabs_image_settings = {
            "api_key": self.api_key_input.text(),
            "aspect_ratio": self.aspect_ratio_combo.currentText(),
            "max_threads": max_threads,
            "proxy_enabled": self.proxy_checkbox.isChecked(),
            "proxy_url": self.proxy_input.text().strip(),
        }
        self.settings.set("elevenlabs_image", elevenlabs_image_settings)
