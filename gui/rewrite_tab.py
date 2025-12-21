from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, 
    QScrollArea, QMessageBox, QGroupBox, QCheckBox, QGridLayout, QStyle
)
from PySide6.QtCore import Qt
from gui.text_tab import DroppableTextEdit, StageSelectionWidget
from utils.translator import translator
from utils.settings import settings_manager
from utils.logger import logger, LogLevel
from utils.flow_layout import FlowLayout
from utils.animator import Animator
from functools import partial
import uuid
import datetime

class RewriteTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.stage_widgets = {} # lang_id -> StageSelectionWidget
        self.language_buttons = {} # lang_id -> QPushButton (toggle)
        self.settings = settings_manager
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)

        # Input Label
        self.input_label = QLabel(translator.translate("enter_links_label", "Enter YouTube Links (one per line):"))
        layout.addWidget(self.input_label)

        # Input Edit
        self.input_edit = DroppableTextEdit()
        self.input_edit.setPlaceholderText("https://www.youtube.com/watch?v=...\nhttps://youtu.be/...")
        self.input_edit.textChanged.connect(self.check_queue_button_visibility)
        layout.addWidget(self.input_edit, 1)

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
        
        self.load_languages_menu()

    def load_languages_menu(self):
        # Clear all previous widgets (language buttons)
        while self.languages_menu_layout.count():
            item = self.languages_menu_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()

        self.language_buttons = {}
        # Clear stage widgets as well since we are reloading buttons
        for widget in self.stage_widgets.values():
            widget.setParent(None)
            widget.deleteLater()
        self.stage_widgets = {}

        languages_config = self.settings.get("languages_config", {})
        
        for lang_id, config in languages_config.items():
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
                # Reuse StageSelectionWidget but with rewrite stages
                rewrite_stages = [
                    'stage_download',
                    'stage_transcription', 
                    'stage_rewrite',
                    'stage_img_prompts',
                    'stage_images',
                    'stage_voiceover',
                    'stage_subtitles', 
                    'stage_montage'
                ]
                stage_widget = StageSelectionWidget(lang_name, lang_id, self, available_stages=rewrite_stages)
                
                # Connect the signal from the widget
                stage_widget.selection_changed.connect(self.check_queue_button_visibility)
                
                self.stages_container_layout.addWidget(stage_widget)
                self.stage_widgets[lang_id] = stage_widget
            
            # Animate appearance
            if not self.stage_widgets[lang_id].isVisible():
                Animator.slide_in_down(self.stage_widgets[lang_id])
            else:
                self.stage_widgets[lang_id].setVisible(True)
        else:
            if lang_id in self.stage_widgets:
                # Animate disappearance
                Animator.slide_out_up(self.stage_widgets[lang_id])
        
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

    def get_lang_config(self, lang_code):
        return self.settings.get("languages_config", {}).get(lang_code, {})

    def update_lang_config(self, lang_code, updates):
        languages = self.settings.get("languages_config", {})
        if lang_code not in languages:
            languages[lang_code] = {}
        
        languages[lang_code].update(updates)
        self.settings.set("languages_config", languages)

    def check_queue_button_visibility(self):
        text_present = bool(self.input_edit.toPlainText().strip())
        lang_selected = any(btn.isChecked() for btn in self.language_buttons.values())
        
        stages_selected = False
        if lang_selected:
            for lang_id, widget in self.stage_widgets.items():
                if widget.isVisible() and any(cb.isChecked() for cb in widget.checkboxes.values()):
                    stages_selected = True
                    break

        self.add_to_queue_button.setEnabled(text_present and lang_selected and stages_selected)

    def add_to_queue(self):
        text = self.input_edit.toPlainText().strip()
        if not text:
            return

        links = [line.strip() for line in text.split('\n') if line.strip()]
        if not links:
            return

        added_count = 0
        languages_config = self.settings.get('languages_config', {})
        
        for link in links:
            # Create a job for this link
            job_id = str(uuid.uuid4())
            job_name = link 
            if len(job_name) > 50:
                job_name = job_name[:47] + "..."
            
            job = {
                'id': job_id,
                'name': job_name,
                'type': 'rewrite',
                'created_at': datetime.datetime.now().isoformat(),
                'status': 'pending',
                'input_source': link,
                'languages': {}
            }

            for lang_id, btn in self.language_buttons.items():
                if btn.isChecked():
                    stage_widget = self.stage_widgets.get(lang_id)
                    if stage_widget and stage_widget.isVisible():
                        selected_stages = []
                        for stage_key, cb in stage_widget.checkboxes.items():
                            if cb.isChecked():
                                selected_stages.append(stage_key)
                        
                        if not selected_stages:
                            continue

                        # Build job-specific language settings
                        lang_config = languages_config.get(lang_id, {})
                        job_lang_settings = lang_config.copy()
                        job_lang_settings['stages'] = selected_stages
                        
                        # Handle template if selected (optional, per TextTab logic)
                        if stage_widget.selected_template:
                            job_lang_settings['template_name'] = stage_widget.selected_template

                        job['languages'][lang_id] = job_lang_settings
                        job['languages'][lang_id]['display_name'] = settings_manager.get_language_name(lang_id)

            if job['languages']:
                self.main_window.queue_manager.add_job(job)
                added_count += 1

        if added_count > 0:
            QMessageBox.information(self, translator.translate("success"), f"Added {added_count} tasks to queue.")
            self.input_edit.clear()
            self.check_queue_button_visibility()

    def retranslate_ui(self):
        self.input_label.setText(translator.translate("enter_links_label", "Enter YouTube Links (one per line):"))
        self.add_to_queue_button.setText(translator.translate("add_to_queue"))
        
        # Reload languages menu to update button texts
        self.load_languages_menu()
        
        # Reload stages translation
        for widget in self.stage_widgets.values():
            widget.retranslate_ui()
