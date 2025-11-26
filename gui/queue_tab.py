from PySide6.QtWidgets import QWidget, QVBoxLayout, QTreeWidget, QTreeWidgetItem, QPushButton, QHBoxLayout, QLabel
from utils.translator import translator

class QueueTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.jobs_expanded = False
        self.all_expanded = False
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)

        top_layout = QHBoxLayout()
        self.balance_label = QLabel()
        top_layout.addWidget(self.balance_label)
        top_layout.addStretch()
        main_layout.addLayout(top_layout)
        
        # Button layout
        button_layout = QHBoxLayout()
        self.expand_jobs_button = QPushButton()
        self.expand_jobs_button.clicked.connect(self.toggle_expand_jobs)
        button_layout.addWidget(self.expand_jobs_button)

        self.expand_all_button = QPushButton()
        self.expand_all_button.clicked.connect(self.toggle_expand_all)
        button_layout.addWidget(self.expand_all_button)
        button_layout.addStretch()

        self.start_processing_button = QPushButton()
        button_layout.addWidget(self.start_processing_button)

        main_layout.addLayout(button_layout)

        self.task_tree = QTreeWidget()
        self.task_tree.setHeaderLabels([translator.translate('task_header')])
        self.task_tree.setIndentation(10)
        self.task_tree.setStyleSheet("QTreeView::item { padding: 1px 0px; }")
        main_layout.addWidget(self.task_tree)
        
        self.retranslate_ui() # Set initial button text

    def add_task(self, job):
        # Create top-level item for the job
        job_item = QTreeWidgetItem(self.task_tree, [job['name']])

        # Create child items for each language
        for lang_id, lang_data in job['languages'].items():
            lang_item = QTreeWidgetItem(job_item, [lang_data['display_name']])
            
            # Create grand-child items for each stage
            for stage_key in lang_data['stages']:
                stage_name = translator.translate(stage_key)
                stage_item = QTreeWidgetItem(lang_item, [stage_name])
        
        # Reset state and expand to job level
        self.jobs_expanded = False 
        self.all_expanded = False
        self.toggle_expand_jobs()

    def toggle_expand_jobs(self):
        if self.jobs_expanded:
            self.task_tree.collapseAll()
            self.jobs_expanded = False
            self.all_expanded = False
        else:
            self.task_tree.collapseAll()
            for i in range(self.task_tree.topLevelItemCount()):
                self.task_tree.topLevelItem(i).setExpanded(True)
            self.jobs_expanded = True
            self.all_expanded = False # Expanding jobs is a subset of expanding all
        self.update_button_text()
            
    def toggle_expand_all(self):
        if self.all_expanded:
            self.task_tree.collapseAll()
            self.all_expanded = False
            self.jobs_expanded = False
        else:
            self.task_tree.expandAll()
            self.all_expanded = True
            self.jobs_expanded = True
        self.update_button_text()

    def update_button_text(self):
        if self.jobs_expanded:
            self.expand_jobs_button.setText(translator.translate('collapse_jobs'))
        else:
            self.expand_jobs_button.setText(translator.translate('expand_jobs'))
            
        if self.all_expanded:
            self.expand_all_button.setText(translator.translate('collapse_all'))
        else:
            self.expand_all_button.setText(translator.translate('expand_all'))

    def update_balance(self, balance_text):
        self.balance_label.setText(balance_text)

    def retranslate_ui(self):
        self.task_tree.setHeaderLabels([translator.translate('task_header')])
        self.start_processing_button.setText(translator.translate('start_processing'))
        self.update_button_text()