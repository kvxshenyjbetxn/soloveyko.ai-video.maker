import os
import time
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer
from utils.logger import logger
from utils.settings import settings_manager
from api.openrouter import OpenRouterAPI

class TaskProcessor(QObject):
    processing_finished = Signal(str)
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status

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
            # Propagate the stage status signal
            worker.signals.stage_status_changed.connect(self.stage_status_changed)
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
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status

class JobWorker(QRunnable):
    def __init__(self, api, settings, job):
        super().__init__()
        self.api = api
        self.settings = settings
        self.job = job
        self.signals = JobWorkerSignals()

    def run(self):
        logger.log(f"Processing job: {self.job['name']}")
        job_id = self.job['id']
        
        base_save_path = self.settings.get('results_path')
        if not base_save_path:
            logger.log("Warning: 'results_path' is not configured in settings. Results will not be saved.")

        all_languages_config = self.settings.get("languages_config", {})
        
        for lang_id, lang_data in self.job['languages'].items():
            lang_dir_path = self._get_save_path(base_save_path, self.job['name'], lang_data['display_name'])
            
            text_for_processing = self.job['text']
            translation_successful = False

            # --- Translation Stage ---
            if 'stage_translation' in lang_data['stages']:
                translated_text = self._perform_translation(job_id, lang_id, lang_data, all_languages_config)
                if translated_text:
                    text_for_processing = translated_text
                    translation_successful = True
                    if lang_dir_path:
                        self.save_translation(lang_dir_path, translated_text)
            
            # --- Image Prompts Stage ---
            if 'stage_img_prompts' in lang_data['stages']:
                self.process_image_prompts(job_id, lang_id, lang_data, text_for_processing, lang_dir_path)

        logger.log(f"Finished processing job: {self.job['name']}")
        self.signals.finished.emit()

    def _get_save_path(self, base_path, job_name, lang_name):
        if not base_path:
            return None
        try:
            safe_job_name = "".join(c for c in job_name if c.isalnum() or c in (' ', '_')).rstrip()
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
            dir_path = os.path.join(base_path, safe_job_name, safe_lang_name)
            os.makedirs(dir_path, exist_ok=True)
            return dir_path
        except Exception as e:
            logger.log(f"    - Failed to create save directory. Error: {e}")
            return None

    def _perform_translation(self, job_id, lang_id, lang_data, all_languages_config):
        stage_key = 'stage_translation'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')
        
        lang_config = all_languages_config.get(lang_id)
        if not lang_config:
            logger.log(f"  - Skipping translation for {lang_data['display_name']}: Configuration not found.")
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None
        
        logger.log(f"  - Starting translation for: {lang_data['display_name']}")
        
        prompt = lang_config.get('prompt', '')
        model = lang_config.get('model', '')
        max_tokens = lang_config.get('max_tokens', 4096)
        
        if not model:
            logger.log(f"  - Skipping translation for {lang_data['display_name']}: Model not configured.")
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None
        
        full_prompt = f"{prompt}\n\n{self.job['text']}"
        
        try:
            logger.log(f"    - Calling model '{model}' with max_tokens={max_tokens}.")
            response = self.api.get_chat_completion(
                model=model,
                messages=[{"role": "user", "content": full_prompt}],
                max_tokens=max_tokens
            )
            
            if response and response['choices'][0]['message']['content']:
                translated_text = response['choices'][0]['message']['content']
                logger.log(f"    - Translation successful for {lang_data['display_name']}.")
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
                return translated_text
            else:
                logger.log(f"    - Translation failed for {lang_data['display_name']}: Empty or invalid response from API.")
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
                return None

        except Exception as e:
            logger.log(f"    - An error occurred during translation for {lang_data['display_name']}: {e}")
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None

    def process_image_prompts(self, job_id, lang_id, lang_data, text_to_use, dir_path):
        stage_key = 'stage_img_prompts'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')
        logger.log(f"  - Starting image prompt generation for: {lang_data['display_name']}")
        
        img_prompt_settings = self.settings.get("image_prompt_settings", {})
        prompt_template = img_prompt_settings.get('prompt', '')
        model = img_prompt_settings.get('model', '')
        max_tokens = img_prompt_settings.get('max_tokens', 4096)

        if not model or not prompt_template:
            logger.log(f"  - Skipping image prompt generation for {lang_data['display_name']}: Model or prompt template not configured.")
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        full_prompt = f"{prompt_template}\n\n{text_to_use}"

        try:
            logger.log(f"    - Calling model '{model}' for image prompts with max_tokens={max_tokens}.")
            response = self.api.get_chat_completion(
                model=model,
                messages=[{"role": "user", "content": full_prompt}],
                max_tokens=max_tokens
            )

            if response and response['choices'][0]['message']['content']:
                image_prompts_text = response['choices'][0]['message']['content']
                logger.log(f"    - Image prompt generation successful for {lang_data['display_name']}.")
                
                if dir_path:
                    self.save_image_prompts(dir_path, image_prompts_text)
                
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
            else:
                logger.log(f"    - Image prompt generation failed for {lang_data['display_name']}: Empty or invalid response from API.")
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

        except Exception as e:
            logger.log(f"    - An error occurred during image prompt generation for {lang_data['display_name']}: {e}")
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

    def save_translation(self, dir_path, content):
        try:
            file_path = os.path.join(dir_path, "translation.txt")
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            logger.log(f"    - Translation result saved to: {file_path}")
        except Exception as e:
            logger.log(f"    - Failed to save translation result. Error: {e}")

    def save_image_prompts(self, dir_path, content):
        try:
            file_path = os.path.join(dir_path, "image_prompts.txt")
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            logger.log(f"    - Image prompts saved to: {file_path}")
        except Exception as e:
            logger.log(f"    - Failed to save image prompts. Error: {e}")