from PySide6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QHBoxLayout, QLabel, QScrollArea, QGroupBox, QMenu, QToolButton, QMessageBox, QTextBrowser, QSizePolicy, QGridLayout, QCheckBox, QComboBox
from PySide6.QtCore import Qt, Signal, QThread
from PySide6.QtGui import QAction, QCursor
from functools import partial
from utils.translator import translator
from utils.flow_layout import FlowLayout
from utils.logger import logger, LogLevel
import os
import sys
import subprocess
import platform

try:
    import psutil
except ImportError:
    psutil = None
from utils.settings import settings_manager

class StatusDot(QLabel):
    """A circular widget to indicate status."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedSize(12, 12)
        self.current_status = 'pending'
        self.set_status('pending')

    def set_status(self, status):
        self.current_status = status
        if status == 'processing':
            color = '#FFD700' # Bright Yellow
        elif status == 'warning':
            color = '#FF8C00' # Rich Orange
        elif status == 'success':
            color = '#28a745' # Green
        elif status == 'error':
            color = '#dc3545' # Red
        else: # 'pending'
            color = '#6c757d' # Grey
        
        self.setStyleSheet(f"background-color: {color}; border-radius: 6px;")

class DeletableStageWidget(QWidget):
    """A widget for a single stage, allowing deletion via context menu."""
    delete_requested = Signal()

    def __init__(self, stage_key, parent=None):
        super().__init__(parent)
        self.setMinimumHeight(24)
        self.stage_key = stage_key
        
        if stage_key.startswith("custom_"):
            # Strip prefix for display custom stages
            self.stage_name = stage_key.replace("custom_", "", 1)
        else:
            self.stage_name = translator.translate(stage_key)
        
        layout = QHBoxLayout()
        layout.setContentsMargins(4, 0, 4, 0)
        self.setLayout(layout)

        self.dot = StatusDot(self)
        self.label = QLabel(self.stage_name, self)
        
        # Metadata label (right-aligned, grey text)
        self.metadata_label = QLabel("", self)
        self.metadata_label.setStyleSheet("color: #888; font-size: 10px;")
        
        layout.addWidget(self.dot)
        layout.addWidget(self.label)
        layout.addStretch()
        layout.addWidget(self.metadata_label)

    def contextMenuEvent(self, event):
        menu = QMenu(self)
        delete_action = QAction(translator.translate("delete_stage_formatted").format(self.stage_name), self)
        delete_action.triggered.connect(self.delete_requested.emit)
        menu.addAction(delete_action)
        menu.exec(event.globalPos())

    def get_dot(self):
        return self.dot
    
    def get_status(self):
        return self.dot.current_status

    def update_metadata(self, metadata_text):
        """Update the metadata display text"""
        self.metadata_label.setText(metadata_text)


class DeletableLanguageHeader(QWidget):
    """A widget for the language header, allowing deletion via context menu and collapsing via click."""
    delete_requested = Signal()
    toggled = Signal(bool)  # is_expanded

    def __init__(self, display_name, template_name=None, parent=None):
        super().__init__(parent)
        self.display_name = display_name
        self.is_expanded = True
        self.setCursor(QCursor(Qt.CursorShape.PointingHandCursor))
        
        # Ensure header has consistent height for alignment in grid
        self.setFixedHeight(30)

        layout = QHBoxLayout()
        layout.setContentsMargins(0, 5, 0, 5)
        layout.setSpacing(5)
        self.setLayout(layout)

        # Chevron indicator
        self.chevron = QLabel("▼", self)
        self.chevron.setStyleSheet("font-size: 10px; color: #888;")
        layout.addWidget(self.chevron)

        self.label = QLabel(f"<b>{self.display_name}</b>", self)
        layout.addWidget(self.label)
        
        # Status Dot
        self.status_dot = StatusDot(self)
        self.status_dot.setVisible(False)
        layout.addWidget(self.status_dot)

        if template_name:
            template_label = QLabel(f"<i>({template_name})</i>")
            template_label.setStyleSheet("color: #999; font-size: 11px;")
            layout.addWidget(template_label)

        layout.addStretch()

    def contextMenuEvent(self, event):
        menu = QMenu(self)
        delete_action = QAction(translator.translate("delete_language_formatted").format(self.display_name), self)
        delete_action.triggered.connect(self.delete_requested.emit)
        menu.addAction(delete_action)
        menu.exec(event.globalPos())

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.toggle()
            event.accept() # Prevent bubbling to TaskCard
        else:
            super().mousePressEvent(event)

    def toggle(self):
        self.is_expanded = not self.is_expanded
        self.chevron.setText("▼" if self.is_expanded else "▶")
        self.status_dot.setVisible(not self.is_expanded)
        self.toggled.emit(self.is_expanded)

    def set_expanded(self, expanded):
        if self.is_expanded != expanded:
            self.toggle()
    
    def set_status(self, status):
        self.status_dot.set_status(status)

class LanguageSection(QWidget):
    """A widget grouping a language header and its stages."""
    def __init__(self, lang_id, lang_data, job_type, parent_card):
        super().__init__(parent_card)
        self.lang_id = lang_id
        self.stage_map = {}
        self.original_text_metadata_label = None
        self.original_text_status = 'pending'
        
        # Main layout for this section
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 10) # Add some bottom spacing
        layout.setSpacing(0)

        # Header
        template_name = lang_data.get("template_name")
        self.header = DeletableLanguageHeader(lang_data['display_name'], template_name, self)
        self.header.delete_requested.connect(lambda: parent_card.on_language_delete(lang_id))
        self.header.toggled.connect(self.on_header_toggled)
        layout.addWidget(self.header)

        # Content container (for collapsing)
        self.content_widget = QWidget(self)
        self.content_layout = QVBoxLayout(self.content_widget)
        self.content_layout.setContentsMargins(15, 0, 0, 0) # Indent content
        self.content_layout.setSpacing(2)
        layout.addWidget(self.content_widget)

        # Build content
        # If translation is not a selected stage, show 'Original'
        if 'stage_translation' not in lang_data['stages'] and job_type != 'rewrite':
            original_widget = QWidget()
            original_layout = QHBoxLayout(original_widget)
            original_layout.setContentsMargins(4, 0, 4, 0)
            
            dot = StatusDot(self)
            dot.set_status('success') # Original text is always 'successful'
            self.original_text_status = 'success'
            label = QLabel(translator.translate('original_text'), self)
            
            # Metadata label for original text
            metadata_label = QLabel("", self)
            metadata_label.setStyleSheet("color: #888; font-size: 10px;")
            self.original_text_metadata_label = metadata_label
            
            original_layout.addWidget(dot)
            original_layout.addWidget(label)
            original_layout.addStretch()
            original_layout.addWidget(metadata_label)
            self.content_layout.addWidget(original_widget)

        pre_found = lang_data.get("pre_found_files", {})
        user_provided = lang_data.get("user_provided_files", {})
        
        for stage_key in lang_data['stages']:
            stage_widget = DeletableStageWidget(stage_key, self)
            stage_widget.delete_requested.connect(
                lambda skey=stage_key: parent_card.on_stage_delete(lang_id, skey)
            )
            
            # Check if this stage was already found in the destination folder or provided by user
            if stage_key in pre_found or stage_key in user_provided:
                stage_widget.get_dot().set_status('success')
            
            # Special case: if user provided images, image prompts are also technically skipped
            if stage_key == 'stage_img_prompts' and 'stage_images' in user_provided:
                stage_widget.get_dot().set_status('success')

            self.stage_map[stage_key] = stage_widget
            self.content_layout.addWidget(stage_widget)
        
        self.update_header_status()

    @property
    def stage_widgets(self):
        return list(self.stage_map.values())

    def set_expanded(self, expanded):
        self.header.set_expanded(expanded)

    def on_header_toggled(self, is_expanded):
        self.content_widget.setVisible(is_expanded)

    def update_stage_status(self, stage_key, status):
        if stage_key in self.stage_map:
            self.stage_map[stage_key].get_dot().set_status(status)
        self.update_header_status()

    def remove_stage(self, stage_key):
        if stage_key in self.stage_map:
            widget = self.stage_map.pop(stage_key)
            widget.setVisible(False)
            self.content_layout.removeWidget(widget)
            widget.deleteLater()
            self.update_header_status()

    def update_header_status(self):
        statuses = [w.get_status() for w in self.stage_map.values()]
        if self.original_text_status != 'pending':
             statuses.append(self.original_text_status)

        if 'error' in statuses:
            final_status = 'error'
        elif 'processing' in statuses:
            final_status = 'processing'
        # Treat 'warning' (e.g. partial images) as success for the group status, 
        # or at least ensure it doesn't fall back to pending. 
        # User requested green even if 49/50 images (which is usually warning/orange).
        elif all(s in ['success', 'warning'] for s in statuses) and statuses:
            final_status = 'success'
        else:
            final_status = 'pending'
        
        self.header.set_status(final_status)


class TaskCard(QGroupBox):
    """A widget that displays a single task and its stage statuses."""
    task_delete_requested = Signal(str) # job_id
    language_delete_requested = Signal(str, str) # job_id, lang_id
    stage_delete_requested = Signal(str, str, str) # job_id, lang_id, stage_key
    retry_requested = Signal(str) # job_id

    def __init__(self, job, log_tab=None, parent=None):
        super().__init__("", parent)
        self.job_id = job['id']
        self.job_name = job['name']
        self.job = job
        self.log_tab = log_tab
        self.is_expanded = False
        self.setStyleSheet("""
            QGroupBox { 
                padding-top: 8px; 
                padding-bottom: 8px; 
            }
            QGroupBox:hover { 
                background-color: rgba(255, 255, 255, 0.05); 
            }
        """)
        self.setCursor(QCursor(Qt.CursorShape.PointingHandCursor))

        # Store references for updates
        self.language_sections = {} # lang_id -> LanguageSection
        self.stage_widgets = {} # lang_id -> list of DeletableStageWidget
        self.original_text_metadata_labels = {}  # lang_id -> QLabel

        self.init_ui(job)

    def init_ui(self, job):
        # Main horizontal layout for left content and right logs
        main_layout = QHBoxLayout(self)
        
        # Left content widget
        left_content = QWidget()
        layout = QVBoxLayout(left_content)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Task Type Badge (at the top)
        type_badge_layout = QHBoxLayout()
        job_type = self.job.get('type', 'text')
        type_label = QLabel(translator.translate(f"type_{job_type}", job_type.capitalize()), self)
        type_label.setStyleSheet(f"""
            background-color: {'#4CAF50' if job_type == 'text' else '#2196F3'};
            color: white;
            border-radius: 4px;
            padding: 2px 6px;
            font-size: 10px;
            font-weight: bold;
        """)
        type_label.setFixedSize(type_label.sizeHint())
        type_badge_layout.addWidget(type_label)
        type_badge_layout.addStretch()
        layout.addLayout(type_badge_layout)

        # Hint label
        hint_label = QLabel(translator.translate("click_for_logs_hint"), self)
        hint_label.setStyleSheet("color: #777; font-size: 10px; margin-bottom: 2px;")
        layout.addWidget(hint_label)

        # Custom header
        header_layout = QHBoxLayout()
        # Візуальне обмеження: 100 символів для тексту мітки
        display_name = self.job_name[:100] + ("..." if len(self.job_name) > 100 else "")
        task_name_label = QLabel(f"<b>{display_name}</b>", self)
        task_name_label.setToolTip(self.job_name)
        task_name_label.setWordWrap(True)
        task_name_label.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        
        # Візуальне обмеження: рівно 3 рядки. 
        # Використовуємо setFixedHeight, щоб наступні рядки не "виглядали" знизу.
        fm = task_name_label.fontMetrics()
        line_height = fm.lineSpacing()
        task_name_label.setFixedHeight(line_height * 3) 
        task_name_label.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        
        self.task_name_label = task_name_label
        header_layout.addWidget(task_name_label)
        header_layout.addStretch()

        delete_button = QToolButton(self)
        delete_button.setText("X")
        delete_button.setFixedSize(20, 20)
        delete_button.setStyleSheet("""
            QToolButton { background-color: #555; color: white; border: none; font-weight: bold; border-radius: 10px; }
            QToolButton:hover { background-color: #dc3545; }
        """)
        delete_button.clicked.connect(self.on_corner_delete_clicked)
        header_layout.addWidget(delete_button)
        layout.addLayout(header_layout)
        
        # Separator line to distinguish title from content
        line = QWidget()
        line.setFixedHeight(1)
        line.setStyleSheet("background-color: #333;")
        layout.addWidget(line)
        layout.addSpacing(5)

        # Languages Container
        languages_container = QWidget()
        num_languages = len(job['languages'])
        
        # Logic: 1-2 languages = Vertical; 3+ languages = 2-Column Grid
        if num_languages >= 3:
            langs_layout = QGridLayout(languages_container)
            langs_layout.setContentsMargins(0, 0, 0, 0)
            langs_layout.setColumnStretch(0, 1)
            langs_layout.setColumnStretch(1, 1)
            langs_layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        else:
            langs_layout = QVBoxLayout(languages_container)
            langs_layout.setContentsMargins(0, 0, 0, 0)

        idx = 0
        for lang_id, lang_data in job['languages'].items():
            lang_section = LanguageSection(lang_id, lang_data, job.get('type'), self)
            self.language_sections[lang_id] = lang_section
            
            # Map exposed widgets for existing logic
            self.stage_widgets[lang_id] = lang_section.stage_widgets
            if lang_section.original_text_metadata_label:
                self.original_text_metadata_labels[lang_id] = lang_section.original_text_metadata_label
            
            if num_languages >= 3:
                row = idx // 2
                col = idx % 2
                langs_layout.addWidget(lang_section, row, col)
            else:
                langs_layout.addWidget(lang_section)
            
            idx += 1

        layout.addWidget(languages_container)
        
        # Retry Button (Small, dynamic)
        self.retry_button = QPushButton(translator.translate("retry_task"))
        self.retry_button.setVisible(False)
        self.retry_button.setStyleSheet("""
            QPushButton {
                background-color: #f39c12;
                color: white;
                font-weight: bold;
                border-radius: 4px;
                font-size: 11px;
                padding: 4px 12px;
                border: none;
            }
            QPushButton:hover {
                background-color: #e67e22;
            }
        """)
        self.retry_button.clicked.connect(lambda: self.retry_requested.emit(self.job_id))
        
        retry_layout = QHBoxLayout()
        retry_layout.addStretch()
        retry_layout.addWidget(self.retry_button)
        layout.addLayout(retry_layout)
        
        layout.addStretch()
        
        # Add left content to main layout
        main_layout.addWidget(left_content)
        
        # Right logs widget
        self.log_browser = QTextBrowser()
        self.log_browser.setReadOnly(True)
        self.log_browser.setVisible(False)
        self.log_browser.setMinimumWidth(600)
        self.log_browser.setMaximumWidth(800)
        self.log_browser.setStyleSheet("QTextBrowser { font-family: 'Courier New', monospace; }")
        main_layout.addWidget(self.log_browser)
    
    def set_all_languages_expanded(self, expanded):
        for section in self.language_sections.values():
            section.set_expanded(expanded)

    def hide_language(self, lang_id):
        if lang_id in self.language_sections:
            self.language_sections[lang_id].setVisible(False)

    def hide_stage(self, lang_id, stage_key):
        if lang_id in self.language_sections:
             section = self.language_sections[lang_id]
             
             if stage_key.startswith("custom_"):
                 stage_name_to_find = stage_key.replace("custom_", "", 1)
                 # We need to find matching key in map if map uses key, not name.
                 # LanguageSection init: self.stage_map[stage_key] = widget
                 # So we should use stage_key directly!
                 if stage_key in section.stage_map:
                     section.remove_stage(stage_key)
                     return
             
             # Fallback if we only have display name logic (old code)
             # But LanguageSection uses stage_key.
             if stage_key in section.stage_map:
                 section.remove_stage(stage_key)
             else:
                 # Try finding via name if needed? 
                 # Code usually passes stage_key.
                 pass
    
    def contextMenuEvent(self, event):
        menu = QMenu(self)
        delete_action = QAction(translator.translate("delete_task"), self)
        delete_action.triggered.connect(self.on_corner_delete_clicked)
        menu.addAction(delete_action)
        menu.exec(event.globalPos())

    def on_corner_delete_clicked(self):
        msg_box = QMessageBox(self)
        msg_box.setWindowTitle(translator.translate("confirm_deletion_title"))
        msg_box.setText(translator.translate("confirm_task_deletion_message").format(self.job_name))
        msg_box.setStandardButtons(QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
        msg_box.setDefaultButton(QMessageBox.StandardButton.No)
        
        if msg_box.exec() == QMessageBox.StandardButton.Yes:
            self.task_delete_requested.emit(self.job_id)

    def on_language_delete(self, lang_id):
        self.language_delete_requested.emit(self.job_id, lang_id)

    def on_stage_delete(self, lang_id, stage_key):
        self.stage_delete_requested.emit(self.job_id, lang_id, stage_key)

    def update_stage_status(self, lang_id, stage_key, status):
        if lang_id in self.language_sections:
            self.language_sections[lang_id].update_stage_status(stage_key, status)
        self._update_retry_visibility()

    def update_name(self, new_name):
        self.job_name = new_name
        self.job['name'] = new_name
        display_name = new_name[:100] + ("..." if len(new_name) > 100 else "")
        self.task_name_label.setText(f"<b>{display_name}</b>")
        self.task_name_label.setToolTip(new_name)

    def _update_retry_visibility(self):
        """Shows retry button if any stage in any language is in 'error' state."""
        has_error = False
        for section in self.language_sections.values():
            statuses = [w.get_status() for w in section.stage_map.values()]
            if 'error' in statuses:
                has_error = True
                break
        
        self.retry_button.setVisible(has_error)
    
    def update_stage_metadata(self, lang_id, stage_key, metadata_text):
        """Update metadata display for a specific stage"""
        # Handle original text metadata
        if stage_key == 'original_text':
            if lang_id in self.original_text_metadata_labels:
                self.original_text_metadata_labels[lang_id].setText(metadata_text)
            return
        
        # Handle regular stage metadata
        if lang_id in self.language_sections:
             section = self.language_sections[lang_id]
             if stage_key in section.stage_map:
                 section.stage_map[stage_key].update_metadata(metadata_text)



    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.toggle_logs()
        super().mousePressEvent(event)
    
    def toggle_logs(self):
        self.is_expanded = not self.is_expanded
        self.log_browser.setVisible(self.is_expanded)
        if self.is_expanded:
            self.load_logs()
            # Subscribe to new logs
            if self.log_tab:
                self.log_tab.log_added.connect(self.on_new_log)
        else:
            # Unsubscribe when collapsed
            if self.log_tab:
                try:
                    self.log_tab.log_added.disconnect(self.on_new_log)
                except:
                    pass  # Already disconnected
    
    def load_logs(self):
        if not self.log_tab:
            return
        
        self.log_browser.clear()
        # Filter logs by job_id (task_id contains job_id)
        # We use specific patterns to avoid matching IDs that are substrings of others (e.g. Task-1 vs Task-10)
        # Generally logs from TaskProcessor/Workers look like: [Task-1_en] or [Task-1]
        pattern1 = f"[{self.job_id}_"
        pattern2 = f"[{self.job_id}]"
        
        for log_data in self.log_tab.all_logs:
            if pattern1 in log_data["message"] or pattern2 in log_data["message"]:
                self._append_log_message(log_data)
    
    def on_new_log(self, log_data):
        """Called when a new log is added to log_tab"""
        if not self.is_expanded:
            return
        
        # Filter by job_id
        pattern1 = f"[{self.job_id}_"
        pattern2 = f"[{self.job_id}]"
        if pattern1 in log_data["message"] or pattern2 in log_data["message"]:
            self._append_log_message(log_data)
            # Auto-scroll to bottom
            self.log_browser.verticalScrollBar().setValue(
                self.log_browser.verticalScrollBar().maximum()
            )
    
    def on_progress_log(self, message):
        """Called for card-only logs (FFmpeg progress, Voicemaker progress)"""
        # Voicemaker logs get special treatment (timestamp, icons, etc.)
        if "Voicemaker" in message:
            import datetime
            timestamp = datetime.datetime.now().strftime("%H:%M:%S")
            
            # Determine level based on message content
            if "success" in message.lower() or "completed" in message.lower():
                level = LogLevel.SUCCESS
            elif "error" in message.lower() or "failed" in message.lower():
                level = LogLevel.ERROR
            else:
                level = LogLevel.INFO

            color = level.to_color()
            icon = level.to_icon()
            
            formatted_message = (
                f'<font color="{color}">'
                f'<b>[{timestamp}]</b> '
                f'{icon} '
                f'<b>{level.name: <7}</b> - '
                f'{message}'
                f'</font>'
            )
            self.log_browser.append(formatted_message)
        
        else:
            # Legacy/FFmpeg logs: Cyan, no timestamp
            formatted_message = f'<font color="#00ffff">{message}</font>'
            self.log_browser.append(formatted_message)

        # Auto-scroll to bottom
        self.log_browser.verticalScrollBar().setValue(
            self.log_browser.verticalScrollBar().maximum()
        )
    
    def _append_log_message(self, log_data):
        """Format and append a log message to the browser"""
        level = log_data["level"]
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
        
        self.log_browser.append(formatted_message)
    
    def __del__(self):
        """Clean up signal connections when card is deleted"""
        if self.log_tab and self.is_expanded:
            try:
                self.log_tab.log_added.disconnect(self.on_new_log)
            except:
                pass

class HardwareMonitorThread(QThread):
    updated = Signal(dict)

    def __init__(self, disk_path):
        super().__init__()
        self.disk_path = disk_path
        self._run_flag = True

    def run(self):
        while self._run_flag:
            stats = {
                'cpu': self._get_cpu_usage(),
                'ram': self._get_ram_usage(),
                'gpu': self._get_gpu_usage(),
                'disks': self._get_all_disks_free(),
            }
            self.updated.emit(stats)
            self.msleep(3000) # Update every 3 seconds

    def stop(self):
        self._run_flag = False

    def _get_cpu_usage(self):
        if psutil:
            return psutil.cpu_percent()
        return 0

    def _get_ram_usage(self):
        if psutil:
            return psutil.virtual_memory().percent
        return 0

    def _get_gpu_usage(self):
        try:
            if platform.system() == "Windows":
                # Метод 1: NVIDIA SMI (найточніша для NVIDIA)
                try:
                    res = subprocess.run(
                        ['nvidia-smi', '--query-gpu=utilization.gpu', '--format=csv,noheader,nounits'],
                        capture_output=True, text=True, check=True, timeout=1,
                        creationflags=subprocess.CREATE_NO_WINDOW if hasattr(subprocess, 'CREATE_NO_WINDOW') else 0
                    )
                    nvidia_val = float(res.stdout.strip())
                    if nvidia_val > 0: return nvidia_val
                except:
                    pass

                # Метод 2: Універсальний через WMI/CIM (для AMD/Intel/NVIDIA)
                # Збираємо навантаження з усіх двигунів (3D, Copy, Video тощо) і беремо максимум
                try:
                    # Отримуємо просто всі значення UtilizationPercentage
                    ps_cmd = 'powershell -Command "(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceAnalyzer_GPUEngine -ErrorAction SilentlyContinue).UtilizationPercentage"'
                    res = subprocess.run(
                        ps_cmd,
                        capture_output=True, text=True, timeout=4,
                        creationflags=subprocess.CREATE_NO_WINDOW if hasattr(subprocess, 'CREATE_NO_WINDOW') else 0
                    )
                    # Парсимо вивід (це буде список чисел)
                    lines = res.stdout.strip().splitlines()
                    nums = []
                    for line in lines:
                        try:
                            # Замінюємо розділювачі для різних локалей
                            n = float(line.strip().replace(',', '.'))
                            nums.append(n)
                        except: continue
                    
                    if nums:
                        total_max = max(nums)
                        if total_max > 0:
                            return round(total_max, 1)
                except:
                    pass
            elif platform.system() == "Darwin":
                return 0
            return 0
        except:
            return 0

    def _get_all_disks_free(self):
        drives = {}
        try:
            if psutil:
                partitions = psutil.disk_partitions(all=False)
                for part in partitions:
                    try:
                        # Skip virtual/special drives
                        if 'cdrom' in part.opts or part.fstype == '':
                            continue
                        
                        usage = psutil.disk_usage(part.mountpoint)
                        free_gb = round(usage.free / (1024**3), 1)
                        
                        # Format label (C: for Windows, Root/Mount for others)
                        label = part.mountpoint
                        if platform.system() == "Windows":
                            label = label.split('\\')[0] # Get C:
                        else:
                            if label == '/': label = '/'
                            else: label = os.path.basename(label) or label
                        
                        if label not in drives:
                            drives[label] = free_gb
                    except:
                        continue
            else:
                # Basic fallback
                path = self.disk_path
                if platform.system() == "Windows":
                    import ctypes
                    free_bytes = ctypes.c_ulonglong(0)
                    ctypes.windll.kernel32.GetDiskFreeSpaceExW(ctypes.c_wchar_p(path), None, None, ctypes.pointer(free_bytes))
                    drives[path[:2]] = round(free_bytes.value / (1024**3), 1)
                else:
                    st = os.statvfs(path)
                    drives["/"] = round(st.f_bavail * st.f_frsize / (1024**3), 1)
        except:
            pass
        return drives

class QueueTab(QWidget):
    def __init__(self, parent=None, main_window=None, log_tab=None):
        super().__init__(parent)
        self.main_window = main_window
        self.log_tab = log_tab
        self.task_cards = {}
        self.all_expanded = True # State for toggle button
        
        self.init_ui()
        
        # Hardware Monitor - Start AFTER UI init to prevent race condition
        results_path = settings_manager.get('results_path', os.path.abspath(os.sep))
        self.monitor_thread = HardwareMonitorThread(results_path)
        self.monitor_thread.updated.connect(self.update_monitor_ui)
        self.monitor_thread.start()

    def __del__(self):
        """Ensure thread stops when tab is destroyed."""
        try:
            if hasattr(self, 'monitor_thread') and self.monitor_thread.isRunning():
                self.monitor_thread.stop()
                self.monitor_thread.wait()
        except:
            pass

    def init_ui(self):
        main_layout = QVBoxLayout(self)

        # Main Header Layout (Horizontal)
        header_layout = QHBoxLayout()
        header_layout.setContentsMargins(0, 0, 0, 0)
        
        # Left side: Balances (FlowLayout to allow wrapping)
        balances_widget = QWidget()
        # Use global FlowLayout from utils.flow_layout
        balances_layout = FlowLayout(balances_widget, hSpacing=20, vSpacing=10)
        balances_layout.setContentsMargins(0, 0, 0, 0)
        
        self.balance_label = QLabel()
        self.googler_usage_label = QLabel()
        self.elevenlabs_balance_label = QLabel()
        self.elevenlabs_unlim_balance_label = QLabel()
        self.voicemaker_balance_label = QLabel()
        self.gemini_tts_balance_label = QLabel()
        
        balances_layout.addWidget(self.balance_label)
        
        self.googler_usage_container = QWidget()
        self.googler_usage_layout = QHBoxLayout(self.googler_usage_container)
        self.googler_usage_layout.setContentsMargins(0, 0, 0, 0)
        self.googler_usage_layout.setSpacing(2)
        self.googler_usage_label = QLabel()
        self.googler_info_btn = QToolButton()
        from PySide6.QtWidgets import QStyle
        self.googler_info_btn.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_MessageBoxInformation))
        self.googler_info_btn.setFixedSize(16, 16)
        self.googler_info_btn.setStyleSheet("QToolButton { border: none; background: transparent; }")
        self.googler_info_btn.setToolTip("Googler Detailed Stats")
        self.googler_usage_layout.addWidget(self.googler_usage_label)
        self.googler_usage_layout.addWidget(self.googler_info_btn)
        
        balances_layout.addWidget(self.googler_usage_container)
        balances_layout.addWidget(self.elevenlabs_balance_label)
        balances_layout.addWidget(self.elevenlabs_unlim_balance_label)
        balances_layout.addWidget(self.voicemaker_balance_label)
        balances_layout.addWidget(self.gemini_tts_balance_label)
        
        # Right side: Controls (Fixed HBoxLayout)
        controls_widget = QWidget()
        controls_layout = QHBoxLayout(controls_widget)
        controls_layout.setContentsMargins(0, 0, 0, 0)
        controls_layout.setSpacing(10)
        
        # Shutdown After Processing Controls
        self.shutdown_container = QWidget()
        self.shutdown_layout = QHBoxLayout(self.shutdown_container)
        self.shutdown_layout.setContentsMargins(0, 0, 0, 0)
        self.shutdown_layout.setSpacing(5)

        self.shutdown_checkbox = QCheckBox(translator.translate('shutdown_after_processing'))
        # Always start unchecked (reset setting for this session)
        self.shutdown_checkbox.setChecked(False)
        settings_manager.set('shutdown_after_processing', False)
        self.shutdown_checkbox.stateChanged.connect(self.on_shutdown_toggled)

        self.shutdown_action_combo = QComboBox()
        self.shutdown_action_combo.setSizeAdjustPolicy(QComboBox.AdjustToContents)
        self.shutdown_action_combo.setMinimumWidth(150)
        self.shutdown_action_combo.addItems([
            translator.translate('action_sleep'),
            translator.translate('action_hibernate'),
            translator.translate('action_shutdown')
        ])
        # Map localized text back to internal keys or just use index logic
        self.shutdown_action_combo.setItemData(0, 'sleep')
        self.shutdown_action_combo.setItemData(1, 'hibernate')
        self.shutdown_action_combo.setItemData(2, 'shutdown')

        saved_action = settings_manager.get('shutdown_action', 'sleep')
        index = self.shutdown_action_combo.findData(saved_action)
        if index >= 0:
            self.shutdown_action_combo.setCurrentIndex(index)
        
        self.shutdown_action_combo.currentIndexChanged.connect(self.on_shutdown_action_changed)
        self.shutdown_action_combo.setVisible(self.shutdown_checkbox.isChecked())

        self.shutdown_layout.addWidget(self.shutdown_checkbox)
        self.shutdown_layout.addWidget(self.shutdown_action_combo)
        
        controls_layout.addWidget(self.shutdown_container)

        # Toggle All Button
        self.toggle_all_button = QPushButton()
        self.toggle_all_button.clicked.connect(self.on_toggle_all_clicked)
        controls_layout.addWidget(self.toggle_all_button)

        self.clear_queue_button = QPushButton()
        self.clear_queue_button.clicked.connect(self.on_clear_queue_clicked)
        self.clear_queue_button.setStyleSheet("""
            QPushButton {
                background-color: #dc3545;
                color: white;
                font-weight: bold;
                padding: 5px 15px;
                border-radius: 4px;
                border: none;
            }
            QPushButton:hover {
                background-color: #c82333;
            }
            QPushButton:disabled {
                background-color: #6c757d;
            }
        """)
        controls_layout.addWidget(self.clear_queue_button)
        
        self.start_processing_button = QPushButton()
        self.start_processing_button.setStyleSheet("""
            QPushButton {
                background-color: #28a745; 
                color: white; 
                font-weight: bold; 
                padding: 5px 15px;
                border-radius: 4px;
                border: none;
            }
            QPushButton:hover {
                background-color: #218838;
            }
            QPushButton:disabled {
                background-color: #6c757d;
            }
        """)
        controls_layout.addWidget(self.start_processing_button)
        
        # Add widgets to header layout
        # Balances need to expand to fill space and wrap
        header_layout.addWidget(balances_widget, 1) # Stretch factor 1
        
        # Controls stay compacted to the right
        header_layout.addWidget(controls_widget, 0)
        
        # Align controls to top so they don't float in middle if balances wrap
        header_layout.setAlignment(controls_widget, Qt.AlignTop)

        main_layout.addLayout(header_layout)

        # Monitoring Layout
        monitor_layout = QHBoxLayout()
        monitor_layout.setContentsMargins(5, 0, 5, 0)
        
        self.cpu_label = QLabel()
        self.ram_label = QLabel()
        self.gpu_label = QLabel()
        self.disk_label = QLabel()
        
        for lbl in [self.cpu_label, self.ram_label, self.gpu_label, self.disk_label]:
            lbl.setStyleSheet("color: #888; font-size: 11px; font-weight: bold;")
            monitor_layout.addWidget(lbl)
            if lbl != self.disk_label:
                monitor_layout.addSpacing(15)
        
        monitor_layout.addStretch()
        main_layout.addLayout(monitor_layout)
        
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        scroll_area.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        
        scroll_content = QWidget()
        self.tasks_layout = FlowLayout(scroll_content, v_align='top')
        
        scroll_area.setWidget(scroll_content)
        main_layout.addWidget(scroll_area)
        
        self.retranslate_ui()

    def set_processing_state(self, is_processing):
        self.start_processing_button.setDisabled(is_processing)
        self.clear_queue_button.setDisabled(is_processing)
        # self.start_processing_button.setText(
        #     "Processing..." if is_processing else translator.translate('start_processing')
        # )

    def on_toggle_all_clicked(self):
        self.all_expanded = not self.all_expanded
        
        # update UI text
        if self.all_expanded:
            self.toggle_all_button.setText(translator.translate('collapse_all'))
        else:
            self.toggle_all_button.setText(translator.translate('expand_all'))
            
        for card in self.task_cards.values():
            card.set_all_languages_expanded(self.all_expanded)

    def on_clear_queue_clicked(self):
        msg_box = QMessageBox(self)
        msg_box.setWindowTitle(translator.translate("confirm_clear_queue_title"))
        msg_box.setText(translator.translate("confirm_clear_queue_message"))
        msg_box.setStandardButtons(QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
        msg_box.setDefaultButton(QMessageBox.StandardButton.No)
        
        if msg_box.exec() == QMessageBox.StandardButton.Yes:
            # Clear the backend queue
            self.main_window.queue_manager.clear_queue()
            
            # Clear the UI cards
            for job_id in list(self.task_cards.keys()):
                card = self.task_cards.pop(job_id)
                self.tasks_layout.removeWidget(card)
                card.deleteLater()
            
            # Clear the gallery
            if hasattr(self.main_window, 'gallery_tab') and self.main_window.gallery_tab:
                self.main_window.gallery_tab.clear_gallery()

    def add_task(self, job):
        task_card = TaskCard(job, log_tab=self.log_tab)
        task_card.task_delete_requested.connect(self.on_task_deleted)
        task_card.language_delete_requested.connect(self.on_partial_delete)
        task_card.retry_requested.connect(self.on_retry_requested)
        
        # Respect current global toggled state
        task_card.set_all_languages_expanded(self.all_expanded)
        task_card.stage_delete_requested.connect(self.on_partial_delete)

        self.tasks_layout.addWidget(task_card)
        self.task_cards[job['id']] = task_card
        
        # Connect to name update signal
        self.main_window.queue_manager.task_name_updated.connect(self.update_task_name)

    def update_task_name(self, task_id, new_name):
        if task_id in self.task_cards:
            self.task_cards[task_id].update_name(new_name)

    def on_retry_requested(self, job_id):
        if hasattr(self.main_window, 'task_processor'):
            self.main_window.task_processor.retry_job(job_id)

    def on_task_deleted(self, job_id):
        if self.main_window.queue_manager.delete_job(job_id):
            if job_id in self.task_cards:
                card = self.task_cards[job_id]
                self.tasks_layout.removeWidget(card)
                card.deleteLater()
                del self.task_cards[job_id]

    def on_partial_delete(self, job_id, lang_id=None, stage_key=None):
        q_manager = self.main_window.queue_manager
        card = self.task_cards.get(job_id)
        if not card:
            return

        if lang_id and not stage_key: # Deleting a language
            if q_manager.delete_language_from_job(job_id, lang_id):
                card.hide_language(lang_id)
        elif lang_id and stage_key: # Deleting a stage
            if q_manager.delete_stage_from_language(job_id, lang_id, stage_key):
                card.hide_stage(lang_id, stage_key)
                
                # Check if the language is now empty
                updated_job = q_manager.get_job(job_id)
                if updated_job and not updated_job['languages'].get(lang_id, {}).get('stages'):
                    if q_manager.delete_language_from_job(job_id, lang_id):
                        card.hide_language(lang_id)

        # Check if the entire job is now empty
        updated_job = q_manager.get_job(job_id)
        if updated_job and not updated_job.get('languages'):
            self.on_task_deleted(job_id)

    def update_stage_status(self, job_id, lang_id, stage_key, status):
        if job_id in self.task_cards:
            card = self.task_cards[job_id]
            card.update_stage_status(lang_id, stage_key, status)
    
    def update_stage_metadata(self, job_id, lang_id, stage_key, metadata_text):
        """Update stage metadata display"""
        if job_id in self.task_cards:
            card = self.task_cards[job_id]
            card.update_stage_metadata(lang_id, stage_key, metadata_text)
    
    def on_task_progress_log(self, job_id, message):
        """Forward progress log to specific task card"""
        if job_id in self.task_cards:
            card = self.task_cards[job_id]
            card.on_progress_log(message)


    def update_balance(self, balance_text):
        self.balance_label.setText(balance_text)

    def update_googler_usage(self, usage_text):
        self.googler_usage_label.setText(usage_text)
        self.googler_usage_container.setVisible(bool(usage_text))

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

    def update_monitor_ui(self, stats):
        self.cpu_label.setText(f"{translator.translate('monitor_cpu')}: {stats['cpu']}%")
        self.ram_label.setText(f"{translator.translate('monitor_ram')}: {stats['ram']}%")
        self.gpu_label.setText(f"{translator.translate('monitor_gpu')}: {stats['gpu']}%")
        
        disks = stats.get('disks', {})
        disk_parts = [f"{name} {free}GB" for name, free in disks.items()]
        disk_str = " | ".join(disk_parts)
        self.disk_label.setText(f"{translator.translate('monitor_disk_free')}: {disk_str}")



    def retranslate_ui(self):
        self.start_processing_button.setText(translator.translate('start_processing'))
        self.clear_queue_button.setText(translator.translate('clear_queue'))
        
        # Update monitor labels initial text (if thread hasn't updated yet)
        if ":" not in self.cpu_label.text():
             self.cpu_label.setText(f"{translator.translate('monitor_cpu')}: --%")
             self.ram_label.setText(f"{translator.translate('monitor_ram')}: --%")
             self.gpu_label.setText(f"{translator.translate('monitor_gpu')}: --%")
             self.disk_label.setText(f"{translator.translate('monitor_disk_free')}: -- GB")

        if self.all_expanded:
            self.toggle_all_button.setText(translator.translate('collapse_all'))
        else:
            self.toggle_all_button.setText(translator.translate('expand_all'))
            
        # Update shutdown controls
        if hasattr(self, 'shutdown_checkbox'):
            self.shutdown_checkbox.setText(translator.translate('shutdown_after_processing'))
            
        if hasattr(self, 'shutdown_action_combo'):
            self.shutdown_action_combo.setItemText(0, translator.translate('action_sleep'))
            self.shutdown_action_combo.setItemText(1, translator.translate('action_hibernate'))
            self.shutdown_action_combo.setItemText(2, translator.translate('action_shutdown'))

        # Retranslating cards would be complex. For now, this is omitted.
        pass

    def on_shutdown_toggled(self, state):
        is_checked = (state == Qt.CheckState.Checked.value)
        self.shutdown_action_combo.setVisible(is_checked)
        settings_manager.set('shutdown_after_processing', is_checked)
        settings_manager.save_settings()

    def on_shutdown_action_changed(self, index):
        action = self.shutdown_action_combo.itemData(index)
        if action:
            settings_manager.set('shutdown_action', action)
            settings_manager.save_settings()
