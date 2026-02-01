from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QLineEdit, QCheckBox,
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
        self.is_loading = True # Prevent saving during init
        self.init_ui()
        self.update_fields()
        self.retranslate_ui()
        self.is_loading = False

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
        self.tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.tokens_spinbox.valueChanged.connect(self.save_settings)

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

        # --- Reordered Layout Construction ---
        
        # 1. Generation Mode & Counts (First)
        
        # Generation Mode Helper + Label
        self.generation_mode_help = HelpLabel("generation_mode_help")
        self.generation_mode_label = QLabel()
        gen_mode_label_container = QWidget()
        gen_mode_label_layout = QHBoxLayout(gen_mode_label_container)
        gen_mode_label_layout.setContentsMargins(0,0,0,0)
        gen_mode_label_layout.setSpacing(5)
        gen_mode_label_layout.addWidget(self.generation_mode_help)
        gen_mode_label_layout.addWidget(self.generation_mode_label)

        self.generation_mode_combo = QComboBox()
        self.generation_mode_combo.currentIndexChanged.connect(self.on_mode_changed)
        self.generation_mode_combo.currentIndexChanged.connect(self.save_settings)
        settings_form_layout.addRow(gen_mode_label_container, self.generation_mode_combo)

        # Sync/Segments Count (Only for Sync Mode)
        self.text_split_count_help = HelpLabel("text_split_count_help")
        self.text_split_count_label = QLabel()
        text_split_container = QWidget()
        text_split_layout = QHBoxLayout(text_split_container)
        text_split_layout.setContentsMargins(0,0,0,0)
        text_split_layout.setSpacing(5)
        text_split_layout.addWidget(self.text_split_count_help)
        text_split_layout.addWidget(self.text_split_count_label)
        
        self.text_split_count_spinbox = QSpinBox()
        self.text_split_count_spinbox.setRange(1, 1000) # 1 minimum for segments
        self.text_split_count_spinbox.valueChanged.connect(self.save_settings)
        self.text_split_row_widget = QWidget() 
        settings_form_layout.addRow(text_split_container, self.text_split_count_spinbox)
        self.text_split_container_widget = text_split_container

        # Prompt Count (Only for Standard Mode)
        
        # NOTE: We do NOT add a checkbox here. The control is enabled globally in General settings.
        # We only show the prompt count input if that global setting is enabled.
        
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
        
        self.prompt_count_container_widget = count_label_container

        # 2. Technical Settings (Model, Tokens, Temp) - Moved below Generation Mode
        settings_form_layout.addRow(model_label_container, self.model_combo)
        settings_form_layout.addRow(tokens_label_container, self.tokens_spinbox)
        settings_form_layout.addRow(temp_label_container, self.temperature_spinbox)
        
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

        # --- Preview Settings Section ---
        preview_group = QGroupBox()
        self.preview_group_layout = QVBoxLayout(preview_group)

        self.preview_prompt_help = HelpLabel("preview_prompt_tooltip")
        self.preview_prompt_label = QLabel()
        preview_prompt_label_container = QWidget()
        preview_prompt_label_layout = QHBoxLayout(preview_prompt_label_container)
        preview_prompt_label_layout.setContentsMargins(0,0,0,0)
        preview_prompt_label_layout.setSpacing(5)
        preview_prompt_label_layout.addWidget(self.preview_prompt_help)
        preview_prompt_label_layout.addWidget(self.preview_prompt_label)

        self.preview_prompt_edit = QTextEdit()
        self.preview_prompt_edit.textChanged.connect(self.save_settings)
        self.preview_prompt_edit.setMinimumHeight(150)

        self.open_preview_editor_button = QPushButton(translator.translate("open_editor_button", "Open Editor"))
        self.open_preview_editor_button.clicked.connect(self.open_preview_prompt_editor)

        preview_settings_form_layout = QFormLayout()

        # Model
        self.preview_model_help = HelpLabel("image_model_label")
        self.preview_model_label = QLabel(translator.translate("image_model_label"))
        preview_model_container = QWidget()
        preview_model_layout = QHBoxLayout(preview_model_container)
        preview_model_layout.setContentsMargins(0,0,0,0)
        preview_model_layout.setSpacing(5)
        preview_model_layout.addWidget(self.preview_model_help)
        preview_model_layout.addWidget(self.preview_model_label)
        
        self.preview_model_combo = QComboBox()
        self.preview_model_combo.currentIndexChanged.connect(self.save_settings)
        preview_settings_form_layout.addRow(preview_model_container, self.preview_model_combo)

        # Tokens
        self.preview_tokens_help = HelpLabel("tokens_label")
        self.preview_tokens_label = QLabel(translator.translate("tokens_label"))
        preview_tokens_container = QWidget()
        preview_tokens_layout = QHBoxLayout(preview_tokens_container)
        preview_tokens_layout.setContentsMargins(0,0,0,0)
        preview_tokens_layout.setSpacing(5)
        preview_tokens_layout.addWidget(self.preview_tokens_help)
        preview_tokens_layout.addWidget(self.preview_tokens_label)

        self.preview_tokens_spinbox = QSpinBox()
        self.preview_tokens_spinbox.setRange(0, 128000)
        self.preview_tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.preview_tokens_spinbox.valueChanged.connect(self.save_settings)
        preview_settings_form_layout.addRow(preview_tokens_container, self.preview_tokens_spinbox)

        # Temperature
        self.preview_temp_help = HelpLabel("temperature_label")
        self.preview_temp_label = QLabel(translator.translate("temperature_label"))
        preview_temp_container = QWidget()
        preview_temp_layout = QHBoxLayout(preview_temp_container)
        preview_temp_layout.setContentsMargins(0,0,0,0)
        preview_temp_layout.setSpacing(5)
        preview_temp_layout.addWidget(self.preview_temp_help)
        preview_temp_layout.addWidget(self.preview_temp_label)

        self.preview_temperature_spinbox = QDoubleSpinBox()
        self.preview_temperature_spinbox.setRange(0.0, 2.0)
        self.preview_temperature_spinbox.setSingleStep(0.1)
        self.preview_temperature_spinbox.valueChanged.connect(self.save_settings)
        preview_settings_form_layout.addRow(preview_temp_container, self.preview_temperature_spinbox)

        # Image Count
        self.preview_image_count_help = HelpLabel("preview_image_count_tooltip")
        self.preview_image_count_label = QLabel()
        preview_image_count_container = QWidget()
        preview_image_count_layout = QHBoxLayout(preview_image_count_container)
        preview_image_count_layout.setContentsMargins(0,0,0,0)
        preview_image_count_layout.setSpacing(5)
        preview_image_count_layout.addWidget(self.preview_image_count_help)
        preview_image_count_layout.addWidget(self.preview_image_count_label)
        
        self.preview_image_count_spinbox = QSpinBox()
        self.preview_image_count_spinbox.setRange(1, 4)
        self.preview_image_count_spinbox.valueChanged.connect(self.save_settings)
        preview_settings_form_layout.addRow(preview_image_count_container, self.preview_image_count_spinbox)

        preview_main_prompt_layout = QHBoxLayout()
        preview_main_prompt_layout.addWidget(self.preview_prompt_edit)

        preview_editor_button_layout = QVBoxLayout()
        preview_editor_button_layout.addWidget(self.open_preview_editor_button)
        preview_editor_button_layout.addStretch()
        preview_main_prompt_layout.addLayout(preview_editor_button_layout)

        self.preview_group_layout.addWidget(preview_prompt_label_container)
        self.preview_group_layout.addLayout(preview_main_prompt_layout)
        self.preview_group_layout.addLayout(preview_settings_form_layout)

        self.content_layout.addWidget(preview_group)

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

    def open_preview_prompt_editor(self):
        dialog = PromptEditorDialog(self, self.preview_prompt_edit.toPlainText())
        if dialog.exec():
            self.preview_prompt_edit.setPlainText(dialog.get_text())

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
        tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
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
        
        # Input Source
        input_source_help = HelpLabel("input_source_label")
        input_source_label = QLabel()
        input_source_container = QWidget()
        input_source_layout = QHBoxLayout(input_source_container)
        input_source_layout.setContentsMargins(0,0,0,0)
        input_source_layout.setSpacing(5)
        input_source_layout.addWidget(input_source_help)
        input_source_layout.addWidget(input_source_label)

        input_source_combo = QComboBox()
        input_source_combo.addItem(translator.translate("input_source_text"), "text")
        input_source_combo.addItem(translator.translate("input_source_task_name"), "task_name")
        
        if stage_data:
            source = stage_data.get("input_source", "text")
            index = input_source_combo.findData(source)
            if index >= 0:
                input_source_combo.setCurrentIndex(index)
        input_source_combo.currentIndexChanged.connect(self.save_custom_stages)
        
        settings_form_layout.addRow(input_source_container, input_source_combo)

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
                "temperature": temp_label,
                "input_source": input_source_label
            },
            "helps": {
                "stage_name": stage_name_help,
                "prompt": prompt_help,
                "model": model_help,
                "tokens": tokens_help,
                "temperature": temp_help,
                "input_source": input_source_help
            },
            "input_source_combo": input_source_combo
        })
        
    def on_mode_changed(self):
        is_sync = self.generation_mode_combo.currentIndex() == 1
        
        # Switch prompt based on mode
        if not self.is_loading:
            self.prompt_edit.blockSignals(True)
            if is_sync:
                # Switching to Sync mode
                prompt = self.settings.get("image_prompt_settings.prompt_sync")
                self.prompt_edit.setPlainText(prompt)
            else:
                # Switching to Standard mode
                prompt = self.settings.get("image_prompt_settings.prompt_standard")
                if not prompt:
                    prompt = self.settings.get("image_prompt_settings.prompt")
                self.prompt_edit.setPlainText(prompt)
            self.prompt_edit.blockSignals(False)
        
        # Sync Mode: Show Segments Count, Hide Prompt Count
        # Standard Mode: Hide Segments Count, Show Prompt Count
        
        self.text_split_container_widget.setVisible(is_sync)
        self.text_split_count_spinbox.setVisible(is_sync)
        
        # Logic for Prompt Count visibility
        # If Standard mode -> check 'prompt_count_control_enabled' setting too?
        # User said: "standard feature... there will be possibility to show prompt count... sync feature... prompt count disabled"
        
        if is_sync:
            # Sync Mode: Checkbox hidden, Count hidden
            self.prompt_count_container_widget.setVisible(False)
            self.prompt_count_spinbox.setVisible(False)
        else:
            # Standard Mode:
            # Check global setting 'prompt_count_control_enabled'
            is_checked = self.settings.get('prompt_count_control_enabled', False)
            
            # Count visible ONLY if global setting enabled
            self.prompt_count_container_widget.setVisible(is_checked)
            self.prompt_count_spinbox.setVisible(is_checked)

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
        self.generation_mode_combo.blockSignals(True)
        self.text_split_count_spinbox.blockSignals(True)
        self.prompt_count_spinbox.blockSignals(True)

        # Logic: Determine mode first to load correct prompt
        split_count = self.settings.get('text_split_count', 0)
        is_sync = split_count > 0
        
        if is_sync:
            prompt = self.settings.get("image_prompt_settings.prompt_sync")
        else:
            prompt = self.settings.get("image_prompt_settings.prompt_standard")
            if not prompt:
                prompt = self.settings.get("image_prompt_settings.prompt")
            
        self.prompt_edit.setPlainText(prompt)
        
        self.load_models()
        current_model = config.get("model", "")
        index = self.model_combo.findText(current_model)
        self.model_combo.setCurrentIndex(index if index >= 0 else 0)
        self.tokens_spinbox.setValue(config.get("max_tokens", 4096))
        self.temperature_spinbox.setValue(config.get("temperature", 0.7))
        
        # Logic: If text_split_count > 0 -> Sync Mode (Index 1). Else Standard (Index 0)
        split_count = self.settings.get('text_split_count', 0)
        last_split_count = self.settings.get('last_text_split_count', 5)
        
        if split_count > 0:
            self.generation_mode_combo.setCurrentIndex(1)
            self.text_split_count_spinbox.setValue(split_count)
        else:
            self.generation_mode_combo.setCurrentIndex(0)
            self.text_split_count_spinbox.setValue(last_split_count) # Restore last used value

        # Apply visibility based on current mode
        self.on_mode_changed()
        
        # Apply visibility based on current mode
        self.on_mode_changed()
        
        self.prompt_count_spinbox.setValue(self.settings.get('prompt_count', 10))

        self.prompt_edit.blockSignals(False)
        self.model_combo.blockSignals(False)
        self.tokens_spinbox.blockSignals(False)
        self.temperature_spinbox.blockSignals(False)
        self.generation_mode_combo.blockSignals(False)
        self.text_split_count_spinbox.blockSignals(False)
        self.prompt_count_spinbox.blockSignals(False)
        
        # Apply visibility based on current mode (AGAIN, after loading checkbox)
        self.on_mode_changed()

        # --- Update Preview Fields ---
        preview_config = self.settings.get("preview_settings", {})
        self.preview_prompt_edit.blockSignals(True)
        self.preview_model_combo.blockSignals(True)
        self.preview_tokens_spinbox.blockSignals(True)
        self.preview_temperature_spinbox.blockSignals(True)
        self.preview_image_count_spinbox.blockSignals(True)

        self.preview_prompt_edit.setPlainText(preview_config.get("prompt", ""))
        self.preview_model_combo.clear()
        self.preview_model_combo.addItems(self.settings.get("openrouter_models", []))
        
        preview_model = preview_config.get("model", "")
        p_index = self.preview_model_combo.findText(preview_model)
        self.preview_model_combo.setCurrentIndex(p_index if p_index >= 0 else 0)

        self.preview_tokens_spinbox.setValue(preview_config.get("max_tokens", 10000))
        self.preview_temperature_spinbox.setValue(preview_config.get("temperature", 1.0))
        self.preview_image_count_spinbox.setValue(preview_config.get("image_count", 3))

        self.preview_prompt_edit.blockSignals(False)
        self.preview_model_combo.blockSignals(False)
        self.preview_tokens_spinbox.blockSignals(False)
        self.preview_temperature_spinbox.blockSignals(False)
        self.preview_image_count_spinbox.blockSignals(False)

        while self.stages_container.count():
            item = self.stages_container.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
        self.stage_widgets = []

        custom_stages = self.settings.get("custom_stages", [])
        for stage in custom_stages:
            self.add_new_stage(stage)

    def load_models(self):
        self.model_combo.blockSignals(True)
        self.preview_model_combo.blockSignals(True)
        
        models = self.settings.get("openrouter_models", [])
        
        current_model = self.model_combo.currentText()
        self.model_combo.clear()
        self.model_combo.addItems(models)
        idx = self.model_combo.findText(current_model)
        if idx >= 0: self.model_combo.setCurrentIndex(idx)
        
        current_preview = self.preview_model_combo.currentText()
        self.preview_model_combo.clear()
        self.preview_model_combo.addItems(models)
        p_idx = self.preview_model_combo.findText(current_preview)
        if p_idx >= 0: self.preview_model_combo.setCurrentIndex(p_idx)
        
        self.model_combo.blockSignals(False)
        self.preview_model_combo.blockSignals(False)

    def save_settings(self):
        if hasattr(self, 'is_loading') and self.is_loading:
            return

        is_sync = self.generation_mode_combo.currentIndex() == 1
        config = self.settings.get("image_prompt_settings", {})
        
        current_prompt = self.prompt_edit.toPlainText()
        
        # Save to specific mode slots
        if is_sync:
            config["prompt_sync"] = current_prompt
        else:
            config["prompt_standard"] = current_prompt
            
        # Keep main 'prompt' key updated for current behavior
        config["prompt"] = current_prompt
        
        config["model"] = self.model_combo.currentText()
        config["max_tokens"] = self.tokens_spinbox.value()
        config["temperature"] = self.temperature_spinbox.value()
        self.settings.set("image_prompt_settings", config)
        
        # Save Logic:
        # If Sync Mode (Index 1) -> save text_split_count value
        # If Standard Mode (Index 0) -> save 0 (disabled)
        # Save Logic:
        # If Sync Mode (Index 1) -> save text_split_count value
        # If Standard Mode (Index 0) -> save 0 (disabled)
        if self.generation_mode_combo.currentIndex() == 1:
            val = self.text_split_count_spinbox.value()
            self.settings.set('text_split_count', val)
            self.settings.set('last_text_split_count', val)
        else:
            self.settings.set('text_split_count', 0)

        # self.settings.set('prompt_count_control_enabled', ...) # Controlled in General Tab
        self.settings.set('prompt_count', self.prompt_count_spinbox.value())

        preview_config = {
            "prompt": self.preview_prompt_edit.toPlainText(),
            "model": self.preview_model_combo.currentText(),
            "max_tokens": self.preview_tokens_spinbox.value(),
            "temperature": self.preview_temperature_spinbox.value(),
            "image_count": self.preview_image_count_spinbox.value()
        }
        self.settings.set("preview_settings", preview_config)

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
            max_tokens = stage["tokens_spinbox"].value()
            temperature = stage["temperature_spinbox"].value()
            input_source = stage["input_source_combo"].currentData()
            
            if name:
                stages_data.append({
                    "name": name,
                    "prompt": prompt,
                    "model": model,
                    "max_tokens": max_tokens,
                    "model": model,
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                    "input_source": input_source
                })
        self.settings.set("custom_stages", stages_data)

    def retranslate_ui(self):
        self.img_group_layout.parentWidget().setTitle(translator.translate("image_prompt_settings_title"))
        self.prompt_content_label.setText(translator.translate("prompt_content_label"))
        self.model_label.setText(translator.translate("image_model_label"))
        self.tokens_label.setText(translator.translate("tokens_label"))
        self.temperature_label.setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")
        self.generation_mode_label.setText(translator.translate("generation_mode_label"))
        
        self.generation_mode_combo.blockSignals(True)
        self.generation_mode_combo.clear()
        self.generation_mode_combo.addItem(translator.translate("gen_mode_standard"))
        self.generation_mode_combo.addItem(translator.translate("gen_mode_sync"))
        # Restore index? No, retranslate usually happens on lang change, we should preserve index? 
        # Actually simplest to just set items. The index is preserved if count doesn't change?
        # Let's re-read from settings or local state?
        # Update_fields calls load logic.
        split_count = self.settings.get('text_split_count', 0)
        self.generation_mode_combo.setCurrentIndex(1 if split_count > 0 else 0)
        self.generation_mode_combo.blockSignals(False)
        self.on_mode_changed()

        self.text_split_count_label.setText(translator.translate("segment_count_label"))
        self.prompt_count_label.setText(translator.translate("prompt_count_label"))
        
        self.prompt_content_help.update_tooltip()
        self.model_help.update_tooltip()
        self.tokens_help.update_tooltip()
        self.temperature_help.update_tooltip()
        self.text_split_count_help.update_tooltip()
        self.prompt_count_help.update_tooltip()
        self.generation_mode_help.update_tooltip()

        self.preview_group_layout.parentWidget().setTitle(translator.translate("preview_settings_group"))
        self.preview_prompt_label.setText(translator.translate("preview_prompt_label"))
        self.preview_image_count_label.setText(translator.translate("preview_image_count_label"))
        self.preview_model_label.setText(translator.translate("image_model_label"))
        self.preview_tokens_label.setText(translator.translate("tokens_label"))
        self.preview_temp_label.setText(translator.translate("temperature_label"))
        
        self.preview_prompt_help.update_tooltip()
        self.preview_model_help.update_tooltip()
        self.preview_tokens_help.update_tooltip()
        self.preview_temp_help.update_tooltip()
        self.preview_image_count_help.update_tooltip()

        self.custom_stages_label.setText(translator.translate("custom_stages_label") if translator.translate("custom_stages_label") != "custom_stages_label" else "Custom Stages")
        self.custom_stages_help.update_tooltip()
        self.add_stage_btn.setText(translator.translate("add_stage_btn") if translator.translate("add_stage_btn") != "add_stage_btn" else "Add Custom Stage")

        self.tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        self.preview_tokens_spinbox.setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))
        for stage in self.stage_widgets:
            stage["tokens_spinbox"].setSpecialValueText(translator.translate("maximum_tokens", "Maximum"))

        for stage in self.stage_widgets:
            stage["labels"]["stage_name"].setText(translator.translate("parallel_stage_name_label"))
            stage["labels"]["prompt"].setText(translator.translate("prompt_label"))
            stage["labels"]["model"].setText(translator.translate("image_model_label"))
            stage["labels"]["tokens"].setText(translator.translate("tokens_label"))
            stage["labels"]["tokens"].setText(translator.translate("tokens_label"))
            stage["labels"]["temperature"].setText(translator.translate("temperature_label") if translator.translate("temperature_label") != "temperature_label" else "Temperature")
            stage["labels"]["input_source"].setText(translator.translate("input_source_label"))
            
            # Update combo items text while keeping data
            input_combo = stage["input_source_combo"]
            current_data = input_combo.currentData()
            input_combo.blockSignals(True)
            input_combo.setItemText(0, translator.translate("input_source_text"))
            input_combo.setItemText(1, translator.translate("input_source_task_name"))
            input_combo.blockSignals(False)
            
            stage["name_edit"].setPlaceholderText(translator.translate("stage_name_placeholder"))
            stage["prompt_edit"].setPlaceholderText(translator.translate("enter_prompt_placeholder"))
            
            for help_widget in stage["helps"].values():
                help_widget.update_tooltip()