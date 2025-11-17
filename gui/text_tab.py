import re
from PySide6.QtWidgets import QWidget, QVBoxLayout, QTextEdit, QHBoxLayout, QLabel
from utils.translator import translator

class TextTab(QWidget):
    def __init__(self):
        super().__init__()
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)

        # Character count labels
        self.char_count_layout = QHBoxLayout()
        
        self.total_chars_label = QLabel()
        self.clean_chars_label = QLabel()
        self.paragraphs_label = QLabel()

        self.char_count_layout.addWidget(self.total_chars_label)
        self.char_count_layout.addStretch()
        self.char_count_layout.addWidget(self.clean_chars_label)
        self.char_count_layout.addStretch()
        self.char_count_layout.addWidget(self.paragraphs_label)

        layout.addLayout(self.char_count_layout)

        self.text_edit = QTextEdit()
        self.text_edit.setObjectName("textEdit") # For styling
        self.text_edit.textChanged.connect(self.update_char_count)
        layout.addWidget(self.text_edit)

        # Status bar
        self.status_bar_layout = QHBoxLayout()
        self.openrouter_balance_label = QLabel()
        self.status_bar_layout.addWidget(self.openrouter_balance_label)
        self.status_bar_layout.addStretch()
        layout.addLayout(self.status_bar_layout)

        self.update_char_count() # Initial count

    def update_char_count(self):
        text = self.text_edit.toPlainText()
        
        # Total characters
        total_chars = len(text)
        
        # Clean characters (no spaces or special symbols)
        clean_text = re.sub(r'[\s\W_]', '', text)
        clean_chars = len(clean_text)
        
        # Paragraphs (blocks of text separated by newlines)
        paragraphs = len([p for p in re.split(r'\n+', text) if p.strip()]) if text else 0
        
        self.total_chars_label.setText(f"{translator.translate('total_chars_label')}: {total_chars}")
        self.clean_chars_label.setText(f"{translator.translate('clean_chars_label')}: {clean_chars}")
        self.paragraphs_label.setText(f"{translator.translate('paragraphs_label')}: {paragraphs}")

    def retranslate_ui(self):
        self.update_char_count()
        self.update_balance()

    def update_balance(self):
        from api.openrouter import OpenRouterAPI
        
        api = OpenRouterAPI()
        usage = api.get_balance()

        if usage is not None:
            self.openrouter_balance_label.setText(f"{translator.translate('balance_label')} {usage:.4f}$")
        elif api.api_key:
            self.openrouter_balance_label.setText(f"{translator.translate('balance_label')} -")
        else:
            self.openrouter_balance_label.setText("")
