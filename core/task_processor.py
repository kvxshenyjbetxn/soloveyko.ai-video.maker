import os
import time
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer
from utils.logger import logger
from utils.settings import settings_manager
from api.openrouter import OpenRouterAPI

class TaskProcessor(QObject):
    processing_finished = Signal(str)

    def __init__(self, queue_manager):
        super().__init__()
        self.queue_manager = queue_manager
        self.settings = settings_manager
        self.openrouter_api = OpenRouterAPI()
        self.threadpool = QThreadPool()
        self.active_jobs = 0
        self.timer = QElapsedTimer()

    def start_processing(self):
        jobs = self.queue_manager.get_tasks()
        if not jobs:
            logger.log("Queue is empty. Nothing to process.")
            return

        self.active_jobs = len(jobs)
        self.timer.start()
        logger.log(f"Starting to process {self.active_jobs} jobs in the queue.")
        
        for job in jobs:
            worker = JobWorker(self.openrouter_api, self.settings, job)
            worker.signals.finished.connect(self.job_finished)
            self.threadpool.start(worker)

    def job_finished(self):
        self.active_jobs -= 1
        if self.active_jobs == 0:
            elapsed_ms = self.timer.elapsed()
            elapsed_str = time.strftime('%H:%M:%S', time.gmtime(elapsed_ms / 1000))
            logger.log(f"Queue processing finished in {elapsed_str}.")
            self.processing_finished.emit(elapsed_str)

class JobWorkerSignals(QObject):
    finished = Signal()

class JobWorker(QRunnable):
    def __init__(self, api, settings, job):
        super().__init__()
        self.api = api
        self.settings = settings
        self.job = job
        self.signals = JobWorkerSignals()

    def run(self):
        logger.log(f"Processing job: {self.job['name']}")
        
        base_save_path = self.settings.get('results_path')
        if not base_save_path:
            logger.log("Warning: 'results_path' is not configured in settings. Results will not be saved.")

        all_languages_config = self.settings.get("languages_config", {})
        
        for lang_id, lang_data in self.job['languages'].items():
            if 'stage_translation' in lang_data['stages']:
                lang_config = all_languages_config.get(lang_id)
                if not lang_config:
                    logger.log(f"  - Skipping translation for {lang_data['display_name']}: Configuration not found.")
                    continue
                
                logger.log(f"  - Starting translation for: {lang_data['display_name']}")
                
                prompt = lang_config.get('prompt', '')
                model = lang_config.get('model', '')
                max_tokens = lang_config.get('max_tokens', 4096)
                
                if not model:
                    logger.log(f"  - Skipping translation for {lang_data['display_name']}: Model not configured.")
                    continue
                
                full_prompt = f"{prompt}\n\n{self.job['text']}"
                
                try:
                    logger.log(f"    - Calling model '{model}' with max_tokens={max_tokens}.")
                    response = self.api.get_chat_completion(
                        model=model,
                        messages=[{"role": "user", "content": full_prompt}],
                        max_tokens=max_tokens
                    )
                    
                    if response:
                        translated_text = response['choices'][0]['message']['content']
                        logger.log(f"    - Translation successful for {lang_data['display_name']}.")
                        
                        if base_save_path:
                            self.save_result(base_save_path, self.job['name'], lang_data['display_name'], translated_text)

                    else:
                        logger.log(f"    - Translation failed for {lang_data['display_name']}: Empty response from API.")

                except Exception as e:
                    logger.log(f"    - An error occurred during translation for {lang_data['display_name']}: {e}")

        logger.log(f"Finished processing job: {self.job['name']}")
        self.signals.finished.emit()

    def save_result(self, base_path, job_name, lang_name, content):
        try:
            # Sanitize names to be safe for directory/file creation
            safe_job_name = "".join(c for c in job_name if c.isalnum() or c in (' ', '_')).rstrip()
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
            
            dir_path = os.path.join(base_path, safe_job_name, safe_lang_name)
            os.makedirs(dir_path, exist_ok=True)
            
            file_path = os.path.join(dir_path, "translation.txt")
            
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            logger.log(f"    - Result saved to: {file_path}")

        except Exception as e:
            logger.log(f"    - Failed to save result. Error: {e}")