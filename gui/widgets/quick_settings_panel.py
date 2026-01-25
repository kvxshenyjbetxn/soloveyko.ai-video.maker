from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QScrollArea, 
                               QCheckBox, QComboBox, QSpinBox, QDoubleSpinBox, 
                               QRadioButton, QButtonGroup, QFrame, QToolButton,
                               QPushButton, QLineEdit, QFileDialog, QColorDialog, QHBoxLayout, QStyle, QSizePolicy, QFormLayout)
from PySide6.QtCore import Qt, QSize, QByteArray
from PySide6.QtGui import QColor, QIcon, QPixmap
from utils.settings import settings_manager
from utils.translator import translator
from gui.settings_metadata import SETTINGS_METADATA, KEY_TO_TRANSLATION_MAP

class QuickSettingsPanel(QWidget):
    def __init__(self, main_window=None, parent=None):
        super().__init__(parent)
        self.main_window = main_window
        self.widgets = {}
        self.init_ui()
        self.refresh()

    def init_ui(self):
        # Regular Vertical Layout (No internal sidebar)
        self.root_layout = QVBoxLayout(self)
        self.root_layout.setContentsMargins(0, 5, 0, 0) 
        self.root_layout.setSpacing(0)
        
        # Header
        header_frame = QFrame()
        header_layout = QHBoxLayout(header_frame)
        header_layout.setContentsMargins(10, 5, 10, 5)
        title_label = QLabel(translator.translate('quick_settings_title', "Quick Settings"))
        title_label.setStyleSheet("font-weight: bold; color: gray;")
        header_layout.addWidget(title_label)
        header_layout.addStretch()
        self.root_layout.addWidget(header_frame)

        # Scroll Area
        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        
        self.settings_container_widget = QWidget()
        self.content_layout = QVBoxLayout(self.settings_container_widget)
        self.content_layout.setContentsMargins(10, 5, 20, 5) # Margin to prevent clipping by scrollbar
        self.content_layout.setSpacing(15)
        self.content_layout.addStretch()
        
        self.scroll_area.setWidget(self.settings_container_widget)
        self.root_layout.addWidget(self.scroll_area)
        
        # Footer
        footer_frame = QFrame()
        footer_layout = QVBoxLayout(footer_frame)
        footer_layout.setContentsMargins(10, 10, 10, 10)
        footer_layout.setSpacing(5) # Add spacing between elements
        
        # Main Footer Text
        footer_text = translator.translate('quick_settings_footer', "Quickly access frequently used settings without switching tabs.")
        self.footer_label = QLabel(footer_text)
        self.footer_label.setWordWrap(True)
        self.footer_label.setStyleSheet("color: gray; font-size: 10px; font-style: italic;")
        self.footer_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer_layout.addWidget(self.footer_label)

        # Close Hint Text
        close_hint_text = translator.translate('quick_settings_close_hint', "To close, drag the slider to the right.")
        self.close_hint_label = QLabel(close_hint_text)
        self.close_hint_label.setWordWrap(True)
        self.close_hint_label.setStyleSheet("color: #666; font-size: 9px;") # Slightly dimmer or different
        self.close_hint_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer_layout.addWidget(self.close_hint_label)

        self.root_layout.addWidget(footer_frame)



    def refresh(self):
        # Clear existing widgets and spacers properly
        while self.content_layout.count():
            item = self.content_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
            elif item.layout():
                # Recursively delete layout items if needed
                while item.layout().count():
                     sub_item = item.layout().takeAt(0)
                     if sub_item.widget():
                         sub_item.widget().setParent(None)
                item.layout().setParent(None)
            # Spacers are just removed by takeAt(0)
        
        # Rebuild based on quick_settings list
        quick_settings = settings_manager.get("quick_settings", [])
        
        # Explicitly filter out disabled settings (just in case they were already saved)
        disabled_quick_settings = ['accent_color', 'detailed_logging_enabled', 'max_download_threads', 'language', 'theme']
        quick_settings = [k for k in quick_settings if k not in disabled_quick_settings]
        
        if 'prompt_count_control_enabled' in quick_settings and 'prompt_count' in quick_settings:
             quick_settings.remove('prompt_count')
        
        # If 'montage.special_processing_mode' is present, filter out its dependent keys
        special_proc_dependents = [
            'montage.special_processing_image_count',
            'montage.special_processing_duration_per_image',
            'montage.special_processing_video_count',
            'montage.special_processing_check_sequence'
        ]
        if 'montage.special_processing_mode' in quick_settings:
             quick_settings = [k for k in quick_settings if k not in special_proc_dependents]
        
        has_items = False
        for key in quick_settings:
            # Check hardcoded first or metadata
            # Check hardcoded first or metadata
            metadata = None
            hardcoded = self._get_hardcoded_metadata(key)
            
            if hardcoded:
                 metadata = hardcoded
            elif key in SETTINGS_METADATA:
                metadata = SETTINGS_METADATA[key]
            elif "." in key:
                 # Try nested metadata
                 parts = key.split(".")
                 current_meta = SETTINGS_METADATA
                 try:
                     for p in parts:
                         current_meta = current_meta[p]
                     metadata = current_meta
                 except (KeyError, TypeError):
                     pass
            
            if metadata:
                self._add_setting_widget(key, metadata)
                has_items = True
 
        self.content_layout.addStretch()

    def _get_hardcoded_metadata(self, key):
        if key == 'language':
            return {'type': 'choice', 'options': ['uk', 'en', 'ru'], 'label': 'language_label'}
        if key == 'theme':
            return {'type': 'choice', 'options': ['light', 'dark', 'black'], 'label': 'theme_label'}
        if key == 'detailed_logging_enabled':
             return {'type': 'bool', 'label': 'detailed_logging_label'}
        return {}

    def _add_setting_widget(self, key, metadata):
        # Container for the setting
        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(5)
        self.content_layout.addWidget(container)
        
        # Label
        label_key = metadata.get('label')
        if not label_key:
             label_key = KEY_TO_TRANSLATION_MAP.get(key, key)
             
        label_text = translator.translate(label_key, label_key)
        label = QLabel(label_text)
        label.setWordWrap(True)
        label.setStyleSheet("font-weight: bold;")
        
        h_header = QWidget()
        h_header_layout = QVBoxLayout(h_header)
        h_header_layout.setContentsMargins(0,0,0,0)
        h_header_layout.addWidget(label)
        layout.addWidget(h_header)

        # Widget Creation
        current_value = settings_manager.get(key)
        setting_type = metadata.get('type')
        
        if key == 'prompt_count_control_enabled':
            # Combined widget for Prompt Count Control + Value
            
            # 1. Checkbox
            checkbox = QCheckBox(translator.translate('enable', 'Enable'))
            current_checked = bool(current_value)
            checkbox.setChecked(current_checked)
            
            # 2. Spinbox (for prompt_count)
            count_val = settings_manager.get('prompt_count', 10)
            spinbox = QSpinBox()
            spinbox.setRange(1, 1000)
            spinbox.setValue(int(count_val))
            
            # Wrap spinbox for alignment
            spin_container = QWidget()
            spin_layout = QHBoxLayout(spin_container)
            spin_layout.setContentsMargins(0, 0, 0, 0)
            spin_layout.setAlignment(Qt.AlignmentFlag.AlignLeft)
            spin_layout.addWidget(spinbox)
            spin_layout.addStretch()
            
            # Set initial visibility
            spin_container.setVisible(current_checked)
            
            # Logic
            def on_check_change(state):
                is_checked = bool(state)
                self._update_setting(key, is_checked)
                spin_container.setVisible(is_checked)
                
            checkbox.stateChanged.connect(on_check_change)
            
            def on_spin_change(val):
                self._update_setting('prompt_count', val)
                
            spinbox.valueChanged.connect(on_spin_change)
            
            layout.addWidget(checkbox)
            layout.addWidget(spin_container)

        elif key == 'montage.special_processing_mode':
             # Compound widget for Special Processing
             
             # 1. Mode ComboBox
             options = metadata.get('options', [])
             mode_combo = QComboBox()
             for opt in options:
                 mode_combo.addItem(translator.translate(f"special_proc_mode_{opt.lower().replace(' ', '_')}", opt), opt)
                 
             idx = mode_combo.findData(current_value)
             if idx >= 0:
                 mode_combo.setCurrentIndex(idx)
             
             layout.addWidget(mode_combo)
             
             # 2. Container for sub-settings
             sub_settings_container = QWidget()
             sub_layout = QFormLayout(sub_settings_container)
             sub_layout.setContentsMargins(10, 0, 0, 0) # Indent
             
             # --- Sub Widgets ---
             
             # Image Count
             img_count_label = QLabel(translator.translate("image_count_label"))
             img_count_spin = QSpinBox()
             img_count_spin.setRange(1, 100)
             img_count_spin.setValue(int(settings_manager.get("montage.special_processing_image_count", 5)))
             img_count_spin.valueChanged.connect(lambda v: self._update_setting("montage.special_processing_image_count", v))
             
             # Duration
             dur_label = QLabel(translator.translate("duration_per_image_label"))
             dur_spin = QDoubleSpinBox()
             dur_spin.setRange(0.1, 10.0)
             dur_spin.setSingleStep(0.1)
             dur_spin.setSuffix(" s")
             dur_spin.setValue(float(settings_manager.get("montage.special_processing_duration_per_image", 2.0)))
             dur_spin.valueChanged.connect(lambda v: self._update_setting("montage.special_processing_duration_per_image", v))
             
             # Video Count
             vid_count_label = QLabel(translator.translate("special_proc_video_count_label"))
             vid_count_spin = QSpinBox()
             vid_count_spin.setRange(1, 100)
             vid_count_spin.setValue(int(settings_manager.get("montage.special_processing_video_count", 1)))
             vid_count_spin.valueChanged.connect(lambda v: self._update_setting("montage.special_processing_video_count", v))
             
             # Check Sequence
             check_seq_cb = QCheckBox(translator.translate("special_proc_check_sequence_label"))
             check_seq_cb.setChecked(settings_manager.get("montage.special_processing_check_sequence", False))
             check_seq_cb.toggled.connect(lambda v: self._update_setting("montage.special_processing_check_sequence", v))
             
             # Add to layout (we will control visibility by Row or Widget)
             # To make it easier, let's group them into "Quick Show Group" and "Video Group" widgets?
             # Or just add all to form and hide rows. QFormLayout doesn't easily hide rows by widget reference usually without iterating.
             # Better: Use distinct widgets for groups.
             
             # Group: Quick Show
             quick_show_widget = QWidget()
             qs_layout = QFormLayout(quick_show_widget)
             qs_layout.setContentsMargins(0,0,0,0)
             qs_layout.addRow(img_count_label, img_count_spin)
             qs_layout.addRow(dur_label, dur_spin)
             
             # Group: Video
             video_widget = QWidget()
             v_layout = QFormLayout(video_widget)
             v_layout.setContentsMargins(0,0,0,0)
             v_layout.addRow(vid_count_label, vid_count_spin)
             v_layout.addRow(check_seq_cb)
             
             sub_layout.addRow(quick_show_widget)
             sub_layout.addRow(video_widget)
             
             layout.addWidget(sub_settings_container)
             
             def update_visibility(index, save=True):
                 data = mode_combo.itemData(index)
                 if save:
                     self._update_setting(key, data)
                 
                 is_quick_show = (data == "Quick show")
                 is_video = (data == "Video at the beginning")
                 
                 quick_show_widget.setVisible(is_quick_show)
                 video_widget.setVisible(is_video)
                 sub_settings_container.setVisible(is_quick_show or is_video)

             mode_combo.currentIndexChanged.connect(lambda idx: update_visibility(idx, save=True))
             
             # Initial state
             update_visibility(mode_combo.currentIndex(), save=False)

        elif setting_type == 'bool':
            widget = QCheckBox(translator.translate('enable', 'Enable'))
            widget.setChecked(bool(current_value))
            widget.stateChanged.connect(lambda state: self._update_setting(key, bool(state)))
            layout.addWidget(widget)
            
        elif setting_type == 'choice':
            options = metadata.get('options', [])
            # Always use ComboBox for consistency
            widget = QComboBox()
            for opt in options:
                display_text = opt
                
                # Check for specific mappings
                if key == 'language':
                    lang_map = {'uk': "Українська", 'en': "English", 'ru': "Русский"}
                    display_text = lang_map.get(opt, opt)
                elif key == 'theme':
                    theme_map = {'light': 'light_theme', 'dark': 'dark_theme', 'black': 'black_theme'}
                    display_text = translator.translate(theme_map.get(opt, opt), opt)
                elif key == 'image_generation_provider':
                    # Special Case: Uppercase or mapped names
                    provider_map = {
                        "pollinations": "Pollinations",
                        "googler": "Googler",
                        "elevenlabs_image": "ElevenLabsImage"
                    }
                    display_text = provider_map.get(opt, opt)
                elif key == 'googler.aspect_ratio' or key == 'elevenlabs_image.aspect_ratio':
                     # Mapping for aspect ratio values if needed, or just nicer display
                     # For now usually they are readable, but metadata has label mapping in json usually
                     # "IMAGE_ASPECT_RATIO_LANDSCAPE" -> translator? 
                     # Let's try to translate the option itself
                     display_text = translator.translate(opt, opt)
                else:
                    pass
                
                widget.addItem(str(display_text), opt)
            
            idx = widget.findData(current_value)
            if idx >= 0:
                widget.setCurrentIndex(idx)
                
            widget.currentIndexChanged.connect(lambda idx, w=widget: self._update_setting(key, w.itemData(idx)))
            layout.addWidget(widget)
                
        elif setting_type == 'int':
            widget = QSpinBox()
            widget.setRange(metadata.get('min', 0), metadata.get('max', 100))
            widget.setValue(int(current_value) if current_value is not None else 0)
            if 'suffix' in metadata:
                widget.setSuffix(metadata['suffix'])
            widget.valueChanged.connect(lambda v: self._update_setting(key, v))
            
            # Wrap in HBox to force left alignment
            h_layout = QHBoxLayout()
            h_layout.setContentsMargins(0,0,0,0)
            h_layout.setAlignment(Qt.AlignmentFlag.AlignLeft)
            h_layout.addWidget(widget)
            h_layout.addStretch()
            layout.addLayout(h_layout)
            
        elif setting_type == 'float':
            widget = QDoubleSpinBox()
            widget.setRange(metadata.get('min', 0.0), metadata.get('max', 100.0))
            widget.setSingleStep(metadata.get('step', 0.1))
            widget.setValue(float(current_value) if current_value is not None else 0.0)
            if 'suffix' in metadata:
                widget.setSuffix(metadata['suffix'])
            widget.valueChanged.connect(lambda v: self._update_setting(key, v))
            
            # Wrap in HBox to force left alignment
            h_layout = QHBoxLayout()
            h_layout.setContentsMargins(0,0,0,0)
            h_layout.setAlignment(Qt.AlignmentFlag.AlignLeft)
            h_layout.addWidget(widget)
            h_layout.addStretch()
            layout.addLayout(h_layout)
            
        elif setting_type == 'text_edit_button' or setting_type == 'str':
             widget = QLineEdit()
             widget.setText(str(current_value) if current_value else "")
             widget.textChanged.connect(lambda t: self._update_setting(key, t))
             layout.addWidget(widget)

        elif setting_type == 'color':
            btn = QPushButton()
            btn.setMinimumHeight(30)
            btn.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Fixed)
            btn.setAutoFillBackground(True)
            
            # Determine initial color and format type
            is_rgb_list = False
            col = QColor('#FFFFFF')
            
            if isinstance(current_value, list) and len(current_value) == 3:
                col = QColor(current_value[0], current_value[1], current_value[2])
                is_rgb_list = True
            elif isinstance(current_value, str) and current_value:
                col = QColor(current_value)
            
            if not col.isValid():
                col = QColor('#FFFFFF')
                
            btn.setStyleSheet(f"background-color: {col.name()}; border: 1px solid gray;")
            
            def pick_color(b=btn, _is_list=is_rgb_list):
                # Fetch current formatted value
                curr_val = settings_manager.get(key)
                start_col = QColor('#FFFFFF')
                
                if isinstance(curr_val, list) and len(curr_val) == 3:
                     start_col = QColor(curr_val[0], curr_val[1], curr_val[2])
                elif isinstance(curr_val, str) and curr_val:
                     start_col = QColor(curr_val)
                
                c = QColorDialog.getColor(start_col, self)
                if c.isValid():
                    hex_c = c.name()
                    b.setStyleSheet(f"background-color: {hex_c}; border: 1px solid gray;")
                    
                    if _is_list:
                        # Save back as list [r, g, b]
                        new_val = [c.red(), c.green(), c.blue()]
                        self._update_setting(key, new_val)
                    else:
                        # Save back as hex string
                        self._update_setting(key, hex_c)
            
            btn.clicked.connect(pick_color)
            
            layout.addWidget(btn)

        elif setting_type == 'folder_path':
             h_path = QHBoxLayout()
             h_path.setContentsMargins(0,0,0,0) 
             h_path.setSpacing(2) # Tighter spacing to push icon right
             le = QLineEdit(str(current_value) if current_value else "")
             le.setReadOnly(True)
             le.setMinimumWidth(0) # Allow shrinking
             btn = QPushButton()
             
             # Yellow folder icon (Simple SVG)
             folder_svg = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#FFC107"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>"""
             pixmap = QPixmap()
             pixmap.loadFromData(QByteArray(folder_svg.encode('utf-8')))
             btn.setIcon(QIcon(pixmap))
             
             btn.setFixedSize(30, 25)
             btn.setToolTip(translator.translate("browse_button", "Browse"))
             
             def browse():
                 d = QFileDialog.getExistingDirectory(self, translator.translate("select_directory", "Select Directory"))
                 if d:
                     le.setText(d)
                     self._update_setting(key, d)
             
             btn.clicked.connect(browse)
             h_path.addWidget(le) # Path on the left
             h_path.addWidget(btn) # Button on the right
             layout.addLayout(h_path)

        elif setting_type == 'model_selection':
             widget = QLabel(translator.translate('complex_setting_placeholder', "Setting available in main tab"))
             layout.addWidget(widget)

        # self.content_layout.addWidget(container) # Moved to top
        
        line = QFrame()
        line.setFrameShape(QFrame.Shape.HLine)
        line.setFrameShadow(QFrame.Shadow.Sunken)
        self.content_layout.addWidget(line)

    def _update_setting(self, key, value):
        settings_manager.set(key, value)
        if self.main_window:
            if hasattr(self.main_window, 'on_quick_setting_changed'):
                self.main_window.on_quick_setting_changed(key)
            elif hasattr(self.main_window, 'refresh_ui_from_settings'):
                # Fallback, but might cause loop if not handled
                # For now we'll implement on_quick_setting_changed in MainWindow next
                pass
