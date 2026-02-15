from PySide6.QtWidgets import (QWidget, QVBoxLayout, QScrollArea,
                               QCheckBox, QDoubleSpinBox, QComboBox, QSpinBox,
                               QFormLayout, QGroupBox, QLabel, QHBoxLayout)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from gui.widgets.slider_spinbox import SliderWithSpinBox
from gui.widgets.help_label import HelpLabel
from gui.widgets.setting_row import add_setting_row

class MontageTab(QWidget):
    def __init__(self, settings_mgr=None, is_template_mode=False):
        super().__init__()
        self.settings = settings_mgr or settings_manager
        self.is_template_mode = is_template_mode
        self.transition_effects = [
            "random", "fade", "wipeleft", "wiperight", "wipeup", "wipedown", 
            "slideleft", "slideright", "slideup", "slidedown", "circlecrop", 
            "rectcrop", "distance", "fadeblack", "fadewhite", "radial", 
            "smoothleft", "smoothright", "smoothup", "smoothdown", 
            "circleopen", "circleclose", "vertopen", "vertclose", 
            "horzopen", "horzclose", "dissolve", "pixelize", "diagtl", 
            "diagtr", "diagbl", "diagbr", "hlslice", "hrslice", "vu"
        ]
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

        def refresh_quick_panel():
            if self.window():
                 if hasattr(self.window(), 'refresh_quick_settings_panels'):
                      self.window().refresh_quick_settings_panels()

        # Determine if we show stars
        show_stars = not self.is_template_mode

        # --- Render Settings ---
        self.render_group = QGroupBox()
        render_layout = QFormLayout()

        self.codec_help = HelpLabel("codec_label")
        self.codec_label = QLabel()
        codec_label_container = QWidget()
        codec_label_layout = QHBoxLayout(codec_label_container)
        codec_label_layout.setContentsMargins(0,0,0,0)
        codec_label_layout.setSpacing(5)
        codec_label_layout.addWidget(self.codec_help)
        codec_label_layout.addWidget(self.codec_label)

        self.codec_combo = QComboBox()
        self.codec_combo.addItem("libx264 (CPU)", "libx264")
        self.codec_combo.addItem("libx265 (CPU)", "libx265")
        self.codec_combo.addItem("h264_nvenc (NVIDIA)", "h264_nvenc")
        self.codec_combo.addItem("h264_amf (AMD)", "h264_amf")
        self.codec_combo.addItem("h264_videotoolbox (Mac)", "h264_videotoolbox")
        
        self.codec_combo.currentIndexChanged.connect(self.save_settings)
        add_setting_row(render_layout, codec_label_container, self.codec_combo, "montage.codec", refresh_quick_panel, show_star=show_stars)


        self.preset_help = HelpLabel("preset_label")
        self.preset_label = QLabel()
        preset_label_container = QWidget()
        preset_label_layout = QHBoxLayout(preset_label_container)
        preset_label_layout.setContentsMargins(0,0,0,0)
        preset_label_layout.setSpacing(5)
        preset_label_layout.addWidget(self.preset_help)
        preset_label_layout.addWidget(self.preset_label)

        self.preset_combo = QComboBox()
        self.preset_combo.addItems(["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"])
        self.preset_combo.currentTextChanged.connect(self.save_settings)
        add_setting_row(render_layout, preset_label_container, self.preset_combo, "montage.preset", refresh_quick_panel, show_star=show_stars)


        self.bitrate_help = HelpLabel("bitrate_label")
        self.bitrate_label = QLabel()
        bitrate_label_container = QWidget()
        bitrate_label_layout = QHBoxLayout(bitrate_label_container)
        bitrate_label_layout.setContentsMargins(0,0,0,0)
        bitrate_label_layout.setSpacing(5)
        bitrate_label_layout.addWidget(self.bitrate_help)
        bitrate_label_layout.addWidget(self.bitrate_label)

        self.bitrate_spin = QSpinBox()
        self.bitrate_spin.setRange(1, 100)
        self.bitrate_spin.setSuffix(" Mbps")
        self.bitrate_spin.valueChanged.connect(self.save_settings)
        add_setting_row(render_layout, bitrate_label_container, self.bitrate_spin, "montage.bitrate_mbps", refresh_quick_panel, show_star=show_stars)
        
        # --- NEW GPU SHADERS & QUALITY ---
        
        if not self.is_template_mode:
            self.use_gpu_help = HelpLabel("use_gpu_shaders_label")
            self.use_gpu_cb = QCheckBox()
            self.use_gpu_label = QLabel()
            use_gpu_container = QWidget()
            use_gpu_layout = QHBoxLayout(use_gpu_container)
            use_gpu_layout.setContentsMargins(0,0,0,0)
            use_gpu_layout.setSpacing(5)
            use_gpu_layout.addWidget(self.use_gpu_help)
            use_gpu_layout.addWidget(self.use_gpu_label)
            use_gpu_layout.addWidget(self.use_gpu_cb)
            use_gpu_layout.addStretch()
            
            self.use_gpu_cb.toggled.connect(self.save_settings)
            add_setting_row(render_layout, None, use_gpu_container, "montage.use_gpu_shaders", refresh_quick_panel, show_star=show_stars)


            self.quality_help = HelpLabel("video_quality_label")
            self.quality_label = QLabel()
            quality_label_container = QWidget()
            quality_label_layout = QHBoxLayout(quality_label_container)
            quality_label_layout.setContentsMargins(0,0,0,0)
            quality_label_layout.setSpacing(5)
            quality_label_layout.addWidget(self.quality_help)
            quality_label_layout.addWidget(self.quality_label)

            self.quality_combo = QComboBox()
            self.quality_combo.addItems(["speed", "balanced", "quality"])
            self.quality_combo.currentTextChanged.connect(self.save_settings)
            add_setting_row(render_layout, quality_label_container, self.quality_combo, "montage.video_quality", refresh_quick_panel, show_star=show_stars)

        # ---------------------------------


        self.upscale_help = HelpLabel("upscale_factor_label")
        self.upscale_label = QLabel()
        upscale_label_container = QWidget()
        upscale_label_layout = QHBoxLayout(upscale_label_container)
        upscale_label_layout.setContentsMargins(0,0,0,0)
        upscale_label_layout.setSpacing(5)
        upscale_label_layout.addWidget(self.upscale_help)
        upscale_label_layout.addWidget(self.upscale_label)

        self.upscale_spin = QDoubleSpinBox()
        self.upscale_spin.setRange(1.0, 5.0)
        self.upscale_spin.setSingleStep(0.1)
        self.upscale_spin.setSuffix("x")
        self.upscale_spin.valueChanged.connect(self.save_settings)
        add_setting_row(render_layout, upscale_label_container, self.upscale_spin, "montage.upscale_factor", refresh_quick_panel, show_star=show_stars)


        self.render_group.setLayout(render_layout)
        self.layout.addWidget(self.render_group)

        # --- Transitions Settings ---
        self.trans_group = QGroupBox()
        trans_layout = QFormLayout()

        self.enable_trans_help = HelpLabel("enable_transitions_label")
        self.enable_trans_cb = QCheckBox()
        self.enable_trans_label = QLabel()
        enable_trans_container = QWidget()
        enable_trans_layout = QHBoxLayout(enable_trans_container)
        enable_trans_layout.setContentsMargins(0,0,0,0)
        enable_trans_layout.setSpacing(5)
        enable_trans_layout.addWidget(self.enable_trans_help)
        enable_trans_layout.addWidget(self.enable_trans_label)
        enable_trans_layout.addWidget(self.enable_trans_cb)
        enable_trans_layout.addStretch()
        
        self.enable_trans_cb.toggled.connect(self.save_settings)
        add_setting_row(trans_layout, None, enable_trans_container, "montage.enable_transitions", refresh_quick_panel, show_star=show_stars)


        self.trans_effect_help = HelpLabel("transition_effect_label")
        self.trans_effect_label = QLabel()
        trans_effect_container = QWidget()
        trans_effect_layout = QHBoxLayout(trans_effect_container)
        trans_effect_layout.setContentsMargins(0,0,0,0)
        trans_effect_layout.setSpacing(5)
        trans_effect_layout.addWidget(self.trans_effect_help)
        trans_effect_layout.addWidget(self.trans_effect_label)

        self.trans_effect_combo = QComboBox()
        for effect in self.transition_effects:
            self.trans_effect_combo.addItem(effect, effect) # Label, Data
        
        self.trans_effect_combo.currentIndexChanged.connect(self.save_settings)
        self.trans_effect_combo.currentIndexChanged.connect(self.update_trans_description)
        add_setting_row(trans_layout, trans_effect_container, self.trans_effect_combo, "montage.transition_effect", refresh_quick_panel, show_star=show_stars)


        # Description Label
        self.trans_desc_label = QLabel()
        self.trans_desc_label.setWordWrap(True)
        self.trans_desc_label.setStyleSheet("color: #888; font-style: italic; font-size: 11px;")
        trans_layout.addRow("", self.trans_desc_label)

        self.trans_dur_help = HelpLabel("duration_label")
        self.trans_dur_label = QLabel()
        trans_dur_container = QWidget()
        trans_dur_layout = QHBoxLayout(trans_dur_container)
        trans_dur_layout.setContentsMargins(0,0,0,0)
        trans_dur_layout.setSpacing(5)
        trans_dur_layout.addWidget(self.trans_dur_help)
        trans_dur_layout.addWidget(self.trans_dur_label)

        self.trans_dur_spin = SliderWithSpinBox()
        self.trans_dur_spin.setRange(0.1, 5.0)
        self.trans_dur_spin.setSingleStep(0.1)
        self.trans_dur_spin.setSuffix(" s")
        self.trans_dur_spin.valueChanged.connect(self.save_settings)
        add_setting_row(trans_layout, trans_dur_container, self.trans_dur_spin, "montage.transition_duration", refresh_quick_panel, show_star=show_stars)


        self.trans_group.setLayout(trans_layout)
        self.layout.addWidget(self.trans_group)

        # --- Zoom Effects ---
        self.zoom_group = QGroupBox()
        zoom_layout = QFormLayout()

        self.enable_zoom_help = HelpLabel("enable_zoom_label")
        self.enable_zoom_cb = QCheckBox()
        self.enable_zoom_label = QLabel()
        enable_zoom_container = QWidget()
        enable_zoom_layout = QHBoxLayout(enable_zoom_container)
        enable_zoom_layout.setContentsMargins(0,0,0,0)
        enable_zoom_layout.setSpacing(5)
        enable_zoom_layout.addWidget(self.enable_zoom_help)
        enable_zoom_layout.addWidget(self.enable_zoom_label)
        enable_zoom_layout.addWidget(self.enable_zoom_cb)
        enable_zoom_layout.addStretch()

        self.enable_zoom_cb.toggled.connect(self.save_settings)
        add_setting_row(zoom_layout, None, enable_zoom_container, "montage.enable_zoom", refresh_quick_panel, show_star=show_stars)


        self.zoom_speed_help = HelpLabel("zoom_speed_factor_label")
        self.zoom_speed_label = QLabel()
        zoom_speed_container = QWidget()
        zoom_speed_layout = QHBoxLayout(zoom_speed_container)
        zoom_speed_layout.setContentsMargins(0,0,0,0)
        zoom_speed_layout.setSpacing(5)
        zoom_speed_layout.addWidget(self.zoom_speed_help)
        zoom_speed_layout.addWidget(self.zoom_speed_label)

        self.zoom_speed_spin = SliderWithSpinBox()
        self.zoom_speed_spin.setRange(0.1, 5.0)
        self.zoom_speed_spin.setSingleStep(0.1)
        self.zoom_speed_spin.valueChanged.connect(self.save_settings)
        add_setting_row(zoom_layout, zoom_speed_container, self.zoom_speed_spin, "montage.zoom_speed_factor", refresh_quick_panel, show_star=show_stars)


        self.zoom_int_help = HelpLabel("zoom_intensity_label")
        self.zoom_int_label = QLabel()
        zoom_int_container = QWidget()
        zoom_int_layout = QHBoxLayout(zoom_int_container)
        zoom_int_layout.setContentsMargins(0,0,0,0)
        zoom_int_layout.setSpacing(5)
        zoom_int_layout.addWidget(self.zoom_int_help)
        zoom_int_layout.addWidget(self.zoom_int_label)

        self.zoom_int_spin = SliderWithSpinBox()
        self.zoom_int_spin.setRange(0.01, 1.0)
        self.zoom_int_spin.setSingleStep(0.05)
        self.zoom_int_spin.valueChanged.connect(self.save_settings)
        add_setting_row(zoom_layout, zoom_int_container, self.zoom_int_spin, "montage.zoom_intensity", refresh_quick_panel, show_star=show_stars)


        self.zoom_group.setLayout(zoom_layout)
        self.layout.addWidget(self.zoom_group)

        # --- Sway Effects ---
        self.sway_group = QGroupBox()
        sway_layout = QFormLayout()

        self.enable_sway_help = HelpLabel("enable_sway_label")
        self.enable_sway_cb = QCheckBox()
        self.enable_sway_label = QLabel()
        enable_sway_container = QWidget()
        enable_sway_layout = QHBoxLayout(enable_sway_container)
        enable_sway_layout.setContentsMargins(0,0,0,0)
        enable_sway_layout.setSpacing(5)
        enable_sway_layout.addWidget(self.enable_sway_help)
        enable_sway_layout.addWidget(self.enable_sway_label)
        enable_sway_layout.addWidget(self.enable_sway_cb)
        enable_sway_layout.addStretch()

        self.enable_sway_cb.toggled.connect(self.save_settings)
        add_setting_row(sway_layout, None, enable_sway_container, "montage.enable_sway", refresh_quick_panel, show_star=show_stars)


        self.sway_speed_help = HelpLabel("sway_speed_factor_label")
        self.sway_speed_label = QLabel()
        sway_speed_container = QWidget()
        sway_speed_layout = QHBoxLayout(sway_speed_container)
        sway_speed_layout.setContentsMargins(0,0,0,0)
        sway_speed_layout.setSpacing(5)
        sway_speed_layout.addWidget(self.sway_speed_help)
        sway_speed_layout.addWidget(self.sway_speed_label)

        self.sway_speed_spin = SliderWithSpinBox()
        self.sway_speed_spin.setRange(0.1, 5.0)
        self.sway_speed_spin.setSingleStep(0.1)
        self.sway_speed_spin.valueChanged.connect(self.save_settings)
        add_setting_row(sway_layout, sway_speed_container, self.sway_speed_spin, "montage.sway_speed_factor", refresh_quick_panel, show_star=show_stars)


        self.sway_group.setLayout(sway_layout)
        self.layout.addWidget(self.sway_group)

        # --- Special Processing ---
        self.special_proc_group = QGroupBox()
        special_proc_layout = QFormLayout()

        self.special_proc_mode_help = HelpLabel("special_proc_mode_label")
        self.special_proc_mode_label = QLabel()
        mode_label_container = QWidget()
        mode_label_layout = QHBoxLayout(mode_label_container)
        mode_label_layout.setContentsMargins(0,0,0,0)
        mode_label_layout.setSpacing(5)
        mode_label_layout.addWidget(self.special_proc_mode_help)
        mode_label_layout.addWidget(self.special_proc_mode_label)

        self.special_proc_mode_combo = QComboBox()
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_disabled"), "Disabled")
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_quick_show"), "Quick show")
        self.special_proc_mode_combo.addItem(translator.translate("special_proc_mode_video_at_beginning"), "Video at the beginning")
        self.special_proc_mode_combo.currentIndexChanged.connect(self.save_settings)
        self.special_proc_mode_combo.currentIndexChanged.connect(self.toggle_special_proc_widgets)
        add_setting_row(special_proc_layout, mode_label_container, self.special_proc_mode_combo, "montage.special_processing_mode", refresh_quick_panel, show_star=show_stars)


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

        self.special_proc_check_sequence_help = HelpLabel("special_proc_check_sequence_label")
        self.special_proc_check_sequence_cb = QCheckBox()
        self.special_proc_check_sequence_label = QLabel()
        self.check_seq_container = QWidget()
        check_seq_layout = QHBoxLayout(self.check_seq_container)
        check_seq_layout.setContentsMargins(0,0,0,0)
        check_seq_layout.setSpacing(5)
        check_seq_layout.addWidget(self.special_proc_check_sequence_help)
        check_seq_layout.addWidget(self.special_proc_check_sequence_label)
        check_seq_layout.addWidget(self.special_proc_check_sequence_cb)
        check_seq_layout.addStretch()

        self.special_proc_check_sequence_cb.toggled.connect(self.save_settings)
        special_proc_layout.addRow(self.check_seq_container)
        
        self.special_proc_group.setLayout(special_proc_layout)
        self.layout.addWidget(self.special_proc_group)

        # --- Performance Settings ---
        self.perf_group = QGroupBox()
        perf_layout = QFormLayout()
        
        if not self.is_template_mode:
            self.max_concurrent_montages_help = HelpLabel("max_concurrent_montages_label")
            self.max_concurrent_montages_label = QLabel()
            max_montages_container = QWidget()
            max_montages_layout = QHBoxLayout(max_montages_container)
            max_montages_layout.setContentsMargins(0,0,0,0)
            max_montages_layout.setSpacing(5)
            max_montages_layout.addWidget(self.max_concurrent_montages_help)
            max_montages_layout.addWidget(self.max_concurrent_montages_label)

            self.max_concurrent_montages_spin = QSpinBox()
            self.max_concurrent_montages_spin.setRange(1, 10)
            self.max_concurrent_montages_spin.valueChanged.connect(self.save_settings)
            add_setting_row(perf_layout, max_montages_container, self.max_concurrent_montages_spin, "montage.max_concurrent_montages", refresh_quick_panel, show_star=show_stars)


        self.perf_group.setLayout(perf_layout)
        self.layout.addWidget(self.perf_group)

        self.layout.addStretch()

    def update_fields(self):
        m_settings = self.settings.get("montage", {})

        for widget in self.findChildren(QWidget):
            if isinstance(widget, (QComboBox, QSpinBox, QDoubleSpinBox, QCheckBox, SliderWithSpinBox)):
                widget.blockSignals(True)

        codec = m_settings.get("codec", "libx264")
        index = self.codec_combo.findData(codec)
        if index != -1:
            self.codec_combo.setCurrentIndex(index)
        else:
             self.codec_combo.setCurrentIndex(0)

        self.preset_combo.setCurrentText(m_settings.get("preset", "medium"))
        self.bitrate_spin.setValue(m_settings.get("bitrate_mbps", 15))
        
        if not self.is_template_mode:
            self.use_gpu_cb.setChecked(m_settings.get("use_gpu_shaders", True))
            self.quality_combo.setCurrentText(m_settings.get("video_quality", "speed"))

        self.upscale_spin.setValue(m_settings.get("upscale_factor", 3.0))

        self.enable_trans_cb.setChecked(m_settings.get("enable_transitions", True))
        
        effect = m_settings.get("transition_effect", "random")
        index = self.trans_effect_combo.findData(effect)
        if index != -1:
            self.trans_effect_combo.setCurrentIndex(index)
        else:
             self.trans_effect_combo.setCurrentIndex(0)
        
        self.update_trans_description()

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

        if not self.is_template_mode:
            self.max_concurrent_montages_spin.setValue(m_settings.get("max_concurrent_montages", 1))

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
            "transition_effect": self.trans_effect_combo.currentData(),
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
        }
        
        if not self.is_template_mode:
            m_settings["video_quality"] = self.quality_combo.currentText()
            m_settings["use_gpu_shaders"] = self.use_gpu_cb.isChecked()
            m_settings["max_concurrent_montages"] = self.max_concurrent_montages_spin.value()
            
        self.settings.set("montage", m_settings)

    def retranslate_ui(self):
        self.render_group.setTitle(translator.translate("render_settings"))
        self.codec_label.setText(translator.translate("codec_label"))
        self.preset_label.setText(translator.translate("preset_label"))
        self.bitrate_label.setText(translator.translate("bitrate_label"))
        if not self.is_template_mode:
            self.use_gpu_label.setText(translator.translate("use_gpu_shaders_label"))
            self.quality_label.setText(translator.translate("video_quality_label"))
        self.upscale_label.setText(translator.translate("upscale_factor_label"))

        self.trans_group.setTitle(translator.translate("transitions_settings"))
        self.enable_trans_label.setText(translator.translate("enable_transitions_label"))
        self.trans_effect_label.setText(translator.translate("transition_effect_label"))
        # Update 'random' label
        random_idx = self.trans_effect_combo.findData("random")
        if random_idx != -1:
            self.trans_effect_combo.setItemText(random_idx, translator.translate("transition_random"))
        
        self.update_trans_description()
        
        self.trans_dur_label.setText(translator.translate("duration_label"))

        self.zoom_group.setTitle(translator.translate("zoom_effects"))
        self.enable_zoom_label.setText(translator.translate("enable_zoom_label"))
        self.zoom_speed_label.setText(translator.translate("zoom_speed_factor_label"))
        self.zoom_int_label.setText(translator.translate("zoom_intensity_label"))

        self.sway_group.setTitle(translator.translate("sway_effects"))
        self.enable_sway_label.setText(translator.translate("enable_sway_label"))
        self.sway_speed_label.setText(translator.translate("sway_speed_factor_label"))

        self.special_proc_group.setTitle(translator.translate("special_processing_group"))
        self.special_proc_mode_label.setText(translator.translate("special_proc_mode_label"))
        self.special_proc_mode_combo.setItemText(0, translator.translate("special_proc_mode_disabled"))
        self.special_proc_mode_combo.setItemText(1, translator.translate("special_proc_mode_quick_show"))
        self.special_proc_mode_combo.setItemText(2, translator.translate("special_proc_mode_video_at_beginning"))
        
        self.special_proc_img_count_label.setText(translator.translate("image_count_label"))
        self.special_proc_dur_label.setText(translator.translate("duration_per_image_label"))
        self.special_proc_video_count_label.setText(translator.translate("special_proc_video_count_label"))
        self.special_proc_check_sequence_label.setText(translator.translate("special_proc_check_sequence_label"))

        if not self.is_template_mode:
            self.perf_group.setTitle(translator.translate("performance_group"))
            self.max_concurrent_montages_label.setText(translator.translate("max_concurrent_montages_label"))

        # Update all hints
        self.codec_help.update_tooltip()
        self.preset_help.update_tooltip()
        self.bitrate_help.update_tooltip()
        if not self.is_template_mode:
            self.use_gpu_help.update_tooltip()
            self.quality_help.update_tooltip()
        self.upscale_help.update_tooltip()
        self.enable_trans_help.update_tooltip()
        self.trans_dur_help.update_tooltip()
        self.enable_zoom_help.update_tooltip()
        self.zoom_speed_help.update_tooltip()
        self.zoom_int_help.update_tooltip()
        self.enable_sway_help.update_tooltip()
        self.sway_speed_help.update_tooltip()
        self.special_proc_mode_help.update_tooltip()
        self.special_proc_check_sequence_help.update_tooltip()
        self.trans_effect_help.update_tooltip()
        if not self.is_template_mode:
            self.max_concurrent_montages_help.update_tooltip()

    def update_trans_description(self):
        effect = self.trans_effect_combo.currentData()
        if effect:
             desc_key = f"trans_desc_{effect}"
             self.trans_desc_label.setText(translator.translate(desc_key, ""))
        else:
             self.trans_desc_label.setText("")

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
        self.check_seq_container.setVisible(is_video)