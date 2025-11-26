import re
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QTextEdit, QHBoxLayout, QLabel,
    QPushButton, QFrame, QCheckBox
)
from PySide6.QtCore import Qt
from functools import partial
from utils.translator import translator
from utils.settings import settings_manager

class StageSelectionWidget(QWidget):
    """A compact widget representing the processing stages for a single language."""
    def __init__(self, language_name):
        super().__init__()
        layout = QHBoxLayout(self)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(6)

        self.lang_label = QLabel(f"{language_name}:")
        layout.addWidget(self.lang_label)

        line = QFrame()
        line.setFrameShape(QFrame.Shape.VLine)
        line.setFrameShadow(QFrame.Shadow.Sunken)
        layout.addWidget(line)
        
        self.translation_check = QCheckBox(translator.translate("stage_translation"))
        self.img_prompts_check = QCheckBox(translator.translate("stage_img_prompts"))
        self.images_check = QCheckBox(translator.translate("stage_images"))
        self.voiceover_check = QCheckBox(translator.translate("stage_voiceover"))
        self.subtitles_check = QCheckBox(translator.translate("stage_subtitles"))
        self.montage_check = QCheckBox(translator.translate("stage_montage"))

        self.stages_layout = QHBoxLayout()
        self.stages_layout.addWidget(self.translation_check)
        self.stages_layout.addWidget(self.img_prompts_check)
        self.stages_layout.addWidget(self.images_check)
        self.stages_layout.addWidget(self.voiceover_check)
        self.stages_layout.addWidget(self.subtitles_check)
        self.stages_layout.addWidget(self.montage_check)
        self.stages_layout.addStretch()
        
        layout.addLayout(self.stages_layout)

class TextTab(QWidget):
    def __init__(self):
        super().__init__()
        self.settings = settings_manager
        self.language_buttons = {}
        self.stage_widgets = {}
        self.init_ui()
        self.load_languages_menu()

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

        # Main text input
        self.text_edit = QTextEdit()
        self.text_edit.setObjectName("textEdit")
        self.text_edit.textChanged.connect(self.update_char_count)
        layout.addWidget(self.text_edit, 1)

        # --- Languages Menu ---
        self.languages_menu_widget = QWidget()
        self.languages_menu_layout = QHBoxLayout(self.languages_menu_widget)
        self.languages_menu_layout.setContentsMargins(0,0,0,0)
        layout.addWidget(self.languages_menu_widget, 0, Qt.AlignmentFlag.AlignLeft)
        
        # --- Stages Container ---
        self.stages_container = QWidget()
        self.stages_container_layout = QVBoxLayout(self.stages_container)
        self.stages_container_layout.setContentsMargins(0,0,0,0)
        self.stages_container_layout.setSpacing(2)
        layout.addWidget(self.stages_container)

        # Status bar
        self.status_bar_layout = QHBoxLayout()
        self.openrouter_balance_label = QLabel()
        self.status_bar_layout.addWidget(self.openrouter_balance_label)
        self.status_bar_layout.addStretch()
        layout.addLayout(self.status_bar_layout)

        self.update_char_count()
        self.retranslate_ui()

    def load_languages_menu(self):
        for i in reversed(range(self.languages_menu_layout.count())):
            widget = self.languages_menu_layout.itemAt(i).widget()
            if widget is not None:
                widget.deleteLater()
        self.language_buttons = {}

        languages = self.settings.get("languages_config", {})
        for lang_id, config in languages.items():
            display_name = config.get("display_name", lang_id)
            btn = QPushButton(display_name)
            btn.setCheckable(True)
            btn.setFixedHeight(25)
            btn.toggled.connect(partial(self.on_language_toggled, lang_id, display_name))
            self.languages_menu_layout.addWidget(btn)
            self.language_buttons[lang_id] = btn
        
        self.languages_menu_layout.addStretch()

    def on_language_toggled(self, lang_id, lang_name, checked):
        if checked:
            if lang_id not in self.stage_widgets:
                stage_widget = StageSelectionWidget(lang_name)
                self.stages_container_layout.addWidget(stage_widget)
                self.stage_widgets[lang_id] = stage_widget
            self.stage_widgets[lang_id].setVisible(True)
        else:
            if lang_id in self.stage_widgets:
                widget_to_remove = self.stage_widgets.pop(lang_id)
                self.stages_container_layout.removeWidget(widget_to_remove)
                widget_to_remove.deleteLater()
        
        self.update_stage_label_widths()

    def update_stage_label_widths(self):
        max_width = 0
        
        visible_labels = [widget.lang_label for widget in self.stage_widgets.values() if widget.isVisible()]

        if not visible_labels:
            return

        # First pass: find the maximum required width
        for label in visible_labels:
            max_width = max(max_width, label.sizeHint().width())
        
        # Second pass: apply the maximum width to all visible labels
        for label in visible_labels:
            label.setMinimumWidth(max_width)

    def update_char_count(self):
        text = self.text_edit.toPlainText()
        total_chars = len(text)
        clean_text = re.sub(r'[\s\W_]', '', text)
        clean_chars = len(clean_text)
        paragraphs = len([p for p in re.split(r'\n+', text) if p.strip()]) if text else 0
        self.total_chars_label.setText(f"{translator.translate('total_chars_label')}: {total_chars}")
        self.clean_chars_label.setText(f"{translator.translate('clean_chars_label')}: {clean_chars}")
        self.paragraphs_label.setText(f"{translator.translate('paragraphs_label')}: {paragraphs}")

    def retranslate_ui(self):
        self.update_char_count()
        self.update_balance()
        # A full retranslation would require recreating the stage widgets.
        # This is a limitation for now.
        self.load_languages_menu()

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
