from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QLineEdit,
    QPushButton, QTextEdit, QComboBox, QLabel, QFormLayout, QSpinBox, QScrollArea, QHBoxLayout, QGroupBox, QMessageBox, QDoubleSpinBox
)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import settings_manager
from gui.widgets.prompt_editor_dialog import PromptEditorDialog
from gui.widgets.help_label import HelpLabel

class PromptsTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
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

        self.prompt_content_help = HelpLabel("prompt_content_label")
        self.prompt_content_label = QLabel()
        prompt_label_container = QWidget()
        prompt_label_layout = QHBoxLayout(prompt_label_container)
        prompt_label_layout.setContentsMargins(0,0,0,0)
        prompt_label_layout.setSpacing(5)
        prompt_label_layout.addWidget(self.prompt_content_help)
        prompt_label_layout.addWidget(self.prompt_content_label)
        
        self.prompt_edit = QTextEdit()
        self.prompt_edit.textChanged.connect(self.save_settings)
        self.prompt_edit.setMinimumHeight(150)

        self.open_editor_button = QPushButton(translator.translate("open_editor_button", "Open Editor"))
        self.open_editor_button.clicked.connect(self.open_main_prompt_editor)
        
        settings_form_layout = QFormLayout()
        
        self.model_help = HelpLabel("image_model_label")
        self.model_label = QLabel()
        model_label_container = QWidget()
        model_label_layout = QHBoxLayout(model_label_container)
        model_label_layout.setContentsMargins(0,0,0,0)
        model_label_layout.setSpacing(5)
        model_label_layout.addWidget(self.model_help)
        model_label_layout.addWidget(self.model_label)

        self.model_combo = QComboBox()
        self.model_combo.currentIndexChanged.connect(self.save_settings)
        settings_form_layout.addRow(model_label_container, self.model_combo)

        self.tokens_help = HelpLabel("tokens_label")
        self.tokens_label = QLabel()
        tokens_label_container = QWidget()
        tokens_label_layout = QHBoxLayout(tokens_label_container)
        tokens_label_layout.setContentsMargins(0,0,0,0)
        tokens_label_layout.setSpacing(5)
        tokens_label_layout.addWidget(self.tokens_help)
        tokens_label_layout.addWidget(self.tokens_label)

        self.tokens_spinbox = QSpinBox()
        self.tokens_spinbox.setRange(0, 128000)
        self.tokens_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(tokens_label_container, self.tokens_spinbox)

        self.temperature_help = HelpLabel("temperature_label")
        self.temperature_label = QLabel()
        temp_label_container = QWidget()
        temp_label_layout = QHBoxLayout(temp_label_container)
        temp_label_layout.setContentsMargins(0,0,0,0)
        temp_label_layout.setSpacing(5)
        temp_label_layout.addWidget(self.temperature_help)
        temp_label_layout.addWidget(self.temperature_label)

        self.temperature_spinbox = QDoubleSpinBox()
        self.temperature_spinbox.setRange(0.0, 2.0)
        self.temperature_spinbox.setSingleStep(0.1)
        self.temperature_spinbox.setValue(0.7)
        self.temperature_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(temp_label_container, self.temperature_spinbox)

        self.prompt_count_help = HelpLabel("prompt_count_label")
        self.prompt_count_label = QLabel()
        count_label_container = QWidget()
        count_label_layout = QHBoxLayout(count_label_container)
        count_label_layout.setContentsMargins(0,0,0,0)
        count_label_layout.setSpacing(5)
        count_label_layout.addWidget(self.prompt_count_help)
        count_label_layout.addWidget(self.prompt_count_label)
        
        self.prompt_count_spinbox = QSpinBox()
        self.prompt_count_spinbox.setRange(1, 1000)
        self.prompt_count_spinbox.valueChanged.connect(self.save_settings)
        settings_form_layout.addRow(count_label_container, self.prompt_count_spinbox)
        
        main_prompt_layout = QHBoxLayout()
        main_prompt_layout.addWidget(self.prompt_edit)
        
        editor_button_layout = QVBoxLayout()
        editor_button_layout.addWidget(self.open_editor_button)
        editor_button_layout.addStretch()
        main_prompt_layout.addLayout(editor_button_layout)
        
        self.img_group_layout.addWidget(prompt_label_container)
        self.img_group_layout.addLayout(main_prompt_layout)
        self.img_group_layout.addLayout(settings_form_layout)
        
        self.content_layout.addWidget(img_group)

        # --- Custom Stages Section ---
        custom_stages_header_layout = QHBoxLayout()
        self.custom_stages_label = QLabel()
        self.custom_stages_label.setStyleSheet("font-size: 16px; font-weight: bold;")
        self.custom_stages_help = HelpLabel("custom_stages_label")
        custom_stages_header_layout.addWidget(self.custom_stages_help)
        custom_stages_header_layout.addWidget(self.custom_stages_label)
        custom_stages_header_layout.addStretch()
        
        self.content_layout.addLayout(custom_stages_header_layout)
        self.stages_container = QVBoxLayout()
        self.content_layout.addLayout(self.stages_container)

        self.add_stage_btn = QPushButton(translator.translate("add_stage_btn"))
        self.add_stage_btn.clicked.connect(lambda: self.add_new_stage())
        self.content_layout.addWidget(self.add_stage_btn)

        self.content_layout.addStretch()

    def open_main_prompt_editor(self):
        dialog = PromptEditorDialog(self, self.prompt_edit.toPlainText())
        if dialog.exec():
            self.prompt_edit.setPlainText(dialog.get_text())

    def open_custom_stage_prompt_editor(self, prompt_edit_widget):
        dialog = PromptEditorDialog(self, prompt_edit_widget.toPlainText())
        if dialog.exec():
            prompt_edit_widget.setPlainText(dialog.get_text())

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
        
        stage_name_help = HelpLabel("parallel_stage_name_label")
        stage_name_label = QLabel()
        
        delete_btn = QPushButton(translator.translate("delete_button"))
        delete_btn.setStyleSheet("background-color: #d32f2f;")
        delete_btn.clicked.connect(lambda: self.delete_stage(stage_group))

        header_layout.addWidget(stage_name_help)
        header_layout.addWidget(stage_name_label) 
        header_layout.addWidget(name_edit)
        header_layout.addWidget(delete_btn)
        stage_layout.addLayout(header_layout)

        # Prompt (first, like in Image Settings)
        prompt_help = HelpLabel("custom_stage_prompt_label")
        prompt_label = QLabel()
        prompt_label_container = QWidget()
        prompt_label_layout = QHBoxLayout(prompt_label_container)
        prompt_label_layout.setContentsMargins(0,0,0,0)
        prompt_label_layout.setSpacing(5)
        prompt_label_layout.addWidget(prompt_help)
        prompt_label_layout.addWidget(prompt_label)

        prompt_edit = QTextEdit()
        prompt_edit.setPlaceholderText(translator.translate("enter_prompt_placeholder"))
        prompt_edit.setMinimumHeight(120) 
        if stage_data:
            prompt_edit.setPlainText(stage_data.get("prompt", ""))
        prompt_edit.textChanged.connect(self.save_custom_stages)

        open_editor_btn = QPushButton(translator.translate("open_editor_button", "Open Editor"))
        open_editor_btn.clicked.connect(lambda _, p=prompt_edit: self.open_custom_stage_prompt_editor(p))

        custom_prompt_layout = QHBoxLayout()
        custom_prompt_layout.addWidget(prompt_edit)
        
        custom_editor_button_layout = QVBoxLayout()
        custom_editor_button_layout.addWidget(open_editor_btn)
        custom_editor_button_layout.addStretch()
        custom_prompt_layout.addLayout(custom_editor_button_layout)

        stage_layout.addWidget(prompt_label_container)
        stage_layout.addLayout(custom_prompt_layout)

        # Settings Form (Model + Tokens) - below prompt
        settings_form_layout = QFormLayout()
        
        model_combo = QComboBox()
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
        
        model_help = HelpLabel("image_model_label")
        model_label = QLabel()
        model_label_container = QWidget()
        model_label_layout = QHBoxLayout(model_label_container)
        model_label_layout.setContentsMargins(0,0,0,0)
        model_label_layout.setSpacing(5)
        model_label_layout.addWidget(model_help)
        model_label_layout.addWidget(model_label)

        tokens_help = HelpLabel("tokens_label")
        tokens_label = QLabel()
        tokens_label_container = QWidget()
        tokens_label_layout = QHBoxLayout(tokens_label_container)
        tokens_label_layout.setContentsMargins(0,0,0,0)
        tokens_label_layout.setSpacing(5)
        tokens_label_layout.addWidget(tokens_help)
        tokens_label_layout.addWidget(tokens_label)

        temp_help = HelpLabel("temperature_label")
        temp_label = QLabel()
        temp_label_container = QWidget()
        temp_label_layout = QHBoxLayout(temp_label_container)
        temp_label_layout.setContentsMargins(0,0,0,0)
        temp_label_layout.setSpacing(5)
        temp_label_layout.addWidget(temp_help)
        temp_label_layout.addWidget(temp_label)

        settings_form_layout.addRow(model_label_container, model_combo)
        settings_form_layout.addRow(tokens_label_container, tokens_spinbox)
        settings_form_layout.addRow(temp_label_container, temperature_spinbox)
        stage_layout.addLayout(settings_form_layout)

        self.stages_container.addWidget(stage_group)
        self.stage_widgets.append({
            "widget": stage_group,
            "name_edit": name_edit,
            "prompt_edit": prompt_edit,
            "model_combo": model_combo,
            "tokens_spinbox": tokens_spinbox,
            "temperature_spinbox": temperature_spinbox,
            "labels": {
                "stage_name": stage_name_label,
                "prompt": prompt_label,
                "model": model_label,
                "tokens": tokens_label,
                "temperature": temp_label
            },
            "helps": {
                "stage_name": stage_name_help,
                "prompt": prompt_help,
                "model": model_help,
                "tokens": tokens_help,
                "temperature": temp_help
            }
        })
        
        if not stage_data:
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
        config = self.settings.get("image_prompt_settings", {})
        
        self.prompt_edit.blockSignals(True)
        self.model_combo.blockSignals(True)
        self.tokens_spinbox.blockSignals(True)
        self.temperature_spinbox.blockSignals(True)
        self.prompt_count_spinbox.blockSignals(True)

        self.prompt_edit.setPlainText(config.get("prompt", ""))
        self.load_models()
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)
        self.tokens_spinbox.setValue(config.get("max_tokens", 4096))
        self.temperature_spinbox.setValue(config.get("temperature", 0.7))
        
        prompt_control_enabled = self.settings.get('prompt_count_control_enabled', False)
        self.prompt_count_help.parentWidget().setVisible(prompt_control_enabled)
        self.prompt_count_spinbox.setVisible(prompt_control_enabled)
        self.prompt_count_spinbox.setValue(self.settings.get('prompt_count', 10))

        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.tokens_spinbox.blockSignals(False)
        self.temperature_spinbox.blockSignals(False)
        self.prompt_count_spinbox.blockSignals(False)

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

    def save_settings(self):
        config = {
            "prompt": self.prompt_edit.toPlainText(),
            "model": self.model_combo.currentText(),
            "max_tokens": self.tokens_spinbox.value(),
            "temperature": self.temperature_spinbox.value()
        }
        self.settings.set("image_prompt_settings", config)
        self.settings.set('prompt_count', self.prompt_count_spinbox.value())

        if self.main_window:
            if hasattr(self.main_window, 'settings_tab') and hasattr(self.main_window.settings_tab, 'general_tab'):
                self.main_window.settings_tab.general_tab.update_fields()

    def save_custom_stages(self):
        stages_data = []
        for stage in self.stage_widgets:
            name = stage["name_edit"].text().strip()
            prompt = stage["prompt_edit"].toPlainText()
            model = stage["model_combo"].currentText()
            max_tokens = stage["tokens_spinbox"].value()
            temperature = stage["temperature_spinbox"].value()
            
            if name:
                stages_data.append({
                    "name": name,
                    "prompt": prompt,
                    "model": model,
                    "max_tokens": max_tokens,
                    "temperature": temperature
                })
        self.settings.set("custom_stages", stages_data)

    def retranslate_ui(self):
        self.img_group_layout.parentWidget().setTitle(translator.translate("image_prompt_settings_title"))
        self.prompt_content_label.setText(translator.translate("prompt_content_label"))
        self.model_label.setText(translator.translate("image_model_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))
        self.temperature_label.setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")
        self.prompt_count_label.setText(translator.translate("prompt_count_label"))
        
        self.prompt_content_help.update_tooltip()
        self.model_help.update_tooltip()
        self.tokens_help.update_tooltip()
        self.temperature_help.update_tooltip()
        self.prompt_count_help.update_tooltip()

        self.custom_stages_label.setText(translator.translate("custom_stages_label") if translator.translate("custom_stages_label") != "custom_stages_label" else "Custom Stages")
        self.custom_stages_help.update_tooltip()
        self.add_stage_btn.setText(translator.translate("add_stage_btn") if translator.translate("add_stage_btn") != "add_stage_btn" else "Add Custom Stage")

        for stage in self.stage_widgets:
            stage["labels"]["stage_name"].setText(translator.translate("parallel_stage_name_label"))
            stage["labels"]["prompt"].setText(translator.translate("prompt_label"))
            stage["labels"]["model"].setText(translator.translate("image_model_label"))
            stage["labels"]["tokens"].setText(translator.translate("tokens_label"))
            stage["labels"]["temperature"].setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")
            
            stage["name_edit"].setPlaceholderText(translator.translate("stage_name_placeholder"))
            stage["prompt_edit"].setPlaceholderText(translator.translate("enter_prompt_placeholder"))
            
            for help_widget in stage["helps"].values():
                help_widget.update_tooltip()