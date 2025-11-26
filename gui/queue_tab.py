from PySide6.QtWidgets import QWidget, QVBoxLayout, QTreeWidget, QTreeWidgetItem
from utils.translator import translator

class QueueTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        self.task_tree = QTreeWidget()
        self.task_tree.setHeaderLabels([translator.translate('task_header')])
        self.task_tree.setIndentation(10)
        self.task_tree.setStyleSheet("QTreeView::item { padding: 1px 0px; }")
        main_layout.addWidget(self.task_tree)

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
        
        self.task_tree.expandAll()


    def retranslate_ui(self):
        self.task_tree.setHeaderLabels([translator.translate('task_header')])
        # Note: Retranslating existing items is more complex and not implemented here.
        # It would require iterating through the tree and updating each item's text.