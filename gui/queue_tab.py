from PySide6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QHBoxLayout, QLabel, QScrollArea, QGroupBox
from PySide6.QtCore import Qt
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

class TaskCard(QGroupBox):
    """A widget that displays a single task and its stage statuses."""
    def __init__(self, job, parent=None):
        super().__init__(job['name'], parent)
        self.job_id = job['id']
        self.stage_dots = {}

        self.setMinimumWidth(250)
        self.setMaximumWidth(350)

        main_layout = QVBoxLayout(self)
        
        for lang_id, lang_data in job['languages'].items():
            lang_label = QLabel(f"<b>{lang_data['display_name']}</b>")
            main_layout.addWidget(lang_label)
            
            for stage_key in lang_data['stages']:
                stage_layout = QHBoxLayout()
                
                dot = StatusDot()
                self.stage_dots[(lang_id, stage_key)] = dot
                
                stage_label = QLabel(translator.translate(stage_key))
                
                stage_layout.addWidget(dot)
                stage_layout.addWidget(stage_label)
                stage_layout.addStretch()
                
                main_layout.addLayout(stage_layout)
        
        main_layout.addStretch()

    def update_stage_status(self, lang_id, stage_key, status):
        if (lang_id, stage_key) in self.stage_dots:
            self.stage_dots[(lang_id, stage_key)].set_status(status)

class QueueTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.task_cards = {} # To store card references by job_id
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)

        top_layout = QHBoxLayout()
        self.balance_label = QLabel()
        top_layout.addWidget(self.balance_label)
        top_layout.addStretch()
        
        self.start_processing_button = QPushButton()
        top_layout.addWidget(self.start_processing_button)
        main_layout.addLayout(top_layout)

        # Scroll Area for task cards
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
        self.tasks_layout.addWidget(task_card)
        self.task_cards[job['id']] = task_card

    def update_stage_status(self, job_id, lang_id, stage_key, status):
        if job_id in self.task_cards:
            card = self.task_cards[job_id]
            card.update_stage_status(lang_id, stage_key, status)

    def update_balance(self, balance_text):
        self.balance_label.setText(balance_text)

    def retranslate_ui(self):
        self.start_processing_button.setText(translator.translate('start_processing'))
        # Retranslating cards would be complex. For now, this is omitted.
        pass