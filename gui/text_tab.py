from PySide6.QtWidgets import QWidget, QVBoxLayout, QTextEdit, QLabel, QHBoxLayout
from PySide6.QtCore import Qt
from utils.translator import translator

class TextTab(QWidget):
    def __init__(self):
        super().__init__()

        # Layout
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(10, 10, 10, 5) # Add some margins

        # Text Edit
        self.text_edit = QTextEdit()
        
        # Add to layout
        self.layout.addWidget(self.text_edit)

        # Bottom layout for version label
        bottom_layout = QHBoxLayout()
        bottom_layout.addStretch()
        self.version_label = QLabel("версія 0.1.1")
        self.version_label.setStyleSheet("color: grey;") # Make it less prominent
        bottom_layout.addWidget(self.version_label)

        self.layout.addLayout(bottom_layout)

        # Connect translator
        translator.language_changed.connect(self.retranslate_ui)
        self.retranslate_ui()

    def retranslate_ui(self):
        self.text_edit.setPlaceholderText(translator.tr("Enter your text here..."))
        # The version label is static and doesn't need translation
