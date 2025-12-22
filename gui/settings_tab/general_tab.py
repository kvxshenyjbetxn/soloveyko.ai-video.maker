from PySide6.QtWidgets import QWidget, QVBoxLayout, QFormLayout, QComboBox, QLabel, QScrollArea, QPushButton, QLineEdit, QFileDialog, QHBoxLayout, QCheckBox, QGroupBox, QColorDialog, QSpinBox
from PySide6.QtGui import QColor
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from utils.logger import logger
import os

class GeneralTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        # main_window is still needed for language/theme change callbacks
        self.main_window = main_window 
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

        # Language selection
        self.language_label = QLabel()
        self.language_combo = QComboBox()
        self.language_combo.addItems(["Українська", "English", "Русский"])
        self.language_combo.currentIndexChanged.connect(self.language_changed)
        form_layout.addRow(self.language_label, self.language_combo)

        # Theme selection
        self.theme_label = QLabel()
        self.theme_combo = QComboBox()
        self.theme_combo.addItem(translator.translate('light_theme'), 'light')
        self.theme_combo.addItem(translator.translate('dark_theme'), 'dark')
        self.theme_combo.addItem(translator.translate('black_theme'), 'black')
        self.theme_combo.currentIndexChanged.connect(self.theme_changed)
        form_layout.addRow(self.theme_label, self.theme_combo)

        # Accent color selection
        self.accent_color_label = QLabel()
        self.accent_color_button = QPushButton()
        self.accent_color_button.setFixedSize(100, 25)
        self.accent_color_button.setAutoFillBackground(True)
        self.accent_color_button.clicked.connect(self.open_color_dialog)
        form_layout.addRow(self.accent_color_label, self.accent_color_button)

        # Image generation provider selection
        self.image_provider_label = QLabel()
        self.image_provider_combo = QComboBox()
        self.image_provider_combo.addItem("Pollinations", "pollinations")
        self.image_provider_combo.addItem("Googler", "googler")
        self.image_provider_combo.currentIndexChanged.connect(self.image_provider_changed)
        form_layout.addRow(self.image_provider_label, self.image_provider_combo)

        # Results path selection
        self.results_path_label = QLabel()
        self.results_path_edit = QLineEdit()
        self.results_path_edit.setReadOnly(True)
        self.browse_button = QPushButton()
        self.browse_button.clicked.connect(self.browse_results_path)
        path_layout = QHBoxLayout()
        path_layout.addWidget(self.results_path_edit)
        path_layout.addWidget(self.browse_button)
        form_layout.addRow(self.results_path_label, path_layout)

        # Detailed logging checkbox
        self.detailed_logging_label = QLabel()
        self.detailed_logging_checkbox = QCheckBox()
        self.detailed_logging_checkbox.stateChanged.connect(self.detailed_logging_changed)
        form_layout.addRow(self.detailed_logging_label, self.detailed_logging_checkbox)

        # --- Controls Group ---
        self.controls_group = QGroupBox()
        self.controls_layout = QFormLayout(self.controls_group)

        # Translation review checkbox
        self.translation_review_label = QLabel()
        self.translation_review_checkbox = QCheckBox()
        self.translation_review_checkbox.stateChanged.connect(self.translation_review_changed)
        self.controls_layout.addRow(self.translation_review_label, self.translation_review_checkbox)

        # Rewrite review checkbox
        self.rewrite_review_label = QLabel()
        self.rewrite_review_checkbox = QCheckBox()
        self.rewrite_review_checkbox.stateChanged.connect(self.rewrite_review_changed)
        self.controls_layout.addRow(self.rewrite_review_label, self.rewrite_review_checkbox)

        # Image review checkbox
        self.image_review_label = QLabel()
        self.image_review_checkbox = QCheckBox()
        self.image_review_checkbox.stateChanged.connect(self.image_review_changed)
        self.controls_layout.addRow(self.image_review_label, self.image_review_checkbox)
        
        # Prompt count control checkbox
        self.prompt_count_control_label = QLabel()
        self.prompt_count_control_checkbox = QCheckBox()
        self.prompt_count_control_checkbox.stateChanged.connect(self.prompt_count_control_changed)
        self.controls_layout.addRow(self.prompt_count_control_label, self.prompt_count_control_checkbox)
        
        # Prompt count spinbox
        self.prompt_count_label = QLabel()
        self.prompt_count_spinbox = QSpinBox()
        self.prompt_count_spinbox.setRange(1, 100)
        self.prompt_count_spinbox.valueChanged.connect(self.prompt_count_changed)
        
        self.prompt_count_widget = QWidget()
        prompt_count_layout = QHBoxLayout(self.prompt_count_widget)
        prompt_count_layout.setContentsMargins(0, 0, 0, 0)
        prompt_count_layout.addWidget(self.prompt_count_label)
        prompt_count_layout.addWidget(self.prompt_count_spinbox)
        prompt_count_layout.addStretch()
        
        self.controls_layout.addRow(self.prompt_count_widget)

        # Max download threads
        self.max_download_threads_label = QLabel()
        self.max_download_threads_spinbox = QSpinBox()
        self.max_download_threads_spinbox.setRange(1, 100)
        self.max_download_threads_spinbox.valueChanged.connect(self.max_download_threads_changed)
        self.controls_layout.addRow(self.max_download_threads_label, self.max_download_threads_spinbox)
        
        form_layout.addRow(self.controls_group)

        content_layout.addLayout(form_layout)
        content_layout.addStretch()
        self.retranslate_ui()

    def update_style(self):
        border_color = os.environ.get('QTMATERIAL_SECONDARYLIGHTCOLOR', '#e0e0e0')
        accent_color = settings_manager.get('accent_color', '#3f51b5')
        self.accent_color_button.setStyleSheet(f"""
            QPushButton {{
                background-color: {accent_color};
                border: 1px solid {border_color};
            }}
        """)

    def update_fields(self):
        # Block signals to prevent triggering save on programmatic changes
        self.language_combo.blockSignals(True)
        self.theme_combo.blockSignals(True)
        self.image_provider_combo.blockSignals(True)
        self.translation_review_checkbox.blockSignals(True)
        self.image_review_checkbox.blockSignals(True)
        self.detailed_logging_checkbox.blockSignals(True)
        self.prompt_count_control_checkbox.blockSignals(True)
        self.prompt_count_spinbox.blockSignals(True)
        self.max_download_threads_spinbox.blockSignals(True)

        lang_map = {"uk": 0, "en": 1, "ru": 2}
        current_lang = settings_manager.get('language')
        self.language_combo.setCurrentIndex(lang_map.get(current_lang, 0))

        current_theme = settings_manager.get('theme', 'light')
        self.theme_combo.setCurrentIndex(self.theme_combo.findData(current_theme))

        current_provider = settings_manager.get('image_generation_provider', 'pollinations')
        self.image_provider_combo.setCurrentIndex(self.image_provider_combo.findData(current_provider))

        self.results_path_edit.setText(settings_manager.get('results_path'))
        self.translation_review_checkbox.setChecked(settings_manager.get('translation_review_enabled', False))
        self.rewrite_review_checkbox.setChecked(settings_manager.get('rewrite_review_enabled', False))
        self.image_review_checkbox.setChecked(settings_manager.get('image_review_enabled', False))
        self.detailed_logging_checkbox.setChecked(settings_manager.get('detailed_logging_enabled', False))
        
        prompt_control_enabled = settings_manager.get('prompt_count_control_enabled', False)
        self.prompt_count_control_checkbox.setChecked(prompt_control_enabled)
        self.prompt_count_widget.setVisible(prompt_control_enabled)
        self.prompt_count_spinbox.setValue(settings_manager.get('prompt_count', 10))
        self.max_download_threads_spinbox.setValue(settings_manager.get('max_download_threads', 5))

        self.update_style() # Set button color and border

        # Unblock signals
        self.language_combo.blockSignals(False)
        self.theme_combo.blockSignals(False)
        self.image_provider_combo.blockSignals(False)
        self.translation_review_checkbox.blockSignals(False)
        self.image_review_checkbox.blockSignals(False)
        self.detailed_logging_checkbox.blockSignals(False)
        self.prompt_count_control_checkbox.blockSignals(False)
        self.prompt_count_spinbox.blockSignals(False)
        self.max_download_threads_spinbox.blockSignals(False)


    def open_color_dialog(self):
        current_color = settings_manager.get('accent_color', '#3f51b5')
        color = QColorDialog.getColor(QColor(current_color), self, translator.translate("pick_accent_color"))

        if color.isValid():
            color_hex = color.name()
            settings_manager.set('accent_color', color_hex)
            self.update_style()
            if self.main_window:
                self.main_window.change_accent_color(color_hex)

    def translation_review_changed(self, state):
        settings_manager.set('translation_review_enabled', state == Qt.CheckState.Checked.value)

    def rewrite_review_changed(self, state):
        settings_manager.set('rewrite_review_enabled', state == Qt.CheckState.Checked.value)

    def image_review_changed(self, state):
        settings_manager.set('image_review_enabled', state == Qt.CheckState.Checked.value)

    def prompt_count_control_changed(self, state):
        is_checked = state == Qt.CheckState.Checked.value
        settings_manager.set('prompt_count_control_enabled', is_checked)
        self.prompt_count_widget.setVisible(is_checked)
        # We might need to inform other tabs about this change.
        # A signal from settings_manager or a direct call could work.
        if self.main_window:
            if hasattr(self.main_window, 'settings_tab') and hasattr(self.main_window.settings_tab, 'prompts_tab'):
                self.main_window.settings_tab.prompts_tab.update_fields()


    def prompt_count_changed(self, value):
        settings_manager.set('prompt_count', value)
        if self.main_window:
            if hasattr(self.main_window, 'settings_tab') and hasattr(self.main_window.settings_tab, 'prompts_tab'):
                self.main_window.settings_tab.prompts_tab.update_fields()

    def detailed_logging_changed(self, state):
        settings_manager.set('detailed_logging_enabled', state == Qt.CheckState.Checked.value)
        logger.reconfigure()

    def max_download_threads_changed(self, value):
        settings_manager.set('max_download_threads', value)

    def language_changed(self, index):
        lang_map = {0: "uk", 1: "en", 2: "ru"}
        lang_code = lang_map.get(index, "uk")
        if self.main_window:
            self.main_window.change_language(lang_code)

    def theme_changed(self, index):
        theme_name = self.theme_combo.itemData(index)
        if self.main_window:
            self.main_window.change_theme(theme_name)
    
    def image_provider_changed(self, index):
        provider_name = self.image_provider_combo.itemData(index)
        settings_manager.set('image_generation_provider', provider_name)

    def browse_results_path(self):
        directory = QFileDialog.getExistingDirectory(self, translator.translate('select_directory'))
        if directory:
            self.results_path_edit.setText(directory)
            settings_manager.set('results_path', directory)

    def retranslate_ui(self):
        self.language_label.setText(translator.translate('language_label'))
        self.theme_label.setText(translator.translate('theme_label'))
        self.theme_combo.setItemText(0, translator.translate('light_theme'))
        self.theme_combo.setItemText(1, translator.translate('dark_theme'))
        self.theme_combo.setItemText(2, translator.translate('black_theme'))
        self.image_provider_label.setText(translator.translate('image_generation_provider_label'))
        self.results_path_label.setText(translator.translate('results_path_label'))
        self.browse_button.setText(translator.translate('browse_button'))
        self.controls_group.setTitle(translator.translate('controls_group_title'))
        self.translation_review_label.setText(translator.translate('translation_review_label'))
        self.rewrite_review_label.setText(translator.translate('rewrite_review_label'))
        self.image_review_label.setText(translator.translate('image_review_label'))
        self.detailed_logging_label.setText(translator.translate('detailed_logging_label'))
        self.accent_color_label.setText(translator.translate('accent_color_label'))
        self.prompt_count_control_label.setText(translator.translate('prompt_count_control_label'))
        self.prompt_count_label.setText(translator.translate('prompt_count_label'))
        self.max_download_threads_label.setText(translator.translate('max_download_threads_label'))
