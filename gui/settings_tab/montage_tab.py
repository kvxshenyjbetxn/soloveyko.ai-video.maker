from PySide6.QtWidgets import (QWidget, QVBoxLayout, QScrollArea,
                               QCheckBox, QDoubleSpinBox, QComboBox, QSpinBox,
                               QFormLayout, QGroupBox, QLabel)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from gui.widgets.slider_spinbox import SliderWithSpinBox

class MontageTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0,0,0,0)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        scroll_content = QWidget()
        scroll_area.setWidget(scroll_content)

        self.layout = QVBoxLayout(scroll_content)
        self.layout.setSpacing(15)
        self.layout.setContentsMargins(10, 10, 10, 10)

        # --- Render Settings ---
        self.render_group = QGroupBox()
        render_layout = QFormLayout()

        self.codec_label = QLabel()
        self.codec_combo = QComboBox()
        # Add codecs with (Label, Value) pairs
        self.codec_combo.addItem("libx264 (CPU)", "libx264")
        self.codec_combo.addItem("libx265 (CPU)", "libx265")
        self.codec_combo.addItem("h264_nvenc (NVIDIA)", "h264_nvenc")
        self.codec_combo.addItem("h264_amf (AMD)", "h264_amf")
        self.codec_combo.addItem("h264_videotoolbox (Mac)", "h264_videotoolbox")
        
        self.codec_combo.currentIndexChanged.connect(self.save_settings)
        render_layout.addRow(self.codec_label, self.codec_combo)

        self.preset_label = QLabel()
        self.preset_combo = QComboBox()
        self.preset_combo.addItems(["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"])
        self.preset_combo.currentTextChanged.connect(self.save_settings)
        render_layout.addRow(self.preset_label, self.preset_combo)

        self.bitrate_label = QLabel()
        self.bitrate_spin = QSpinBox()
        self.bitrate_spin.setRange(1, 100)
        self.bitrate_spin.setSuffix(" Mbps")
        self.bitrate_spin.valueChanged.connect(self.save_settings)
        render_layout.addRow(self.bitrate_label, self.bitrate_spin)

        self.upscale_label = QLabel()
        self.upscale_spin = QDoubleSpinBox()
        self.upscale_spin.setRange(1.0, 5.0)
        self.upscale_spin.setSingleStep(0.1)
        self.upscale_spin.setSuffix("x")
        self.upscale_spin.setToolTip("Image upscale factor before processing (Default: 3.0)")
        self.upscale_spin.valueChanged.connect(self.save_settings)
        render_layout.addRow(self.upscale_label, self.upscale_spin)

        self.render_group.setLayout(render_layout)
        self.layout.addWidget(self.render_group)

        # --- Transitions Settings ---
        self.trans_group = QGroupBox()
        trans_layout = QFormLayout()

        self.enable_trans_cb = QCheckBox()
        self.enable_trans_cb.toggled.connect(self.save_settings)
        trans_layout.addRow(self.enable_trans_cb)

        self.trans_dur_label = QLabel()
        self.trans_dur_spin = SliderWithSpinBox()
        self.trans_dur_spin.setRange(0.1, 5.0)
        self.trans_dur_spin.setSingleStep(0.1)
        self.trans_dur_spin.setSuffix(" s")
        self.trans_dur_spin.valueChanged.connect(self.save_settings)
        trans_layout.addRow(self.trans_dur_label, self.trans_dur_spin)

        self.trans_group.setLayout(trans_layout)
        self.layout.addWidget(self.trans_group)

        # --- Zoom Effects ---
        self.zoom_group = QGroupBox()
        zoom_layout = QFormLayout()

        self.enable_zoom_cb = QCheckBox()
        self.enable_zoom_cb.toggled.connect(self.save_settings)
        zoom_layout.addRow(self.enable_zoom_cb)

        self.zoom_speed_label = QLabel()
        self.zoom_speed_spin = SliderWithSpinBox()
        self.zoom_speed_spin.setRange(0.1, 5.0)
        self.zoom_speed_spin.setSingleStep(0.1)
        self.zoom_speed_spin.valueChanged.connect(self.save_settings)
        zoom_layout.addRow(self.zoom_speed_label, self.zoom_speed_spin)

        self.zoom_int_label = QLabel()
        self.zoom_int_spin = SliderWithSpinBox()
        self.zoom_int_spin.setRange(0.01, 1.0)
        self.zoom_int_spin.setSingleStep(0.05)
        self.zoom_int_spin.valueChanged.connect(self.save_settings)
        zoom_layout.addRow(self.zoom_int_label, self.zoom_int_spin)

        self.zoom_group.setLayout(zoom_layout)
        self.layout.addWidget(self.zoom_group)

        # --- Sway Effects ---
        self.sway_group = QGroupBox()
        sway_layout = QFormLayout()

        self.enable_sway_cb = QCheckBox()
        self.enable_sway_cb.toggled.connect(self.save_settings)
        sway_layout.addRow(self.enable_sway_cb)

        self.sway_speed_label = QLabel()
        self.sway_speed_spin = SliderWithSpinBox()
        self.sway_speed_spin.setRange(0.1, 5.0)
        self.sway_speed_spin.setSingleStep(0.1)
        self.sway_speed_spin.valueChanged.connect(self.save_settings)
        sway_layout.addRow(self.sway_speed_label, self.sway_speed_spin)

        self.sway_group.setLayout(sway_layout)
        self.layout.addWidget(self.sway_group)

        # --- Special Processing ---
        self.special_proc_group = QGroupBox()
        special_proc_layout = QFormLayout()

        self.special_proc_mode_label = QLabel()
        self.special_proc_mode_combo = QComboBox()
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_disabled"), "Disabled")
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_quick_show"), "Quick show")
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_video_at_beginning"), "Video at the beginning")
        self.special_proc_mode_combo.currentIndexChanged.connect(self.save_settings)
        self.special_proc_mode_combo.currentIndexChanged.connect(self.toggle_special_proc_widgets)
        special_proc_layout.addRow(self.special_proc_mode_label, self.special_proc_mode_combo)

        # Quick show settings
        self.special_proc_img_count_label = QLabel()
        self.special_proc_img_count_spin = QSpinBox()
        self.special_proc_img_count_spin.setRange(1, 100)
        self.special_proc_img_count_spin.valueChanged.connect(self.save_settings)
        special_proc_layout.addRow(self.special_proc_img_count_label, self.special_proc_img_count_spin)

        self.special_proc_dur_label = QLabel()
        self.special_proc_dur_spin = SliderWithSpinBox()
        self.special_proc_dur_spin.setRange(0.1, 10.0)
        self.special_proc_dur_spin.setSingleStep(0.1)
        self.special_proc_dur_spin.setSuffix(" s")
        self.special_proc_dur_spin.valueChanged.connect(self.save_settings)
        special_proc_layout.addRow(self.special_proc_dur_label, self.special_proc_dur_spin)

        # Video at the beginning settings
        self.special_proc_video_count_label = QLabel()
        self.special_proc_video_count_spin = QSpinBox()
        self.special_proc_video_count_spin.setRange(1, 100)
        self.special_proc_video_count_spin.valueChanged.connect(self.save_settings)
        special_proc_layout.addRow(self.special_proc_video_count_label, self.special_proc_video_count_spin)

        self.special_proc_check_sequence_cb = QCheckBox()
        self.special_proc_check_sequence_cb.toggled.connect(self.save_settings)
        special_proc_layout.addRow(self.special_proc_check_sequence_cb)
        
        self.special_proc_group.setLayout(special_proc_layout)
        self.layout.addWidget(self.special_proc_group)

        # --- Performance Settings ---
        self.perf_group = QGroupBox()
        perf_layout = QFormLayout()

        self.max_concurrent_montages_label = QLabel()
        self.max_concurrent_montages_spin = QSpinBox()
        self.max_concurrent_montages_spin.setRange(1, 10)
        self.max_concurrent_montages_spin.valueChanged.connect(self.save_settings)
        perf_layout.addRow(self.max_concurrent_montages_label, self.max_concurrent_montages_spin)

        self.perf_group.setLayout(perf_layout)
        self.layout.addWidget(self.perf_group)

        self.layout.addStretch()

    def update_fields(self):
        m_settings = self.settings.get("montage", {})

        # Block signals
        for widget in self.findChildren(QWidget):
            if isinstance(widget, (QComboBox, QSpinBox, QDoubleSpinBox, QCheckBox, SliderWithSpinBox)):
                widget.blockSignals(True)

        codec = m_settings.get("codec", "libx264")
        index = self.codec_combo.findData(codec)
        if index != -1:
            self.codec_combo.setCurrentIndex(index)
        else:
             # Fallback if the saved codec isn't in our list (e.g., config file manual edit)
             self.codec_combo.setCurrentIndex(0)

        self.preset_combo.setCurrentText(m_settings.get("preset", "medium"))
        self.bitrate_spin.setValue(m_settings.get("bitrate_mbps", 15))
        self.upscale_spin.setValue(m_settings.get("upscale_factor", 3.0))

        self.enable_trans_cb.setChecked(m_settings.get("enable_transitions", True))
        self.trans_dur_spin.setValue(m_settings.get("transition_duration", 0.5))

        self.enable_zoom_cb.setChecked(m_settings.get("enable_zoom", True))
        self.zoom_speed_spin.setValue(m_settings.get("zoom_speed_factor", 1.0))
        self.zoom_int_spin.setValue(m_settings.get("zoom_intensity", 0.15))

        self.enable_sway_cb.setChecked(m_settings.get("enable_sway", False))
        self.sway_speed_spin.setValue(m_settings.get("sway_speed_factor", 1.0))

        mode = m_settings.get("special_processing_mode", "Disabled")
        index = self.special_proc_mode_combo.findData(mode)
        if index != -1:
            self.special_proc_mode_combo.setCurrentIndex(index)

        self.special_proc_img_count_spin.setValue(m_settings.get("special_processing_image_count", 5))
        self.special_proc_dur_spin.setValue(m_settings.get("special_processing_duration_per_image", 2.0))
        self.special_proc_video_count_spin.setValue(m_settings.get("special_processing_video_count", 1))
        self.special_proc_check_sequence_cb.setChecked(m_settings.get("special_processing_check_sequence", False))

        self.max_concurrent_montages_spin.setValue(m_settings.get("max_concurrent_montages", 1))

        # Unblock signals
        for widget in self.findChildren(QWidget):
            if isinstance(widget, (QComboBox, QSpinBox, QDoubleSpinBox, QCheckBox, SliderWithSpinBox)):
                widget.blockSignals(False)

        self.toggle_special_proc_widgets()

    def save_settings(self, *args):
        m_settings = {
            "codec": self.codec_combo.currentData(),
            "preset": self.preset_combo.currentText(),
            "bitrate_mbps": self.bitrate_spin.value(),
            "upscale_factor": self.upscale_spin.value(),
            "enable_transitions": self.enable_trans_cb.isChecked(),
            "transition_duration": self.trans_dur_spin.value(),
            "enable_zoom": self.enable_zoom_cb.isChecked(),
            "zoom_speed_factor": self.zoom_speed_spin.value(),
            "zoom_intensity": self.zoom_int_spin.value(),
            "enable_sway": self.enable_sway_cb.isChecked(),
            "sway_speed_factor": self.sway_speed_spin.value(),
            "special_processing_mode": self.special_proc_mode_combo.currentData(),
            "special_processing_image_count": self.special_proc_img_count_spin.value(),
            "special_processing_duration_per_image": self.special_proc_dur_spin.value(),
            "special_processing_video_count": self.special_proc_video_count_spin.value(),
            "special_processing_check_sequence": self.special_proc_check_sequence_cb.isChecked(),
            "max_concurrent_montages": self.max_concurrent_montages_spin.value()
        }
        self.settings.set("montage", m_settings)

    def retranslate_ui(self):
        self.render_group.setTitle(translator.translate("render_settings"))
        self.codec_label.setText(translator.translate("codec_label"))
        self.preset_label.setText(translator.translate("preset_label"))
        self.bitrate_label.setText(translator.translate("bitrate_label"))
        self.upscale_label.setText(translator.translate("upscale_factor_label"))

        self.trans_group.setTitle(translator.translate("transitions_settings"))
        self.enable_trans_cb.setText(translator.translate("enable_transitions_label"))
        self.trans_dur_label.setText(translator.translate("duration_label"))

        self.zoom_group.setTitle(translator.translate("zoom_effects"))
        self.enable_zoom_cb.setText(translator.translate("enable_zoom_label"))
        self.zoom_speed_label.setText(translator.translate("zoom_speed_factor_label"))
        self.zoom_int_label.setText(translator.translate("zoom_intensity_label"))

        self.sway_group.setTitle(translator.translate("sway_effects"))
        self.enable_sway_cb.setText(translator.translate("enable_sway_label"))
        self.sway_speed_label.setText(translator.translate("sway_speed_factor_label"))

        self.special_proc_group.setTitle(translator.translate("special_processing_group"))
        self.special_proc_mode_label.setText(translator.translate("special_proc_mode_label"))
        self.special_proc_mode_combo.setItemText(0, translator.translate("special_proc_mode_disabled"))
        self.special_proc_mode_combo.setItemText(1, translator.translate("special_proc_mode_quick_show"))
        self.special_proc_mode_combo.setItemText(2, translator.translate("special_proc_mode_video_at_beginning"))
        
        self.special_proc_img_count_label.setText(translator.translate("image_count_label"))
        self.special_proc_dur_label.setText(translator.translate("duration_per_image_label"))
        self.special_proc_video_count_label.setText(translator.translate("special_proc_video_count_label"))
        self.special_proc_check_sequence_cb.setText(translator.translate("special_proc_check_sequence_label"))

        self.perf_group.setTitle(translator.translate("performance_group"))
        self.max_concurrent_montages_label.setText(translator.translate("max_concurrent_montages_label"))

    def toggle_special_proc_widgets(self):
        mode = self.special_proc_mode_combo.currentData()

        is_quick_show = (mode == "Quick show")
        is_video = (mode == "Video at the beginning")
        
        self.special_proc_img_count_spin.setVisible(is_quick_show)
        self.special_proc_img_count_label.setVisible(is_quick_show)
        self.special_proc_dur_spin.setVisible(is_quick_show)
        self.special_proc_dur_label.setVisible(is_quick_show)
        
        self.special_proc_video_count_spin.setVisible(is_video)
        self.special_proc_video_count_label.setVisible(is_video)
        self.special_proc_check_sequence_cb.setVisible(is_video)