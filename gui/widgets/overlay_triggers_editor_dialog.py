from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QTableWidget, 
                               QTableWidgetItem, QPushButton, QHeaderView, QLineEdit, 
                               QComboBox, QWidget, QFileDialog, QLabel, QSpinBox)
from PySide6.QtCore import Qt, Signal
import os
import json
from utils.translator import translator

class TriggerPositionDialog(QDialog):
    def __init__(self, x, y, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("overlay_settings_title", "Position Settings"))
        self.setFixedSize(300, 150)
        
        layout = QVBoxLayout(self)
        
        # Grid layout for inputs
        grid = QVBoxLayout()
        
        # X
        row_x = QHBoxLayout()
        label_x = QLabel(translator.translate("trigger_horizontal", "Horizontal (X):"))
        self.spin_x = QSpinBox()
        self.spin_x.setRange(-10000, 10000)
        self.spin_x.setValue(int(x))
        self.spin_x.setSuffix(" px")
        row_x.addWidget(label_x)
        row_x.addWidget(self.spin_x)
        grid.addLayout(row_x)
        
        # Y
        row_y = QHBoxLayout()
        label_y = QLabel(translator.translate("trigger_vertical", "Vertical (Y):"))
        self.spin_y = QSpinBox()
        self.spin_y.setRange(-10000, 10000)
        self.spin_y.setValue(int(y))
        self.spin_y.setSuffix(" px")
        row_y.addWidget(label_y)
        row_y.addWidget(self.spin_y)
        grid.addLayout(row_y)
        
        layout.addLayout(grid)
        layout.addStretch()
        
        # Buttons
        buttons = QHBoxLayout()
        save_btn = QPushButton(translator.translate("save_button", "Save"))
        save_btn.clicked.connect(self.accept)
        cancel_btn = QPushButton(translator.translate("cancel_button", "Cancel"))
        cancel_btn.clicked.connect(self.reject)
        
        buttons.addWidget(save_btn)
        buttons.addWidget(cancel_btn)
        layout.addLayout(buttons)

    def get_values(self):
        return self.spin_x.value(), self.spin_y.value()

class OverlayTriggersEditorDialog(QDialog):
    def __init__(self, triggers_data, parent=None):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("overlay_triggers_group", "Dynamic Overlays (Triggers)"))
        self.resize(900, 400)
        
        # Ensure deep copy of data to avoid direct modifications if cancelled
        if isinstance(triggers_data, str):
            try:
                self.triggers = json.loads(triggers_data)
            except:
                self.triggers = []
        elif isinstance(triggers_data, list):
            import copy
            self.triggers = copy.deepcopy(triggers_data)
        else:
            self.triggers = []
            
        self.layout = QVBoxLayout(self)
        
        # Description
        desc_label = QLabel(translator.translate("trigger_dialog_desc", "Configure dynamic overlay triggers such as appearing images or videos based on text phrases or time."))
        desc_label.setWordWrap(True)
        self.layout.addWidget(desc_label)
        
        # Table
        self.triggers_table = QTableWidget()
        self.triggers_table.setColumnCount(4)
        self.triggers_table.setHorizontalHeaderLabels([
            translator.translate("trigger_column", "Trigger"),
            translator.translate("type_column", "Type"),
            translator.translate("file_column", "Effect File"),
            translator.translate("actions_column", "Actions")
        ])
        self.triggers_table.horizontalHeader().setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        self.triggers_table.horizontalHeader().setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        self.triggers_table.horizontalHeader().setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)
        self.triggers_table.horizontalHeader().setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        
        self.layout.addWidget(self.triggers_table)
        
        # Add Button
        self.add_btn = QPushButton(translator.translate("add_trigger_button", "Add Trigger"))
        self.add_btn.clicked.connect(self.add_empty_row)
        self.layout.addWidget(self.add_btn)
        
        # Bottom Buttons
        btn_layout = QHBoxLayout()
        save_btn = QPushButton(translator.translate("save_button", "Save"))
        save_btn.clicked.connect(self.accept)
        cancel_btn = QPushButton(translator.translate("close_button", "Close"))
        cancel_btn.clicked.connect(self.reject)
        
        btn_layout.addStretch()
        btn_layout.addWidget(save_btn)
        btn_layout.addWidget(cancel_btn)
        self.layout.addLayout(btn_layout)
        
        self.populate_table()

    def populate_table(self):
        self.triggers_table.setRowCount(0)
        for trigger in self.triggers:
            self.add_row_ui(trigger)

    def add_empty_row(self):
        new_trigger = {
            "type": "text",
            "value": "",
            "path": "",
            "x": 0,
            "y": 0
        }
        self.triggers.append(new_trigger)
        self.add_row_ui(new_trigger)

    def add_row_ui(self, trigger_data):
        row = self.triggers_table.rowCount()
        self.triggers_table.insertRow(row)

        # Trigger Value (Phrase or Time)
        val_text = trigger_data.get("value") or trigger_data.get("phrase") or trigger_data.get("keyword") or trigger_data.get("text") or ""
        val_input = QLineEdit(str(val_text))
        val_input.textChanged.connect(lambda text, r=row: self.update_data(r, "value", text))
        self.triggers_table.setCellWidget(row, 0, val_input)

        # Type ComboBox
        type_combo = QComboBox()
        type_combo.addItem(translator.translate("trigger_type_text", "Text Phrase"), "text")
        type_combo.addItem(translator.translate("trigger_type_time", "Time (SS or MM:SS)"), "time")
        
        current_type = trigger_data.get("type", "text")
        idx = type_combo.findData(current_type)
        if idx >= 0: type_combo.setCurrentIndex(idx)
        
        type_combo.currentIndexChanged.connect(lambda idx, r=row: self.update_data(r, "type", type_combo.itemData(idx)))
        self.triggers_table.setCellWidget(row, 1, type_combo)

        # Path Selection
        path_container = QWidget()
        path_layout = QHBoxLayout(path_container)
        path_layout.setContentsMargins(2, 2, 2, 2)
        
        full_path = trigger_data.get("path", "")
        path_input = QLineEdit()
        if full_path:
            path_input.setText(os.path.basename(full_path))
        
        path_input.setReadOnly(True)
        path_input.setPlaceholderText(translator.translate("no_file_selected", "No file selected"))
        path_input.setToolTip(full_path)
        path_input.setStyleSheet("background: #2b2b2b; color: #ffffff; border: 1px solid #444; padding: 4px;")
        
        browse_btn = QPushButton(translator.translate("browse_button", "Browse"))
        browse_btn.clicked.connect(lambda: self.browse_file(row, path_input))
        
        path_layout.addWidget(path_input)
        path_layout.addWidget(browse_btn)
        self.triggers_table.setCellWidget(row, 2, path_container)

        # Actions Layout (Delete + Settings)
        actions_container = QWidget()
        actions_layout = QHBoxLayout(actions_container)
        actions_layout.setContentsMargins(5, 0, 5, 0)
        actions_layout.setSpacing(5)
        
        # Settings Button
        settings_btn = QPushButton(translator.translate("trigger_settings", "Position"))
        settings_btn.setToolTip(translator.translate("trigger_settings_tooltip", "Adjust Position"))
        settings_btn.clicked.connect(lambda: self.open_pos_settings(row))
        
        # Remove Button
        remove_btn = QPushButton(translator.translate("trigger_delete", "Delete"))
        remove_btn.setStyleSheet("color: #ff6666;")
        remove_btn.clicked.connect(lambda: self.remove_row(row))
        
        actions_layout.addWidget(settings_btn)
        actions_layout.addWidget(remove_btn)
        self.triggers_table.setCellWidget(row, 3, actions_container)
        
        self.triggers_table.setRowHeight(row, 40)

    def update_data(self, row, key, value):
        if 0 <= row < len(self.triggers):
            self.triggers[row][key] = value

    def browse_file(self, row, display_widget):
        file_path, _ = QFileDialog.getOpenFileName(
            self, 
            translator.translate("select_file", "Select File"),
            "",
            "Media Files (*.png *.jpg *.jpeg *.mp4 *.mov *.webm *.gif);;All Files (*.*)"
        )
        if file_path:
            if 0 <= row < len(self.triggers):
                self.triggers[row]["path"] = file_path
                display_widget.setText(os.path.basename(file_path))
                display_widget.setToolTip(file_path)

    def open_pos_settings(self, row):
        if 0 <= row < len(self.triggers):
            data = self.triggers[row]
            dialog = TriggerPositionDialog(data.get("x", 0), data.get("y", 0), self)
            if dialog.exec():
                x, y = dialog.get_values()
                self.triggers[row]["x"] = x
                self.triggers[row]["y"] = y

    def remove_row(self, row):
        if 0 <= row < len(self.triggers):
            self.triggers.pop(row)
            self.populate_table()

    def get_triggers(self):
        return self.triggers
