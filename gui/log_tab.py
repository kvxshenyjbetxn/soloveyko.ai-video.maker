from PySide6.QtWidgets import QWidget, QVBoxLayout, QTextBrowser, QHBoxLayout, QLineEdit, QComboBox, QCheckBox
from PySide6.QtCore import Qt
from utils.logger import LogLevel
from utils.translator import translator

class LogTab(QWidget):
    def __init__(self):
        super().__init__()
        self.all_logs = []
        self.init_ui()
        self.retranslate_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        
        # --- Filter and Search Controls ---
        controls_layout = QHBoxLayout()
        
        self.filter_combo = QComboBox()
        self.filter_combo.addItem(translator.translate("all_levels"), "ALL")
        for level in LogLevel:
            self.filter_combo.addItem(f"{level.to_icon()} {level.name}", level.name)
        self.filter_combo.currentIndexChanged.connect(self.apply_filters)
        
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText(translator.translate("search_logs"))
        self.search_input.textChanged.connect(self.apply_filters)
        
        self.autoscroll_checkbox = QCheckBox(translator.translate("autoscroll"))
        self.autoscroll_checkbox.setChecked(True)
        
        controls_layout.addWidget(self.filter_combo)
        controls_layout.addWidget(self.search_input, 1) # Give search more space
        controls_layout.addWidget(self.autoscroll_checkbox)
        layout.addLayout(controls_layout)

        # --- Log Output ---
        self.log_output = QTextBrowser()
        self.log_output.setReadOnly(True)
        self.log_output.setStyleSheet("QTextBrowser { font-family: 'Courier New', monospace; }")
        layout.addWidget(self.log_output)

    def add_log_message(self, log_data):
        self.all_logs.append(log_data)
        self.display_log(log_data)

    def display_log(self, log_data):
        level = log_data["level"]
        
        # Check against current filters
        current_filter_level = self.filter_combo.currentData()
        search_text = self.search_input.text().lower()

        if current_filter_level != "ALL" and level.name != current_filter_level:
            return
        if search_text and search_text not in log_data["message"].lower():
            return

        # Format and append
        color = level.to_color()
        icon = level.to_icon()
        
        formatted_message = (
            f'<font color="{color}">'
            f'<b>[{log_data["timestamp"]}]</b> '
            f'{icon} '
            f'<b>{level.name: <7}</b> - '
            f'{log_data["message"]}'
            f'</font>'
        )
        
        self.log_output.append(formatted_message)

        if self.autoscroll_checkbox.isChecked():
            self.log_output.verticalScrollBar().setValue(self.log_output.verticalScrollBar().maximum())

    def apply_filters(self):
        self.log_output.clear()
        for log_data in self.all_logs:
            self.display_log(log_data)
            
    def retranslate_ui(self):
        self.filter_combo.setItemText(0, translator.translate("all_levels"))
        # The rest of the combo box is dynamically generated, no need to retranslate
        self.search_input.setPlaceholderText(translator.translate("search_logs"))
        self.autoscroll_checkbox.setText(translator.translate("autoscroll"))