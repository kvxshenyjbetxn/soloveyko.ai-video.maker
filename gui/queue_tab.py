from PySide6.QtWidgets import QWidget, QVBoxLayout, QListWidget, QListWidgetItem
from utils.translator import translator

class QueueTab(QWidget):
    def __init__(self, main_window=None):
        super().__init__()
        self.main_window = main_window
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        self.task_list = QListWidget()
        main_layout.addWidget(self.task_list)

    def add_task(self, task):
        item = QListWidgetItem(task['name'])
        self.task_list.addItem(item)

    def retranslate_ui(self):
        # The queue tab itself doesn't have much text to retranslate
        # but we might need it later.
        pass
