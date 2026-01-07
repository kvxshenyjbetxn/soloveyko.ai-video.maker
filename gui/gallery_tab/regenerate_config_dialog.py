from PySide6.QtWidgets import QDialog, QVBoxLayout, QFormLayout, QComboBox, QTextEdit, QDialogButtonBox, QWidget, QLineEdit, QSpinBox
from utils.translator import translator
from utils.settings import settings_manager

class RegenerateConfigDialog(QDialog):
    def __init__(self, image_data, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("regenerate_image_title"))
        self.settings = settings_manager
        
        # Extract initial data, providing defaults
        self.initial_provider = image_data.get('provider', self.settings.get('image_generation_provider', 'pollinations'))
        self.initial_prompt = image_data.get('prompt', '')
        self.initial_googler_config = image_data.get('googler_config', self.settings.get('googler', {}))
        self.initial_pollinations_config = image_data.get('pollinations_config', self.settings.get('pollinations', {}))

        # Layouts
        main_layout = QVBoxLayout(self)
        form_layout = QFormLayout()

        # Service selector
        self.service_combo = QComboBox()
        self.service_combo.addItems(['pollinations', 'googler', 'elevenlabs'])
        self.service_combo.setCurrentText(self.initial_provider)
        self.service_combo.currentTextChanged.connect(self.update_visible_options)
        form_layout.addRow(translator.translate("image_generation_provider_label"), self.service_combo)

        # Prompt editor
        self.prompt_edit = QTextEdit()
        self.prompt_edit.setPlainText(self.initial_prompt)
        form_layout.addRow(translator.translate("prompt_content_label"), self.prompt_edit)

        # --- Pollinations Specific Options ---
        self.pollinations_options_widget = QWidget()
        pollinations_layout = QFormLayout(self.pollinations_options_widget)
        pollinations_layout.setContentsMargins(0, 0, 0, 0)
        self.pollinations_model_combo = QComboBox()
        self.pollinations_model_combo.addItems(["flux", "flux-realism", "flux-3d", "flux-cablyai", "dall-e-3", "midjourney", "boreal"])
        self.pollinations_model_combo.setCurrentText(self.initial_pollinations_config.get('model', 'flux'))
        pollinations_layout.addRow(translator.translate("pollinations_model_label"), self.pollinations_model_combo)
        form_layout.addRow(self.pollinations_options_widget)

        # --- Googler Specific Options ---
        self.googler_options_widget = QWidget()
        googler_layout = QFormLayout(self.googler_options_widget)
        googler_layout.setContentsMargins(0, 0, 0, 0)

        self.aspect_ratio_combo = QComboBox()
        self.aspect_ratio_combo.addItems([
            "IMAGE_ASPECT_RATIO_UNSPECIFIED", "IMAGE_ASPECT_RATIO_SQUARE",
            "IMAGE_ASPECT_RATIO_PORTRAIT", "IMAGE_ASPECT_RATIO_LANDSCAPE"
        ])
        self.aspect_ratio_combo.setCurrentText(self.initial_googler_config.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE'))
        googler_layout.addRow(translator.translate("googler_aspect_ratio_label"), self.aspect_ratio_combo)

        self.seed_spinbox = QSpinBox()
        self.seed_spinbox.setRange(0, 999999999)
        self.seed_spinbox.setValue(self.initial_googler_config.get('seed') or 0) # Set to 0 if None
        googler_layout.addRow(translator.translate("googler_seed_label"), self.seed_spinbox)
        
        self.negative_prompt_edit = QLineEdit()
        self.negative_prompt_edit.setPlaceholderText(translator.translate("googler_negative_prompt_placeholder"))
        self.negative_prompt_edit.setText(self.initial_googler_config.get('negative_prompt', ''))
        googler_layout.addRow(translator.translate("googler_negative_prompt_label"), self.negative_prompt_edit)

        form_layout.addRow(self.googler_options_widget)
        
        main_layout.addLayout(form_layout)

        # Dialog buttons
        self.button_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        self.button_box.accepted.connect(self.accept)
        self.button_box.rejected.connect(self.reject)
        main_layout.addWidget(self.button_box)

        self.setMinimumSize(450, 400)
        self.update_visible_options(self.initial_provider)

    def update_visible_options(self, provider):
        self.googler_options_widget.setVisible(provider == 'googler')
        self.pollinations_options_widget.setVisible(provider == 'pollinations')

    def get_values(self):
        config = {
            "provider": self.service_combo.currentText(),
            "prompt": self.prompt_edit.toPlainText()
        }
        if config["provider"] == 'googler':
            config['googler_config'] = {
                "aspect_ratio": self.aspect_ratio_combo.currentText(),
                "seed": self.seed_spinbox.value(),
                "negative_prompt": self.negative_prompt_edit.text()
            }
        elif config["provider"] == 'pollinations':
            config['pollinations_config'] = {
                "model": self.pollinations_model_combo.currentText()
            }
            
        return config