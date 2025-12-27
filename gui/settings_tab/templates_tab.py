import ast
import copy

from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QHBoxLayout, QComboBox, 
                               QLineEdit, QPushButton, QMessageBox, QInputDialog,
                               QSpacerItem, QSizePolicy, QDialog, QTreeView, QGroupBox, QTextEdit,
                               QStyledItemDelegate, QCheckBox, QSpinBox, QDoubleSpinBox, QStyleOptionViewItem,
                               QApplication, QStyle, QColorDialog, QFontComboBox, QFileDialog)
from PySide6.QtGui import QStandardItemModel, QStandardItem, QColor, QBrush
from PySide6.QtCore import Qt, Signal, QTimer, QModelIndex, QEvent

from utils.translator import translator
from utils.settings import settings_manager, template_manager
from gui.widgets.prompt_editor_dialog import PromptEditorDialog
import json
import os
import sys

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

VOICEMAKER_VOICES = load_json_assets("voicemaker_voices.json")
GEMINI_VOICES = load_json_assets("gemini_tts_voices.json")

# --- Settings Metadata ---
# This dictionary will define the editor type and data for each setting.
# 'type' can be 'bool', 'int', 'float', 'choice', 'str'
# 'options' is for 'choice' type
SETTINGS_METADATA = {
    'openrouter_models': {'type': 'string_list', 'label': 'openrouter_models'},
    'image_generation_provider': {'type': 'choice', 'options': ["pollinations", "googler"], 'label': 'image_generation_provider'},
    'image_review_enabled': {'type': 'bool', 'label': 'image_review_enabled'},
    'results_path': {'type': 'folder_path', 'label': 'results_path'},
    'montage': {
        # Render
        'codec': {'type': 'choice', 'options': ["libx264", "h264_nvenc", "h264_amf"]},
        'preset': {'type': 'choice', 'options': ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"]},
        'bitrate_mbps': {'type': 'int', 'min': 1, 'max': 100, 'suffix': ' Mbps'},
        'upscale_factor': {'type': 'float', 'min': 1.0, 'max': 5.0, 'step': 0.1, 'suffix': 'x'},
        # Transitions
        'enable_transitions': {'type': 'bool'},
        'transition_duration': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1, 'suffix': ' s'},
        # Zoom
        'enable_zoom': {'type': 'bool'},
        'zoom_speed_factor': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1},
        'zoom_intensity': {'type': 'float', 'min': 0.01, 'max': 1.0, 'step': 0.05},
        # Sway
        'enable_sway': {'type': 'bool'},
        'sway_speed_factor': {'type': 'float', 'min': 0.1, 'max': 5.0, 'step': 0.1},
        # Special Processing
        'special_processing_mode': {'type': 'choice', 'options': ["Disabled", "Quick show", "Video at the beginning"]},
        'special_processing_image_count': {'type': 'int', 'min': 1, 'max': 100},
        'special_processing_duration_per_image': {'type': 'float', 'min': 0.1, 'max': 10.0, 'step': 0.1, 'suffix': ' s'},
        'special_processing_video_count': {'type': 'int', 'min': 1, 'max': 100},
        'special_processing_check_sequence': {'type': 'bool'},
        'max_concurrent_montages': {'type': 'int', 'min': 1, 'max': 10}
    },
    'subtitles': {
        'whisper_type': {'type': 'choice', 'options': ['standard', 'amd', 'assemblyai']},
        'whisper_model': {'type': 'choice', 'options': ["tiny", "base", "small", "medium", "large", "base.bin", "small.bin", "medium.bin", "large.bin"]},
        'font': {'type': 'font'},
        'fontsize': {'type': 'int', 'min': 10, 'max': 200},
        'margin_v': {'type': 'int', 'min': 0, 'max': 500},
        'fade_in': {'type': 'int', 'min': 0, 'max': 5000, 'suffix': ' ms'},
        'fade_out': {'type': 'int', 'min': 0, 'max': 5000, 'suffix': ' ms'},
        'max_words': {'type': 'int', 'min': 1, 'max': 50},
        'color': {'type': 'color'},
    },
    'googler': {
        'api_key': {'type': 'str'}, # Assuming basic string
        'aspect_ratio': {'type': 'choice', 'options': ["IMAGE_ASPECT_RATIO_LANDSCAPE", "IMAGE_ASPECT_RATIO_PORTRAIT", "IMAGE_ASPECT_RATIO_SQUARE"]},
        'video_prompt': {'type': 'text_edit_button'},
        'negative_prompt': {'type': 'str'},
        'seed': {'type': 'str'},
    },
    'pollinations': {
        'model': {'type': 'choice', 'options': ["flux", "flux-realism", "flux-3d", "flux-cablyai", "dall-e-3", "midjourney", "boreal"]},
        'token': {'type': 'str'},
        'width': {'type': 'int', 'min': 64, 'max': 4096},
        'height': {'type': 'int', 'min': 64, 'max': 4096},
        'nologo': {'type': 'bool'},
        'enhance': {'type': 'bool'},
    },
     'image_prompt_settings': {
        'prompt': {'type': 'text_edit_button'},
        'model': {'type': 'model_selection'},
        'max_tokens': {'type': 'int', 'min': 1, 'max': 128000},
        'temperature': {'type': 'float', 'min': 0.0, 'max': 2.0, 'step': 0.1},
    },
    'languages_config': {
        'prompt': {'type': 'text_edit_button'},
        'model': {'type': 'model_selection'},
        'max_tokens': {'type': 'int', 'min': 1, 'max': 128000},
        'temperature': {'type': 'float', 'min': 0.0, 'max': 2.0, 'step': 0.1},
        'background_music_path': {'type': 'file_path'},
        'background_music_volume': {'type': 'int', 'min': 0, 'max': 100},
        'tts_provider': {'type': 'choice', 'options': ["ElevenLabs", "VoiceMaker", "GeminiTTS"]},
        'voicemaker_voice_id': {'type': 'voicemaker_voice'},
        'gemini_voice': {'type': 'gemini_voice'},
        'gemini_tone': {'type': 'str'},
        'edgetts_voice': {'type': 'str'}, 
        'edgetts_rate': {'type': 'int', 'min': -100, 'max': 100, 'suffix': '%'},
        'edgetts_pitch': {'type': 'int', 'min': -100, 'max': 100, 'suffix': ' Hz'},
        'elevenlabs_template_uuid': {'type': 'str'},
        'default_template': {'type': 'str'}, 
    }
}

class SettingsDelegate(QStyledItemDelegate):
    def __init__(self, parent=None):
        super().__init__(parent)

    
    def get_metadata_for_index(self, index):
        key_path = index.model().data(index, Qt.UserRole + 1)
        if not key_path:
            return {}

        # Traverse metadata using the key path
        metadata = SETTINGS_METADATA
        # Special handling for languages_config which has dynamic keys (language IDs)
        # Structure: languages_config -> [lang_id] -> [setting_key]
        if len(key_path) >= 3 and key_path[0] == 'languages_config':
             # key_path[0] is 'languages_config'
             # key_path[1] is 'uk', 'en', etc (the lang_id)
             # key_path[2] is the actual setting, e.g., 'prompt'
             
             # We want to look up 'prompt' inside 'languages_config' in our METADATA
             # So we skip the lang_id level in metadata lookup
             setting_key = key_path[-1]
             
             # Check if defined directly under languages_config
             if setting_key in SETTINGS_METADATA['languages_config']:
                 return SETTINGS_METADATA['languages_config'][setting_key]

        
        for key in key_path:
            if isinstance(metadata, dict):
                metadata = metadata.get(key, {})
            else:
                return {} 
        return metadata or {}


    def createEditor(self, parent, option, index):
        # We only want editors for the value column (1)
        if index.column() != 1:
            return None

        metadata = self.get_metadata_for_index(index)
        editor_type = metadata.get('type')
        
        editor = None
        if editor_type == 'bool' or editor_type == 'color':
            # For checkboxes and color pickers, we handle interaction in editorEvent
            return None
        elif editor_type in ['text_edit', 'text_edit_button']:
            # For text edit, we handle it via double-click in editorEvent to open dialog
            return None
        elif editor_type == 'choice':
            editor = QComboBox(parent)
            editor.addItems(metadata.get('options', []))
        elif editor_type == 'int':
            editor = QSpinBox(parent)
            editor.setRange(metadata.get('min', -2147483647), metadata.get('max', 2147483647))
            if 'suffix' in metadata: editor.setSuffix(metadata.get('suffix'))
        elif editor_type == 'float':
            editor = QDoubleSpinBox(parent)
            editor.setRange(metadata.get('min', -1e9), metadata.get('max', 1e9))
            editor.setSingleStep(metadata.get('step', 0.1))
            if 'suffix' in metadata: editor.setSuffix(metadata.get('suffix'))
        elif editor_type == 'font':
            editor = QFontComboBox(parent)
        elif editor_type == 'folder_path':
            editor = PathEditor(parent, mode='directory')
        elif editor_type == 'file_path':
            editor = PathEditor(parent, mode='file')
        elif editor_type == 'model_selection':
            editor = QComboBox(parent)
            models = settings_manager.get("openrouter_models", [])
            editor.addItems(models)
            editor.setEditable(True) # Allow typing custom models
        elif editor_type == 'voicemaker_voice':
            editor = QComboBox(parent)
            # Populate with all voices
            for lang_data in VOICEMAKER_VOICES:
                 lang_code = lang_data.get("LanguageCode", "Unknown")
                 for voice_id in lang_data.get("Voices", []):
                     editor.addItem(f"{lang_code}: {voice_id}", voice_id)
        elif editor_type == 'gemini_voice':
            editor = QComboBox(parent)
            for voice in GEMINI_VOICES:
                editor.addItem(f"{voice['name']} ({voice['description']})", voice['value'])
        
        if editor:
            editor.setAutoFillBackground(True)
            return editor
            
        return super().createEditor(parent, option, index)

    def setEditorData(self, editor, index):
        value = index.model().data(index, Qt.EditRole)
        
        if isinstance(editor, QComboBox) and not isinstance(editor, QFontComboBox):
            index = editor.findText(str(value))
            if index >= 0:
                editor.setCurrentIndex(index)
            else:
                 # If editable, set text
                 if editor.isEditable():
                     editor.setCurrentText(str(value))
        elif isinstance(editor, QFontComboBox):
            editor.setCurrentFont(str(value))
        elif isinstance(editor, (QSpinBox, QDoubleSpinBox)):
            try:
                editor.setValue(float(value))
            except (ValueError, TypeError):
                pass
        elif isinstance(editor, PathEditor):
            editor.setText(str(value))
        else:
            super().setEditorData(editor, index)

    def setModelData(self, editor, model, index):
        if isinstance(editor, QComboBox) and not isinstance(editor, QFontComboBox):
            model.setData(index, editor.currentText(), Qt.EditRole)
        elif isinstance(editor, QFontComboBox):
            model.setData(index, editor.currentFont().family(), Qt.EditRole)
        elif isinstance(editor, (QSpinBox, QDoubleSpinBox)):
            model.setData(index, editor.value(), Qt.EditRole)
        elif isinstance(editor, PathEditor):
            model.setData(index, editor.text(), Qt.EditRole)
        else:
            super().setModelData(editor, model, index)

    def paint(self, painter, option, index):
        if index.column() == 1:
            metadata = self.get_metadata_for_index(index)
            if metadata.get('type') == 'color':
                value = index.model().data(index, Qt.EditRole)
                if isinstance(value, list) and len(value) >= 3:
                     try:
                        color = QColor(int(value[0]), int(value[1]), int(value[2]))
                        painter.save()
                        
                        # Draw color swatch
                        rect = option.rect.adjusted(4, 4, -4, -4)
                        painter.setBrush(QBrush(color))
                        painter.setPen(Qt.NoPen)
                        painter.drawRoundedRect(rect, 2, 2)
                        
                        painter.restore()
                     except (ValueError, TypeError):
                         pass
                return

            if metadata.get('type') == 'bool':
                # Manually paint a checkbox
                check_box_option = QStyleOptionViewItem(option)
                check_box_option.features |= QStyleOptionViewItem.HasCheckIndicator
                
                value = index.model().data(index, Qt.DisplayRole)
                check_box_option.checkState = Qt.Checked if value else Qt.Unchecked

                # Center the checkbox
                style = QApplication.style()
                rect = style.subElementRect(QStyle.SE_CheckBoxIndicator, check_box_option, None)
                check_box_option.rect.setLeft(option.rect.left() + (option.rect.width() - rect.width()) // 2)
                
                style.drawControl(QStyle.CE_ItemViewItem, check_box_option, painter)
                return

        super().paint(painter, option, index)

    def editorEvent(self, event, model, option, index):
        if index.column() == 1:
            metadata = self.get_metadata_for_index(index)
            editor_type = metadata.get('type')
            
            if editor_type == 'color':
                if event.type() == QEvent.MouseButtonRelease and event.button() == Qt.LeftButton:
                    current_value = model.data(index, Qt.EditRole)
                    initial_color = Qt.white
                    if isinstance(current_value, list) and len(current_value) >= 3:
                        initial_color = QColor(current_value[0], current_value[1], current_value[2])
                    
                    color = QColorDialog.getColor(initial_color, None, translator.translate("pick_color_title", "Select Color"))
                    if color.isValid():
                        model.setData(index, [color.red(), color.green(), color.blue()], Qt.EditRole)
                    return True

            if editor_type == 'bool':
                if event.type() == QEvent.MouseButtonRelease and event.button() == Qt.LeftButton:
                    current_value = model.data(index, Qt.DisplayRole)
                    model.setData(index, not current_value)
                    return True
            
            if editor_type in ['text_edit', 'text_edit_button']:
                 if event.type() == QEvent.MouseButtonDblClick and event.button() == Qt.LeftButton:
                     current_value = model.data(index, Qt.EditRole)
                     # Find parent widget to use as parent for dialog
                     parent_widget = option.widget
                     dialog = PromptEditorDialog(parent_widget, str(current_value))
                     if dialog.exec():
                         model.setData(index, dialog.get_text(), Qt.EditRole)
                     return True

        return super().editorEvent(event, model, option, index)

    def updateEditorGeometry(self, editor, option, index):
        editor.setGeometry(option.rect)

class TextEditorButton(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.line_edit = QLineEdit()
        self.line_edit.setReadOnly(True)
        self.edit_btn = QPushButton(translator.translate("edit_button", "Edit"))
        self.edit_btn.setMaximumWidth(50)
        self.edit_btn.clicked.connect(self.open_editor)
        layout.addWidget(self.line_edit)
        layout.addWidget(self.edit_btn)
        self.setFocusProxy(self.edit_btn)
        self._text = ""

    def setText(self, text):
        self._text = text
        # Show only first line or snippet in line edit
        snippet = text.split('\n')[0]
        if len(text) > len(snippet):
            snippet += "..."
        self.line_edit.setText(snippet)
        
    def text(self):
        return self._text
        
    def open_editor(self):
        dialog = PromptEditorDialog(self, self._text)
        if dialog.exec():
            self.setText(dialog.get_text())

class PathEditor(QWidget):
    def __init__(self, parent=None, mode='directory'):
        super().__init__(parent)
        self.mode = mode
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        self.line_edit = QLineEdit()
        self.browse_btn = QPushButton("...")
        self.browse_btn.setMaximumWidth(30)
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

class TemplateEditorDialog(QDialog):
    def __init__(self, title, data, parent=None, template_name=None):
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setMinimumSize(600, 500)
        self.template_name = template_name
        
        # Make a deep copy to avoid modifying the original dict from template_manager
        self.original_data = copy.deepcopy(data)

        layout = QVBoxLayout(self)

        # Display note as read-only label
        note = self.original_data.get("__note__", "")
        if note:
            note_label = QLabel(note)
            note_label.setWordWrap(True)
            note_label.setTextInteractionFlags(Qt.TextSelectableByMouse)
            note_label.setStyleSheet("background-color: #f0f0f0; border: 1px solid #ccc; padding: 5px; border-radius: 3px; min-height: 40px;")
            layout.addWidget(note_label)

        self.tree_view = QTreeView()
        layout.addWidget(self.tree_view)

        self.model = QStandardItemModel()
        self.model.setHorizontalHeaderLabels([translator.translate("setting_header", "Setting"), translator.translate("value_header", "Value")])
        self.tree_view.setModel(self.model)
        
        self.delegate = SettingsDelegate(self.tree_view)
        self.tree_view.setItemDelegateForColumn(1, self.delegate)
        
        # We don't want the note in the tree view
        tree_data = {k: v for k, v in self.original_data.items() if k != "__note__"}

        self.populate_tree(tree_data, self.model.invisibleRootItem(), [])

        self.tree_view.expandAll()
        self.tree_view.resizeColumnToContents(0)
        
        buttons_layout = QHBoxLayout()
        self.save_button = QPushButton(translator.translate("save_button", "Save"))
        self.save_button.clicked.connect(self._on_save)
        
        close_button = QPushButton(translator.translate("close_button", "Close"))
        close_button.clicked.connect(self.accept)
        
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.save_button)
        buttons_layout.addWidget(close_button)
        layout.addLayout(buttons_layout)

    def _get_metadata_for_path(self, key_path):
        metadata = SETTINGS_METADATA
        
        # Similar logic to Delegate for languages_config wildcard
        if len(key_path) >= 3 and key_path[0] == 'languages_config':
             setting_key = key_path[-1]
             if setting_key in SETTINGS_METADATA['languages_config']:
                 return SETTINGS_METADATA['languages_config'][setting_key]

        for key in key_path:
            if isinstance(metadata, dict):
                metadata = metadata.get(key)
            else:
                return {} # Path is longer than metadata structure
        return metadata or {}

    def _on_save(self):
        new_data = self.get_data_from_tree()
        # Preserve the original note
        if "__note__" in self.original_data:
            new_data["__note__"] = self.original_data["__note__"]
        
        if self.template_name:
            template_manager.save_template(self.template_name, new_data)
            if self.parent() and isinstance(self.parent(), TemplatesTab):
                self.parent()._on_template_select(self.template_name)

        self.accept()

    def get_data_from_tree(self):
        return self._get_children_data(self.model.invisibleRootItem())

    def _get_children_data(self, parent_item):
        data = {}
        for row in range(parent_item.rowCount()):
            key_item = parent_item.child(row, 0)
            key_path = key_item.data(Qt.UserRole + 1)
            if not key_path: continue
            
            key = key_path[-1]
            
            if key_item.hasChildren():
                data[key] = self._get_children_data(key_item)
            else:
                value_item = parent_item.child(row, 1)
                data[key] = self._get_value(value_item)
        return data

    def _get_value(self, value_item):
        value = value_item.data(Qt.EditRole)
        metadata = self.delegate.get_metadata_for_index(value_item.index())
        
        if metadata.get('type') == 'string_list':
            if isinstance(value, str):
                return [item.strip() for item in value.split(',') if item.strip()]
            else:
                return []

        if isinstance(value, str):
            # Attempt to convert to bool
            if value.lower() == 'true': return True
            if value.lower() == 'false': return False
            
            # Attempt to convert to number
            try: return int(value)
            except (ValueError, TypeError): pass
            try: return float(value)
            except (ValueError, TypeError): pass

            # Attempt to safely evaluate as a Python literal (list, dict, etc.)
            try:
                return ast.literal_eval(value)
            except (ValueError, SyntaxError, TypeError):
                pass # If it's not a valid literal, return as string

        return value

    def populate_tree(self, data, parent_item, key_path):
        if not isinstance(data, dict):
            return

        for key, value in sorted(data.items()):
            current_path = key_path + [key]
            
            # Use original key for lookup, translated key for display
            display_key = translator.translate(f"{key}_label", key.replace('_', ' ').title())
            
            key_item = QStandardItem(display_key)
            key_item.setEditable(False)
            key_item.setData(current_path, Qt.UserRole + 1) # Store the key path

            value_item = QStandardItem()
            metadata = self._get_metadata_for_path(current_path)

            if isinstance(value, dict):
                parent_item.appendRow([key_item, value_item])
                value_item.setEditable(False)
                self.populate_tree(value, key_item, current_path)
            elif metadata.get('type') == 'string_list':
                value_item.setText(", ".join(value) if isinstance(value, list) else "")
                value_item.setEditable(True)
                value_item.setData(current_path, Qt.UserRole + 1)
                parent_item.appendRow([key_item, value_item])
            elif metadata.get('type') == 'color':
                # Handle color
                value_item.setData(value, Qt.EditRole)
                value_item.setEditable(True)
                value_item.setData(current_path, Qt.UserRole + 1)
                parent_item.appendRow([key_item, value_item])
            elif isinstance(value, list):
                # Handle other lists (like color) - for now, just display them as non-editable strings
                parent_item.appendRow([key_item, value_item])
                value_item.setEditable(False)
                value_item.setText(str(value))
            else:
                value_item.setData(value, Qt.EditRole)
                value_item.setEditable(True)
                value_item.setData(current_path, Qt.UserRole + 1)
                parent_item.appendRow([key_item, value_item])


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
        self.edit_button = QPushButton()
        
        buttons_layout.addWidget(self.save_button)
        buttons_layout.addWidget(self.apply_button)
        buttons_layout.addStretch()
        buttons_layout.addWidget(self.rename_button)
        buttons_layout.addWidget(self.edit_button)
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
        self.edit_button.setText(translator.translate("edit_template_button", "Edit"))
        self.template_label.setText(translator.translate("template_label"))
        self.template_name_label.setText(translator.translate("template_name_label"))
        self.notes_group.setTitle(translator.translate("template_notes_group_title", "Notes"))

    def connect_signals(self):
        self.templates_combo.currentTextChanged.connect(self._on_template_select)
        self.save_button.clicked.connect(self._on_save)
        self.apply_button.clicked.connect(self._on_apply)
        self.edit_button.clicked.connect(self._on_edit)
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
            'prompt_count_control_enabled',
            'prompt_count',
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

    def _on_edit(self):
        name = self.templates_combo.currentText()
        if not name:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("no_template_selected_error"))
            return

        template_data = template_manager.load_template(name)
        if not template_data:
            QMessageBox.warning(self, translator.translate("error"), translator.translate("template_not_found_error"))
            return

        dialog = TemplateEditorDialog(
            translator.translate("template_editor_title", "Template Editor") + f" - {name}",
            template_data,
            self,
            template_name=name
        )
        dialog.exec()
