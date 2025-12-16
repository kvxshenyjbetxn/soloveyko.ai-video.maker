import copy

from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QHBoxLayout, QComboBox, 
                               QLineEdit, QPushButton, QMessageBox, QInputDialog,
                               QSpacerItem, QSizePolicy, QDialog, QTreeView, QGroupBox, QTextEdit)
from PySide6.QtGui import QStandardItemModel, QStandardItem
from PySide6.QtCore import Qt, Signal, QTimer

from utils.translator import translator
from utils.settings import settings_manager, template_manager

class TemplateViewerDialog(QDialog):
    def __init__(self, title, data, parent=None):
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setMinimumSize(600, 500)

        layout = QVBoxLayout(self)

        # Display note if it exists
        note = data.pop("__note__", None)
        if note:
            note_label = QLabel(note)
            note_label.setWordWrap(True)
            note_label.setTextInteractionFlags(Qt.TextSelectableByMouse)
            note_label.setStyleSheet("background-color: #f0f0f0; border: 1px solid #ccc; padding: 5px; border-radius: 3px;")
            layout.addWidget(note_label)

        self.tree_view = QTreeView()
        self.tree_view.setEditTriggers(QTreeView.NoEditTriggers)
        layout.addWidget(self.tree_view)

        model = QStandardItemModel()
        model.setHorizontalHeaderLabels([translator.translate("setting_header", "Setting"), translator.translate("value_header", "Value")])
        self.tree_view.setModel(model)

        self.populate_tree(data, model.invisibleRootItem())

        self.tree_view.expandAll()
        self.tree_view.resizeColumnToContents(0)

        close_button = QPushButton(translator.translate("close_button", "Close"))
        close_button.clicked.connect(self.accept)
        layout.addWidget(close_button)

    def populate_tree(self, data, parent_item, parent_key=""):
        for key, value in sorted(data.items()):
            if key.endswith('_api_key') and not value:
                continue
            
            if parent_key == 'languages_config':
                key_name = key 
            else:
                key_name = translator.translate(f"{key}_label", key.replace('_', ' ').title())

            if isinstance(value, dict):
                row_item = QStandardItem(key_name)
                parent_item.appendRow([row_item, QStandardItem("")])
                self.populate_tree(value, row_item, parent_key=key)
            elif isinstance(value, list):
                row_item = QStandardItem(key_name)
                parent_item.appendRow([row_item, QStandardItem("")])
                if not value:
                    row_item.appendRow([QStandardItem("(empty)"), QStandardItem("")])
                else:
                    for i, item in enumerate(value):
                        if isinstance(item, dict):
                            item_node = QStandardItem(f"Item {i+1}")
                            row_item.appendRow([item_node, QStandardItem("")])
                            self.populate_tree(item, item_node, parent_key=key)
                        else:
                            row_item.appendRow([QStandardItem(str(item)), QStandardItem("")])
            else:
                display_v = str(value)
                parent_item.appendRow([QStandardItem(key_name), QStandardItem(display_v)])


class TemplatesTab(QWidget):
    template_applied = Signal()

    def __init__(self):
        super().__init__()
        self.init_ui()
        self.retranslate_ui()
        
        self.note_save_timer = QTimer(self)
        self.note_save_timer.setSingleShot(True)
        self.note_save_timer.setInterval(1000) # 1 second delay
        
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
        self.view_button = QPushButton()
        
        buttons_layout.addWidget(self.save_button)
        buttons_layout.addWidget(self.apply_button)
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.rename_button)
        buttons_layout.addWidget(self.view_button)
        buttons_layout.addWidget(self.delete_button)
        main_layout.addLayout(buttons_layout)

        # Notes section
        self.notes_group = QGroupBox()
        notes_layout = QVBoxLayout(self.notes_group)
        self.notes_edit = QTextEdit()
        self.notes_edit.setPlaceholderText(translator.translate("template_notes_placeholder", "Enter notes for the selected template here..."))
        notes_layout.addWidget(self.notes_edit)
        main_layout.addWidget(self.notes_group)
        
        main_layout.addStretch()

    def retranslate_ui(self):
        self.save_button.setText(translator.translate("save_button"))
        self.apply_button.setText(translator.translate("apply_template_button", "Apply"))
        self.delete_button.setText(translator.translate("delete_button"))
        self.rename_button.setText(translator.translate("rename_button"))
        self.view_button.setText(translator.translate("view_template_button", "View"))
        self.template_label.setText(translator.translate("template_label"))
        self.template_name_label.setText(translator.translate("template_name_label"))
        self.notes_group.setTitle(translator.translate("template_notes_group_title", "Notes"))

    def connect_signals(self):
        self.templates_combo.currentTextChanged.connect(self._on_template_select)
        self.save_button.clicked.connect(self._on_save)
        self.apply_button.clicked.connect(self._on_apply)
        self.view_button.clicked.connect(self._on_view)
        self.delete_button.clicked.connect(self._on_delete)
        self.rename_button.clicked.connect(self._on_rename)
        
        # Auto-save for notes
        self.notes_edit.textChanged.connect(self._on_note_changed)
        self.note_save_timer.timeout.connect(self._save_current_note)

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
        # Stop any pending save operation from the previous selection
        if self.note_save_timer.isActive():
            self.note_save_timer.stop()
            self._save_current_note()

        self.template_name_edit.setText(name)
        if name:
            template_data = template_manager.load_template(name)
            note = template_data.get("__note__", "")
            # Block signals to prevent triggering save on programmatic change
            self.notes_edit.blockSignals(True)
            self.notes_edit.setText(note)
            self.notes_edit.blockSignals(False)
        else:
            self.notes_edit.clear()

    def _on_note_changed(self):
        """Starts the auto-save timer when the user types in the notes editor."""
        self.note_save_timer.start()

    def _save_current_note(self):
        """Saves the current content of the notes editor to the selected template file."""
        name = self.templates_combo.currentText()
        if not name:
            return

        template_data = template_manager.load_template(name)
        # If template was deleted or is empty, do nothing
        if not template_data:
            return
            
        current_note = self.notes_edit.toPlainText()
        
        # Only save if the note has actually changed
        if template_data.get("__note__") != current_note:
            template_data["__note__"] = current_note
            template_manager.save_template(name, template_data)


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
                # Exclude whisper_type (engine). Keep model? User said "transcription engine". 
                # Existing code excluded model too. I'll maintain that for now to be safe or maybe user wants model in template?
                # User said "ignore ... transcription engine ...". 
                # If I switch engine, model changes. So excluding model is safer.
                elif key == 'subtitles' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['whisper_type', 'whisper_model']}

                # Filter googler settings to exclude thread counts (global hardware settings)
                elif key == 'googler' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['max_threads', 'max_video_threads']}
                
                elif key == 'languages_config' and isinstance(value, dict):
                    # Create a deep copy to modify, ensuring the original settings object is not changed
                    new_lang_config = copy.deepcopy(value)
                    for lang_id, lang_settings in new_lang_config.items():
                        if 'default_template' in lang_settings:
                            del lang_settings['default_template']
                    value = new_lang_config

                template_data[key] = value
        
        return template_data

    def _on_save(self):
        name = self.template_name_edit.text().strip()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_name_empty_error"))
            return

        data_to_save = self._gather_current_settings()
        data_to_save["__note__"] = self.notes_edit.toPlainText()

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

        reply = QMessageBox.question(self, translator.translate("confirm_apply_title", "Confirm Apply"),
                                     translator.translate("confirm_apply_template_text", 
                                                          "This will overwrite your current global settings with the settings from the template '{name}'. Are you sure?").format(name=name),
                                     QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                                     QMessageBox.StandardButton.No)

        if reply != QMessageBox.StandardButton.Yes:
            return
            
        template_data = template_manager.load_template(name)
        if not template_data:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_not_found_error"))
            return

        def deep_merge(source, destination):
            for key, value in source.items():
                if isinstance(value, dict) and key in destination and isinstance(destination[key], dict):
                    deep_merge(value, destination[key])
                else:
                    destination[key] = value
            return destination

        # Perform a deep merge
        deep_merge(template_data, settings_manager.settings)
        
        # Set last applied template and save all settings
        settings_manager.set("last_applied_template", name)
        self.template_applied.emit()

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

    def _on_view(self):
        name = self.templates_combo.currentText()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        template_data = template_manager.load_template(name)
        if not template_data:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_not_found_error"))
            return

        dialog = TemplateViewerDialog(
            translator.translate("template_details_title", "Template Details") + f" - {name}",
            template_data,
            self
        )
        dialog.exec()
