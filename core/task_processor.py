import os
import sys
import time
import platform
import threading
import shutil
import collections
import copy
from concurrent.futures import ThreadPoolExecutor

from PySide6.QtCore import QObject, Signal, QThreadPool, QElapsedTimer, QSemaphore, Slot

from utils.logger import logger, LogLevel
from utils.settings import settings_manager, template_manager
from utils.translator import translator
from core.notification_manager import notification_manager

from core.task_state import TaskState

# Mixins
from core.mixins.download_mixin import DownloadMixin
from core.mixins.translation_mixin import TranslationMixin
from core.mixins.subtitle_mixin import SubtitleMixin
from core.mixins.image_mixin import ImageMixin
from core.mixins.video_mixin import VideoMixin

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

class TaskProcessor(QObject, DownloadMixin, TranslationMixin, SubtitleMixin, ImageMixin, VideoMixin):
    processing_finished = Signal(str)
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status
    image_generated = Signal(str, str, str, str, str) # job_name, lang_name, image_path, prompt, thumbnail_path
    video_generated = Signal(str, str) # old_image_path, new_video_path
    task_progress_log = Signal(str, str) # job_id, log_message (for card-only logs)
    image_review_required = Signal()
    translation_review_required = Signal(str, str) # task_id, translated_text
    translation_regenerated = Signal(str, str) # task_id, new_text
    rewrite_review_required = Signal(str, str) # task_id, rewritten_text
    rewrite_regenerated = Signal(str, str) # task_id, new_text
    stage_metadata_updated = Signal(str, str, str, str) # job_id, lang_id, stage_key, metadata_text


    def __init__(self, queue_manager):
        super().__init__()
        self.queue_manager = queue_manager
        self.settings = settings_manager
        self.threadpool = QThreadPool()
        # Limit global threads to prevent resource exhaustion
        max_threads = min(os.cpu_count() * 2, 16) 
        self.threadpool.setMaxThreadCount(max_threads)
        logger.log(f"Thread Pool initialized with max count: {self.threadpool.maxThreadCount()}", level=LogLevel.INFO)
        
        self.timer = QElapsedTimer()
        self.voicemaker_voices = self._load_voicemaker_voices()
        
        self.task_states = {}
        self.total_subtitle_tasks = 0
        self.completed_subtitle_tasks = 0
        self.subtitle_barrier_passed = False
        self.is_finished = False
        self.subtitle_lock = threading.Lock() # Used to sync counter access
        self.tasks_awaiting_review = []
        self.montage_tasks_ids = set()
        self.failed_montage_tasks_ids = set()

        # --- Semaphores for concurrency control ---
        self.subtitle_semaphore = QSemaphore(1)
        montage_settings = self.settings.get("montage", {})
        max_montage = montage_settings.get("max_concurrent_montages", 1)
        self.montage_semaphore = QSemaphore(max_montage)

        googler_settings = self.settings.get("googler", {})
        max_googler = googler_settings.get("max_threads", 1)
        self.googler_semaphore = QSemaphore(max_googler)
        self.image_gen_executor = ThreadPoolExecutor(max_workers=max_googler)
        
        max_video = googler_settings.get("max_video_threads", 1)
        self.video_semaphore = QSemaphore(max_video)
        
        # Queues for preventing thread starvation
        self.whisper_queue = collections.deque()
        self.pending_montages = collections.deque()
        
        # Download concurrency
        max_downloads = self.settings.get("max_download_threads", 5)
        self.download_semaphore = QSemaphore(max_downloads)

        # OpenRouter concurrency
        self.openrouter_active_count = 0
        self.openrouter_queue = collections.deque()

        # ElevenLabs concurrency
        self.elevenlabs_active_count = 0
        self.elevenlabs_queue = collections.deque()

        # EdgeTTS concurrency
        self.edgetts_active_count = 0
        self.edgetts_queue = collections.deque()

        # Determine yt-dlp path
        yt_dlp_name = "yt-dlp.exe" if platform.system() == "Windows" else "yt-dlp"
        
        if getattr(sys, 'frozen', False):
            path_in_data = os.path.join(self.settings.base_path, yt_dlp_name)
            if os.path.exists(path_in_data):
                self.yt_dlp_path = path_in_data
            else:
                self.yt_dlp_path = os.path.join(sys._MEIPASS, "assets", yt_dlp_name)
        else:
            self.yt_dlp_path = os.path.join(BASE_PATH, "assets", yt_dlp_name)

        if not os.path.exists(self.yt_dlp_path):
            logger.log(f"Warning: yt-dlp.exe not found at {self.yt_dlp_path}", level=LogLevel.WARNING)

        # Queues for preventing thread starvation
        self.pending_subtitles = collections.deque()
        self.active_workers = set() # Track for Segfault prevention
        
        logger.log(f"Task Processor initialized. Download concurrency: {max_downloads}, Subtitle concurrency: 1, Montage concurrency: {max_montage}, Googler concurrency: {max_googler}, Video concurrency: {max_video}", level=LogLevel.INFO)

    def start_processing(self):
        jobs = self.queue_manager.get_tasks()
        if not jobs:
            logger.log("Queue is empty.", level=LogLevel.INFO)
            return

        self.timer.start()
        self.task_states = {}
        self.total_subtitle_tasks = 0
        self.completed_subtitle_tasks = 0
        self.subtitle_barrier_passed = False
        self.is_finished = False
        self.tasks_awaiting_review = []
        self.montage_tasks_ids = set()
        self.failed_montage_tasks_ids = set()
        
        # 1. Initialize all task states
        for job in jobs:
            for lang_id, lang_data in job['languages'].items():
                
                current_settings = copy.deepcopy(self.settings.settings)

                global_lang_config = current_settings.get("languages_config", {}).get(lang_id, {})
                if global_lang_config:
                    lang_data.update(global_lang_config)

                template_name = lang_data.get('template_name')
                
                if template_name:
                    template_data = template_manager.load_template(template_name)
                    if template_data:
                        for key, value in template_data.items():
                            if isinstance(value, dict) and key in current_settings and isinstance(current_settings[key], dict):
                                current_settings[key].update(value)
                            else:
                                current_settings[key] = value
                        logger.log(f"[{job['name']}_{lang_id}] Applied template: {template_name}", level=LogLevel.INFO)
                        template_lang_cfg = template_data.get('languages_config', {}).get(lang_id, {})
                        if template_lang_cfg:
                            lang_data.update(template_lang_cfg)
                    else:
                        logger.log(f"[{job['name']}_{lang_id}] Template '{template_name}' not found. Using global settings.", level=LogLevel.WARNING)

                base_save_path = current_settings.get('results_path')

                merged_lang_config = current_settings.get("languages_config", {}).get(lang_id, {})
                if merged_lang_config:
                     current_settings['montage']['overlay_effect_path'] = merged_lang_config.get('overlay_effect_path')
                     current_settings['montage']['watermark_path'] = merged_lang_config.get('watermark_path')
                     current_settings['montage']['watermark_size'] = merged_lang_config.get('watermark_size', 20)
                     current_settings['montage']['watermark_position'] = merged_lang_config.get('watermark_position', 8)

                state = TaskState(job, lang_id, lang_data, base_save_path, current_settings)

                if not state.dir_path:
                    logger.log(f"[{state.task_id}] CRITICAL: Directory path could not be created. Aborting this task.", level=LogLevel.ERROR)
                    for stage_key in state.stages:
                        self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, 'error')
                    continue
                
                # --- NEW LOGIC: Pre-process user-provided files ---
                user_files = state.lang_data.get('user_provided_files', {})
                if user_files and state.dir_path:
                    if 'pre_found_files' not in state.lang_data:
                        state.lang_data['pre_found_files'] = {}
                    
                    try:
                        if 'stage_images' in user_files:
                            stage_key = 'stage_images'
                            source_list = user_files[stage_key]
                            logger.log(f"[{state.task_id}] Copying {len(source_list)} user-provided images...", level=LogLevel.INFO)
                            image_dir = os.path.join(state.dir_path, "images")
                            os.makedirs(image_dir, exist_ok=True)
                            for i, src_path in enumerate(source_list):
                                _, ext = os.path.splitext(src_path)
                                dest_path = os.path.join(image_dir, f"{i+1}{ext}")
                                shutil.copy(src_path, dest_path)
                            state.lang_data['pre_found_files'][stage_key] = image_dir
                            
                            if 'stage_img_prompts' in state.stages:
                                prompt_dummy_path = os.path.join(state.dir_path, "image_prompts.txt")
                                with open(prompt_dummy_path, 'w', encoding='utf-8') as f:
                                    f.write("Prompts skipped due to user-provided images.")
                                state.lang_data['pre_found_files']['stage_img_prompts'] = prompt_dummy_path

                        if 'stage_voiceover' in user_files:
                            stage_key = 'stage_voiceover'
                            source_path = user_files[stage_key]
                            logger.log(f"[{state.task_id}] Copying user-provided voiceover...", level=LogLevel.INFO)
                            _, ext = os.path.splitext(source_path)
                            if ext.lower() in ['.mp3', '.wav']:
                                dest_path = os.path.join(state.dir_path, f"voice{ext}")
                                shutil.copy(source_path, dest_path)
                                state.lang_data['pre_found_files'][stage_key] = dest_path
                            else:
                                logger.log(f"[{state.task_id}] Unsupported audio format '{ext}' for user-provided file. Skipping.", level=LogLevel.WARNING)

                    except Exception as e:
                        logger.log(f"[{state.task_id}] Error processing user-provided files: {e}", level=LogLevel.ERROR)
                
                self.task_states[state.task_id] = state
                if 'stage_subtitles' in state.stages:
                    self.total_subtitle_tasks += 1
                if 'stage_montage' in state.stages:
                    self.montage_tasks_ids.add(state.task_id)

        logger.log(f"Starting processing for {len(self.task_states)} language tasks. Montage tasks: {len(self.montage_tasks_ids)}", level=LogLevel.INFO)
        
        if self.total_subtitle_tasks == 0:
            self.subtitle_barrier_passed = True
            logger.log("No subtitle tasks in queue. Barrier passed immediately.", level=LogLevel.INFO)

        for task_id, state in self.task_states.items():
            if 'stage_download' in state.stages:
                self._start_download(task_id)
            elif 'stage_translation' in state.stages:
                self._start_translation(task_id)
                time.sleep(0.5)
            else:
                state.text_for_processing = state.original_text
                self._on_text_ready(task_id)
    
    def _start_worker(self, worker_class, task_id, stage_key, config, on_finish_slot, on_error_slot):
        state = self.task_states[task_id]
        pre_found_files = state.lang_data.get('pre_found_files', {})

        if stage_key in pre_found_files:
            file_path = pre_found_files[stage_key]
            logger.log(f"[{task_id}] Skipping stage '{stage_key}' using existing file: {os.path.basename(file_path)}", level=LogLevel.INFO)
            
            result = None
            try:
                if stage_key in ['stage_translation', 'stage_rewrite', 'stage_img_prompts', 'stage_transcription']:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        result = f.read()
                elif stage_key == 'stage_images':
                    image_paths = sorted(
                        [os.path.join(file_path, f) for f in os.listdir(file_path) if os.path.isfile(os.path.join(file_path, f))],
                        key=lambda x: int(os.path.splitext(os.path.basename(x))[0]) if os.path.splitext(os.path.basename(x))[0].isdigit() else -1
                    )
                    result = {'paths': image_paths, 'total_prompts': len(image_paths)}
                else:
                    result = file_path
                
                on_finish_slot(task_id, result)
            except Exception as e:
                error_msg = f"Failed to process existing file for stage '{stage_key}': {e}"
                on_error_slot(task_id, error_msg)
            return

        self.stage_status_changed.emit(self.task_states[task_id].job_id, self.task_states[task_id].lang_id, stage_key, 'processing')
        worker = worker_class(task_id, config)
        self.active_workers.add(worker)
        
        def wrapped_finish(*args):
             self.active_workers.discard(worker)
             on_finish_slot(*args)
        
        def wrapped_error(*args):
             self.active_workers.discard(worker)
             on_error_slot(*args)

        worker.signals.finished.connect(wrapped_finish)
        worker.signals.error.connect(wrapped_error)
        worker.signals.status_changed.connect(self._on_worker_status_changed)
        worker.signals.video_generated.connect(self.video_generated)
        worker.signals.video_progress.connect(self._on_video_progress)
        worker.signals.metadata_updated.connect(self._on_metadata_updated)
        worker.signals.progress_log.connect(self.task_progress_log) # Forward generic logs if needed
        self.threadpool.start(worker)

    @Slot(str, str, str, str)
    def _on_worker_status_changed(self, task_id, image_path, prompt, thumbnail_path):
        state = self.task_states.get(task_id)
        if state:
            self.image_generated.emit(state.job_name, state.lang_name, image_path, prompt, thumbnail_path)
            state.images_generated_count += 1
            metadata_text = f"{state.images_generated_count}/{state.images_total_count}"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)

    @Slot(str)
    def _on_video_progress(self, task_id):
        state = self.task_states.get(task_id)
        if state:
            state.videos_generated_count += 1
            img_meta = f"{state.images_generated_count}/{state.images_total_count}"
            vid_meta = f"{state.videos_generated_count}/{state.videos_total_count}"
            metadata_text = f"img: {img_meta}, vid: {vid_meta}"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)

    @Slot(str, str, str)
    def _on_metadata_updated(self, task_id, stage_key, text):
        state = self.task_states.get(task_id)
        if state:
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, stage_key, text)

    def _set_stage_status(self, task_id, stage_key, status, error_message=None):
        state = self.task_states.get(task_id)
        if not state: return

        state.status[stage_key] = status
        self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, status)
        
        if status == 'review_required':
             # Notify user about review
             task_name = state.job_name
             if stage_key == 'stage_translation':
                 title = translator.translate('notification_translation_review_title')
                 body = translator.translate('notification_translation_review_body').format(task_name=task_name, lang_id=state.lang_id)
                 notification_manager.send_notification(f"{title}\n{body}")
             elif stage_key == 'stage_images':
                 title = translator.translate('notification_image_review_title')
                 body = translator.translate('notification_image_review_body')
                 notification_manager.send_notification(f"{title}\n{body}")

        if status == 'error':
            if task_id in self.montage_tasks_ids and stage_key != 'stage_montage':
                self.failed_montage_tasks_ids.add(task_id)
                self._check_if_all_are_ready_or_failed()

            stage_names = {
                'stage_translation': 'Translation', 'stage_img_prompts': 'Image prompts generation',
                'stage_voiceover': 'Voiceover generation', 'stage_subtitles': 'Subtitle generation',
                'stage_images': 'Image generation', 'stage_montage': 'Video montage',
                'stage_download': 'Download', 'stage_transcription': 'Transcription',
                'stage_rewrite': 'Rewrite'
            }
            stage_name = stage_names.get(stage_key, stage_key)
            logger.log(f"[{task_id}] {stage_name} failed: {error_message}", level=LogLevel.ERROR)

            title = translator.translate('notification_task_error_title')
            # Assuming 'task_name' is initialized above in review_required block, wait, it's inside `if status == 'review_required'`. 
            # I need to get task_name again or move it up.
            task_name = state.job_name # Get it safely here
            body = translator.translate('notification_task_error_body').format(
                task_name=task_name, lang_id=state.lang_id, stage_name=stage_name, error_message=error_message)
            notification_manager.send_notification(f"{title}\n{body}")
        
        self.check_if_all_finished()

    @Slot(str)
    def _on_image_deleted(self, image_path):
        logger.log(f"Received request to remove image '{image_path}' from task states.", level=LogLevel.INFO)
        for task_id, state in self.task_states.items():
            if state.image_paths and image_path in state.image_paths:
                state.image_paths.remove(image_path)
                logger.log(f"Removed '{os.path.basename(image_path)}' from task {task_id}. Remaining images: {len(state.image_paths)}", level=LogLevel.INFO)

    @Slot(str, str)
    def _on_image_regenerated(self, old_path, new_path):
        logger.log(f"Received request to update image path from '{old_path}' to '{new_path}'.", level=LogLevel.INFO)
        for task_id, state in self.task_states.items():
            if state.image_paths and old_path in state.image_paths:
                index = state.image_paths.index(old_path)
                state.image_paths[index] = new_path
                logger.log(f"Updated path in task {task_id}. New path: '{new_path}'", level=LogLevel.INFO)

    def check_if_all_finished(self):
        if self.is_finished:
            return

        all_done = True
        for state in self.task_states.values():
            for stage_key in state.stages:
                if state.status.get(stage_key) == 'pending' or state.status.get(stage_key) == 'processing':
                    all_done = False
                    break
            if not all_done:
                break
        
        if all_done:
            self.is_finished = True
            elapsed_ms = self.timer.elapsed()
            elapsed_str = time.strftime('%H:%M:%S', time.gmtime(elapsed_ms / 1000))
            logger.log(f"Queue processing finished in {elapsed_str}.", level=LogLevel.SUCCESS)
            
            title = translator.translate('notification_queue_finished_title')
            body = translator.translate('notification_queue_finished_body').format(elapsed_time=elapsed_str)
            notification_manager.send_notification(f"{title}\n{body}")

            self.processing_finished.emit(elapsed_str)
    
    def cleanup(self):
        logger.log("Cleaning up TaskProcessor resources...", level=LogLevel.INFO)
        
        self.is_finished = True # Signal that we are finishing
        
        if hasattr(self, 'image_gen_executor'):
            try:
                self.image_gen_executor.shutdown(wait=False, cancel_futures=True)
                logger.log("Image generation executor shut down successfully.", level=LogLevel.INFO)
            except Exception as e:
                logger.log(f"Error shutting down image_gen_executor: {e}", level=LogLevel.WARNING)
        
        if hasattr(self, 'threadpool'):
            try:
                self.threadpool.clear()
                # Wait for active threads to finish (up to 3000ms) to prevent 0x8001010d
                # This ensures we don't kill threads mid-COM-call or mid-network-request if possible
                if self.threadpool.activeThreadCount() > 0:
                    logger.log(f"Waiting for {self.threadpool.activeThreadCount()} active threads to finish...", level=LogLevel.INFO)
                    self.threadpool.waitForDone(3000)
                logger.log("Thread pool cleared and waited successfully.", level=LogLevel.INFO)
            except Exception as e:
                logger.log(f"Error clearing thread pool: {e}", level=LogLevel.WARNING)
