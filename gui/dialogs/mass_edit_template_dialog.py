from PySide6.QtWidgets import (QDialog, QVBoxLayout, QLabel, QComboBox, 
                               QLineEdit, QListWidget, QListWidgetItem, 
                               QPushButton, QMessageBox, QGroupBox, QHBoxLayout,
                               QSpinBox, QDoubleSpinBox, QFontComboBox, QFileDialog, QWidget)
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.settings import template_manager, settings_manager
from gui.settings_metadata import SETTINGS_METADATA, KEY_TO_TRANSLATION_MAP, VOICEMAKER_VOICES, GEMINI_VOICES
from gui.dialogs.template_changes_dialog import TemplateChangesDialog
import copy

class PathEditor(QWidget):
    def __init__(self, parent=None, mode='directory'):
        super().__init__(parent)
        self.mode = mode
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.line_edit = QLineEdit()
        self.browse_btn = QPushButton(translator.translate("browse_button", "Select"))
        self.browse_btn.clicked.connect(self.browse)
        layout.addWidget(self.line_edit)
        layout.addWidget(self.browse_btn)
        self.setFocusProxy(self.line_edit)
        
    def setText(self, text):
        self.line_edit.setText(text)
        
    def text(self):
        return self.line_edit.text()
        
    def browse(self):
        if self.mode == 'directory':
            path = QFileDialog.getExistingDirectory(self, translator.translate("select_directory", "Select Directory"))
        else:
            path, _ = QFileDialog.getOpenFileName(self, translator.translate("select_file", "Select File"))
            
        if path:
            self.line_edit.setText(path)
            self.line_edit.setFocus()

class MassEditTemplateDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("mass_edit_title"))
        self.setMinimumSize(600, 700)
        
        self.layout = QVBoxLayout(self)
        
        # Helper label
        hint_label = QLabel(translator.translate("select_setting_hint"))
        hint_label.setWordWrap(True)
        self.layout.addWidget(hint_label)

        # Parameter Selection
        self.param_group = QGroupBox(translator.translate("select_parameter"))
        self.param_layout = QVBoxLayout(self.param_group)
        
        # Searchable combo logic is tricky without custom widget, but standard QComboBox with setEditable(True) works okay-ish for filtering if configured, 
        # but for now standard combo.
        self.param_combo = QComboBox()
        self.param_combo.setPlaceholderText(translator.translate("select_setting"))
        
        # Populate params dynamically
        self.params_data = [] # List of (path, metadata)
        self._populate_params()
        
        self.param_combo.currentIndexChanged.connect(self._update_value_widget)
        self.param_layout.addWidget(self.param_combo)
        self.layout.addWidget(self.param_group)
        
        # Value Selection
        self.value_group = QGroupBox(translator.translate("new_value"))
        self.value_layout = QVBoxLayout(self.value_group)
        self.value_widget = None 
        self.layout.addWidget(self.value_group)
        
        # Templates Selection
        self.templates_group = QGroupBox(translator.translate("target_templates"))
        self.templates_layout = QVBoxLayout(self.templates_group)
        
        # Select All / Deselect All
        self.sel_buttons_layout = QHBoxLayout()
        self.btn_select_all = QPushButton(translator.translate("select_all"))
        self.btn_deselect_all = QPushButton(translator.translate("deselect_all"))
        self.btn_select_all.clicked.connect(self._select_all)
        self.btn_deselect_all.clicked.connect(self._deselect_all)
        self.sel_buttons_layout.addWidget(self.btn_select_all)
        self.sel_buttons_layout.addWidget(self.btn_deselect_all)
        self.templates_layout.addLayout(self.sel_buttons_layout)
        
        self.templates_list = QListWidget()
        self.templates_layout.addWidget(self.templates_list)
        self.layout.addWidget(self.templates_group)
        
        # Buttons
        self.buttons_layout = QHBoxLayout()
        self.apply_btn = QPushButton(translator.translate("apply_changes"))
        self.apply_btn.clicked.connect(self._apply)
        self.close_btn = QPushButton(translator.translate("close_button"))
        self.close_btn.clicked.connect(self.reject)
        
        self.buttons_layout.addStretch()
        self.buttons_layout.addWidget(self.apply_btn)
        self.buttons_layout.addWidget(self.close_btn)
        self.layout.addLayout(self.buttons_layout)
        
        self._load_templates()
        # Initialize with first item if any
        if self.param_combo.count() > 0:
            self._update_value_widget()
        
    def _populate_params(self):
        # Flatten metadata
        items = []
        self._flatten_metadata_recursive(SETTINGS_METADATA, [], items)
        
        # Sort by label for better UX
        items.sort(key=lambda x: x[0])
        
        for label, path, metadata in items:
            self.param_combo.addItem(label, (path, metadata))
            
    def _flatten_metadata_recursive(self, metadata, current_path, items):
        for key, value in metadata.items():
            if key == 'whisper_type':
                continue
            if 'type' in value:
                # Leaf node
                # Construct path
                path = current_path + [key]
                
                # Check if inside languages_config to inject wildcard
                if 'languages_config' in current_path:
                    # Logic: if languages_config is the direct parent or ancestor
                    # Current structure: languages_config -> key
                    # So path is ['languages_config', 'prompt']
                    # We want ['languages_config', '*', 'prompt']
                    
                    # More generic handle: find where 'languages_config' is and insert '*' after it if not present
                    try:
                        idx = path.index('languages_config')
                        if idx + 1 < len(path) and path[idx+1] != '*':
                             path.insert(idx + 1, '*')
                    except ValueError:
                         pass

                # --- Exclusion Logic for Mass Editing ---
                full_path_str = ".".join(path)
                # Exclusion patterns (mostly hardware/system)
                EXCLUSIONS = {
                    "accent_color", "detailed_logging_enabled", "max_download_threads", 
                    "theme", "language", "last_applied_template",
                    "elevenlabs_image.max_threads",
                    "googler.max_threads", "googler.max_video_threads",
                    "montage.max_concurrent_montages"
                }
                
                if any(p in EXCLUSIONS for p in [key, full_path_str]):
                    continue
                
                # Construct Label
                label = self._build_friendly_label(path, value)
                items.append((label, path, value))
            elif isinstance(value, dict):
                # Branch
                self._flatten_metadata_recursive(value, current_path + [key], items)

    def _build_friendly_label(self, path, leaf_metadata=None):
        parts = []
        for i, p in enumerate(path):
            if p == '*':
                parts.append("(All Languages)")
                continue
                
            # If it's the leaf node, try to use its label from metadata
            if i == len(path) - 1 and leaf_metadata and 'label' in leaf_metadata:
                part_label = translator.translate(leaf_metadata['label'])
            else:
                # Try to find translation for key
                trans_key = KEY_TO_TRANSLATION_MAP.get(p)
                if trans_key:
                    part_label = translator.translate(trans_key)
                else:
                    # Try section titles or fallbacks
                    if p == 'languages_config': part_label = translator.translate("languages_tab")
                    elif p == 'montage': part_label = translator.translate("montage_tab")
                    elif p == 'subtitles': part_label = translator.translate("subtitles_tab")
                    elif p == 'googler': part_label = "Googler"
                    elif p == 'pollinations': part_label = "Pollinations"
                    else: 
                         # Fallback to titling the key
                         part_label = p.replace('_', ' ').title()
            
            parts.append(part_label)
            
        return " -> ".join(parts)

    def _load_templates(self):
        templates = template_manager.get_templates()
        for t in templates:
            item = QListWidgetItem(t)
            item.setCheckState(Qt.Checked)
            self.templates_list.addItem(item)
            
    def _select_all(self):
        for i in range(self.templates_list.count()):
            self.templates_list.item(i).setCheckState(Qt.Checked)
            
    def _deselect_all(self):
        for i in range(self.templates_list.count()):
            self.templates_list.item(i).setCheckState(Qt.Unchecked)

    def _update_value_widget(self):
        # Clear old widget
        if self.value_widget:
            self.value_layout.removeWidget(self.value_widget)
            self.value_widget.deleteLater()
            self.value_widget = None
            
        data = self.param_combo.currentData()
        if not data: return
        
        path, metadata = data
        editor_type = metadata.get('type')
        
        # Widget creation logic mirrored from SettingsDelegate
        if editor_type == 'bool':
            self.value_widget = QComboBox()
            self.value_widget.addItems(["True", "False"])
        elif editor_type == 'choice':
            self.value_widget = QComboBox()
            self.value_widget.addItems(metadata.get('options', []))
        elif editor_type == 'int':
            self.value_widget = QSpinBox()
            self.value_widget.setRange(metadata.get('min', -2147483647), metadata.get('max', 2147483647))
            if 'suffix' in metadata: self.value_widget.setSuffix(metadata.get('suffix'))
        elif editor_type == 'float':
            self.value_widget = QDoubleSpinBox()
            self.value_widget.setRange(metadata.get('min', -1e9), metadata.get('max', 1e9))
            self.value_widget.setSingleStep(metadata.get('step', 0.1))
            if 'suffix' in metadata: self.value_widget.setSuffix(metadata.get('suffix'))
        elif editor_type == 'font':
            self.value_widget = QFontComboBox()
        elif editor_type == 'folder_path':
            self.value_widget = PathEditor(mode='directory')
        elif editor_type == 'file_path':
            self.value_widget = PathEditor(mode='file')
        elif editor_type == 'model_selection':
            self.value_widget = QComboBox()
            models = settings_manager.get("openrouter_models", [])
            self.value_widget.addItems(models)
            self.value_widget.setEditable(True)
        elif editor_type == 'voicemaker_voice':
            self.value_widget = QComboBox()
            for lang_data in VOICEMAKER_VOICES:
                 lang_code = lang_data.get("LanguageCode", "Unknown")
                 for voice_id in lang_data.get("Voices", []):
                     self.value_widget.addItem(f"{lang_code}: {voice_id}", voice_id)
        elif editor_type == 'gemini_voice':
            self.value_widget = QComboBox()
            for voice in GEMINI_VOICES:
                self.value_widget.addItem(f"{voice['name']} ({voice['description']})", voice['value'])
        elif editor_type == 'string_list':
             self.value_widget = QLineEdit()
             self.value_widget.setPlaceholderText("Comma separated values")
        else:
             self.value_widget = QLineEdit()
             
        self.value_layout.addWidget(self.value_widget)
        
    def _apply(self):
        data_item = self.param_combo.currentData()
        if not data_item: return
        
        path, metadata = data_item
        editor_type = metadata.get('type')
        
        new_value = None
        
        # Get value logic
        if isinstance(self.value_widget, QComboBox) and not isinstance(self.value_widget, QFontComboBox):
            if editor_type == 'bool':
                 new_value = (self.value_widget.currentText() == "True")
            elif editor_type in ['voicemaker_voice', 'gemini_voice']:
                 new_value = self.value_widget.currentData()
                 if new_value is None: 
                      new_value = self.value_widget.currentText()
            else:
                 new_value = self.value_widget.currentText()
        elif isinstance(self.value_widget, QFontComboBox):
            new_value = self.value_widget.currentFont().family()
        elif isinstance(self.value_widget, (QSpinBox, QDoubleSpinBox)):
            new_value = self.value_widget.value()
        elif isinstance(self.value_widget, PathEditor):
            new_value = self.value_widget.text()
        elif isinstance(self.value_widget, QLineEdit):
            text = self.value_widget.text()
            if editor_type == 'string_list':
                 new_value = [x.strip() for x in text.split(',') if x.strip()]
            else:
                 new_value = text
                 
        # Collect selected templates
        selected_templates = []
        for i in range(self.templates_list.count()):
            item = self.templates_list.item(i)
            if item.checkState() == Qt.Checked:
                selected_templates.append(item.text())
        
        if not selected_templates:
            return

        updates_to_apply = []
        changes_for_dialog = {}

        for t_name in selected_templates:
            original_data = template_manager.load_template(t_name)
            if not original_data: continue
            
            temp_data = copy.deepcopy(original_data)
            modified = self._recursive_update(temp_data, path, new_value)
            
            if modified:
                diff = self._recursive_diff(original_data, temp_data)
                if diff:
                    changes_for_dialog[t_name] = diff
                updates_to_apply.append((t_name, temp_data))
        
        if not updates_to_apply:
            QMessageBox.information(self, translator.translate("info"), translator.translate("template_changes_no_changes", "No changes detected."))
            return

        # Show confirmation dialog
        dialog = TemplateChangesDialog(changes_for_dialog, self)
        if dialog.exec():
            count = 0
            for t_name, data in updates_to_apply:
                template_manager.save_template(t_name, data)
                count += 1
                    
            QMessageBox.information(self, translator.translate("success"), 
                                    translator.translate("mass_edit_success").format(count=count))
            self.accept()
            
    def _recursive_update(self, current_data, path, value):
        if not path:
            return False 
            
        key = path[0]
        modified = False
        
        if len(path) == 1:
            # Leaf node
            if key == "*":
                # Apply to all keys in current dict
                if isinstance(current_data, dict):
                    for k, v in current_data.items():
                        if isinstance(v, dict):
                            pass
                pass
            else:
                if key in current_data:
                     if current_data.get(key) != value:
                        current_data[key] = value
                        modified = True
                else:
                     current_data[key] = value
                     modified = True
            return modified
            
        # Branch node
        if key == "*":
             if isinstance(current_data, dict):
                 for k, v in current_data.items():
                     if isinstance(v, dict):
                         if self._recursive_update(v, path[1:], value):
                             modified = True
        else:
            if key in current_data:
                 if isinstance(current_data[key], dict):
                     if self._recursive_update(current_data[key], path[1:], value):
                        modified = True
            else:
                 pass
                    
        return modified

    def _recursive_diff(self, d1, d2, path=""):
        diffs = {}
        # Keys in d2 that are different from d1
        # We assume d2 is a modified version of d1, so keys match mostly.
        # But we might have added keys.
        
        all_keys = set(d1.keys()) | set(d2.keys())
        
        for k in all_keys:
            if k == "__note__": continue 
            
            val1 = d1.get(k)
            val2 = d2.get(k)
            
            current_path = f"{path} -> {k}" if path else k
            
            if isinstance(val1, dict) and isinstance(val2, dict):
                nested_diffs = self._recursive_diff(val1, val2, current_path)
                diffs.update(nested_diffs)
            elif val1 != val2:
                # Key changed
                # Make the key look nice if possible? 
                # Already handled by path construction
                diffs[current_path] = (val1, val2)
                
        return diffs
