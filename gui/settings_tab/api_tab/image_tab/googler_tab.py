
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QPushButton, QHBoxLayout
from utils.settings import settings_manager
from utils.translator import translator
from api.googler import GooglerAPI

class GooglerTab(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.aspect_ratios = ["IMAGE_ASPECT_RATIO_LANDSCAPE", "IMAGE_ASPECT_RATIO_PORTRAIT", "IMAGE_ASPECT_RATIO_SQUARE"]
        self.initUI()
        self.update_fields()
        self.connect_signals()

    def initUI(self):
        layout = QFormLayout(self)

        # API Key
        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        layout.addRow(self.api_key_label, self.api_key_input)

        # Usage
        self.usage_label = QLabel()
        usage_layout = QHBoxLayout()
        self.check_usage_button = QPushButton()
        self.usage_display_label = QLabel("N/A")
        usage_layout.addWidget(self.check_usage_button)
        usage_layout.addWidget(self.usage_display_label)
        self.check_usage_button.clicked.connect(self.check_usage)
        layout.addRow(self.usage_label, usage_layout)

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

        # Max Video Threads
        self.max_video_threads_label = QLabel()
        self.max_video_threads_spinbox = QSpinBox()
        self.max_video_threads_spinbox.setRange(1, 25)
        layout.addRow(self.max_video_threads_label, self.max_video_threads_spinbox)

        # Video Prompt
        self.video_prompt_label = QLabel()
        self.video_prompt_input = QLineEdit()
        layout.addRow(self.video_prompt_label, self.video_prompt_input)

        # Seed
        self.seed_label = QLabel()
        self.seed_input = QLineEdit()
        layout.addRow(self.seed_label, self.seed_input)

        # Negative Prompt
        self.negative_prompt_label = QLabel()
        self.negative_prompt_input = QLineEdit()
        layout.addRow(self.negative_prompt_label, self.negative_prompt_input)

        self.setLayout(layout)

    def connect_signals(self):
        self.api_key_input.textChanged.connect(self.save_settings)
        self.aspect_ratio_combo.currentIndexChanged.connect(self.save_settings)
        self.max_threads_spinbox.valueChanged.connect(self.save_settings)
        self.max_video_threads_spinbox.valueChanged.connect(self.save_settings)
        self.video_prompt_input.textChanged.connect(self.save_settings)
        self.seed_input.textChanged.connect(self.save_settings)
        self.negative_prompt_input.textChanged.connect(self.save_settings)

    def translate_ui(self):
        self.api_key_label.setText(translator.translate("googler_api_key_label"))
        self.api_key_input.setPlaceholderText(translator.translate("googler_api_key_placeholder"))
        self.usage_label.setText(translator.translate("googler_usage_label"))
        self.check_usage_button.setText(translator.translate("googler_check_usage_button"))
        self.aspect_ratio_label.setText(translator.translate("googler_aspect_ratio_label"))
        self.max_threads_label.setText(translator.translate("googler_max_threads_label"))
        self.max_video_threads_label.setText(translator.translate("googler_max_video_threads_label"))
        self.video_prompt_label.setText(translator.translate("googler_video_prompt_label"))
        self.video_prompt_input.setPlaceholderText(translator.translate("googler_video_prompt_placeholder"))
        self.seed_label.setText(translator.translate("googler_seed_label"))
        self.negative_prompt_label.setText(translator.translate("googler_negative_prompt_label"))
        self.negative_prompt_input.setPlaceholderText(translator.translate("googler_negative_prompt_placeholder"))

    def update_fields(self):
        self.api_key_input.blockSignals(True)
        self.aspect_ratio_combo.blockSignals(True)
        self.max_threads_spinbox.blockSignals(True)
        self.max_video_threads_spinbox.blockSignals(True)
        self.video_prompt_input.blockSignals(True)
        self.seed_input.blockSignals(True)
        self.negative_prompt_input.blockSignals(True)

        googler_settings = settings_manager.get("googler", {})
        self.api_key_input.setText(googler_settings.get("api_key", ""))
        self.aspect_ratio_combo.setCurrentText(googler_settings.get("aspect_ratio", "IMAGE_ASPECT_RATIO_LANDSCAPE"))
        self.max_threads_spinbox.setValue(googler_settings.get("max_threads", 1))
        self.max_video_threads_spinbox.setValue(googler_settings.get("max_video_threads", 1))
        self.video_prompt_input.setText(googler_settings.get("video_prompt", "Animate this scene, cinematic movement, 4k"))
        self.seed_input.setText(str(googler_settings.get("seed", "")))
        self.negative_prompt_input.setText(googler_settings.get("negative_prompt", ""))

        self.api_key_input.blockSignals(False)
        self.aspect_ratio_combo.blockSignals(False)
        self.max_threads_spinbox.blockSignals(False)
        self.max_video_threads_spinbox.blockSignals(False)
        self.video_prompt_input.blockSignals(False)
        self.seed_input.blockSignals(False)
        self.negative_prompt_input.blockSignals(False)

    def save_settings(self):
        googler_settings = {
            "api_key": self.api_key_input.text(),
            "aspect_ratio": self.aspect_ratio_combo.currentText(),
            "max_threads": self.max_threads_spinbox.value(),
            "max_video_threads": self.max_video_threads_spinbox.value(),
            "video_prompt": self.video_prompt_input.text(),
            "seed": self.seed_input.text(),
            "negative_prompt": self.negative_prompt_input.text(),
        }
        settings_manager.set("googler", googler_settings)

    def check_usage(self):
        self.save_settings()
        api_key = self.api_key_input.text()
        if not api_key:
            self.usage_display_label.setText(translator.translate("googler_api_key_missing"))
            return

        googler_api = GooglerAPI(api_key=api_key)
        usage_data = googler_api.get_usage()

        if usage_data:
            img_usage = usage_data.get("current_usage", {}).get("hourly_usage", {}).get("image_generation", {})
            current_usage = img_usage.get("current_usage", "N/A")
            
            img_limits = usage_data.get("account_limits", {})
            limit = img_limits.get("img_gen_per_hour_limit", "N/A")

            self.usage_display_label.setText(f"{current_usage} / {limit}")
        else:
            self.usage_display_label.setText(translator.translate("googler_usage_failed"))
