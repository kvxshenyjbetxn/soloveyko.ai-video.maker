from PySide6.QtWidgets import QDialog, QVBoxLayout, QFormLayout, QComboBox, QTextEdit, QDialogButtonBox
from utils.translator import translator
from utils.settings import settings_manager

class RegenerateConfigDialog(QDialog):
    def __init__(self, initial_prompt, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("regenerate_image_title"))
        
        self.current_provider = settings_manager.get('image_generation_provider', 'pollinations')

        # Layouts
        main_layout = QVBoxLayout(self)
        form_layout = QFormLayout()

        # Service selector
        self.service_combo = QComboBox()
        self.service_combo.addItems(['pollinations', 'googler'])
        self.service_combo.setCurrentText(self.current_provider)
        form_layout.addRow(translator.translate("image_generation_provider_label"), self.service_combo)

        # Prompt editor
        self.prompt_edit = QTextEdit()
        self.prompt_edit.setPlainText(initial_prompt)
        form_layout.addRow(translator.translate("prompt_content_label"), self.prompt_edit)

        main_layout.addLayout(form_layout)

        # Dialog buttons
        self.button_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        self.button_box.accepted.connect(self.accept)
        self.button_box.rejected.connect(self.reject)
        main_layout.addWidget(self.button_box)

        self.setMinimumSize(400, 300)

    def get_values(self):
        return {
            "provider": self.service_combo.currentText(),
            "prompt": self.prompt_edit.toPlainText()
        }
