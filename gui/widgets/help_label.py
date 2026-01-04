from PySide6.QtWidgets import QLabel, QMessageBox, QWidget
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QCursor
from utils.hint_manager import hint_manager
from utils.translator import translator

class HelpLabel(QLabel):
    clicked = Signal()

    def __init__(self, hint_key, parent=None):
        super().__init__("?", parent)
        self.hint_key = hint_key
        self.setFixedSize(18, 18)
        self.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.setCursor(QCursor(Qt.CursorShape.PointingHandCursor))
        self.setObjectName("help_label")
        self.update_tooltip()
        
        # Style: Circle with question mark
        self.setStyleSheet("""
            QLabel#help_label {
                background-color: #444444;
                border: 1px solid #999999;
                border-radius: 9px;
                color: #ffffff;
                font-weight: bold;
                font-size: 11px;
            }
            QLabel#help_label:hover {
                background-color: #666666;
                color: #ffffff;
                border: 1px solid #bbbbbb;
            }
        """)

    def update_tooltip(self):
        hint_text = hint_manager.get_hint(self.hint_key)
        if hint_text:
            self.setToolTip(hint_text)
        else:
            self.setToolTip("")

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.show_help_dialog()
            self.clicked.emit()
            
    def show_help_dialog(self):
        hint_text = hint_manager.get_hint(self.hint_key)
        if hint_text:
            msg = QMessageBox(self)
            msg.setWindowTitle(translator.translate("help_title", "Інформація"))
            msg.setText(hint_text)
            msg.setIcon(QMessageBox.Icon.Information)
            msg.setStandardButtons(QMessageBox.StandardButton.Ok)
            msg.exec()
