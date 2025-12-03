from PySide6.QtWidgets import (QWidget, QVBoxLayout, QScrollArea,
                               QCheckBox, QDoubleSpinBox, QComboBox, QSpinBox,
                               QFormLayout, QGroupBox, QLabel)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager

class MontageTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.init_ui()
        self.load_settings()
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
        self.codec_combo.addItems(["libx264", "h264_nvenc", "h264_amf"])
        self.codec_combo.currentTextChanged.connect(self.save_settings)
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
        self.trans_dur_spin = QDoubleSpinBox()
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
        self.zoom_speed_spin = QDoubleSpinBox()
        self.zoom_speed_spin.setRange(0.1, 5.0)
        self.zoom_speed_spin.setSingleStep(0.1)
        self.zoom_speed_spin.valueChanged.connect(self.save_settings)
        zoom_layout.addRow(self.zoom_speed_label, self.zoom_speed_spin)

        self.zoom_int_label = QLabel()
        self.zoom_int_spin = QDoubleSpinBox()
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
        self.sway_speed_spin = QDoubleSpinBox()
        self.sway_speed_spin.setRange(0.1, 5.0)
        self.sway_speed_spin.setSingleStep(0.1)
        self.sway_speed_spin.valueChanged.connect(self.save_settings)
        sway_layout.addRow(self.sway_speed_label, self.sway_speed_spin)

        self.sway_group.setLayout(sway_layout)
        self.layout.addWidget(self.sway_group)

        self.layout.addStretch()

    def load_settings(self):
        m_settings = self.settings.get("montage", {})

        self.codec_combo.setCurrentText(m_settings.get("codec", "libx264"))
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

    def save_settings(self, *args):
        m_settings = {
            "codec": self.codec_combo.currentText(),
            "preset": self.preset_combo.currentText(),
            "bitrate_mbps": self.bitrate_spin.value(),
            "upscale_factor": self.upscale_spin.value(),
            "enable_transitions": self.enable_trans_cb.isChecked(),
            "transition_duration": self.trans_dur_spin.value(),
            "enable_zoom": self.enable_zoom_cb.isChecked(),
            "zoom_speed_factor": self.zoom_speed_spin.value(),
            "zoom_intensity": self.zoom_int_spin.value(),
            "enable_sway": self.enable_sway_cb.isChecked(),
            "sway_speed_factor": self.sway_speed_spin.value()
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