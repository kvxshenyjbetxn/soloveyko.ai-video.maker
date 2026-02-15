import ast
import copy

from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QHBoxLayout, QComboBox, 
                               QLineEdit, QPushButton, QMessageBox, QInputDialog,
                               QSpacerItem, QSizePolicy, QDialog, QTreeView, QGroupBox, QTextEdit,
                               QStyledItemDelegate, QCheckBox, QSpinBox, QDoubleSpinBox, QStyleOptionViewItem,
                               QApplication, QStyle, QColorDialog, QFontComboBox, QFileDialog, QAbstractItemView, QStyleOptionButton)
from PySide6.QtGui import QStandardItemModel, QStandardItem, QColor, QBrush
from PySide6.QtCore import Qt, Signal, QTimer, QModelIndex, QEvent

from utils.translator import translator
from utils.settings import settings_manager, template_manager
from gui.widgets.prompt_editor_dialog import PromptEditorDialog
from gui.widgets.help_label import HelpLabel
from gui.dialogs.mass_edit_template_dialog import MassEditTemplateDialog
from gui.dialogs.template_changes_dialog import TemplateChangesDialog
from gui.widgets.overlay_triggers_editor_dialog import OverlayTriggersEditorDialog
from gui.dialogs.template_editor_window import TemplateEditorWindow
import json
import os
import sys

def calculate_diff(d1, d2, path=""):
    diffs = {}
    
    # Handle One-sided None (Treat as empty dict if the other is a dict)
    is_d1_dict = isinstance(d1, dict)
    is_d2_dict = isinstance(d2, dict)
    
    if d1 is None and is_d2_dict: d1 = {}
    if d2 is None and is_d1_dict: d2 = {}
    
    # Re-check types after None adjustment
    if isinstance(d1, dict) and isinstance(d2, dict):
        all_keys = set(d1.keys()) | set(d2.keys())
        for k in all_keys:
            if k == "__note__": continue 
            
            val1 = d1.get(k)
            val2 = d2.get(k)
            
            current_path = f"{path} -> {k}" if path else k
            
            # Recurse
            nested_diffs = calculate_diff(val1, val2, current_path)
            diffs.update(nested_diffs)
            
    else:
        # One is not a dict (leaf comparison)
        if d1 != d2:
            diffs[path] = (d1, d2)
            
    return diffs

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(".")

def load_json_assets(filename):
    try:
        path = os.path.join(BASE_PATH, "assets", filename)
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading {filename}: {e}")
        return []

from gui.settings_metadata import (SETTINGS_METADATA, KEY_TO_TRANSLATION_MAP, 
                                     VOICEMAKER_VOICES, GEMINI_VOICES)




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
        
        self.template_list_help = HelpLabel("template_list_hint")
        self.template_name_help = HelpLabel("template_name_hint")

        selection_layout.addWidget(self.template_list_help)
        selection_layout.addWidget(self.template_label)
        selection_layout.addWidget(self.templates_combo)
        selection_layout.addSpacing(10)
        selection_layout.addWidget(self.template_name_help)
        selection_layout.addWidget(self.template_name_label)
        selection_layout.addWidget(self.template_name_edit)
        main_layout.addLayout(selection_layout)

        # Action buttons
        buttons_layout = QHBoxLayout()
        self.save_button = QPushButton()
        self.apply_button = QPushButton()
        self.delete_button = QPushButton()
        self.rename_button = QPushButton()
        self.edit_button = QPushButton()
        
        self.save_help = HelpLabel("template_save_hint")
        self.apply_help = HelpLabel("template_apply_hint")
        self.edit_help = HelpLabel("template_edit_hint")

        buttons_layout.addWidget(self.save_help)
        buttons_layout.addWidget(self.save_button)
        buttons_layout.addWidget(self.apply_help)
        buttons_layout.addWidget(self.apply_button)
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.rename_button)
        buttons_layout.addWidget(self.edit_help)
        buttons_layout.addWidget(self.edit_button)
        buttons_layout.addWidget(self.delete_button)
        
        self.mass_edit_button = QPushButton()
        self.mass_edit_help = HelpLabel("mass_edit_hint")
        buttons_layout.addWidget(self.mass_edit_help)
        buttons_layout.addWidget(self.mass_edit_button)
        main_layout.addLayout(buttons_layout)

        # Notes section
        self.notes_group = QGroupBox()
        notes_layout = QVBoxLayout(self.notes_group)
        self.notes_edit = QTextEdit()
        self.notes_edit.setPlaceholderText(translator.translate("template_notes_placeholder", "Enter notes for the selected template here..."))
        
        self.notes_help = HelpLabel("template_notes_hint")
        # To put help label in group box title area, we can either use a layout or just add it to the group layout.
        # User said "біля поля для вводу нотатки", so let's put it in a horizontal layout at the top of notes_layout.
        note_header_layout = QHBoxLayout()
        note_header_layout.addWidget(self.notes_help)
        note_header_layout.addStretch()
        notes_layout.addLayout(note_header_layout)
        
        notes_layout.addWidget(self.notes_edit)
        main_layout.addWidget(self.notes_group)
        
        main_layout.addStretch()

    def retranslate_ui(self):
        self.save_button.setText(translator.translate("save_button"))
        self.apply_button.setText(translator.translate("apply_template_button", "Apply"))
        self.delete_button.setText(translator.translate("delete_button"))
        self.rename_button.setText(translator.translate("rename_button"))
        self.edit_button.setText(translator.translate("edit_template_button", "Edit"))
        self.mass_edit_button.setText(translator.translate("mass_edit_button", "Mass Edit"))
        self.template_label.setText(translator.translate("template_label"))
        self.template_name_label.setText(translator.translate("template_name_label"))
        self.notes_group.setTitle(translator.translate("template_notes_group_title", "NOTES"))
        self.notes_edit.setPlaceholderText(translator.translate("template_notes_placeholder"))

        self.template_list_help.update_tooltip()
        self.template_name_help.update_tooltip()
        self.save_help.update_tooltip()
        self.apply_help.update_tooltip()
        self.edit_help.update_tooltip()
        self.notes_help.update_tooltip()
        self.mass_edit_help.update_tooltip()

    def connect_signals(self):
        self.templates_combo.currentTextChanged.connect(self._on_template_select)
        self.save_button.clicked.connect(self._on_save)
        self.apply_button.clicked.connect(self._on_apply)
        self.edit_button.clicked.connect(self._on_edit)
        self.delete_button.clicked.connect(self._on_delete)
        self.rename_button.clicked.connect(self._on_rename)
        self.mass_edit_button.clicked.connect(self._on_mass_edit)
        
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
            'prompt_count_control_enabled',
            'prompt_count',
            'image_generation_provider',
            'text_split_count',
            
            # API Tab (Note: some might be excluded later if user wants no keys in templates, but for now we follow list)
            'openrouter_models',
            'openrouter_api_key',
            'elevenlabs_api_key',
            'elevenlabs_unlim_api_key',
            'voicemaker_api_key',
            'voicemaker_char_limit',
            'gemini_tts_api_key',
            'pollinations',  # This is a dictionary
            'googler',       # This is a dictionary
            'elevenlabs_image', # Dictionary
            'assemblyai_api_key',
            # Explicitly EXCLUDED following user request: 'detailed_logging_enabled', 'accent_color', 'max_download_threads'
            
            # Review Settings
            'translation_review_enabled',
            'rewrite_review_enabled',

            # Full tabs saved as dictionaries
            'languages_config',
            'image_prompt_settings',
            'preview_settings',
            'montage',    # Will be filtered to exclude codec and preset
            'subtitles'   # Will be filtered to exclude whisper_type
        ]
        
        template_data = {}
        for key in template_keys:
            if key in all_settings:
                value = all_settings[key]
                
                # Filter montage settings to exclude codec and preset (hardware-specific), and now GPU/Quality settings (global)
                if key == 'montage' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['codec', 'preset', 'use_gpu_shaders', 'video_quality', 'max_concurrent_montages']}
                
                # Filter subtitles settings to exclude hardware-specific keys
                # Exclude whisper_type (engine). Keep model? User said "transcription engine". 
                # Existing code excluded model too. I'll maintain that for now to be safe or maybe user wants model in template?
                # User said "ignore ... transcription engine ...". 
                # If I switch engine, model changes. So excluding model is safer.
                elif key == 'subtitles' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['whisper_type']}

                # Filter googler settings to exclude thread counts (global hardware settings)
                elif key == 'googler' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['max_threads', 'max_video_threads']}

                elif key == 'elevenlabs_image' and isinstance(value, dict):
                    value = {k: v for k, v in value.items() if k not in ['max_threads', 'proxy_enabled', 'proxy_url']}
                
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

        # Check existing
        existing_data = template_manager.load_template(name)
        if existing_data:
            diff = calculate_diff(existing_data, data_to_save)
        else:
            diff = calculate_diff({}, data_to_save)
            
        if diff:
            changes = {name: diff}
            dialog = TemplateChangesDialog(changes, self)
            if not dialog.exec():
                return
        else:
             QMessageBox.information(self, translator.translate("info"), translator.translate("template_changes_no_changes"))
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

        # Explicitly remove global settings that should not be applied from templates
        if 'simultaneous_montage_and_subs' in template_data:
            del template_data['simultaneous_montage_and_subs']
        
        if 'ai_assistant_model' in template_data:
            del template_data['ai_assistant_model']

        # Remove global montage settings from template if present
        if 'montage' in template_data and isinstance(template_data['montage'], dict):
            for key in ['use_gpu_shaders', 'video_quality', 'max_concurrent_montages']:
                if key in template_data['montage']:
                    del template_data['montage'][key]

        def deep_merge(source, destination):
            for key, value in source.items():
                if isinstance(value, dict) and key in destination and isinstance(destination[key], dict):
                    deep_merge(value, destination[key])
                else:
                    destination[key] = value
            return destination

        # Backward compatibility: Default to Standard mode (0) if template doesn't specify text_split_count
        if 'text_split_count' not in template_data:
             settings_manager.settings['text_split_count'] = 0

        # Backward compatibility: Handle missing triggers in old templates
        if 'languages_config' in template_data:
            for lang_id, lang_cfg in template_data['languages_config'].items():
                if 'overlay_triggers' not in lang_cfg:
                    # Ensure we don't carry over triggers if the template explicitly lacks them
                    current_cfg = settings_manager.settings.get('languages_config', {}).get(lang_id, {})
                    if isinstance(current_cfg, dict):
                        current_cfg['overlay_triggers'] = []

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

    def _on_edit(self):
        name = self.templates_combo.currentText()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        template_data = template_manager.load_template(name)
        if not template_data:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_not_found_error"))
            return

        dialog = TemplateEditorWindow(
            template_data,
            self,
            template_name=name
        )
        if dialog.exec():
            new_data = dialog.get_template_data()
            
            # Explicitly remove global settings that should not be in templates
            if 'simultaneous_montage_and_subs' in new_data:
                del new_data['simultaneous_montage_and_subs']
            if 'ai_assistant_model' in new_data:
                del new_data['ai_assistant_model']

            diffs = calculate_diff(template_data, new_data)
            
            if not diffs:
                 QMessageBox.information(self, translator.translate("no_changes", "No Changes"), translator.translate("no_changes_detected", "No changes detected."))
                 return

            confirm_dialog = TemplateChangesDialog({name: diffs}, self)
            if confirm_dialog.exec():
                 template_manager.save_template(name, new_data)


    def _on_mass_edit(self):
        dialog = MassEditTemplateDialog(self)
        dialog.exec()
