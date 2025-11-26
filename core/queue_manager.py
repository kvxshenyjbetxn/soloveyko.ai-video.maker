from PySide6.QtCore import QObject, Signal

class QueueManager(QObject):
    task_added = Signal(dict)

    def __init__(self):
        super().__init__()
        self.tasks = []

    def add_task(self, task):
        self.tasks.append(task)
        self.task_added.emit(task)

    def get_tasks(self):
        return self.tasks

    def get_task_count(self):
        return len(self.tasks)
