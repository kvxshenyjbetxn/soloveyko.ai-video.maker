import copy

from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QHBoxLayout, QComboBox, 
                               QLineEdit, QPushButton, QMessageBox, QInputDialog,
                               QSpacerItem, QSizePolicy)
from PySide6.QtCore import Qt, Signal

from utils.translator import translator
from utils.settings import settings_manager, template_manager

class TemplatesTab(QWidget):
    template_applied = Signal()

    def __init__(self):
        super().__init__()
        self.init_ui()
        self.retranslate_ui()
        self.connect_signals()
        self.populate_templates_combo()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(10, 10, 10, 10)
        main_layout.setSpacing(10)

        # Template selection and naming
        selection_layout = QHBoxLayout()
        self.template_label = QLabel()
        self.templates_combo = QComboBox()
        self.templates_combo.setMinimumWidth(200)
        self.template_name_label = QLabel()
        self.template_name_edit = QLineEdit()
        
        selection_layout.addWidget(self.template_label)
        selection_layout.addWidget(self.templates_combo)
        selection_layout.addWidget(self.template_name_label)
        selection_layout.addWidget(self.template_name_edit)
        main_layout.addLayout(selection_layout)

        # Action buttons
        buttons_layout = QHBoxLayout()
        self.save_button = QPushButton()
        self.apply_button = QPushButton()
        self.delete_button = QPushButton()
        self.rename_button = QPushButton()
        
        buttons_layout.addWidget(self.save_button)
        buttons_layout.addWidget(self.apply_button)
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.rename_button)
        buttons_layout.addWidget(self.delete_button)
        main_layout.addLayout(buttons_layout)
        
        main_layout.addStretch()

    def retranslate_ui(self):
        self.save_button.setText(translator.translate("save_button"))
        self.apply_button.setText(translator.translate("apply_button"))
        self.delete_button.setText(translator.translate("delete_button"))
        self.rename_button.setText(translator.translate("rename_button"))
        self.template_label.setText(translator.translate("template_label"))
        self.template_name_label.setText(translator.translate("template_name_label"))

    def connect_signals(self):
        self.templates_combo.currentTextChanged.connect(self._on_template_select)
        self.save_button.clicked.connect(self._on_save)
        self.apply_button.clicked.connect(self._on_apply)
        self.delete_button.clicked.connect(self._on_delete)
        self.rename_button.clicked.connect(self._on_rename)

    def populate_templates_combo(self):
        self.templates_combo.blockSignals(True)
        current_text = self.templates_combo.currentText()
        self.templates_combo.clear()
        templates = template_manager.get_templates()
        if templates:
            self.templates_combo.addItems(templates)
            if current_text in templates:
                self.templates_combo.setCurrentText(current_text)
            else:
                self.templates_combo.setCurrentIndex(0)
        self.templates_combo.blockSignals(False)
        self._on_template_select(self.templates_combo.currentText())

    def _on_template_select(self, name):
        self.template_name_edit.setText(name)

    def _gather_current_settings(self):
        """Gathers settings from the global settings_manager to be saved in a template."""
        all_settings = copy.deepcopy(settings_manager.settings)
        template_keys = [
            # General Tab (WITHOUT language and theme - they are session-specific)
            'results_path', 
            'image_review_enabled', 
            'image_generation_provider',
            
            # API Tab
            'openrouter_api_key',
            'openrouter_models',
            'elevenlabs_api_key',
            'voicemaker_api_key',
            'voicemaker_char_limit',
            'gemini_tts_api_key',
            'pollinations',  # This is a dictionary
            'googler',       # This is a dictionary

            # Full tabs saved as dictionaries
            'languages_config',
            'image_prompt_settings',
            'montage',    # Will be filtered to exclude codec and preset
            'subtitles'   # Will be filtered to exclude whisper_type
        ]
        
        template_data = {}
        for key in template_keys:
            if key in all_settings:
                value = all_settings[key]
                
                # Filter montage settings to exclude codec and preset (hardware-specific)
                if key == 'montage' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['codec', 'preset']}
                
                # Filter subtitles settings to exclude hardware-specific keys
                elif key == 'subtitles' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['whisper_type', 'whisper_model']}
                
                template_data[key] = value
        
        return template_data

    def _on_save(self):
        name = self.template_name_edit.text().strip()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_name_empty_error"))
            return

        data_to_save = self._gather_current_settings()

        # Confirmation for saving
        reply = QMessageBox.question(self, translator.translate("confirm_save_title"), 
                                     translator.translate("confirm_save_template_text").format(name=name),
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No, 
                                     QMessageBox.StandardButton.No)

        if reply != QMessageBox.StandardButton.Yes:
            return

        template_manager.save_template(name, data_to_save)
        self.populate_templates_combo()
        self.templates_combo.setCurrentText(name)
        QMessageBox.information(self, translator.translate("success"), translator.translate("template_saved_success").format(name=name))
    def _on_apply(self):
        name = self.templates_combo.currentText()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        # Confirmation for applying
        reply = QMessageBox.question(self, translator.translate("confirm_apply_title"), 
                                     translator.translate("confirm_apply_template_text").format(name=name),
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No, 
                                     QMessageBox.StandardButton.No)

        if reply != QMessageBox.StandardButton.Yes:
            return

        template_data = template_manager.load_template(name)
        if not template_data:
            QMessageBox.critical(self, translator.translate("error"), translator.translate("template_load_error").format(name=name))
            return
        
        # Ignore subtitle settings from templates, as they are user/environment-specific
        template_data.pop('subtitles', None)
        
        # Update settings
        for key, value in template_data.items():
            # Special handling for dictionaries like 'montage' and 'subtitles'
            if isinstance(value, dict) and key in settings_manager.settings and isinstance(settings_manager.settings[key], dict):
                settings_manager.settings[key].update(value)
            else:
                settings_manager.settings[key] = value
        
        settings_manager.set('last_used_template_name', name)
        settings_manager.save_settings()
        self.template_applied.emit()
        QMessageBox.information(self, translator.translate("success"), translator.translate("template_applied_success").format(name=name))

    def _on_delete(self):
        name = self.templates_combo.currentText()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        reply = QMessageBox.question(self, translator.translate("confirm_delete_title"), 
                                     translator.translate("confirm_delete_template_text").format(name=name),
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No, 
                                     QMessageBox.StandardButton.No)

        if reply == QMessageBox.StandardButton.Yes:
            template_manager.delete_template(name)
            self.populate_templates_combo()

    def _on_rename(self):
        old_name = self.templates_combo.currentText()
        if not old_name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        new_name, ok = QInputDialog.getText(self, translator.translate("rename_template_title"), 
                                          translator.translate("enter_new_name_for_template").format(name=old_name), 
                                          text=old_name)

        if ok and new_name and new_name != old_name:
            template_manager.rename_template(old_name, new_name)
            self.populate_templates_combo()
            self.templates_combo.setCurrentText(new_name)