from PySide6.QtWidgets import QDialog, QTextEdit, QDialogButtonBox, QVBoxLayout
from PySide6.QtCore import Qt
from utils.translator import translator

class PromptEditorDialog(QDialog):
    def __init__(self, parent=None, initial_text=""):
        super().__init__(parent)
        self.setWindowTitle(translator.translate("prompt_editor_title", "Prompt Editor"))
        self.setMinimumSize(700, 500)
        
        # Start maximized
        # self.setWindowState(self.windowState() | Qt.WindowState.WindowMaximized)

        layout = QVBoxLayout(self)
        self.text_edit = QTextEdit()
        self.text_edit.setPlainText(initial_text)
        layout.addWidget(self.text_edit)

        button_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        button_box.accepted.connect(self.accept)
        button_box.rejected.connect(self.reject)
        layout.addWidget(button_box)

    def get_text(self):
        return self.text_edit.toPlainText()