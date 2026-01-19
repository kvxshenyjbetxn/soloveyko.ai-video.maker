
from utils.settings import settings_manager
from utils.translator import translator
from api.googler import GooglerAPI
from gui.widgets.help_label import HelpLabel
from gui.widgets.setting_row import add_setting_row
from PySide6.QtWidgets import QWidget, QFormLayout, QLabel, QComboBox, QLineEdit, QSpinBox, QPushButton, QHBoxLayout, QWidget

class GooglerTab(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.aspect_ratios = ["IMAGE_ASPECT_RATIO_LANDSCAPE", "IMAGE_ASPECT_RATIO_PORTRAIT"]
        self.initUI()
        self.update_fields()
        self.connect_signals()

    def initUI(self):
        layout = QFormLayout(self)


        self.api_key_label = QLabel()
        self.api_key_input = QLineEdit()
        
        def refresh_quick_panel():
            if self.window():
                 if hasattr(self.window(), 'refresh_quick_settings_panels'):
                      self.window().refresh_quick_settings_panels()

        add_setting_row(layout, self.api_key_label, self.api_key_input, "googler.api_key", refresh_quick_panel)

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
        self.aspect_ratio_help = HelpLabel("googler_aspect_ratio_label")
        self.aspect_ratio_label = QLabel()
        aspect_label_container = QWidget()
        aspect_label_layout = QHBoxLayout(aspect_label_container)
        aspect_label_layout.setContentsMargins(0, 0, 0, 0)
        aspect_label_layout.setSpacing(5)
        aspect_label_layout.addWidget(self.aspect_ratio_help)
        aspect_label_layout.addWidget(self.aspect_ratio_label)
        
        self.aspect_ratio_combo = QComboBox()
        for ratio in self.aspect_ratios:
             self.aspect_ratio_combo.addItem(ratio, ratio)
        add_setting_row(layout, aspect_label_container, self.aspect_ratio_combo, "googler.aspect_ratio", refresh_quick_panel)

        # Max Threads
        self.max_threads_help = HelpLabel("googler_max_threads_label")
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

        # Max Video Threads
        self.max_video_threads_help = HelpLabel("googler_max_video_threads_label")
        self.max_video_threads_label = QLabel()
        max_video_label_container = QWidget()
        max_video_label_layout = QHBoxLayout(max_video_label_container)
        max_video_label_layout.setContentsMargins(0, 0, 0, 0)
        max_video_label_layout.setSpacing(5)
        max_video_label_layout.addWidget(self.max_video_threads_help)
        max_video_label_layout.addWidget(self.max_video_threads_label)
        
        self.max_video_threads_spinbox = QSpinBox()
        self.max_video_threads_spinbox.setRange(1, 25)
        layout.addRow(max_video_label_container, self.max_video_threads_spinbox)

        # Video Prompt
        self.video_prompt_help = HelpLabel("googler_video_prompt_label")
        self.video_prompt_label = QLabel()
        video_prompt_label_container = QWidget()
        video_prompt_label_layout = QHBoxLayout(video_prompt_label_container)
        video_prompt_label_layout.setContentsMargins(0, 0, 0, 0)
        video_prompt_label_layout.setSpacing(5)
        video_prompt_label_layout.addWidget(self.video_prompt_help)
        video_prompt_label_layout.addWidget(self.video_prompt_label)
        
        self.video_prompt_input = QLineEdit()
        add_setting_row(layout, video_prompt_label_container, self.video_prompt_input, "googler.video_prompt", refresh_quick_panel)

        # Seed
        self.seed_help = HelpLabel("googler_seed_label")
        self.seed_label = QLabel()
        seed_label_container = QWidget()
        seed_label_layout = QHBoxLayout(seed_label_container)
        seed_label_layout.setContentsMargins(0, 0, 0, 0)
        seed_label_layout.setSpacing(5)
        seed_label_layout.addWidget(self.seed_help)
        seed_label_layout.addWidget(self.seed_label)
        
        self.seed_input = QLineEdit()
        add_setting_row(layout, seed_label_container, self.seed_input, "googler.seed", refresh_quick_panel)

        # Negative Prompt
        self.negative_prompt_help = HelpLabel("googler_negative_prompt_label")
        self.negative_prompt_label = QLabel()
        negative_prompt_label_container = QWidget()
        negative_prompt_label_layout = QHBoxLayout(negative_prompt_label_container)
        negative_prompt_label_layout.setContentsMargins(0, 0, 0, 0)
        negative_prompt_label_layout.setSpacing(5)
        negative_prompt_label_layout.addWidget(self.negative_prompt_help)
        negative_prompt_label_layout.addWidget(self.negative_prompt_label)
        
        self.negative_prompt_input = QLineEdit()
        add_setting_row(layout, negative_prompt_label_container, self.negative_prompt_input, "googler.negative_prompt", refresh_quick_panel)

        # Buy API Key Link
        self.buy_info_layout = QHBoxLayout()
        self.buy_info_label = QLabel()
        self.buy_link_label = QLabel('<a href="https://t.me/logovo_shop_bot" style="color: #0078d4;">@logovo_shop_bot</a>')
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
        self.max_video_threads_spinbox.valueChanged.connect(self.save_settings)
        self.video_prompt_input.textChanged.connect(self.save_settings)
        self.seed_input.textChanged.connect(self.save_settings)
        self.negative_prompt_input.textChanged.connect(self.save_settings)

    def translate_ui(self):
        self.api_key_label.setText(translator.translate("googler_api_key_label"))
        self.api_key_input.setPlaceholderText(translator.translate("googler_api_key_placeholder"))
        self.buy_info_label.setText(translator.translate("googler_buy_info"))
        self.usage_label.setText(translator.translate("googler_usage_label"))
        self.check_usage_button.setText(translator.translate("googler_check_usage_button"))
        self.aspect_ratio_label.setText(translator.translate("googler_aspect_ratio_label"))
        self.aspect_ratio_combo.setItemText(0, translator.translate("aspect_ratio_landscape"))
        self.aspect_ratio_combo.setItemText(1, translator.translate("aspect_ratio_portrait"))
        self.max_threads_label.setText(translator.translate("googler_max_threads_label"))
        self.max_video_threads_label.setText(translator.translate("googler_max_video_threads_label"))
        self.video_prompt_label.setText(translator.translate("googler_video_prompt_label"))
        self.video_prompt_input.setPlaceholderText(translator.translate("googler_video_prompt_placeholder"))
        self.seed_label.setText(translator.translate("googler_seed_label"))
        self.negative_prompt_label.setText(translator.translate("googler_negative_prompt_label"))
        self.negative_prompt_input.setPlaceholderText(translator.translate("googler_negative_prompt_placeholder"))
        
        # Update hints
        self.aspect_ratio_help.update_tooltip()
        self.max_threads_help.update_tooltip()
        self.max_video_threads_help.update_tooltip()
        self.video_prompt_help.update_tooltip()
        self.seed_help.update_tooltip()
        self.negative_prompt_help.update_tooltip()

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
        idx = self.aspect_ratio_combo.findData(googler_settings.get("aspect_ratio", "IMAGE_ASPECT_RATIO_LANDSCAPE"))
        if idx >= 0:
            self.aspect_ratio_combo.setCurrentIndex(idx)
        self.max_threads_spinbox.setValue(googler_settings.get("max_threads", 25))
        self.max_video_threads_spinbox.setValue(googler_settings.get("max_video_threads", 10))
        self.video_prompt_input.setText(googler_settings.get("video_prompt", "Animate this scene, cinematic movement, 4k"))
        self.seed_input.setText(str(googler_settings.get("seed", "")))
        self.negative_prompt_input.setText(googler_settings.get("negative_prompt", "blood"))

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
            "aspect_ratio": self.aspect_ratio_combo.currentData(),
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
            # New v3 structure
            try:
                # Images
                img_limits = usage_data.get("account_limits") or {}
                img_limit = img_limits.get("img_gen_per_hour_limit", 0)
                
                cur_usage = usage_data.get("current_usage") or {}
                hourly = cur_usage.get("hourly_usage") or {}
                img_stats = hourly.get("image_generation") or {}
                img_cur = img_stats.get("current_usage", 0)
                
                # Videos
                vid_limit = img_limits.get("video_gen_per_hour_limit", 0)
                vid_stats = hourly.get("video_generation") or {}
                vid_cur = vid_stats.get("current_usage", 0)
                
                # Threads
                active = cur_usage.get("active_threads") or {}
                img_threads = active.get("image_threads", 0)
                vid_threads = active.get("video_threads", 0)
                
                img_thread_limit = img_limits.get("img_generation_threads_allowed", 0)
                vid_thread_limit = img_limits.get("video_generation_threads_allowed", 0)

                summary = f"Img: {img_cur}/{img_limit} | Vid: {vid_cur}/{vid_limit}"
                self.usage_display_label.setText(summary)
                
                tooltip = (
                    f"Googler Usage Stats:\n"
                    f"Images: {img_cur} / {img_limit} (Hourly)\n"
                    f"Videos: {vid_cur} / {vid_limit} (Hourly)\n"
                    f"Image Threads: {img_threads} / {img_thread_limit}\n"
                    f"Video Threads: {vid_threads} / {vid_thread_limit}"
                )
                self.usage_display_label.setToolTip(tooltip)
                
            except Exception as e:
                logger.log(f"Error parsing usage data: {e}", level=LogLevel.ERROR)
                self.usage_display_label.setText("Error parsing")
        else:
            self.usage_display_label.setText(translator.translate("googler_usage_failed"))
