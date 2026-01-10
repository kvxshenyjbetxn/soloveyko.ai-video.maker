from PySide6.QtWidgets import (QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem, 
                               QPushButton, QHBoxLayout, QHeaderView, QAbstractItemView)
from PySide6.QtCore import Qt
from utils.translator import translator

from gui.settings_metadata import KEY_TO_TRANSLATION_MAP

class TemplateChangesDialog(QDialog):
    def __init__(self, changes, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("template_changes_title", "Confirm Template Changes"))
        self.setMinimumSize(800, 600)
        self.changes = changes
        
        layout = QVBoxLayout(self)
        
        # Intro label
        intro_label = QLabel(translator.translate("template_changes_intro", "The following settings will be changed:"))
        intro_label.setWordWrap(True)
        layout.addWidget(intro_label)
        
        # Tree Widget
        self.tree = QTreeWidget()
        self.tree.setVerticalScrollMode(QAbstractItemView.ScrollMode.ScrollPerPixel)
        self.tree.setHeaderLabels([
            translator.translate("template_changes_template_column", "Template / Setting"),
            translator.translate("template_changes_old_value_column", "Old Value"),
            translator.translate("template_changes_new_value_column", "New Value")
        ])
        self.tree.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.tree.header().setSectionResizeMode(1, QHeaderView.ResizeToContents)
        self.tree.header().setSectionResizeMode(2, QHeaderView.ResizeToContents)
        
        layout.addWidget(self.tree)
        
        self.populate_tree()
        
        # Buttons
        btn_layout = QHBoxLayout()
        self.confirm_btn = QPushButton(translator.translate("template_changes_confirm", "Confirm"))
        self.confirm_btn.clicked.connect(self.accept)
        # Apply style to confirm button to make it prominent (e.g., success color if supports theme, or just default)
        
        self.cancel_btn = QPushButton(translator.translate("template_changes_cancel", "Cancel"))
        self.cancel_btn.clicked.connect(self.reject)
        
        btn_layout.addStretch()
        btn_layout.addWidget(self.confirm_btn)
        btn_layout.addWidget(self.cancel_btn)
        layout.addLayout(btn_layout)
        
    def populate_tree(self):
        self.tree.clear()
        
        # Structure of changes:
        # { 
        #   "Template Name": { 
        #       "setting_key": (old_val, new_val), 
        #       "group -> setting": (old_val, new_val) 
        #   } 
        # }
        
        for template_name, diffs in self.changes.items():
            if not diffs:
                continue
                
            template_item = QTreeWidgetItem(self.tree)
            template_item.setText(0, template_name)
            # Make template item bold/different
            font = template_item.font(0)
            font.setBold(True)
            template_item.setFont(0, font)
            template_item.setExpanded(True)
            
    # Definitions strictly matching TemplateEditorDialog
    GROUPS = {
        "general_tab": ["results_path", "image_review_enabled", "rewrite_review_enabled", "translation_review_enabled", "prompt_count_control_enabled", "prompt_count", "image_generation_provider"],
        "api_tab": ["openrouter_models", "openrouter_api_key", "elevenlabs_api_key", "elevenlabs_unlim_api_key", "voicemaker_api_key", "voicemaker_char_limit", "gemini_tts_api_key", "assemblyai_api_key", "googler", "pollinations", "elevenlabs_image"],
        "languages_tab": ["languages_config"],
        "prompts_tab": ["image_prompt_settings", "preview_settings"],
        "montage_tab": ["montage"],
        "subtitles_tab": ["subtitles"],
        "elevenlabs_unlim_settings_title": ["eleven_unlim_settings"]
    }

    FLATTEN_GROUPS = {
        "montage_tab": "montage",
        "subtitles_tab": "subtitles",
        "languages_tab": "languages_config"
    }

    def populate_tree(self):
        self.tree.clear()
        
        # Build reverse lookup map for groups
        key_to_group = {}
        for group, keys in self.GROUPS.items():
            for k in keys:
                key_to_group[k] = group
        
        for template_name, diffs in self.changes.items():
            if not diffs:
                continue
                
            template_item = QTreeWidgetItem(self.tree)
            template_item.setText(0, template_name)
            font = template_item.font(0)
            font.setBold(True)
            template_item.setFont(0, font)
            template_item.setExpanded(True)
            
            # Sort paths for consistent display
            sorted_paths = sorted(diffs.keys())
            
            for key_path_str in sorted_paths:
                old_val, new_val = diffs[key_path_str]
                
                parts = key_path_str.split(" -> ")
                root_key = parts[0]
                
                # Default parent is the template root
                current_parent = template_item
                
                # Determine Group
                group_key = key_to_group.get(root_key)
                
                # Logic to determine if we consume the root key (Flattening)
                consume_root_key = False
                
                if group_key:
                    # Check if this group flattens this root key
                    if self.FLATTEN_GROUPS.get(group_key) == root_key:
                        consume_root_key = True
                        
                    # Find or Create Group Item
                    group_item = None
                    for k in range(template_item.childCount()):
                        child = template_item.child(k)
                        # We use UserRole to store the ID of the group/item to identify it reliably
                        if child.data(0, Qt.UserRole) == group_key:
                            group_item = child
                            break
                    
                    if not group_item:
                        group_item = QTreeWidgetItem(template_item)
                        group_item.setData(0, Qt.UserRole, group_key)
                        group_item.setText(0, translator.translate(group_key, group_key.replace("_", " ").title()))
                        group_item.setExpanded(True)
                        # Optional: Style group item (e.g. bold or slightly different color)
                        f = group_item.font(0)
                        f.setBold(True)
                        group_item.setFont(0, f)
                    
                    current_parent = group_item
                
                # Determine parts to generate items for
                if consume_root_key:
                    parts_to_show = parts[1:]
                else:
                    parts_to_show = parts
                    # However, if the item IS put in a group (but not flattened, e.g. General -> results_path),
                    # 'results_path' is still parts[0]. But we are now adding it under 'group_item'.
                    # So we just iterate parts_to_show.
                
                # If path is empty (e.g. key itself was flattened and had no children? Unlikely for settings), handle gracefull
                if not parts_to_show:
                    # Should not happen for leaf settings unless the dict itself is the value
                    pass

                for i, part in enumerate(parts_to_show):
                    is_leaf = (i == len(parts_to_show) - 1)
                    
                    # Search for child in current_parent
                    found_item = None
                    for k in range(current_parent.childCount()):
                        child = current_parent.child(k)
                        if child.data(0, Qt.UserRole) == part:
                            found_item = child
                            break
                    
                    if not found_item:
                        found_item = QTreeWidgetItem(current_parent)
                        found_item.setData(0, Qt.UserRole, part)
                        
                        display_text = self._translate_part(part)
                        found_item.setText(0, display_text)
                        found_item.setExpanded(True)
                    
                    current_parent = found_item
                    
                    if is_leaf:
                        current_parent.setText(1, self.format_value(old_val))
                        current_parent.setText(2, self.format_value(new_val))
                        current_parent.setForeground(2, Qt.darkGreen)
                        if old_val is None:
                             current_parent.setText(1, "---")

    def _translate_part(self, part):
        if part == '*':
            return translator.translate("all_levels", "(All Languages)")
            
        trans_key = KEY_TO_TRANSLATION_MAP.get(part)
        if trans_key:
             val = translator.translate(trans_key)
             if val and val != trans_key: return val
             
        # Fallbacks 
        if part == 'googler': return "Googler"
        elif part == 'pollinations': return "Pollinations"
        elif part == 'elevenlabs_image': return "ElevenLabsImage"
        
        # Fallback to titling
        return part.replace('_', ' ').title()
                     
    def format_value(self, value):
        if value is None:
            return ""
        if isinstance(value, bool):
            return str(value)
        if isinstance(value, (list, dict)):
            # Truncate if too long?
            s = str(value)
            if len(s) > 100:
                return s[:100] + "..."
            return s
        
        # For strings (including prompts)
        s_val = str(value)
        if len(s_val) > 100:
            return s_val[:100] + "..."
        return s_val
