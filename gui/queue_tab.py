from PySide6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QHBoxLayout, QLabel, QScrollArea, QGroupBox
from PySide6.QtCore import Qt
from utils.translator import translator
from utils.flow_layout import FlowLayout

class TaskCard(QGroupBox):
    """A widget that displays a single task as a card."""
    def __init__(self, job, parent=None):
        super().__init__(job['name'], parent)
        self.setMinimumWidth(250)
        self.setMaximumWidth(350)

        layout = QVBoxLayout(self)
        
        for lang_id, lang_data in job['languages'].items():
            lang_label = QLabel(f"<b>{lang_data['display_name']}</b>")
            layout.addWidget(lang_label)
            
            # Use a single label with rich text for stages
            stages_text = "<br>".join(f"&nbsp;&nbsp;• {translator.translate(stage_key)}" for stage_key in lang_data['stages'])
            stages_label = QLabel(stages_text)
            stages_label.setWordWrap(True)
            stages_label.setAlignment(Qt.AlignmentFlag.AlignTop)
            layout.addWidget(stages_label)
        
        layout.addStretch()

class QueueTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
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

    def update_balance(self, balance_text):
        self.balance_label.setText(balance_text)

    def retranslate_ui(self):
        self.start_processing_button.setText(translator.translate('start_processing'))
        # We might need to retranslate existing cards if language changes
        for i in range(self.tasks_layout.count()):
            widget = self.tasks_layout.itemAt(i).widget()
            if isinstance(widget, TaskCard):
                # This is complex, as the card is built with translated text.
                # For now, we assume cards are not re-translated after creation.
                pass