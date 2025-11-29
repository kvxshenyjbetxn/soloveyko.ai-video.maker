from PySide6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QHBoxLayout, QLabel, QScrollArea, QGroupBox, QMenu, QToolButton, QMessageBox
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QAction
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
            color = '#ffa500' # Orange
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
        self.stage_name = translator.translate(stage_key)
        
        layout = QHBoxLayout()
        layout.setContentsMargins(4, 0, 4, 0)
        self.setLayout(layout)

        self.dot = StatusDot(self)
        self.label = QLabel(self.stage_name, self)
        
        layout.addWidget(self.dot)
        layout.addWidget(self.label)
        layout.addStretch()

    def contextMenuEvent(self, event):
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
        layout.setContentsMargins(0, 0, 0, 0)
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

class TaskCard(QGroupBox):
    """A widget that displays a single task and its stage statuses."""
    task_delete_requested = Signal(str) # job_id
    language_delete_requested = Signal(str, str) # job_id, lang_id
    stage_delete_requested = Signal(str, str, str) # job_id, lang_id, stage_key

    def __init__(self, job, parent=None):
        super().__init__("", parent)
        self.job_id = job['id']
        self.job_name = job['name']
        self.setStyleSheet("QGroupBox { padding-top: 8px; padding-bottom: 8px; }")

        self.language_widgets = {}
        self.stage_widgets = {}

        self.init_ui(job)

    def init_ui(self, job):
        layout = QVBoxLayout(self)

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
            self.language_widgets[lang_id] = lang_header
            layout.addWidget(lang_header)
            
            self.stage_widgets[lang_id] = []

            # If translation is not a selected stage, show 'Original'
            if 'stage_translation' not in lang_data['stages']:
                original_widget = QWidget()
                original_layout = QHBoxLayout(original_widget)
                original_layout.setContentsMargins(4, 0, 4, 0)
                
                dot = StatusDot(self)
                dot.set_status('success') # Original text is always 'successful'
                label = QLabel(translator.translate('original_text'), self)
                
                original_layout.addWidget(dot)
                original_layout.addWidget(label)
                original_layout.addStretch()
                layout.addWidget(original_widget)

            for stage_key in lang_data['stages']:
                stage_widget = DeletableStageWidget(stage_key, self)
                stage_widget.delete_requested.connect(lambda lid=lang_id, skey=stage_key: self.on_stage_delete(lid, skey))
                
                self.stage_widgets[lang_id].append(stage_widget)
                layout.addWidget(stage_widget)
        
        layout.addStretch()

    def hide_language(self, lang_id):
        if lang_id in self.language_widgets:
            self.language_widgets[lang_id].setVisible(False)
        if lang_id in self.stage_widgets:
            for stage_widget in self.stage_widgets[lang_id]:
                stage_widget.setVisible(False)

    def hide_stage(self, lang_id, stage_key):
        if lang_id in self.stage_widgets:
            stage_name_to_find = translator.translate(stage_key)
            for stage_widget in self.stage_widgets[lang_id]:
                if stage_widget.label.text() == stage_name_to_find:
                    stage_widget.setVisible(False)
                    # Also remove it from the list to avoid finding it again
                    self.stage_widgets[lang_id].remove(stage_widget)
                    break
    
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
            stage_name_to_find = translator.translate(stage_key)
            for stage_widget in self.stage_widgets[lang_id]:
                if stage_widget.label.text() == stage_name_to_find:
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
        top_layout.addWidget(self.balance_label)
        top_layout.addSpacing(20)
        top_layout.addWidget(self.googler_usage_label)
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

    def retranslate_ui(self):
        self.start_processing_button.setText(translator.translate('start_processing'))
        # Retranslating cards would be complex. For now, this is omitted.
        pass
