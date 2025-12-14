from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QFormLayout, QSpinBox, QScrollArea, QHBoxLayout, QGroupBox, QMessageBox, QDoubleSpinBox
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager

class PromptsTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.stage_widgets = []
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

        self.content_layout = QVBoxLayout(scroll_content)
        
        # --- Image Prompts Section ---
        img_group = QGroupBox()
        self.img_group_layout = QVBoxLayout(img_group)

        self.prompt_content_label = QLabel()
        self.prompt_edit = QTextEdit()
        self.prompt_edit.textChanged.connect(self.save_settings)
        
        settings_form_layout = QFormLayout()
        
        self.model_label = QLabel()
        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.model_label, self.model_combo)

        self.tokens_label = QLabel()
        self.tokens_spinbox = QSpinBox()
        self.tokens_spinbox.setRange(0, 128000)
        self.tokens_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.tokens_label, self.tokens_spinbox)

        self.temperature_label = QLabel(translator.translate("temperature_label"))
        self.temperature_spinbox = QDoubleSpinBox()
        self.temperature_spinbox.setRange(0.0, 2.0)
        self.temperature_spinbox.setSingleStep(0.1)
        self.temperature_spinbox.setValue(0.7)
        self.temperature_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.temperature_label, self.temperature_spinbox)

        self.image_prompt_count_label = QLabel()
        self.image_prompt_count_spinbox = QSpinBox()
        self.image_prompt_count_spinbox.setRange(1, 1000)
        self.image_prompt_count_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(self.image_prompt_count_label, self.image_prompt_count_spinbox)

        self.img_group_layout.addWidget(self.prompt_content_label)
        self.img_group_layout.addWidget(self.prompt_edit)
        self.img_group_layout.addLayout(settings_form_layout)
        
        self.content_layout.addWidget(img_group)


        # --- Custom Stages Section ---
        self.custom_stages_label = QLabel(translator.translate("custom_stages_label"))
        self.custom_stages_label.setStyleSheet("font-size: 16px; font-weight: bold; margin-top: 20px;")
        self.content_layout.addWidget(self.custom_stages_label)
        self.stages_container = QVBoxLayout()
        self.content_layout.addLayout(self.stages_container)

        self.add_stage_btn = QPushButton(translator.translate("add_stage_btn"))
        self.add_stage_btn.clicked.connect(lambda: self.add_new_stage())
        self.content_layout.addWidget(self.add_stage_btn)

        self.content_layout.addStretch()

    def add_new_stage(self, stage_data=None):
        stage_group = QGroupBox()
        stage_layout = QVBoxLayout(stage_group)
        
        # Header: Name + Delete
        header_layout = QHBoxLayout()
        name_edit = QLineEdit()
        name_edit.setPlaceholderText(translator.translate("stage_name_placeholder"))
        if stage_data:
            name_edit.setText(stage_data.get("name", ""))
        name_edit.textChanged.connect(self.save_custom_stages)
        
        delete_btn = QPushButton(translator.translate("delete_button"))
        delete_btn.setStyleSheet("background-color: #d32f2f;")
        delete_btn.clicked.connect(lambda: self.delete_stage(stage_group))

        header_layout.addWidget(QLabel(translator.translate("parallel_stage_name_label"))) 
        header_layout.addWidget(name_edit)
        header_layout.addWidget(delete_btn)
        stage_layout.addLayout(header_layout)

        # Prompt (first, like in Image Settings)
        prompt_label = QLabel(translator.translate("prompt_label"))
        prompt_edit = QTextEdit()
        prompt_edit.setPlaceholderText(translator.translate("enter_prompt_placeholder"))
        prompt_edit.setMinimumHeight(80) 
        if stage_data:
            prompt_edit.setPlainText(stage_data.get("prompt", ""))
        prompt_edit.textChanged.connect(self.save_custom_stages)
        stage_layout.addWidget(prompt_label)
        stage_layout.addWidget(prompt_edit)

        # Settings Form (Model + Tokens) - below prompt
        settings_form_layout = QFormLayout()
        
        model_combo = QComboBox()
        # Populate models
        models = self.settings.get("openrouter_models", [])
        model_combo.addItems(models)
        if stage_data and stage_data.get("model"):
            index = model_combo.findText(stage_data.get("model"))
            if index >= 0:
                model_combo.setCurrentIndex(index)
        model_combo.currentIndexChanged.connect(self.save_custom_stages)
        
        tokens_spinbox = QSpinBox()
        tokens_spinbox.setRange(0, 128000)
        tokens_spinbox.setValue(stage_data.get("max_tokens", 4096) if stage_data else 4096)
        tokens_spinbox.valueChanged.connect(self.save_custom_stages)

        temperature_spinbox = QDoubleSpinBox()
        temperature_spinbox.setRange(0.0, 2.0)
        temperature_spinbox.setSingleStep(0.1)
        temperature_spinbox.setValue(stage_data.get("temperature", 0.7) if stage_data else 0.7)
        temperature_spinbox.valueChanged.connect(self.save_custom_stages)
        
        settings_form_layout.addRow(translator.translate("image_model_label"), model_combo)
        settings_form_layout.addRow(translator.translate("tokens_label"), tokens_spinbox)
        settings_form_layout.addRow(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature", temperature_spinbox)
        stage_layout.addLayout(settings_form_layout)

        self.stages_container.addWidget(stage_group)
        self.stage_widgets.append({
            "widget": stage_group,
            "name_edit": name_edit,
            "prompt_edit": prompt_edit,
            "model_combo": model_combo,
            "tokens_spinbox": tokens_spinbox,
            "temperature_spinbox": temperature_spinbox
        })
        
        if not stage_data: # If adding manually, save immediately
            self.save_custom_stages()

    def delete_stage(self, stage_widget):
        for i, stage in enumerate(self.stage_widgets):
            if stage["widget"] == stage_widget:
                self.stages_container.removeWidget(stage_widget)
                stage_widget.deleteLater()
                self.stage_widgets.pop(i)
                break
        self.save_custom_stages()

    def update_fields(self):
        # Image Settings
        config = self.settings.get("image_prompt_settings", {})
        
        self.prompt_edit.blockSignals(True)
        self.model_combo.blockSignals(True)
        self.tokens_spinbox.blockSignals(True)
        self.temperature_spinbox.blockSignals(True)
        self.image_prompt_count_spinbox.blockSignals(True)

        self.prompt_edit.setPlainText(config.get("prompt", ""))
        self.load_models()
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)
        self.tokens_spinbox.setValue(config.get("max_tokens", 4096))
        self.temperature_spinbox.setValue(config.get("temperature", 0.7))
        self.image_prompt_count_spinbox.setValue(self.settings.get('image_prompt_count', 50))


        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.tokens_spinbox.blockSignals(False)
        self.temperature_spinbox.blockSignals(False)
        self.image_prompt_count_spinbox.blockSignals(False)


        # Custom Stages
        # Clear existing
        while self.stages_container.count():
            item = self.stages_container.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
        self.stage_widgets = []

        custom_stages = self.settings.get("custom_stages", [])
        for stage in custom_stages:
            self.add_new_stage(stage)

    def load_models(self):
        self.model_combo.clear()
        models = self.settings.get("openrouter_models", [])
        self.model_combo.addItems(models)
        # We could update custom stage combos here too if needed, but update_fields handles it on refresh

    def save_settings(self):
        # Save Image Settings
        config = {
            "prompt": self.prompt_edit.toPlainText(),
            "model": self.model_combo.currentText(),
            "max_tokens": self.tokens_spinbox.value(),
            "temperature": self.temperature_spinbox.value()
        }
        self.settings.set("image_prompt_settings", config)
        self.settings.set('image_prompt_count', self.image_prompt_count_spinbox.value())


    def save_custom_stages(self):
        stages_data = []
        for stage in self.stage_widgets:
            name = stage["name_edit"].text().strip()
            prompt = stage["prompt_edit"].toPlainText()
            model = stage["model_combo"].currentText()
            max_tokens = stage["tokens_spinbox"].value()
            temperature = stage["temperature_spinbox"].value()
            
            if name: # Only save if name exists
                stages_data.append({
                    "name": name,
                    "prompt": prompt,
                    "model": model,
                    "max_tokens": max_tokens,
                    "temperature": temperature
                })
        self.settings.set("custom_stages", stages_data)

    def retranslate_ui(self):
        self.img_group_layout.parentWidget().setTitle(translator.translate("image_prompt_settings_title") if translator.translate("image_prompt_settings_title") != "image_prompt_settings_title" else "Image Prompt Settings")
        self.prompt_content_label.setText(translator.translate("prompt_content_label"))
        self.model_label.setText(translator.translate("image_model_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))
        self.image_prompt_count_label.setText(translator.translate("image_prompt_count_label"))
        self.custom_stages_label.setText(translator.translate("custom_stages_label") if translator.translate("custom_stages_label") != "custom_stages_label" else "Custom Stages")
        self.add_stage_btn.setText(translator.translate("add_stage_btn") if translator.translate("add_stage_btn") != "add_stage_btn" else "Add Custom Stage")