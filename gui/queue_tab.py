from PySide6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QHBoxLayout, QLabel, QScrollArea, QGroupBox, QMenu, QToolButton, QMessageBox, QTextEdit, QFrame, QGridLayout
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QAction, QCursor, QPalette, QColor
from functools import partial
from utils.translator import translator
from utils.flow_layout import FlowLayout

class StatusDot(QLabel):
    """A circular widget to indicate status."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedSize(12, 12)
        self.set_status('pending')

    def set_status(self, status):
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

class CompactStageWidget(QWidget):
    """Compact horizontal widget for a single stage."""
    delete_requested = Signal()

    def __init__(self, stage_key, parent=None, is_original=False):
        super().__init__(parent)
        self.stage_key = stage_key
        self.stage_name = translator.translate(stage_key) if not is_original else stage_key
        self.is_original = is_original
        
        layout = QHBoxLayout()
        layout.setContentsMargins(2, 2, 2, 2)
        layout.setSpacing(4)
        self.setLayout(layout)

        self.dot = StatusDot(self)
        self.label = QLabel(self.stage_name, self)
        
        layout.addWidget(self.dot)
        layout.addWidget(self.label)
        
        self.setStyleSheet("""
            CompactStageWidget {
                background-color: transparent;
                border: 1px solid palette(mid);
                border-radius: 4px;
                padding: 2px;
            }
        """)

    def contextMenuEvent(self, event):
        if not self.is_original:
            menu = QMenu(self)
            delete_action = QAction(translator.translate("delete_stage_formatted").format(self.stage_name), self)
            delete_action.triggered.connect(self.delete_requested.emit)
            menu.addAction(delete_action)
            menu.exec(event.globalPos())

    def get_dot(self):
        return self.dot

class DeletableLanguageHeader(QWidget):
    """A widget for the language header, allowing deletion via context menu."""
    delete_requested = Signal()

    def __init__(self, display_name, parent=None):
        super().__init__(parent)
        self.display_name = display_name

        layout = QHBoxLayout()
        layout.setContentsMargins(0, 4, 0, 2)
        self.setLayout(layout)

        self.label = QLabel(f"<b>{self.display_name}</b>", self)
        layout.addWidget(self.label)
        layout.addStretch()

    def contextMenuEvent(self, event):
        menu = QMenu(self)
        delete_action = QAction(translator.translate("delete_language_formatted").format(self.display_name), self)
        delete_action.triggered.connect(self.delete_requested.emit)
        menu.addAction(delete_action)
        menu.exec(event.globalPos())

class LogWidget(QTextEdit):
    """Widget for displaying task logs."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setReadOnly(True)
        self.setMaximumHeight(150)
        self.setMinimumHeight(150)
        self.setPlaceholderText("Лог завантажується...")
        self.setVisible(False)
        self.setStyleSheet("""
            QTextEdit {
                background-color: #1e1e1e;
                color: #cccccc;
                border: 1px solid #444;
                border-radius: 4px;
                padding: 4px;
                font-family: 'Consolas', 'Courier New', monospace;
                font-size: 10pt;
            }
        """)

class TaskCard(QGroupBox):
    """A widget that displays a single task and its stage statuses."""
    task_delete_requested = Signal(str) # job_id
    language_delete_requested = Signal(str, str) # job_id, lang_id
    stage_delete_requested = Signal(str, str, str) # job_id, lang_id, stage_key

    def __init__(self, job, parent=None):
        super().__init__("", parent)
        self.job_id = job['id']
        self.job_name = job['name']
        self.log_expanded = False
        
        self.setStyleSheet("""
            QGroupBox { 
                padding: 8px; 
                border: 1px solid #555;
                border-radius: 6px;
                background-color: transparent;
            }
            QGroupBox:hover {
                background-color: rgba(255, 255, 255, 0.05);
            }
        """)
        self.setCursor(QCursor(Qt.CursorShape.PointingHandCursor))

        self.language_widgets = {}
        self.stage_widgets = {}
        self.stages_containers = {}

        self.init_ui(job)

    def _create_branched_layout(self, lang_id, lang_data):
        """Create a branched visualization of stages."""
        stages_container = QWidget()
        grid_layout = QGridLayout(stages_container)
        grid_layout.setContentsMargins(0, 0, 0, 4)
        grid_layout.setHorizontalSpacing(8)
        grid_layout.setVerticalSpacing(2)
        
        self.stages_containers[lang_id] = stages_container
        self.stage_widgets[lang_id] = []
        
        # Classify stages into branches
        start_stage = None
        upper_branch = []  # Visual stages (prompts, images)
        lower_branch = []  # Audio stages (voiceover, subtitles)
        end_stage = None
        
        # Add original/translation as start
        if 'stage_translation' not in lang_data['stages']:
            start_stage = ('original_text', translator.translate('original_text'), True)
        else:
            if 'stage_translation' in lang_data['stages']:
                start_stage = ('stage_translation', None, False)
        
        # Classify stages
        for stage_key in lang_data['stages']:
            if stage_key == 'stage_translation':
                continue
            elif stage_key in ['stage_img_prompts', 'stage_images']:
                upper_branch.append(stage_key)
            elif stage_key in ['stage_voiceover', 'stage_subtitles']:
                lower_branch.append(stage_key)
            elif stage_key == 'stage_montage':
                end_stage = stage_key
        
        col = 0
        
        # Add start stage (middle row)
        if start_stage:
            stage_key, stage_name, is_original = start_stage
            if is_original:
                start_widget = CompactStageWidget(stage_name, self, is_original=True)
                start_widget.get_dot().set_status('success')
            else:
                start_widget = CompactStageWidget(stage_key, self)
                start_widget.delete_requested.connect(
                    lambda checked=False, lid=lang_id, skey=stage_key: self.on_stage_delete(lid, skey)
                )
            
            self.stage_widgets[lang_id].append(start_widget)
            grid_layout.addWidget(start_widget, 1, col)
            col += 1
            
            # Add diagonal arrows if there are branches
            if upper_branch or lower_branch:
                arrow_container = QWidget()
                arrow_layout = QVBoxLayout(arrow_container)
                arrow_layout.setContentsMargins(0, 0, 0, 0)
                arrow_layout.setSpacing(15)
                
                if upper_branch:
                    arrow_up = QLabel("↗", self)
                    arrow_up.setStyleSheet("color: #888; font-size: 18pt; padding: 0px; margin: 0px;")
                    arrow_layout.addWidget(arrow_up, alignment=Qt.AlignmentFlag.AlignCenter)
                else:
                    arrow_layout.addStretch()
                
                if lower_branch:
                    arrow_down = QLabel("↘", self)
                    arrow_down.setStyleSheet("color: #888; font-size: 18pt; padding: 0px; margin: 0px;")
                    arrow_layout.addWidget(arrow_down, alignment=Qt.AlignmentFlag.AlignCenter)
                else:
                    arrow_layout.addStretch()
                
                grid_layout.addWidget(arrow_container, 0, col, 3, 1)
                col += 1
        
        # Add both branches in parallel columns
        max_branch_len = max(len(upper_branch), len(lower_branch))
        
        for i in range(max_branch_len):
            # Add arrows before stages (except first column)
            if i > 0:
                if i < len(upper_branch):
                    arrow = QLabel("→", self)
                    arrow.setStyleSheet("color: #888; font-size: 14pt;")
                    grid_layout.addWidget(arrow, 0, col)
                
                if i < len(lower_branch):
                    arrow = QLabel("→", self)
                    arrow.setStyleSheet("color: #888; font-size: 14pt;")
                    grid_layout.addWidget(arrow, 2, col)
                
                col += 1
            
            # Add upper branch stage
            if i < len(upper_branch):
                stage_key = upper_branch[i]
                stage_widget = CompactStageWidget(stage_key, self)
                stage_widget.delete_requested.connect(
                    lambda checked=False, lid=lang_id, skey=stage_key: self.on_stage_delete(lid, skey)
                )
                self.stage_widgets[lang_id].append(stage_widget)
                grid_layout.addWidget(stage_widget, 0, col)
            
            # Add lower branch stage
            if i < len(lower_branch):
                stage_key = lower_branch[i]
                stage_widget = CompactStageWidget(stage_key, self)
                stage_widget.delete_requested.connect(
                    lambda checked=False, lid=lang_id, skey=stage_key: self.on_stage_delete(lid, skey)
                )
                self.stage_widgets[lang_id].append(stage_widget)
                grid_layout.addWidget(stage_widget, 2, col)
            
            col += 1
        
        # Add end stage (montage) in the middle row
        if end_stage:
            # Diagonal arrows converging to montage
            arrow_container = QWidget()
            arrow_layout = QVBoxLayout(arrow_container)
            arrow_layout.setContentsMargins(0, 0, 0, 0)
            arrow_layout.setSpacing(15)
            
            if upper_branch:
                arrow_down = QLabel("↘", self)
                arrow_down.setStyleSheet("color: #888; font-size: 18pt; padding: 0px; margin: 0px;")
                arrow_layout.addWidget(arrow_down, alignment=Qt.AlignmentFlag.AlignCenter)
            else:
                arrow_layout.addStretch()
            
            if lower_branch:
                arrow_up = QLabel("↗", self)
                arrow_up.setStyleSheet("color: #888; font-size: 18pt; padding: 0px; margin: 0px;")
                arrow_layout.addWidget(arrow_up, alignment=Qt.AlignmentFlag.AlignCenter)
            else:
                arrow_layout.addStretch()
            
            grid_layout.addWidget(arrow_container, 0, col, 3, 1)
            col += 1
            
            end_widget = CompactStageWidget(end_stage, self)
            end_widget.delete_requested.connect(
                lambda checked=False, lid=lang_id, skey=end_stage: self.on_stage_delete(lid, skey)
            )
            self.stage_widgets[lang_id].append(end_widget)
            grid_layout.addWidget(end_widget, 1, col)
        
        # Add stretch to push everything to the left
        grid_layout.setColumnStretch(col + 1, 1)
        
        return stages_container

    def init_ui(self, job):
        layout = QVBoxLayout(self)
        layout.setSpacing(4)

        # Custom header
        header_layout = QHBoxLayout()
        task_name_label = QLabel(f"<b>{self.job_name}</b>", self)
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

        for lang_id, lang_data in job['languages'].items():
            lang_header = DeletableLanguageHeader(lang_data['display_name'], self)
            lang_header.delete_requested.connect(
                lambda checked=False, lid=lang_id: self.on_language_delete(lid)
            )
            self.language_widgets[lang_id] = lang_header
            layout.addWidget(lang_header)
            
            # Create branched layout
            stages_container = self._create_branched_layout(lang_id, lang_data)
            layout.addWidget(stages_container)
        
        self.log_widget = LogWidget(self)
        layout.addWidget(self.log_widget)

    def hide_language(self, lang_id):
        if lang_id in self.language_widgets:
            self.language_widgets[lang_id].setVisible(False)
        if lang_id in self.stages_containers:
            self.stages_containers[lang_id].setVisible(False)

    def hide_stage(self, lang_id, stage_key):
        if lang_id in self.stage_widgets:
            stage_name_to_find = translator.translate(stage_key)
            for stage_widget in self.stage_widgets[lang_id]:
                if hasattr(stage_widget, 'stage_key') and stage_widget.stage_key == stage_key:
                    stage_widget.setVisible(False)
                    self.stage_widgets[lang_id].remove(stage_widget)
                    break
    
    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.log_expanded = not self.log_expanded
            self.log_widget.setVisible(self.log_expanded)
        super().mousePressEvent(event)
    
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
        if lang_id in self.stage_widgets:
            for stage_widget in self.stage_widgets[lang_id]:
                if hasattr(stage_widget, 'stage_key') and stage_widget.stage_key == stage_key:
                    stage_widget.get_dot().set_status(status)
                    break

class QueueTab(QWidget):
    def __init__(self, parent=None, main_window=None):
        super().__init__(parent)
        self.main_window = main_window
        self.task_cards = {}
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)

        top_layout = QHBoxLayout()
        self.balance_label = QLabel()
        self.googler_usage_label = QLabel()
        self.elevenlabs_balance_label = QLabel()
        self.voicemaker_balance_label = QLabel()
        self.gemini_tts_balance_label = QLabel()
        top_layout.addWidget(self.balance_label)
        top_layout.addSpacing(20)
        top_layout.addWidget(self.googler_usage_label)
        top_layout.addSpacing(20)
        top_layout.addWidget(self.elevenlabs_balance_label)
        top_layout.addSpacing(20)
        top_layout.addWidget(self.voicemaker_balance_label)
        top_layout.addSpacing(20)
        top_layout.addWidget(self.gemini_tts_balance_label)
        top_layout.addStretch()
        
        self.start_processing_button = QPushButton()
        top_layout.addWidget(self.start_processing_button)
        main_layout.addLayout(top_layout)

        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        
        scroll_content = QWidget()
        self.tasks_layout = FlowLayout(scroll_content)
        
        scroll_area.setWidget(scroll_content)
        main_layout.addWidget(scroll_area)
        
        self.retranslate_ui()

    def add_task(self, job):
        task_card = TaskCard(job)
        task_card.task_delete_requested.connect(self.on_task_deleted)
        task_card.language_delete_requested.connect(self.on_partial_delete)
        task_card.stage_delete_requested.connect(self.on_partial_delete)

        self.tasks_layout.addWidget(task_card)
        self.task_cards[job['id']] = task_card

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

    def update_balance(self, balance_text):
        self.balance_label.setText(balance_text)

    def update_googler_usage(self, usage_text):
        self.googler_usage_label.setText(usage_text)

    def update_elevenlabs_balance(self, balance_text):
        self.elevenlabs_balance_label.setText(balance_text)

    def update_voicemaker_balance(self, balance_text):
        self.voicemaker_balance_label.setText(balance_text)

    def update_gemini_tts_balance(self, balance_text):
        self.gemini_tts_balance_label.setText(balance_text)

    def retranslate_ui(self):
        self.start_processing_button.setText(translator.translate('start_processing'))
        # Retranslating cards would be complex. For now, this is omitted.
        pass
