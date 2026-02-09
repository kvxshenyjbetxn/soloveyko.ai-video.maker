from PySide6.QtWidgets import QWidget, QVBoxLayout, QFormLayout, QComboBox, QLabel, QScrollArea, QPushButton, QLineEdit, QFileDialog, QHBoxLayout, QCheckBox, QGroupBox, QColorDialog, QSpinBox
from PySide6.QtGui import QColor
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from utils.logger import logger
from gui.widgets.setting_row import add_setting_row
import os

class GeneralTab(QWidget):
    def __init__(self, main_window=None, settings_mgr=None, is_template_mode=False):
        super().__init__()
        # main_window is still needed for language/theme change callbacks
        self.main_window = main_window 
        self.settings_manager = settings_mgr or settings_manager
        self.is_template_mode = is_template_mode
        self.init_ui()
        self.update_fields()
        self.update_style()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        main_layout.addWidget(scroll_area)

        scroll_content = QWidget()
        scroll_area.setWidget(scroll_content)

        content_layout = QVBoxLayout(scroll_content)
        form_layout = QFormLayout()

        # Callback for refreshing quick settings panels
        # In template mode, we don't have a quick settings panel to refresh
        if self.is_template_mode:
            refresh_cb = None
        else:
            refresh_cb = self.main_window.refresh_quick_settings_panels if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels') else None

        # Determine if we show stars (only in global mode)
        show_stars = not self.is_template_mode

        # Language selection - Global Only
        if not self.is_template_mode:
            self.language_label = QLabel()
            self.language_combo = QComboBox()
            self.language_combo.addItems(["Українська", "English", "Русский"])
            self.language_combo.currentIndexChanged.connect(self.language_changed)
            add_setting_row(form_layout, self.language_label, self.language_combo, "language", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # Theme selection - Global Only
        if not self.is_template_mode:
            self.theme_label = QLabel()
            self.theme_combo = QComboBox()
            self.theme_combo.addItem(translator.translate('light_theme'), 'light')
            self.theme_combo.addItem(translator.translate('dark_theme'), 'dark')
            self.theme_combo.addItem(translator.translate('black_theme'), 'black')
            self.theme_combo.currentIndexChanged.connect(self.theme_changed)
            add_setting_row(form_layout, self.theme_label, self.theme_combo, "theme", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # Accent color selection - Global Only
        if not self.is_template_mode:
            self.accent_color_label = QLabel()
            self.accent_color_button = QPushButton()
            self.accent_color_button.setFixedSize(100, 25)
            self.accent_color_button.setAutoFillBackground(True)
            self.accent_color_button.clicked.connect(self.open_color_dialog)
            add_setting_row(form_layout, self.accent_color_label, self.accent_color_button, "accent_color", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # Image generation provider selection
        from gui.widgets.help_label import HelpLabel
        self.image_provider_help = HelpLabel("image_generation_provider_label")
        self.image_provider_label = QLabel()
        
        provider_label_widget = QWidget()
        provider_label_layout = QHBoxLayout(provider_label_widget)
        provider_label_layout.setContentsMargins(0, 0, 0, 0)
        provider_label_layout.setSpacing(5)
        provider_label_layout.addWidget(self.image_provider_help)
        provider_label_layout.addWidget(self.image_provider_label)
        
        self.image_provider_combo = QComboBox()
        self.image_provider_combo.addItem("Pollinations", "pollinations")
        self.image_provider_combo.addItem("Googler", "googler")
        self.image_provider_combo.addItem("ElevenLabsImage", "elevenlabs_image")
        self.image_provider_combo.currentIndexChanged.connect(self.image_provider_changed)
        add_setting_row(form_layout, provider_label_widget, self.image_provider_combo, "image_generation_provider", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)

        # Results path selection
        self.results_path_help = HelpLabel("results_path_label")
        self.results_path_label = QLabel()
        
        results_label_widget = QWidget()
        results_label_layout = QHBoxLayout(results_label_widget)
        results_label_layout.setContentsMargins(0, 0, 0, 0)
        results_label_layout.setSpacing(5)
        results_label_layout.addWidget(self.results_path_help)
        results_label_layout.addWidget(self.results_path_label)
        
        self.results_path_edit = QLineEdit()
        self.results_path_edit.setReadOnly(True)
        self.browse_button = QPushButton()
        self.browse_button.clicked.connect(self.browse_results_path)
        
        path_container = QWidget()
        path_layout = QHBoxLayout(path_container)
        path_layout.setContentsMargins(0,0,0,0)
        path_layout.addWidget(self.results_path_edit)
        path_layout.addWidget(self.browse_button)
        
        add_setting_row(form_layout, results_label_widget, path_container, "results_path", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)

        if not self.is_template_mode:
            # Simultaneous Montage and Subtitles
            self.simultaneous_montage_help = HelpLabel("simultaneous_montage_label")
            self.simultaneous_montage_label = QLabel()

            simultaneous_label_widget = QWidget()
            simultaneous_label_layout = QHBoxLayout(simultaneous_label_widget)
            simultaneous_label_layout.setContentsMargins(0, 0, 0, 0)
            simultaneous_label_layout.setSpacing(5)
            simultaneous_label_layout.addWidget(self.simultaneous_montage_help)
            simultaneous_label_layout.addWidget(self.simultaneous_montage_label)

            self.simultaneous_montage_checkbox = QCheckBox()
            self.simultaneous_montage_checkbox.stateChanged.connect(self.simultaneous_montage_changed)
            add_setting_row(form_layout, simultaneous_label_widget, self.simultaneous_montage_checkbox, "simultaneous_montage_and_subs", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # Simulation Target selection
        self.simulation_target_help = HelpLabel("simulation_target_help")
        self.simulation_target_label = QLabel()
        
        simulation_target_widget = QWidget()
        simulation_target_layout = QHBoxLayout(simulation_target_widget)
        simulation_target_layout.setContentsMargins(0, 0, 0, 0)
        simulation_target_layout.setSpacing(5)
        simulation_target_layout.addWidget(self.simulation_target_help)
        simulation_target_layout.addWidget(self.simulation_target_label)

        self.simulation_target_combo = QComboBox()
        self.simulation_target_combo.addItem("DaVinci Resolve Studio", "DaVinci Resolve Studio")
        # Можна додати інші пізніше: Adobe Premiere Pro, Final Cut Pro, None
        self.simulation_target_combo.currentIndexChanged.connect(self.simulation_target_changed)
        
        add_setting_row(form_layout, simulation_target_widget, self.simulation_target_combo, "simulation_target", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)

        # Max download threads
        if not self.is_template_mode:
            self.max_download_threads_help = HelpLabel("max_download_threads_label")
            self.max_download_threads_label = QLabel()
            
            threads_label_widget = QWidget()
            threads_label_layout = QHBoxLayout(threads_label_widget)
            threads_label_layout.setContentsMargins(0, 0, 0, 0)
            threads_label_layout.setSpacing(5)
            threads_label_layout.addWidget(self.max_download_threads_help)
            threads_label_layout.addWidget(self.max_download_threads_label)
            
            self.max_download_threads_spinbox = QSpinBox()
            self.max_download_threads_spinbox.setRange(1, 100)
            self.max_download_threads_spinbox.valueChanged.connect(self.max_download_threads_changed)
            add_setting_row(form_layout, threads_label_widget, self.max_download_threads_spinbox, "max_download_threads", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # Detailed logging checkbox - Global Only or Template?
        # Logging usually global. Let's hide in template mode.
        if not self.is_template_mode:
            self.detailed_logging_help = HelpLabel("detailed_logging_label")
            self.detailed_logging_label = QLabel()
            
            logging_label_widget = QWidget()
            logging_label_layout = QHBoxLayout(logging_label_widget)
            logging_label_layout.setContentsMargins(0, 0, 0, 0)
            logging_label_layout.setSpacing(5)
            logging_label_layout.addWidget(self.detailed_logging_help)
            logging_label_layout.addWidget(self.detailed_logging_label)
            
            self.detailed_logging_checkbox = QCheckBox()
            self.detailed_logging_checkbox.stateChanged.connect(self.detailed_logging_changed)
            add_setting_row(form_layout, logging_label_widget, self.detailed_logging_checkbox, "detailed_logging_enabled", quick_panel_refresh_callback=refresh_cb, show_star=False)

        # --- Controls Group ---
        self.controls_group = QGroupBox()
        self.controls_layout = QFormLayout(self.controls_group)

        # Translation review checkbox
        self.translation_review_help = HelpLabel("translation_review_label")
        self.translation_review_label = QLabel()
        
        translation_label_widget = QWidget()
        translation_label_layout = QHBoxLayout(translation_label_widget)
        translation_label_layout.setContentsMargins(0, 0, 0, 0)
        translation_label_layout.setSpacing(5)
        translation_label_layout.addWidget(self.translation_review_help)
        translation_label_layout.addWidget(self.translation_review_label)
        
        self.translation_review_checkbox = QCheckBox()
        self.translation_review_checkbox.stateChanged.connect(self.translation_review_changed)
        add_setting_row(self.controls_layout, translation_label_widget, self.translation_review_checkbox, "translation_review_enabled", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)

        # Rewrite review checkbox
        self.rewrite_review_help = HelpLabel("rewrite_review_label")
        self.rewrite_review_label = QLabel()
        
        rewrite_label_widget = QWidget()
        rewrite_label_layout = QHBoxLayout(rewrite_label_widget)
        rewrite_label_layout.setContentsMargins(0, 0, 0, 0)
        rewrite_label_layout.setSpacing(5)
        rewrite_label_layout.addWidget(self.rewrite_review_help)
        rewrite_label_layout.addWidget(self.rewrite_review_label)
        
        self.rewrite_review_checkbox = QCheckBox()
        self.rewrite_review_checkbox.stateChanged.connect(self.rewrite_review_changed)
        add_setting_row(self.controls_layout, rewrite_label_widget, self.rewrite_review_checkbox, "rewrite_review_enabled", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)


        # Image review checkbox
        self.image_review_help = HelpLabel("image_review_label")
        self.image_review_label = QLabel()
        
        image_label_widget = QWidget()
        image_label_layout = QHBoxLayout(image_label_widget)
        image_label_layout.setContentsMargins(0, 0, 0, 0)
        image_label_layout.setSpacing(5)
        image_label_layout.addWidget(self.image_review_help)
        image_label_layout.addWidget(self.image_review_label)
        
        self.image_review_checkbox = QCheckBox()
        self.image_review_checkbox.stateChanged.connect(self.image_review_changed)
        add_setting_row(self.controls_layout, image_label_widget, self.image_review_checkbox, "image_review_enabled", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)
        
        # Prompt count control checkbox
        self.prompt_count_control_help = HelpLabel("prompt_count_control_label")
        self.prompt_count_control_label = QLabel()
        
        prompt_control_label_widget = QWidget()
        prompt_control_label_layout = QHBoxLayout(prompt_control_label_widget)
        prompt_control_label_layout.setContentsMargins(0, 0, 0, 0)
        prompt_control_label_layout.setSpacing(5)
        prompt_control_label_layout.addWidget(self.prompt_count_control_help)
        prompt_control_label_layout.addWidget(self.prompt_count_control_label)
        
        self.prompt_count_control_checkbox = QCheckBox()
        self.prompt_count_control_checkbox.stateChanged.connect(self.prompt_count_control_changed)
        add_setting_row(self.controls_layout, prompt_control_label_widget, self.prompt_count_control_checkbox, "prompt_count_control_enabled", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)
        
        # Prompt count spinbox
        self.prompt_count_label = QLabel()
        self.prompt_count_spinbox = QSpinBox()
        self.prompt_count_spinbox.setRange(1, 100)
        self.prompt_count_spinbox.valueChanged.connect(self.prompt_count_changed)
        
        # Widget for the field side (spinbox only)
        self.prompt_count_field_widget = QWidget()
        pc_layout = QHBoxLayout(self.prompt_count_field_widget)
        pc_layout.setContentsMargins(0, 0, 0, 0)
        pc_layout.addWidget(self.prompt_count_spinbox)
        pc_layout.addStretch()
        
        self.prompt_count_container = add_setting_row(
            self.controls_layout, 
            self.prompt_count_label, 
            self.prompt_count_field_widget, 
            "prompt_count", 
            quick_panel_refresh_callback=refresh_cb,
            show_star=False 
        )
        
        form_layout.addRow(self.controls_group)

        # --- Cleanup Group ---
        self.cleanup_group = QGroupBox()
        self.cleanup_layout = QFormLayout(self.cleanup_group)

        # Master toggle
        self.auto_cleanup_label = QLabel()
        self.auto_cleanup_help = HelpLabel("auto_cleanup_help")
        
        cleanup_label_widget = QWidget()
        cleanup_label_layout = QHBoxLayout(cleanup_label_widget)
        cleanup_label_layout.setContentsMargins(0, 0, 0, 0)
        cleanup_label_layout.setSpacing(5)
        cleanup_label_layout.addWidget(self.auto_cleanup_help)
        cleanup_label_layout.addWidget(self.auto_cleanup_label)

        self.auto_cleanup_checkbox = QCheckBox()
        self.auto_cleanup_checkbox.stateChanged.connect(self.auto_cleanup_changed)
        add_setting_row(self.cleanup_layout, cleanup_label_widget, self.auto_cleanup_checkbox, "auto_cleanup_enabled", quick_panel_refresh_callback=refresh_cb, show_star=show_stars)

        # Individual files checkboxes
        self.cleanup_files_layout = QVBoxLayout() # Sub-layout for files
        
        self.cleanup_images_cb = QCheckBox("images folder")
        self.cleanup_images_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_images', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_images_cb)

        self.cleanup_prompts_cb = QCheckBox("image_prompts.txt")
        self.cleanup_prompts_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_image_prompts', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_prompts_cb)

        self.cleanup_translation_cb = QCheckBox("translation.txt")
        self.cleanup_translation_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_translation', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_translation_cb)

        self.cleanup_translation_orig_cb = QCheckBox("translation_orig.txt")
        self.cleanup_translation_orig_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_translation_orig', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_translation_orig_cb)

        self.cleanup_voice_ass_cb = QCheckBox("voice.ass")
        self.cleanup_voice_ass_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_voice_ass', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_voice_ass_cb)

        self.cleanup_voice_mp3_cb = QCheckBox("voice.mp3")
        self.cleanup_voice_mp3_cb.stateChanged.connect(lambda s: self.settings_manager.set('cleanup_voice_audio', s == Qt.CheckState.Checked.value))
        self.cleanup_files_layout.addWidget(self.cleanup_voice_mp3_cb)

        # Wrap sub-layout in a widget to add to form layout
        self.cleanup_files_widget = QWidget()
        self.cleanup_files_widget.setLayout(self.cleanup_files_layout)
        
        # We add it as a full row or under the master toggle? 
        # Better: add it as a row with empty label to indent, or just add to layout.
        # But `add_setting_row` works with FormLayout rows. 
        # Let's just add it to the cleanup_layout directly.
        self.cleanup_layout.addRow(self.cleanup_files_widget)

        form_layout.addRow(self.cleanup_group)

        content_layout.addLayout(form_layout)
        content_layout.addStretch()
        
        # Do not call retranslate_ui here automatically if it depends on global translator which might need refreshing?
        # Actually it's fine.
        self.retranslate_ui()

    def update_style(self):
        border_color = os.environ.get('QTMATERIAL_SECONDARYLIGHTCOLOR', '#e0e0e0')
        accent_color = self.settings_manager.get('accent_color', '#3f51b5')
        if hasattr(self, 'accent_color_button'): # Might not exist in template mode
             self.accent_color_button.setStyleSheet(f"""
                QPushButton {{
                    background-color: {accent_color};
                    border: 1px solid {border_color};
                }}
            """)

    def update_fields(self):
        # Block signals to prevent triggering save on programmatic changes
        if hasattr(self, 'language_combo'): self.language_combo.blockSignals(True)
        if hasattr(self, 'theme_combo'): self.theme_combo.blockSignals(True)
        self.image_provider_combo.blockSignals(True)
        self.translation_review_checkbox.blockSignals(True)
        self.image_review_checkbox.blockSignals(True)
        if hasattr(self, 'simultaneous_montage_checkbox'): self.simultaneous_montage_checkbox.blockSignals(True)
        self.simulation_target_combo.blockSignals(True)
        if hasattr(self, 'detailed_logging_checkbox'): self.detailed_logging_checkbox.blockSignals(True)
        self.prompt_count_control_checkbox.blockSignals(True)
        self.prompt_count_spinbox.blockSignals(True)
        if hasattr(self, 'max_download_threads_spinbox'): self.max_download_threads_spinbox.blockSignals(True)

        if hasattr(self, 'language_combo'):
            lang_map = {"uk": 0, "en": 1, "ru": 2}
            current_lang = self.settings_manager.get('language')
            self.language_combo.setCurrentIndex(lang_map.get(current_lang, 0))

        if hasattr(self, 'theme_combo'):
            current_theme = self.settings_manager.get('theme', 'light')
            self.theme_combo.setCurrentIndex(self.theme_combo.findData(current_theme))

        current_provider = self.settings_manager.get('image_generation_provider', 'pollinations')
        self.image_provider_combo.setCurrentIndex(self.image_provider_combo.findData(current_provider))

        # Handle path - if None, use empty string for display
        res_path = self.settings_manager.get('results_path') or ""
        self.results_path_edit.setText(res_path)

        self.translation_review_checkbox.setChecked(self.settings_manager.get('translation_review_enabled', False))
        self.rewrite_review_checkbox.setChecked(self.settings_manager.get('rewrite_review_enabled', False))
        self.image_review_checkbox.setChecked(self.settings_manager.get('image_review_enabled', False))
        if hasattr(self, 'simultaneous_montage_checkbox'):
            self.simultaneous_montage_checkbox.setChecked(bool(self.settings_manager.get('simultaneous_montage_and_subs', False)))
        
        current_sim_target = self.settings_manager.get('simulation_target', 'DaVinci Resolve Studio')
        idx = self.simulation_target_combo.findData(current_sim_target)
        if idx >= 0:
            self.simulation_target_combo.setCurrentIndex(idx)
        else:
            self.simulation_target_combo.setCurrentIndex(0)

        if hasattr(self, 'detailed_logging_checkbox'):
            self.detailed_logging_checkbox.setChecked(self.settings_manager.get('detailed_logging_enabled', False))
        
        prompt_control_enabled = self.settings_manager.get('prompt_count_control_enabled', False)
        self.prompt_count_control_checkbox.setChecked(prompt_control_enabled)
        
        if hasattr(self, 'prompt_count_label'):
             self.prompt_count_label.setVisible(prompt_control_enabled)
        if hasattr(self, 'prompt_count_container'):
             self.prompt_count_container.setVisible(prompt_control_enabled)
             
        self.prompt_count_spinbox.setValue(int(self.settings_manager.get('prompt_count', 50) or 50))
        if hasattr(self, 'max_download_threads_spinbox'):
            self.max_download_threads_spinbox.setValue(int(self.settings_manager.get('max_download_threads', 5) or 5))

        self.update_style() 

        # Unblock signals
        if hasattr(self, 'language_combo'): self.language_combo.blockSignals(False)
        if hasattr(self, 'theme_combo'): self.theme_combo.blockSignals(False)
        self.image_provider_combo.blockSignals(False)
        self.translation_review_checkbox.blockSignals(False)
        self.image_review_checkbox.blockSignals(False)
        if hasattr(self, 'simultaneous_montage_checkbox'): self.simultaneous_montage_checkbox.blockSignals(False)
        self.simulation_target_combo.blockSignals(False)
        if hasattr(self, 'detailed_logging_checkbox'): self.detailed_logging_checkbox.blockSignals(False)
        self.prompt_count_control_checkbox.blockSignals(False)
        self.prompt_count_spinbox.blockSignals(False)
        if hasattr(self, 'max_download_threads_spinbox'): self.max_download_threads_spinbox.blockSignals(False)

        # Cleanup settings update
        self.auto_cleanup_checkbox.blockSignals(True)
        auto_cleanup_enabled = self.settings_manager.get('auto_cleanup_enabled', False)
        self.auto_cleanup_checkbox.setChecked(auto_cleanup_enabled)
        self.cleanup_files_widget.setVisible(auto_cleanup_enabled)
        self.auto_cleanup_checkbox.blockSignals(False)
        
        self.cleanup_images_cb.blockSignals(True)
        self.cleanup_images_cb.setChecked(self.settings_manager.get('cleanup_images', False))
        self.cleanup_images_cb.blockSignals(False)

        self.cleanup_prompts_cb.blockSignals(True)
        self.cleanup_prompts_cb.setChecked(self.settings_manager.get('cleanup_image_prompts', False))
        self.cleanup_prompts_cb.blockSignals(False)

        self.cleanup_translation_cb.blockSignals(True)
        self.cleanup_translation_cb.setChecked(self.settings_manager.get('cleanup_translation', False))
        self.cleanup_translation_cb.blockSignals(False)

        self.cleanup_translation_orig_cb.blockSignals(True)
        self.cleanup_translation_orig_cb.setChecked(self.settings_manager.get('cleanup_translation_orig', False))
        self.cleanup_translation_orig_cb.blockSignals(False)

        self.cleanup_voice_ass_cb.blockSignals(True)
        self.cleanup_voice_ass_cb.setChecked(self.settings_manager.get('cleanup_voice_ass', False))
        self.cleanup_voice_ass_cb.blockSignals(False)

        self.cleanup_voice_mp3_cb.blockSignals(True)
        self.cleanup_voice_mp3_cb.setChecked(self.settings_manager.get('cleanup_voice_audio', False))
        self.cleanup_voice_mp3_cb.blockSignals(False)


    def open_color_dialog(self):
        current_color = self.settings_manager.get('accent_color', '#3f51b5')
        color = QColorDialog.getColor(QColor(current_color), self, translator.translate("pick_accent_color"))

        if color.isValid():
            color_hex = color.name()
            self.settings_manager.set('accent_color', color_hex)
            self.update_style()
            if self.main_window and hasattr(self.main_window, 'change_accent_color'):
                self.main_window.change_accent_color(color_hex)

    def translation_review_changed(self, state):
        self.settings_manager.set('translation_review_enabled', state == Qt.CheckState.Checked.value)
        if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels'):
            self.main_window.refresh_quick_settings_panels()

    def rewrite_review_changed(self, state):
        self.settings_manager.set('rewrite_review_enabled', state == Qt.CheckState.Checked.value)
        if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels'):
            self.main_window.refresh_quick_settings_panels()

    def image_review_changed(self, state):
        self.settings_manager.set('image_review_enabled', state == Qt.CheckState.Checked.value)
        if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels'):
            self.main_window.refresh_quick_settings_panels()

    def simultaneous_montage_changed(self, state):
        self.settings_manager.set('simultaneous_montage_and_subs', state == Qt.CheckState.Checked.value)

    def simulation_target_changed(self, index):
        target = self.simulation_target_combo.itemData(index)
        self.settings_manager.set('simulation_target', target)

    def auto_cleanup_changed(self, state):
        is_checked = state == Qt.CheckState.Checked.value
        self.settings_manager.set('auto_cleanup_enabled', is_checked)
        self.cleanup_files_widget.setVisible(is_checked)

    def prompt_count_control_changed(self, state):
        is_checked = state == Qt.CheckState.Checked.value
        self.settings_manager.set('prompt_count_control_enabled', is_checked)
        
        # Toggle visibility
        if hasattr(self, 'prompt_count_label'):
             self.prompt_count_label.setVisible(is_checked)
        if hasattr(self, 'prompt_count_container'):
             self.prompt_count_container.setVisible(is_checked)
             
        # Notify other tabs or refresh logic
        # For template mode, maybe just local update is enough as tabs aren't interconnected the same way?
        # But prompts_tab might be present.
        if self.main_window and hasattr(self.main_window, 'settings_tab') and hasattr(self.main_window.settings_tab, 'prompts_tab'):
             self.main_window.settings_tab.prompts_tab.update_fields()


    def prompt_count_changed(self, value):
        self.settings_manager.set('prompt_count', value)
        if self.main_window:
            if hasattr(self.main_window, 'settings_tab') and hasattr(self.main_window.settings_tab, 'prompts_tab'):
                self.main_window.settings_tab.prompts_tab.update_fields()
            if hasattr(self.main_window, 'refresh_quick_settings_panels'):
                self.main_window.refresh_quick_settings_panels()

    def detailed_logging_changed(self, state):
        self.settings_manager.set('detailed_logging_enabled', state == Qt.CheckState.Checked.value)
        logger.reconfigure()

    def max_download_threads_changed(self, value):
        self.settings_manager.set('max_download_threads', value)

    def language_changed(self, index):
        lang_map = {0: "uk", 1: "en", 2: "ru"}
        lang_code = lang_map.get(index, "uk")
        if self.main_window and hasattr(self.main_window, 'change_language'):
            self.main_window.change_language(lang_code)

    def theme_changed(self, index):
        theme_name = self.theme_combo.itemData(index)
        if self.main_window and hasattr(self.main_window, 'change_theme'):
            self.main_window.change_theme(theme_name)
    
    def image_provider_changed(self, index):
        provider_name = self.image_provider_combo.itemData(index)
        self.settings_manager.set('image_generation_provider', provider_name)
        if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels'):
            self.main_window.refresh_quick_settings_panels()

    def browse_results_path(self):
        directory = QFileDialog.getExistingDirectory(self, translator.translate('select_directory'))
        if directory:
            self.results_path_edit.setText(directory)
            self.settings_manager.set('results_path', directory)
            if self.main_window and hasattr(self.main_window, 'refresh_quick_settings_panels'):
                self.main_window.refresh_quick_settings_panels()

    def retranslate_ui(self):
        if hasattr(self, 'language_label'): self.language_label.setText(translator.translate('language_label'))
        if hasattr(self, 'theme_label'): self.theme_label.setText(translator.translate('theme_label'))
        if hasattr(self, 'theme_combo'):
            self.theme_combo.setItemText(0, translator.translate('light_theme'))
            self.theme_combo.setItemText(1, translator.translate('dark_theme'))
            self.theme_combo.setItemText(2, translator.translate('black_theme'))
        self.image_provider_label.setText(translator.translate('image_generation_provider_label'))
        self.image_provider_help.update_tooltip()
        self.results_path_label.setText(translator.translate('results_path_label'))
        self.results_path_help.update_tooltip()
        self.browse_button.setText(translator.translate('browse_button'))
        
        self.controls_group.setTitle(translator.translate('controls_group_title'))
        self.translation_review_label.setText(translator.translate('translation_review_label'))
        self.translation_review_help.update_tooltip()
        self.rewrite_review_label.setText(translator.translate('rewrite_review_label'))
        self.rewrite_review_help.update_tooltip()
        self.image_review_label.setText(translator.translate('image_review_label'))
        self.image_review_help.update_tooltip()
        
        self.cleanup_group.setTitle(translator.translate('cleanup_group_title'))
        self.auto_cleanup_label.setText(translator.translate('auto_cleanup_label'))
        self.auto_cleanup_help.update_tooltip()
        self.cleanup_images_cb.setText(translator.translate('cleanup_images_label'))
        self.cleanup_prompts_cb.setText(translator.translate('cleanup_image_prompts_label'))
        self.cleanup_translation_cb.setText(translator.translate('cleanup_translation_label'))
        self.cleanup_translation_orig_cb.setText(translator.translate('cleanup_translation_orig_label'))
        self.cleanup_voice_ass_cb.setText(translator.translate('cleanup_voice_ass_label'))
        self.cleanup_voice_mp3_cb.setText(translator.translate('cleanup_voice_mp3_label'))

        if hasattr(self, 'simultaneous_montage_label'):
            self.simultaneous_montage_label.setText(translator.translate('simultaneous_montage_label'))
            self.simultaneous_montage_help.update_tooltip()
        self.simulation_target_label.setText(translator.translate('simulation_target_label'))
        self.simulation_target_help.update_tooltip()
        if hasattr(self, 'detailed_logging_label'):
             self.detailed_logging_label.setText(translator.translate('detailed_logging_label'))
             self.detailed_logging_help.update_tooltip()
        if hasattr(self, 'accent_color_label'): self.accent_color_label.setText(translator.translate('accent_color_label'))
        self.prompt_count_control_label.setText(translator.translate('prompt_count_control_label'))
        self.prompt_count_control_help.update_tooltip()
        self.prompt_count_label.setText(translator.translate('prompt_count_label'))
        if hasattr(self, 'max_download_threads_label'):
            self.max_download_threads_label.setText(translator.translate('max_download_threads_label'))
            self.max_download_threads_help.update_tooltip()
