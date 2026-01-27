from PySide6.QtCore import QObject, Signal
from core.history_manager import history_manager

class QueueManager(QObject):
    task_added = Signal(dict)
    queue_updated = Signal()

    def __init__(self):
        super().__init__()
        self.tasks = []
        self.task_counter = 0

    def generate_task_id(self):
        self.task_counter += 1
        return f"Task-{self.task_counter}"

    def add_task(self, task):
        # Generate simple ID if not provided
        if 'id' not in task or not task['id']:
            task['id'] = self.generate_task_id()
        
        # Register in recent history for recovery (if not already restored from history)
        if not task.get('is_restored'):
            history_manager.register_recent_job(task)
        
        self.tasks.append(task)
        self.task_added.emit(task)
        self.queue_updated.emit()

    def get_tasks(self):
        return self.tasks

    def clear_queue(self):
        self.tasks.clear()
        self.task_counter = 0
        self.queue_updated.emit()

    def get_task_count(self):
        return len(self.tasks)

    def get_job(self, job_id):
        for job in self.tasks:
            if job['id'] == job_id:
                return job
        return None

    def delete_job(self, job_id):
        job_to_delete = None
        for job in self.tasks:
            if job['id'] == job_id:
                job_to_delete = job
                break
        if job_to_delete:
            self.tasks.remove(job_to_delete)
            if not self.tasks:
                self.task_counter = 0
            self.queue_updated.emit()
            return True
        return False

    def delete_language_from_job(self, job_id, lang_id):
        for job in self.tasks:
            if job['id'] == job_id:
                if lang_id in job['languages']:
                    del job['languages'][lang_id]
                    # If no languages are left, should we delete the job?
                    # For now, let's leave the job, it will just be empty.
                    return True
        return False

    def delete_stage_from_language(self, job_id, lang_id, stage_key):
        for job in self.tasks:
            if job['id'] == job_id:
                if lang_id in job['languages']:
                    if stage_key in job['languages'][lang_id]['stages']:
                        job['languages'][lang_id]['stages'].remove(stage_key)
                        # If no stages are left, should we delete the language?
                        # For now, let's leave it.
                        return True
        return False
