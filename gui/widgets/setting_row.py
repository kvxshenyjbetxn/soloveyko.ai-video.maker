from PySide6.QtWidgets import QWidget, QHBoxLayout, QToolButton, QLabel, QFormLayout
from PySide6.QtCore import Qt
from utils.settings import settings_manager
from utils.translator import translator

class QuickSettingButton(QToolButton):
    def __init__(self, setting_key, quick_panel_refresh_callback=None, parent=None):
        super().__init__(parent)
        self.setting_key = setting_key
        self.refresh_callback = quick_panel_refresh_callback
        
        self.setCheckable(True)
        self.setText("★")
        self.setToolTip(translator.translate("add_to_quick_settings", "Add to Quick Settings"))
        self.setFixedSize(30, 30)
        
        self.base_style = "QToolButton { border: none; background: transparent; font-size: 20px; padding: 0; margin: 0; }"
        self.style_checked = self.base_style + "QToolButton { color: #FFD700; } QToolButton:hover { background: rgba(255, 215, 0, 0.1); border-radius: 15px; }"
        self.style_unchecked = self.base_style + "QToolButton { color: #505050; } QToolButton:hover { color: #A0A0A0; background: rgba(128, 128, 128, 0.1); border-radius: 15px; }"
    
        self.update_state()
        self.toggled.connect(self.on_toggle)
        
    def update_state(self):
        quick_settings = settings_manager.get("quick_settings", [])
        if self.setting_key in quick_settings:
            self.setChecked(True)
            self.setStyleSheet(self.style_checked)
        else:
            self.setChecked(False)
            self.setStyleSheet(self.style_unchecked)
            
    def on_toggle(self, checked):
        current_quick = list(settings_manager.get("quick_settings", []))
        
        if checked:
            if self.setting_key not in current_quick:
                current_quick.append(self.setting_key)
                self.setStyleSheet(self.style_checked)
        else:
            if self.setting_key in current_quick:
                while self.setting_key in current_quick:
                    current_quick.remove(self.setting_key)
                self.setStyleSheet(self.style_unchecked)
        
        settings_manager.set("quick_settings", current_quick)
        
        if self.refresh_callback:
            try:
                self.refresh_callback()
            except Exception as e:
                print(f"Error in refresh callback: {e}")

def add_setting_row(layout, label, widget, setting_key, quick_panel_refresh_callback=None, show_star=True):
    """
    Helper to add a row to a QFormLayout or QVBoxLayout with a Quick Settings toggle button.
    
    Args:
        layout: The layout to add to (must be QFormLayout or compatible with addRow)
        label: QLabel or string for the setting label
        widget: The editor widget (QComboBox, QSpinBox, etc.)
        setting_key: The string key in settings_manager
        quick_panel_refresh_callback: Optional function to call when setting is toggled (to refresh side panel)
    """
    
    # Container for [Widget | Star Button]
    container = QWidget()
    h_layout = QHBoxLayout(container)
    h_layout.setContentsMargins(0, 0, 0, 0)
    h_layout.setSpacing(5)
    
    # Ensure widget expands
    h_layout.addWidget(widget, 1)
    
    if show_star:
        star_btn = QuickSettingButton(setting_key, quick_panel_refresh_callback)
        h_layout.addWidget(star_btn)
    
    h_layout.addStretch()
    
    # Add to main layout
    if isinstance(layout, QFormLayout):
        if label is None:
            layout.addRow(container)
        else:
            layout.addRow(label, container)
    else:
        # Fallback 
        row_widget = QWidget()
        row_layout = QHBoxLayout(row_widget)
        row_layout.setContentsMargins(0,0,0,0)
        
        if label is not None:
            if isinstance(label, str):
                lbl = QLabel(label)
                row_layout.addWidget(lbl)
            else:
                row_layout.addWidget(label)
        
        row_layout.addWidget(container)
        layout.addWidget(row_widget)
    
    return container
