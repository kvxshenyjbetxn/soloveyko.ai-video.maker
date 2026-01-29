from PySide6.QtWidgets import (QDialog, QVBoxLayout, QTextEdit, QComboBox, 
                               QDoubleSpinBox, QSpinBox, QLabel, QDialogButtonBox, 
                               QHBoxLayout, QFormLayout)
from PySide6.QtCore import Qt
from utils.translator import translator

class PromptSettingsDialog(QDialog):
    def __init__(self, parent, current_prompt, current_model, current_temp, current_tokens, available_models):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("edit_prompt_title"))
        self.setMinimumSize(600, 500)
        self.setModal(True)

        layout = QVBoxLayout(self)

        # Prompt Editor
        prompt_label = QLabel(translator.translate("prompt_label"))
        layout.addWidget(prompt_label)
        
        self.prompt_edit = QTextEdit()
        self.prompt_edit.setPlainText(current_prompt)
        layout.addWidget(self.prompt_edit)

        # Settings Form
        form_layout = QFormLayout()
        
        # Model Selection
        self.model_combo = QComboBox()
        self.model_combo.setEditable(True) # Allow custom model names
        if available_models:
            self.model_combo.addItems(available_models)
        
        if current_model:
            index = self.model_combo.findText(current_model)
            if index != -1:
                self.model_combo.setCurrentIndex(index)
            else:
                self.model_combo.setCurrentText(current_model)
                
        form_layout.addRow(translator.translate("model_label"), self.model_combo)

        # Temperature
        self.temp_spin = QDoubleSpinBox()
        self.temp_spin.setRange(0.0, 2.0)
        self.temp_spin.setSingleStep(0.1)
        self.temp_spin.setValue(float(current_temp) if current_temp is not None else 0.7)
        form_layout.addRow(translator.translate("temperature_label"), self.temp_spin)

        # Max Tokens
        self.tokens_spin = QSpinBox()
        self.tokens_spin.setRange(0, 128000) # 0 for Maximum
        self.tokens_spin.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.tokens_spin.setSingleStep(128)
        self.tokens_spin.setValue(int(current_tokens) if current_tokens is not None else 0)
        form_layout.addRow(translator.translate("tokens_label"), self.tokens_spin)

        layout.addLayout(form_layout)

        # Buttons
        button_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        button_box.accepted.connect(self.accept)
        button_box.rejected.connect(self.reject)
        layout.addWidget(button_box)

    def get_data(self):
        return {
            'prompt': self.prompt_edit.toPlainText(),
            'model': self.model_combo.currentText(),
            'temperature': self.temp_spin.value(),
            'max_tokens': self.tokens_spin.value()
        }
