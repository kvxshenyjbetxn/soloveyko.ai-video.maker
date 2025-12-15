import os
import re
import uuid
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QTextEdit, QHBoxLayout, QLabel,
    QPushButton, QFrame, QCheckBox, QToolButton, QInputDialog, QGridLayout, QMessageBox, QStyle
)
from PySide6.QtGui import QDragEnterEvent, QDropEvent, QColor
from PySide6.QtCore import Qt, QMimeData
from utils.flow_layout import FlowLayout
from functools import partial
from utils.translator import translator
from utils.settings import settings_manager
from gui.file_dialog import FileDialog
from utils.animator import Animator

def get_text_color_for_background(bg_color_hex):
    """Determines if black or white text is more readable on a given background color."""
    try:
        color = QColor(bg_color_hex)
        # Using the perceived luminance formula
        luminance = (0.299 * color.red() + 0.587 * color.green() + 0.114 * color.blue()) / 255
        return "#000000" if luminance > 0.5 else "#ffffff"
    except Exception:
        return "#000000"

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

from gui.file_dialog import FileDialog

class StageSelectionWidget(QWidget):
    """A compact widget representing the processing stages for a single language."""
    def __init__(self, language_name, parent_tab):
        super().__init__()
        self.parent_tab = parent_tab
        self.user_images = []
        self.user_audio = None
        self.user_background_music = None

        layout = QHBoxLayout(self)
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(6)

        self.lang_label = QLabel(f"{language_name}:")
        layout.addWidget(self.lang_label)

        self.toggle_all_button = QToolButton()
        self.toggle_all_button.setObjectName("toggleAllButton")
        self.toggle_all_button.setCheckable(True)
        self.toggle_all_button.clicked.connect(self.toggle_all)
        self.toggle_all_button.setFixedHeight(25)

        self.update_style()
        
        layout.addWidget(self.toggle_all_button)

        self.add_bg_music_button = QPushButton(translator.translate("add_background_music", "Add Music"))
        self.add_bg_music_button.setObjectName("addBgMusicButton")
        self.add_bg_music_button.setCheckable(True)
        self.add_bg_music_button.clicked.connect(self.open_background_music_dialog)
        self.add_bg_music_button.setFixedHeight(25)
        layout.addWidget(self.add_bg_music_button)

        line = QFrame()
        line.setFrameShape(QFrame.Shape.VLine)
        line.setFrameShadow(QFrame.Shadow.Sunken)
        layout.addWidget(line)
        
        self.checkboxes = {}
        self.add_buttons = {}
        stage_keys = ["stage_translation", "stage_img_prompts", "stage_images", 
                      "stage_voiceover", "stage_subtitles", "stage_montage"]
        
        for key in stage_keys:
            checkbox = QCheckBox(translator.translate(key))
            checkbox.setChecked(True)
            checkbox.stateChanged.connect(self.update_toggle_button_text)
            checkbox.stateChanged.connect(self.parent_tab.check_queue_button_visibility)
            layout.addWidget(checkbox)
            self.checkboxes[key] = checkbox

            # --- Custom Stages (Insert after Translation) ---
            if key == "stage_translation":
                custom_stages = self.parent_tab.settings.get("custom_stages", [])
                for stage in custom_stages:
                    stage_name = stage.get("name")
                    if stage_name:
                        key_custom = f"custom_{stage_name}"
                        checkbox_custom = QCheckBox(stage_name)
                        checkbox_custom.setChecked(True)
                        checkbox_custom.stateChanged.connect(self.update_toggle_button_text)
                        checkbox_custom.stateChanged.connect(self.parent_tab.check_queue_button_visibility)
                        layout.addWidget(checkbox_custom)
                        self.checkboxes[key_custom] = checkbox_custom

            if key in ["stage_images", "stage_voiceover"]:
                add_button = QToolButton()
                add_button.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_ToolBarHorizontalExtensionButton))
                add_button.setObjectName(f"addButton_{key}")
                add_button.setFixedSize(22, 22)
                add_button.setStyleSheet("QToolButton { border: none; background-color: transparent; }")
                layout.addWidget(add_button)
                self.add_buttons[key] = add_button

                if key == "stage_images":
                    add_button.clicked.connect(self.open_image_dialog)
                else:
                    add_button.clicked.connect(self.open_audio_dialog)

        layout.addStretch()
        self.update_toggle_button_text()

    def open_image_dialog(self):
        dialog = FileDialog(
            self,
            title=translator.translate("add_images_title"),
            description=translator.translate("add_images_desc"),
            extensions=[".png", ".jpg", ".jpeg"],
            multi_file=True
        )
        dialog.files_selected.connect(self.set_user_images)
        dialog.exec()

    def open_audio_dialog(self):
        dialog = FileDialog(
            self,
            title=translator.translate("add_audio_title"),
            description=translator.translate("add_audio_desc"),
            extensions=[".mp3", ".wav"],
            multi_file=False
        )
        dialog.files_selected.connect(self.set_user_audio)
        dialog.exec()

    def open_background_music_dialog(self, checked):
        if not checked:
            self.set_user_background_music([])
            return

        dialog = FileDialog(
            self,
            title=translator.translate("add_background_music_title", "Add Background Music"),
            description=translator.translate("add_background_music_desc", "Select an audio file to use as background music."),
            extensions=[".mp3", ".wav"],
            multi_file=False
        )
        dialog.finished.connect(lambda: self.add_bg_music_button.setChecked(self.user_background_music is not None))
        dialog.files_selected.connect(self.set_user_background_music)
        dialog.exec()

    def set_user_background_music(self, files):
        self.user_background_music = files[0] if files else None
        if self.user_background_music:
            self.add_bg_music_button.setText(os.path.basename(self.user_background_music))
            self.add_bg_music_button.setToolTip(self.user_background_music)
            self.add_bg_music_button.setChecked(True)
        else:
            self.add_bg_music_button.setText(translator.translate("add_background_music", "Add Music"))
            self.add_bg_music_button.setToolTip("")
            self.add_bg_music_button.setChecked(False)

    def set_user_images(self, files):
        self.user_images = sorted(files)
        button = self.add_buttons.get("stage_images")
        if button:
            if self.user_images:
                button.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_DialogApplyButton))
                self.checkboxes["stage_img_prompts"].setChecked(False)
                self.checkboxes["stage_img_prompts"].setEnabled(False)
            else:
                button.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_ToolBarHorizontalExtensionButton))
                self.checkboxes["stage_img_prompts"].setEnabled(True)

    def set_user_audio(self, files):
        self.user_audio = files[0] if files else None
        button = self.add_buttons.get("stage_voiceover")
        translation_checkbox = self.checkboxes.get("stage_translation")

        if button:
            if self.user_audio:
                button.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_DialogApplyButton))
                if translation_checkbox:
                    translation_checkbox.setChecked(False)
                    translation_checkbox.setEnabled(False)
            else:
                button.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_ToolBarHorizontalExtensionButton))
                if translation_checkbox:
                    translation_checkbox.setEnabled(True)
        
    def get_user_files(self):
        files = {}
        if self.user_images:
            files["stage_images"] = self.user_images
        if self.user_audio:
            files["stage_voiceover"] = self.user_audio
        if self.user_background_music:
            files["background_music"] = self.user_background_music
        return files
        
    def retranslate_ui(self):
        for key, checkbox in self.checkboxes.items():
            if not key.startswith("custom_"):
                checkbox.setText(translator.translate(key))
        self.update_toggle_button_text()
        self.add_bg_music_button.setText(translator.translate("add_background_music", "Add Music"))
        self.add_bg_music_button.setToolTip("")
        if self.user_background_music:
             self.add_bg_music_button.setText(os.path.basename(self.user_background_music))
             self.add_bg_music_button.setToolTip(self.user_background_music)

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

    def update_style(self):
        primary_color = self.parent_tab.settings.get('accent_color', '#3f51b5')
        text_color_on_primary = get_text_color_for_background(primary_color)

        self.toggle_all_button.setStyleSheet(f"""
            QToolButton#toggleAllButton {{
                color: {primary_color};
                border: 1px solid {primary_color};
                border-radius: 8px;
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
        
        self.char_count_layout.addSpacing(20)
        self.template_name_label = QLabel()
        self.char_count_layout.addWidget(self.template_name_label)
        
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
            
            base_save_path = self.settings.get('results_path')
            
            found_files_per_lang = {}
            found_files_details = {}

            # --- Check for existing files ---
            if base_save_path:
                safe_job_name = "".join(c for c in task_name if c.isalnum() or c in (' ', '_')).rstrip()
                for lang_id, btn in self.language_buttons.items():
                    if btn.isChecked():
                        lang_name = btn.text()
                        safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
                        dir_path = os.path.join(base_save_path, safe_job_name, safe_lang_name)
                        
                        if os.path.isdir(dir_path):
                            found_files_for_lang = {}
                            details_for_lang = {}
                            
                            def add_found(stage_key, path, display_name=None):
                                if display_name is None:
                                    display_name = translator.translate(stage_key)
                                found_files_for_lang[stage_key] = display_name
                                details_for_lang[stage_key] = path

                            # Check for files
                            translation_path = os.path.join(dir_path, "translation.txt")
                            if os.path.isfile(translation_path):
                                add_found("stage_translation", translation_path)

                            prompts_path = os.path.join(dir_path, "image_prompts.txt")
                            if os.path.isfile(prompts_path):
                                add_found("stage_img_prompts", prompts_path)

                            images_dir = os.path.join(dir_path, "images")
                            if os.path.isdir(images_dir):
                                files = os.listdir(images_dir)
                                if files:
                                    image_ext = ('.png', '.jpg', '.jpeg')
                                    video_ext = ('.mp4',)
                                    image_count = len([f for f in files if f.lower().endswith(image_ext)])
                                    video_count = len([f for f in files if f.lower().endswith(video_ext)])
                                    
                                    display_name = translator.translate("stage_images")
                                    counts = []
                                    if image_count > 0:
                                        counts.append(f"{image_count} {translator.translate('images_label')}")
                                    if video_count > 0:
                                        counts.append(f"{video_count} {translator.translate('videos_label')}")
                                    
                                    if counts:
                                        display_name += f" ({', '.join(counts)})"

                                    add_found("stage_images", images_dir, display_name=display_name)

                            voice_mp3_path = os.path.join(dir_path, "voice.mp3")
                            voice_wav_path = os.path.join(dir_path, "voice.wav")
                            if os.path.isfile(voice_mp3_path):
                                add_found("stage_voiceover", voice_mp3_path)
                            elif os.path.isfile(voice_wav_path):
                                add_found("stage_voiceover", voice_wav_path)

                            subtitles_path = os.path.join(dir_path, "voice.ass")
                            if os.path.isfile(subtitles_path):
                                add_found("stage_subtitles", subtitles_path)
                            
                            if found_files_for_lang:
                                found_files_per_lang[lang_name] = found_files_for_lang
                                found_files_details[lang_name] = details_for_lang

            use_existing = False
            if found_files_per_lang:
                display_order = ["stage_translation", "stage_img_prompts", "stage_images", "stage_voiceover", "stage_subtitles"]
                
                message = translator.translate("found_existing_files_prompt") + f" '{task_name}':\n\n"
                for lang_name, found_stages in found_files_per_lang.items():
                    message += f"<b>{lang_name}:</b>\n"
                    # Iterate in the specified order
                    for stage_key in display_order:
                        if stage_key in found_stages:
                            message += f"- {found_stages[stage_key]}\n"
                    message += "\n"
                message += translator.translate("use_existing_files_question")

                msg_box = QMessageBox(self)
                msg_box.setWindowTitle(translator.translate("found_existing_files_title"))
                msg_box.setTextFormat(Qt.TextFormat.RichText)
                msg_box.setText(message)
                msg_box.setStandardButtons(QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
                msg_box.setDefaultButton(QMessageBox.StandardButton.Yes)
                
                reply = msg_box.exec()
                if reply == QMessageBox.StandardButton.Yes:
                    use_existing = True
            
            # --- Prepare and add the job ---
            languages_data = {}
            for lang_id, btn in self.language_buttons.items():
                if btn.isChecked():
                    stage_widget = self.stage_widgets.get(lang_id)
                    if stage_widget and stage_widget.isVisible():
                        selected_stages = stage_widget.get_selected_stages()
                        
                        if selected_stages:
                            lang_name = btn.text()
                            lang_data = {
                                "display_name": lang_name,
                                "stages": selected_stages
                            }
                            
                            if use_existing and lang_name in found_files_details:
                                lang_data["pre_found_files"] = found_files_details[lang_name]
                            
                            user_files = stage_widget.get_user_files()
                            if user_files:
                                lang_data["user_provided_files"] = user_files

                            languages_data[lang_id] = lang_data
            
            if languages_data:
                job = {
                    "id": None,
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
        self.load_languages_menu()
        self.add_to_queue_button.setText(translator.translate('add_to_queue'))
        self.update_styles()
        self.apply_text_color_to_text_edit()

    def update_styles(self):
        for widget in self.stage_widgets.values():
            if widget.isVisible():
                widget.update_style()
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

    def update_template_name(self, name):
        if name:
            self.template_name_label.setText(f"{translator.translate('current_template_label', 'Template')}: {name}")
        else:
            self.template_name_label.setText("")

    def apply_text_color_to_text_edit(self):
        current_theme = self.settings.get('theme', 'dark')
        if current_theme == 'light':
            text_color = 'black'
        else:
            text_color = 'white'
        self.text_edit.setStyleSheet(f"QTextEdit#textEdit {{ color: {text_color}; }}")


