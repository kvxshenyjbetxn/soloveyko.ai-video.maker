from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, 
    QScrollArea, QMessageBox, QGroupBox, QCheckBox, QGridLayout, QStyle, QInputDialog, QSplitter,
    QToolButton, QApplication
)
from PySide6.QtCore import Qt, QByteArray
from gui.text_tab import DroppableTextEdit, StageSelectionWidget
from utils.translator import translator
from utils.settings import settings_manager, template_manager
from utils.youtube_downloader import YouTubeDownloader
from utils.logger import logger, LogLevel
from utils.flow_layout import FlowLayout
from utils.animator import Animator
from functools import partial
import os
import re
import uuid
import datetime
import datetime
from PySide6.QtWidgets import QSplitter
from gui.widgets.quick_settings_panel import QuickSettingsPanel
from gui.widgets.recent_tasks_panel import RecentTasksPanel

class RewriteTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.stage_widgets = {} # lang_id -> StageSelectionWidget
        self.language_buttons = {} # lang_id -> QPushButton (toggle)
        self.settings = settings_manager
        self.init_ui()

    def init_ui(self):
        root_layout = QHBoxLayout(self)
        root_layout.setContentsMargins(0, 0, 0, 0)

        self.splitter = QSplitter(Qt.Orientation.Horizontal)
        root_layout.addWidget(self.splitter)
        
        # Recent Tasks Panel (Left)
        self.recent_tasks_panel = RecentTasksPanel(main_window=getattr(self, 'main_window', None))
        self.recent_tasks_panel.setMinimumWidth(250)
        self.recent_tasks_panel.task_selected.connect(self.restore_job)
        self.splitter.addWidget(self.recent_tasks_panel)

        # Main Content Container
        self.content_container = QWidget()
        self.content_container.setMinimumWidth(0)
        layout = QVBoxLayout(self.content_container)
        layout.setContentsMargins(10, 10, 10, 10)
        
        self.splitter.addWidget(self.content_container)

        # Input Header
        input_header_layout = QHBoxLayout()
        self.input_label = QLabel(translator.translate("enter_links_label", "Enter YouTube Links (one per line):"))
        self.link_count_label = QLabel("")
        input_header_layout.addWidget(self.input_label)
        input_header_layout.addStretch()
        input_header_layout.addWidget(self.link_count_label)
        layout.addLayout(input_header_layout)

        # Input Edit
        self.input_edit = DroppableTextEdit()
        self.input_edit.setMinimumWidth(0)
        self.input_edit.setPlaceholderText("https://www.youtube.com/watch?v=...\nhttps://youtu.be/...")
        self.input_edit.textChanged.connect(self.check_queue_button_visibility)
        self.input_edit.textChanged.connect(self.update_link_count)
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
        self.add_to_queue_button.clicked.connect(lambda: self.add_to_queue())
        self.add_to_queue_button.setFixedHeight(25)
        self.languages_menu_grid_layout.addWidget(self.add_to_queue_button, 0, 1)

        self.languages_menu_grid_layout.setColumnStretch(0, 1)
        self.languages_menu_grid_layout.setColumnStretch(1, 0)

        layout.addWidget(self.languages_menu_container)
        

        # Status bar
        self.status_bar_layout = QHBoxLayout()
        self.openrouter_balance_label = QLabel()
        self.openrouter_balance_label.setMinimumWidth(0)

        self.googler_usage_layout = QHBoxLayout()
        self.googler_usage_layout.setSpacing(2)
        self.googler_usage_label = QLabel()
        self.googler_usage_label.setMinimumWidth(0)
        self.googler_info_btn = QToolButton()
        self.googler_info_btn.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_MessageBoxInformation))
        self.googler_info_btn.setFixedSize(16, 16)
        self.googler_info_btn.setStyleSheet("QToolButton { border: none; background: transparent; }")
        self.googler_info_btn.setToolTip("Googler Detailed Stats")
        self.googler_usage_layout.addWidget(self.googler_usage_label)
        self.googler_usage_layout.addWidget(self.googler_info_btn)

        self.elevenlabs_balance_label = QLabel()
        self.elevenlabs_balance_label.setMinimumWidth(0)
        self.elevenlabs_unlim_balance_label = QLabel()
        self.elevenlabs_unlim_balance_label.setMinimumWidth(0)
        self.voicemaker_balance_label = QLabel()
        self.voicemaker_balance_label.setMinimumWidth(0)
        self.gemini_tts_balance_label = QLabel()
        self.gemini_tts_balance_label.setMinimumWidth(0)
        self.status_bar_layout.addWidget(self.openrouter_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addLayout(self.googler_usage_layout)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.elevenlabs_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.elevenlabs_unlim_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.voicemaker_balance_label)
        self.status_bar_layout.addSpacing(20)
        self.status_bar_layout.addWidget(self.gemini_tts_balance_label)
        self.status_bar_layout.addStretch()
        layout.addLayout(self.status_bar_layout)
        
        # Quick Settings Panel
        self.quick_settings_panel = QuickSettingsPanel(main_window=getattr(self, 'main_window', None))
        self.quick_settings_panel.setMinimumWidth(300)
        self.splitter.addWidget(self.quick_settings_panel)
        # Make side panels collapsible
        self.splitter.setCollapsible(0, True)
        self.splitter.setCollapsible(2, True)
        
        # Stretch factors
        self.splitter.setStretchFactor(0, 0) # Recent Tasks
        self.splitter.setStretchFactor(1, 1) # Main Content
        self.splitter.setStretchFactor(2, 0) # Quick Settings

        # Restore splitter state
        saved_state = self.settings.get("unified_splitter_state")
        if saved_state:
            self.splitter.restoreState(QByteArray.fromHex(saved_state.encode()))
        else:
            # Default: set balanced widths for side panels
            self.splitter.setSizes([280, 800, 280])
        
        # Connect signal to save state
        self.splitter.splitterMoved.connect(self.save_splitter_state)

        self.load_languages_menu()

    def save_splitter_state(self):
        state = self.splitter.saveState().toHex().data().decode()
        self.settings.set("unified_splitter_state", state)
        
        # Sync other tabs if they exist
        if self.main_window:
            if hasattr(self.main_window, 'text_tab') and self.main_window.text_tab:
                self.main_window.text_tab.sync_splitter(state)
            if hasattr(self.main_window, 'rewrite_tab') and self.main_window.rewrite_tab != self:
                self.main_window.rewrite_tab.sync_splitter(state)

    def sync_splitter(self, state_hex):
        """External sync without triggering double save"""
        self.splitter.blockSignals(True)
        self.splitter.restoreState(QByteArray.fromHex(state_hex.encode()))
        self.splitter.blockSignals(False)

    def restore_job(self, job):
        if not job: return
        
        # Check job type
        if job.get('type') != 'rewrite':
             return

        # Handle multiple links if saved as input_source or just text
        input_data = job.get('input_source', '')
        # If it was a list of links, it might be stored differently? 
        # In RewriteTab.add_to_queue, it processes links in a loop.
        # But RecentTasksPanel saves the job as it was added.
        
        self.input_edit.setText(input_data)
        
        langs_data = job.get('languages', {})
        for lang_id, btn in self.language_buttons.items():
            if lang_id in langs_data:
                btn.setChecked(True)
                stage_widget = self.stage_widgets.get(lang_id)
                if stage_widget:
                    config = langs_data[lang_id]
                    stage_widget.apply_config(config.get('stages', []), config.get('template_name'))
            else:
                btn.setChecked(False)

        # Automatically trigger add to queue
        self.add_to_queue(task_name_input=job.get('name'), is_restored=True)

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
                    'stage_preview',
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

    def add_to_queue(self, task_name_input=None, is_restored=False):
        links = self.input_edit.toPlainText().split('\n')
        links = [link.strip() for link in links if link.strip()]
        
        if not links:
            return

        # Check for AMD Whisper specific requirement
        whisper_type = self.settings.get("subtitles", {}).get("whisper_type", "amd")
        source_lang_code = None
        
        if whisper_type == 'amd' and not is_restored:
            dialog_label = translator.translate("enter_source_lang_label", "Enter Source Language Code (e.g. en, uk) for AMD Whisper:")
            dialog_title = translator.translate("enter_source_lang_title", "Source Language")
            
            lang, ok = QInputDialog.getText(self, dialog_title, dialog_label)
            if ok and lang.strip():
                source_lang_code = lang.strip()
            # If user cancels but AMD is required, we probably proceed without code and rely on default
            
        added_count = 0
        languages_config = self.settings.get('languages_config', {})
        
        initial_task_count = self.main_window.queue_manager.get_task_count()

        for i, link in enumerate(links):
            QApplication.processEvents() # maintain UI responsiveness
            
            # Determine job name automatically
            job_name = task_name_input 
            
            if not job_name or not job_name.strip():
                # Try to fetch title
                title = YouTubeDownloader.get_video_title(link)
                if title:
                    job_name = title
                else:
                    count = initial_task_count + 1 + i
                    job_name = f"{translator.translate('default_task_name', 'Task')} {count}"

            # --- Sanitize Job Name (Copied logic for consistency) ---
            # Remove ellipsis and invalid chars
            clean_job_name = job_name.replace('…', '').replace('...', '')
            safe_job_name = re.sub(r'[<>:"/\\|?*]', '', clean_job_name)
            safe_job_name = safe_job_name[:100].strip('. ')
            if not safe_job_name: 
                 safe_job_name = f"Task {initial_task_count + 1 + i}"
            
            # Use sanitized name as the actual displayed job name too, to avoid confusion
            job_name = safe_job_name

            # Create a job for this link
            job_id = str(uuid.uuid4())
            
            # --- Check for existing files ---
            found_files_per_lang = {}
            found_files_details = {}

            for lang_id, btn in self.language_buttons.items():
                if btn.isChecked():
                    stage_widget = self.stage_widgets.get(lang_id)
                    
                    # Determine the correct base_save_path
                    base_save_path = self.settings.get('results_path')
                    if stage_widget and stage_widget.selected_template:
                        template_data = template_manager.load_template(stage_widget.selected_template)
                        if template_data and template_data.get('results_path'):
                            base_save_path = template_data['results_path']
                            
                    if not base_save_path:
                        continue
                    
                    lang_name = btn.text()
                    
                    # Path logic same as TaskState
                    safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).strip('. ')
                    dir_path = os.path.abspath(os.path.join(base_save_path, safe_job_name, safe_lang_name))
                    
                    if os.path.isdir(dir_path):
                        found_files_for_lang = {}
                        details_for_lang = {}
                        
                        def add_found(stage_key, path, display_name=None):
                            if display_name is None:
                                display_name = translator.translate(stage_key)
                            found_files_for_lang[stage_key] = display_name
                            details_for_lang[stage_key] = path

                        # Check for files
                        download_path = os.path.join(dir_path, "downloaded_audio.mp3")
                        if os.path.isfile(download_path):
                            add_found("stage_download", download_path)

                        transcription_path = os.path.join(dir_path, "transcription.txt")
                        if os.path.isfile(transcription_path):
                            add_found("stage_transcription", transcription_path)

                        # Check for custom stages
                        custom_stages = self.settings.get("custom_stages", [])
                        for stage in custom_stages:
                            stage_name = stage.get("name")
                            if stage_name:
                                safe_name = "".join(c for c in stage_name if c.isalnum() or c in (' ', '_')).rstrip()
                                custom_path = os.path.join(dir_path, f"{safe_name}.txt")
                                if os.path.isfile(custom_path):
                                    add_found(f"custom_{stage_name}", custom_path, display_name=stage_name)

                        # For rewrite, the output is 'translation.txt' for compatibility
                        rewrite_path = os.path.join(dir_path, "translation.txt")
                        if os.path.isfile(rewrite_path):
                            add_found("stage_rewrite", rewrite_path)

                        preview_dir = os.path.join(dir_path, "preview")
                        if os.path.isdir(preview_dir):
                            # ImageGenerationWorker saves to preview/images
                            preview_images_dir = os.path.join(preview_dir, "images")
                            count = 0
                            
                            if os.path.isdir(preview_images_dir):
                                preview_files = os.listdir(preview_images_dir)
                                image_ext = ('.png', '.jpg', '.jpeg', '.webp')
                                preview_images = [f for f in preview_files if f.lower().endswith(image_ext)]
                                count = len(preview_images)
                            
                            # Also check for prompts
                            prompts_exist = os.path.isfile(os.path.join(preview_dir, "preview_prompts.txt"))
                            
                            if count > 0 or prompts_exist:
                                display_name = translator.translate('stage_preview')
                                if count > 0:
                                    display_name += f" ({count} {translator.translate('images_label')})"
                                add_found("stage_preview", preview_dir, display_name=display_name)

                        prompts_path = os.path.join(dir_path, "image_prompts.txt")
                        if os.path.isfile(prompts_path):
                            add_found("stage_img_prompts", prompts_path)

                        images_dir = os.path.join(dir_path, "images")
                        if os.path.isdir(images_dir):
                            files = os.listdir(images_dir)
                            if files:
                                image_ext = ('.png', '.jpg', '.jpeg')
                                video_ext = ('.mp4',)
                                image_count = len([f for f in files if f.lower().endswith(image_ext) and not f.lower().endswith('_thumb.jpg')])
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
                standard_order = ["stage_download", "stage_transcription", "stage_rewrite", "stage_preview", "stage_img_prompts", "stage_images", "stage_voiceover", "stage_subtitles"]
                
                # Get all unique custom stage keys found
                custom_keys = set()
                for found_stages in found_files_per_lang.values():
                    for key in found_stages.keys():
                        if key.startswith("custom_"):
                            custom_keys.add(key)
                
                # Insert custom keys after rewrite if it exists
                display_order = []
                for s in ["stage_download", "stage_transcription", "stage_rewrite"]:
                    if s in standard_order:
                        display_order.append(s)
                
                # Add custom stages
                display_order.extend(sorted(list(custom_keys)))
                
                # Add rest
                for s in standard_order:
                    if s not in display_order:
                        display_order.append(s)

                message = translator.translate("found_existing_files_prompt") + f" '{job_name}':<br><br>"
                for lang_name, found_stages in found_files_per_lang.items():
                    message += f"<b>{lang_name}:</b><ul>"
                    for stage_key in display_order:
                        if stage_key in found_stages:
                            message += f"<li>{found_stages[stage_key]}</li>"
                    message += "</ul>"
                message += translator.translate("use_existing_files_question")

                msg_box = QMessageBox(self)
                msg_box.setWindowTitle(translator.translate("found_existing_files_title"))
                msg_box.setTextFormat(Qt.TextFormat.RichText)
                msg_box.setText(message)
                msg_box.setStandardButtons(QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
                msg_box.setDefaultButton(QMessageBox.StandardButton.Yes)
                
                if msg_box.exec() == QMessageBox.StandardButton.Yes:
                    use_existing = True

            job = {
                'id': job_id,
                'name': job_name,
                'type': 'rewrite',
                'created_at': datetime.datetime.now().isoformat(),
                'status': 'pending',
                'input_source': link,
                'languages': {},
                'is_restored': is_restored
            }
            
            for lang_id, btn in self.language_buttons.items():
                if btn.isChecked():
                    stage_widget = self.stage_widgets.get(lang_id)
                    if stage_widget and stage_widget.isVisible():
                        selected_stages = stage_widget.get_selected_stages()
                        if not selected_stages:
                            continue

                        lang_config = languages_config.get(lang_id, {})
                        job_lang_settings = lang_config.copy()
                        job_lang_settings['stages'] = selected_stages
                        
                        # INJECT SOURCE LANGUAGE FOR AMD FORK
                        if source_lang_code:
                            job_lang_settings['source_language'] = source_lang_code

                        if use_existing and btn.text() in found_files_details:
                            job_lang_settings["pre_found_files"] = found_files_details[btn.text()]

                        if stage_widget.selected_template:
                            job_lang_settings['template_name'] = stage_widget.selected_template

                        job['languages'][lang_id] = job_lang_settings
                        job['languages'][lang_id]['display_name'] = lang_config.get('display_name', lang_id)

            if job['languages']:
                self.main_window.queue_manager.add_task(job)
                added_count += 1
        
        if added_count > 0:
            self.recent_tasks_panel.refresh()
            self.input_edit.clear()
            self.check_queue_button_visibility()
            
            QMessageBox.information(
                self, 
                translator.translate("success", "Success"), 
                translator.translate("tasks_added_success", "Task(s) successfully added to queue.") + f" ({added_count})"
            )

    def retranslate_ui(self):
        self.input_label.setText(translator.translate("enter_links_label", "Enter YouTube Links (one per line):"))
        self.add_to_queue_button.setText(translator.translate("add_to_queue"))
        
        # Reload languages menu to update button texts
        self.load_languages_menu()
        
        # Reload stages translation
        for widget in self.stage_widgets.values():
            widget.retranslate_ui()

        self.update_link_count() 
        
    def update_balance(self, balance_text):
        self.openrouter_balance_label.setText(balance_text)

    def update_googler_usage(self, usage_text):
        self.googler_usage_label.setText(usage_text)

    def update_googler_usage_detailed(self, usage_text, usage_data):
        self.update_googler_usage(usage_text)
        if not usage_data:
            return

        try:
            img_limits = usage_data.get("account_limits") or {}
            img_limit = img_limits.get("img_gen_per_hour_limit", 0)
            img_threads_limit = img_limits.get("img_generation_threads_allowed", 0)
            
            vid_limit = img_limits.get("video_gen_per_hour_limit", 0)
            vid_threads_limit = img_limits.get("video_generation_threads_allowed", 0)
            
            cur_usage = usage_data.get("current_usage") or {}
            hourly = cur_usage.get("hourly_usage") or {}
            img_stats = hourly.get("image_generation") or {}
            img_cur = img_stats.get("current_usage", 0)
            vid_stats = hourly.get("video_generation") or {}
            vid_cur = vid_stats.get("current_usage", 0)
            
            active = cur_usage.get("active_threads") or {}
            img_threads = active.get("image_threads", 0)
            vid_threads = active.get("video_threads", 0)
            
            tooltip = (
                f"<b>Googler Usage Stats:</b><br><br>"
                f"Images: {img_cur} / {img_limit} (Hourly)<br>"
                f"Videos: {vid_cur} / {vid_limit} (Hourly)<br>"
                f"Image Threads: {img_threads} / {img_threads_limit}<br>"
                f"Video Threads: {vid_threads} / {vid_threads_limit}"
            )
            self.googler_info_btn.setToolTip(tooltip)
        except:
            pass

    def update_elevenlabs_balance(self, balance_text):
        self.elevenlabs_balance_label.setText(balance_text)

    def update_elevenlabs_unlim_balance(self, balance_text):
        self.elevenlabs_unlim_balance_label.setText(balance_text)

    def update_voicemaker_balance(self, balance_text):
        self.voicemaker_balance_label.setText(balance_text)

    def update_gemini_tts_balance(self, balance_text):
        self.gemini_tts_balance_label.setText(balance_text)

    def update_link_count(self):
        text = self.input_edit.toPlainText().strip()
        if not text:
            count = 0
        else:
            count = len([line for line in text.split('\n') if line.strip()])
        self.link_count_label.setText(f"{translator.translate('links_count_label', 'Links count')}: {count}")
