import re
import uuid
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QTextEdit, QHBoxLayout, QLabel,
    QPushButton, QFrame, QCheckBox, QToolButton, QInputDialog, QGridLayout
)
from PySide6.QtGui import QDragEnterEvent, QDropEvent
from PySide6.QtCore import Qt, QMimeData
from utils.flow_layout import FlowLayout
from functools import partial
from utils.translator import translator
from utils.settings import settings_manager

class DroppableTextEdit(QTextEdit):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAcceptDrops(True)

    def dragEnterEvent(self, event: QDragEnterEvent):
        if event.mimeData().hasUrls():
            urls = event.mimeData().urls()
            if any(url.toLocalFile().endswith('.txt') for url in urls):
                event.acceptProposedAction()
        else:
            super().dragEnterEvent(event)

    def dropEvent(self, event: QDropEvent):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()
            for url in event.mimeData().urls():
                file_path = url.toLocalFile()
                if file_path.endswith('.txt'):
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            self.insertPlainText(f.read())
                    except Exception as e:
                        print(f"Error reading file {file_path}: {e}")
        else:
            super().dropEvent(event)

class StageSelectionWidget(QWidget):
    """A compact widget representing the processing stages for a single language."""
    def __init__(self, language_name, parent_tab):
        super().__init__()
        self.parent_tab = parent_tab
        layout = QHBoxLayout(self)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(6)

        self.lang_label = QLabel(f"{language_name}:")
        layout.addWidget(self.lang_label)

        self.toggle_all_button = QToolButton()
        self.toggle_all_button.setObjectName("toggleAllButton")
        self.toggle_all_button.setCheckable(True)
        self.toggle_all_button.clicked.connect(self.toggle_all)

        theme_colors = {
            'light': {'primary': '#2979ff', 'text': '#3c3c3c'},
            'dark':  {'primary': '#2979ff', 'text': '#000000'},
            'black': {'primary': '#2979ff', 'text': '#ffffff'}
        }
        current_theme = self.parent_tab.settings.get('theme', 'dark')
        primary_color = theme_colors.get(current_theme, {}).get('primary', '#2979ff')
        text_color_on_primary = theme_colors.get(current_theme, {}).get('text', '#000000')


        self.toggle_all_button.setStyleSheet(f"""
            QToolButton#toggleAllButton {{
                color: {primary_color};
                border: 1px solid {primary_color};
                border-radius: 12px;
                padding: 0 8px;
                background-color: transparent;
            }}
            QToolButton#toggleAllButton:hover {{
                background-color: {primary_color}20; /* 20 is for alpha */
            }}
            QToolButton#toggleAllButton:checked, QToolButton#toggleAllButton:pressed {{
                background-color: {primary_color};
                color: {text_color_on_primary};
            }}
        """)
        self.toggle_all_button.setFixedHeight(25)
        layout.addWidget(self.toggle_all_button)

        line = QFrame()
        line.setFrameShape(QFrame.Shape.VLine)
        line.setFrameShadow(QFrame.Shadow.Sunken)
        layout.addWidget(line)
        
        self.checkboxes = {}
        stage_keys = ["stage_translation", "stage_img_prompts", "stage_images", 
                      "stage_voiceover", "stage_subtitles", "stage_montage"]
        
        for key in stage_keys:
            checkbox = QCheckBox(translator.translate(key))
            checkbox.setChecked(True)
            checkbox.stateChanged.connect(self.update_toggle_button_text)
            checkbox.stateChanged.connect(self.parent_tab.check_queue_button_visibility)
            layout.addWidget(checkbox)
            self.checkboxes[key] = checkbox
            
        layout.addStretch()
        self.update_toggle_button_text()
        
    def retranslate_ui(self):
        for key, checkbox in self.checkboxes.items():
            checkbox.setText(translator.translate(key))
        self.update_toggle_button_text()

    def are_all_selected(self):
        return all(checkbox.isChecked() for checkbox in self.checkboxes.values())

    def toggle_all(self):
        new_state = not self.are_all_selected()
        for checkbox in self.checkboxes.values():
            checkbox.setChecked(new_state)
        self.update_toggle_button_text()

    def update_toggle_button_text(self, state=None):
        all_selected = self.are_all_selected()
        self.toggle_all_button.setChecked(all_selected)
        if all_selected:
            self.toggle_all_button.setText(translator.translate("deselect_all"))
        else:
            self.toggle_all_button.setText(translator.translate("select_all"))

    def get_selected_stages(self):
        return [key for key, checkbox in self.checkboxes.items() if checkbox.isChecked()]

class TextTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
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
        self.text_edit = DroppableTextEdit()
        self.text_edit.setObjectName("textEdit")
        self.text_edit.textChanged.connect(self.update_char_count)
        self.text_edit.textChanged.connect(self.check_queue_button_visibility)
        layout.addWidget(self.text_edit, 1)

        self.apply_text_color_to_text_edit()

        # --- Stages Container ---
        self.stages_container = QWidget()
        self.stages_container_layout = QVBoxLayout(self.stages_container)
        self.stages_container_layout.setContentsMargins(0,0,0,0)
        self.stages_container_layout.setSpacing(2)
        layout.addWidget(self.stages_container)

        # --- Languages Menu ---
        self.languages_menu_container = QWidget()
        self.languages_menu_grid_layout = QGridLayout(self.languages_menu_container)
        self.languages_menu_grid_layout.setContentsMargins(0,0,0,0)

        self.languages_menu_widget = QWidget()
        self.languages_menu_layout = FlowLayout(self.languages_menu_widget)
        self.languages_menu_layout.setContentsMargins(0,0,0,0)
        self.languages_menu_layout.setSpacing(5)

        self.languages_menu_grid_layout.addWidget(self.languages_menu_widget, 0, 0)
        
        # --- Add to Queue Button ---
        self.add_to_queue_button = QPushButton(translator.translate('add_to_queue'))
        self.add_to_queue_button.setEnabled(False)
        self.add_to_queue_button.clicked.connect(self.add_to_queue)
        self.add_to_queue_button.setFixedHeight(25)
        self.languages_menu_grid_layout.addWidget(self.add_to_queue_button, 0, 1)

        self.languages_menu_grid_layout.setColumnStretch(0, 1)
        self.languages_menu_grid_layout.setColumnStretch(1, 0)

        layout.addWidget(self.languages_menu_container)

        # Status bar
        self.status_bar_layout = QHBoxLayout()
        self.openrouter_balance_label = QLabel()
        self.googler_usage_label = QLabel()
        self.elevenlabs_balance_label = QLabel()
        self.voicemaker_balance_label = QLabel()
        self.gemini_tts_balance_label = QLabel()
        self.status_bar_layout.addWidget(self.openrouter_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.googler_usage_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.elevenlabs_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.voicemaker_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.gemini_tts_balance_label)
        self.status_bar_layout.addStretch()
        layout.addLayout(self.status_bar_layout)

        self.update_char_count()
        self.retranslate_ui()

    def load_languages_menu(self):
        # Clear all previous widgets (language buttons)
        while self.languages_menu_layout.count():
            item = self.languages_menu_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()

        self.language_buttons = {}

        languages = self.settings.get("languages_config", {})
        for lang_id, config in languages.items():
            display_name = config.get("display_name", lang_id)
            btn = QPushButton(display_name)
            btn.setCheckable(True)
            btn.setFixedHeight(25)
            btn.toggled.connect(partial(self.on_language_toggled, lang_id, display_name))
            self.language_buttons[lang_id] = btn
            self.languages_menu_layout.addWidget(btn)

    def on_language_toggled(self, lang_id, lang_name, checked):
        if checked:
            if lang_id not in self.stage_widgets:
                stage_widget = StageSelectionWidget(lang_name, self)
                self.stages_container_layout.addWidget(stage_widget)
                self.stage_widgets[lang_id] = stage_widget
            self.stage_widgets[lang_id].setVisible(True)
        else:
            if lang_id in self.stage_widgets:
                self.stage_widgets[lang_id].setVisible(False)
        
        self.update_stage_label_widths()
        self.check_queue_button_visibility()

    def update_stage_label_widths(self):
        max_width = 0
        
        visible_labels = [widget.lang_label for widget in self.stage_widgets.values() if widget.isVisible()]

        if not visible_labels:
            return

        for label in visible_labels:
            max_width = max(max_width, label.sizeHint().width())
        
        for label in visible_labels:
            label.setMinimumWidth(max_width)

    def check_queue_button_visibility(self):
        lang_selected = any(btn.isChecked() for btn in self.language_buttons.values())
        
        stages_selected = False
        if lang_selected:
            for lang_id, widget in self.stage_widgets.items():
                if widget.isVisible() and any(cb.isChecked() for cb in widget.checkboxes.values()):
                    stages_selected = True
                    break

        self.add_to_queue_button.setEnabled(lang_selected and stages_selected)

    def add_to_queue(self):
        task_name, ok = QInputDialog.getText(self, translator.translate('enter_task_name_title'), translator.translate('enter_task_name_label'))
        if ok:
            if not task_name:
                task_count = self.main_window.queue_manager.get_task_count()
                task_name = f"{translator.translate('default_task_name')} {task_count + 1}"

            text = self.text_edit.toPlainText()
            
            languages_data = {}
            for lang_id, btn in self.language_buttons.items():
                if btn.isChecked():
                    stage_widget = self.stage_widgets.get(lang_id)
                    if stage_widget and stage_widget.isVisible():
                        selected_stages = stage_widget.get_selected_stages()
                        if selected_stages:
                            languages_data[lang_id] = {
                                "display_name": btn.text(),
                                "stages": selected_stages
                            }
            
            if languages_data:
                job = {
                    "id": None,  # Will be auto-generated by queue_manager
                    "name": task_name,
                    "text": text,
                    "languages": languages_data
                }
                self.main_window.queue_manager.add_task(job)

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
        # self.update_balance() # This will be called from main_window
        self.load_languages_menu()
        self.add_to_queue_button.setText(translator.translate('add_to_queue'))

        for widget in self.stage_widgets.values():
            widget.retranslate_ui()

        self.apply_text_color_to_text_edit()


    def update_balance(self, balance_text):
        self.openrouter_balance_label.setText(balance_text)

    def update_googler_usage(self, usage_text):
        self.googler_usage_label.setText(usage_text)

    def update_elevenlabs_balance(self, balance_text):
        self.elevenlabs_balance_label.setText(balance_text)

    def update_voicemaker_balance(self, balance_text):
        self.voicemaker_balance_label.setText(balance_text)

    def update_gemini_tts_balance(self, balance_text):
        self.gemini_tts_balance_label.setText(balance_text)

    def apply_text_color_to_text_edit(self):
        current_theme = self.settings.get('theme', 'dark')
        if current_theme == 'light':
            text_color = 'black'
        else:
            text_color = 'white'
        self.text_edit.setStyleSheet(f"QTextEdit#textEdit {{ color: {text_color}; }}")


