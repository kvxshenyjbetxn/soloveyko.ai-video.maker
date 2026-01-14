from PySide6.QtWidgets import QWidget, QHBoxLayout, QToolButton, QLabel, QFormLayout
from PySide6.QtCore import Qt
from utils.settings import settings_manager
from utils.translator import translator

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
    
    # Star Button
    star_btn = QToolButton()
    star_btn.setCheckable(True)
    star_btn.setText("★") # Star character
    star_btn.setToolTip(translator.translate("add_to_quick_settings", "Add to Quick Settings"))
    star_btn.setFixedSize(30, 30)
    # Make styling clear
    # Define styles for checked/unchecked
    # Use padding 0 and margin 0 to avoid clipping, specific font family if needed, but default usually works if size is enough.
    # Added border-radius for hover effect.
    base_style = "QToolButton { border: none; background: transparent; font-size: 20px; padding: 0; margin: 0; }"
    style_checked = base_style + "QToolButton { color: #FFD700; } QToolButton:hover { background: rgba(255, 215, 0, 0.1); border-radius: 15px; }"
    style_unchecked = base_style + "QToolButton { color: #505050; } QToolButton:hover { color: #A0A0A0; background: rgba(128, 128, 128, 0.1); border-radius: 15px; }"

    # Check initial state
    quick_settings = settings_manager.get("quick_settings", [])
    if setting_key in quick_settings:
        star_btn.setChecked(True)
        star_btn.setStyleSheet(style_checked)
    else:
        star_btn.setChecked(False)
        star_btn.setStyleSheet(style_unchecked)
        
    def on_toggle(checked):
        # Re-fetch mostly to be safe
        current_quick = list(settings_manager.get("quick_settings", []))
        
        if checked:
            if setting_key not in current_quick:
                current_quick.append(setting_key)
                star_btn.setStyleSheet(style_checked)
        else:
            if setting_key in current_quick:
                while setting_key in current_quick:
                    current_quick.remove(setting_key)
                star_btn.setStyleSheet(style_unchecked)
        
        settings_manager.set("quick_settings", current_quick)
        
        if quick_panel_refresh_callback:
            try:
                quick_panel_refresh_callback()
            except Exception as e:
                print(f"Error in refresh callback: {e}")
            
    star_btn.toggled.connect(on_toggle)
    if show_star:
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
