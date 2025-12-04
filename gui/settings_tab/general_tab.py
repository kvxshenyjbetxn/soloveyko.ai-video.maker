from PySide6.QtWidgets import QWidget, QVBoxLayout, QFormLayout, QComboBox, QLabel, QScrollArea, QPushButton, QLineEdit, QFileDialog, QHBoxLayout, QCheckBox
from PySide6.QtCore import Qt
from utils.translator import translator

class GeneralTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

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
        self.language_label = QLabel(translator.translate('language_label'))
        self.language_combo = QComboBox()
        self.language_combo.addItems(["Українська", "English", "Русский"])
        
        lang_map = {"uk": 0, "en": 1, "ru": 2}
        current_lang = self.main_window.settings_manager.get('language')
        self.language_combo.setCurrentIndex(lang_map.get(current_lang, 0))
        
        self.language_combo.currentIndexChanged.connect(self.language_changed)
        
        form_layout.addRow(self.language_label, self.language_combo)

        # Theme selection
        self.theme_label = QLabel(translator.translate('theme_label'))
        self.theme_combo = QComboBox()
        self.theme_combo.addItem(translator.translate('light_theme'), 'light')
        self.theme_combo.addItem(translator.translate('dark_theme'), 'dark')
        self.theme_combo.addItem(translator.translate('black_theme'), 'black')

        current_theme = self.main_window.settings_manager.get('theme', 'light')
        self.theme_combo.setCurrentIndex(self.theme_combo.findData(current_theme))

        self.theme_combo.currentIndexChanged.connect(self.theme_changed)

        form_layout.addRow(self.theme_label, self.theme_combo)

        # Image generation provider selection
        self.image_provider_label = QLabel(translator.translate('image_generation_provider_label'))
        self.image_provider_combo = QComboBox()
        self.image_provider_combo.addItem("Pollinations", "pollinations")
        self.image_provider_combo.addItem("Googler", "googler")

        current_provider = self.main_window.settings_manager.get('image_generation_provider', 'pollinations')
        self.image_provider_combo.setCurrentIndex(self.image_provider_combo.findData(current_provider))

        self.image_provider_combo.currentIndexChanged.connect(self.image_provider_changed)

        form_layout.addRow(self.image_provider_label, self.image_provider_combo)

        # Results path selection
        self.results_path_label = QLabel(translator.translate('results_path_label'))
        self.results_path_edit = QLineEdit()
        self.results_path_edit.setReadOnly(True)
        self.results_path_edit.setText(self.main_window.settings_manager.get('results_path'))
        
        self.browse_button = QPushButton(translator.translate('browse_button'))
        self.browse_button.clicked.connect(self.browse_results_path)

        path_layout = QHBoxLayout()
        path_layout.addWidget(self.results_path_edit)
        path_layout.addWidget(self.browse_button)

        form_layout.addRow(self.results_path_label, path_layout)

        # Image review checkbox
        self.image_review_label = QLabel(translator.translate('image_review_label'))
        self.image_review_checkbox = QCheckBox()
        self.image_review_checkbox.setChecked(self.main_window.settings_manager.get('image_review_enabled', False))
        self.image_review_checkbox.stateChanged.connect(self.image_review_changed)
        form_layout.addRow(self.image_review_label, self.image_review_checkbox)

        content_layout.addLayout(form_layout)
        content_layout.addStretch()

    def image_review_changed(self, state):
        self.main_window.settings_manager.set('image_review_enabled', state == Qt.CheckState.Checked.value)

    def language_changed(self, index):
        lang_map = {0: "uk", 1: "en", 2: "ru"}
        lang_code = lang_map.get(index, "uk")
        self.main_window.change_language(lang_code)

    def theme_changed(self, index):
        theme_name = self.theme_combo.itemData(index)
        self.main_window.change_theme(theme_name)
    
    def image_provider_changed(self, index):
        provider_name = self.image_provider_combo.itemData(index)
        self.main_window.settings_manager.set('image_generation_provider', provider_name)

    def browse_results_path(self):
        directory = QFileDialog.getExistingDirectory(self, translator.translate('select_directory'))
        if directory:
            self.results_path_edit.setText(directory)
            self.main_window.settings_manager.set('results_path', directory)

    def retranslate_ui(self):
        self.language_label.setText(translator.translate('language_label'))
        self.theme_label.setText(translator.translate('theme_label'))
        self.theme_combo.setItemText(0, translator.translate('light_theme'))
        self.theme_combo.setItemText(1, translator.translate('dark_theme'))
        self.theme_combo.setItemText(2, translator.translate('black_theme'))
        self.image_provider_label.setText(translator.translate('image_generation_provider_label'))
        self.results_path_label.setText(translator.translate('results_path_label'))
        self.browse_button.setText(translator.translate('browse_button'))
        self.image_review_label.setText(translator.translate('image_review_label'))