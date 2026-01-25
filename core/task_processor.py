import os
import sys
import time
import platform
import threading
import shutil
import collections
import copy
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

from PySide6.QtCore import QObject, Signal, QThreadPool, QElapsedTimer, QSemaphore, Slot

from utils.logger import logger, LogLevel
from utils.settings import settings_manager, template_manager
from utils.translator import translator
from core.notification_manager import notification_manager
from core.history_manager import history_manager

from core.task_state import TaskState

# Mixins
from core.mixins.download_mixin import DownloadMixin
from core.mixins.translation_mixin import TranslationMixin
from core.mixins.subtitle_mixin import SubtitleMixin
from core.mixins.image_mixin import ImageMixin
from core.mixins.video_mixin import VideoMixin
from core.mixins.preview_mixin import PreviewMixin

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

class TaskProcessor(QObject, DownloadMixin, TranslationMixin, SubtitleMixin, ImageMixin, VideoMixin, PreviewMixin):
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
    balance_updated = Signal(str, object) # provider, data


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
        self.image_review_notification_emitted = False

        # --- Semaphores for concurrency control ---
        self.subtitle_semaphore = QSemaphore(1)
        montage_settings = self.settings.get("montage", {})
        max_montage = montage_settings.get("max_concurrent_montages", 1)
        self.montage_semaphore = QSemaphore(max_montage)

        googler_settings = self.settings.get("googler", {})
        max_googler = googler_settings.get("max_threads", 1)
        self.googler_semaphore = QSemaphore(max_googler)
        
        elevenlabs_image_settings = self.settings.get("elevenlabs_image", {})
        max_elevenlabs_image = elevenlabs_image_settings.get("max_threads", 5)
        self.elevenlabs_image_semaphore = QSemaphore(max_elevenlabs_image)
        self.elevenlabs_executor = ThreadPoolExecutor(max_workers=max_elevenlabs_image)
        
        # Restore original executor for Googler/Pollinations
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
        
        # ElevenLabs Unlim concurrency
        self.elevenlabs_unlim_active_count = 0
        self.elevenlabs_unlim_queue = collections.deque()

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

    def _are_subtitles_running(self):
        """Checks if any subtitle or transcription workers are currently active."""
        # Using self.subtitle_semaphore.available() < 1 is not fully reliable if we start allowing >1,
        # but for now Subtitle concurrency is 1.
        # Better: check active_workers list for instances of SubtitleWorker or TranscriptionWorker.
        # Or check if semaphore is acquired.
        from core.workers import SubtitleWorker, TranscriptionWorker
        for worker in self.active_workers:
             if isinstance(worker, (SubtitleWorker, TranscriptionWorker)):
                 return True
        return False

    def _are_montages_running(self):
        """Checks if any montage workers are currently active."""
        from core.workers import MontageWorker
        for worker in self.active_workers:
             if isinstance(worker, MontageWorker):
                 return True
        return False

    def start_processing(self):
        # Reset finish state
        self.is_finished = False
        
        # Load any new tasks from the queue manager
        new_tasks_count = self._load_new_tasks_from_queue()
        
        if not self.task_states and new_tasks_count == 0:
            logger.log("Queue is empty.", level=LogLevel.INFO)
            return

        self.timer.start()
        
        logger.log(f"Starting/Resuming processing. Total tracked tasks: {len(self.task_states)}. New tasks added: {new_tasks_count}", level=LogLevel.INFO)
        
        # Check if we have any work to do
        self._start_pending_tasks()
        self.check_if_all_finished()

    def _load_new_tasks_from_queue(self):
        """
        Fetches tasks from QueueManager and adds them to task_states if not already present.
        Returns the number of new tasks added.
        """
        jobs = self.queue_manager.get_tasks()
        new_tasks_count = 0
        
        for job in jobs:
            for lang_id, lang_data in job['languages'].items():
                
                # Construct a temporary ID to check existence (QueueManager doesn't seem to give unique IDs per lang variant easily available without logic duplication, 
                # but TaskState constructor does. Let's pre-calculate or check by job_id + lang_id)
                # TaskState generates ID as f"{job_id}_{lang_id}"
                
                potential_task_id = f"{job['id']}_{lang_id}"
                
                # improved check: if task exists, we might want to check if it's 'pending' vs 'success' 
                # but for now, we assume if it's in task_states, it's being handled or done.
                if potential_task_id in self.task_states:
                    continue

                # --- Task Initialization Logic (Moved from start_processing) ---
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
                state.start_time = datetime.now()

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
                            
                            # Filter source list
                            valid_exts = ('.jpg', '.jpeg', '.png', '.webp', '.bmp', '.mp4', '.mkv', '.mov', '.avi', '.webm')
                            cleaned_list = [
                                p for p in source_list 
                                if os.path.basename(p).lower().endswith(valid_exts) 
                                and not os.path.basename(p).startswith('.')
                            ]
                            
                            logger.log(f"[{state.task_id}] Processing {len(cleaned_list)} user-provided images...", level=LogLevel.INFO)
                            image_dir = os.path.join(state.dir_path, "images")
                            abs_image_dir = os.path.abspath(image_dir)
                            
                            # 1. Check if we need to preserve local files ("Move to Temp")
                            # Put temp dir OUTSIDE image_dir to survive cleanup
                            temp_source_dir = os.path.join(state.dir_path, "temp_images_processing")
                            files_preserved = False
                            
                            final_source_list = []
                            
                            # Identify which files are local
                            local_indices = []
                            for idx, p in enumerate(cleaned_list):
                                if os.path.dirname(os.path.abspath(p)) == abs_image_dir:
                                    local_indices.append(idx)
                            
                            if local_indices:
                                os.makedirs(temp_source_dir, exist_ok=True)
                                logger.log(f"[{state.task_id}] Preserving {len(local_indices)} local files to {temp_source_dir}", level=LogLevel.INFO)
                                
                                for idx, p in enumerate(cleaned_list):
                                    if idx in local_indices: # It's local
                                        fname = os.path.basename(p)
                                        temp_path = os.path.join(temp_source_dir, fname)
                                        try:
                                            # Move safe
                                            if os.path.exists(p):
                                                shutil.move(p, temp_path)
                                                final_source_list.append(temp_path)
                                                files_preserved = True
                                            elif os.path.exists(temp_path):
                                                final_source_list.append(temp_path) # Already moved?
                                            else:
                                                 logger.log(f"[{state.task_id}] Warning: Source file missing for move: {p}", level=LogLevel.WARNING)
                                        except Exception as e:
                                            logger.log(f"[{state.task_id}] Error moving to temp: {e}", level=LogLevel.ERROR)
                                    else:
                                        final_source_list.append(p)
                            else:
                                final_source_list = cleaned_list

                            # 2. CLEANUP TARGET DIRECTORY
                            # This ensures no stale files (like 11..20 from previous run) remain
                            if os.path.exists(image_dir):
                                try:
                                    shutil.rmtree(image_dir)
                                except Exception as e:
                                    logger.log(f"[{state.task_id}] Warning: Failed to clean image dir {image_dir}: {e}", level=LogLevel.WARNING)
                            
                            os.makedirs(image_dir, exist_ok=True)

                            # 3. POPULATE (Copy back)
                            for i, src_path in enumerate(final_source_list):
                                try:
                                    _, ext = os.path.splitext(src_path)
                                    dest_path = os.path.join(image_dir, f"{i+1}{ext}")
                                    shutil.copy(src_path, dest_path)
                                except Exception as e:
                                    logger.log(f"[{state.task_id}] Error copying {src_path} -> {i+1}: {e}", level=LogLevel.ERROR)

                            # 4. Remove Temp
                            if files_preserved and os.path.exists(temp_source_dir):
                                try:
                                    shutil.rmtree(temp_source_dir)
                                except Exception as e:
                                    logger.log(f"[{state.task_id}] Warning: Failed to remove temp dir: {e}", level=LogLevel.WARNING)

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
                new_tasks_count += 1
                
                if 'stage_subtitles' in state.stages:
                    self.total_subtitle_tasks += 1
                    # CRITICAL: Reset barrier if new subtitle tasks are added, so they correctly trigger montage start logic later
                    self.subtitle_barrier_passed = False
                
                if 'stage_images' in state.stages:
                    self.image_review_notification_emitted = False
                
                if 'stage_montage' in state.stages:
                    self.montage_tasks_ids.add(state.task_id)

                # Announce initial status and sync state
                pre_found = state.lang_data.get('pre_found_files', {})
                for stage_key in state.stages:
                   if stage_key in pre_found:
                       state.status[stage_key] = 'success'
                       self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, 'success')
                   else:
                       self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, 'pending')

        return new_tasks_count

    def _start_pending_tasks(self):
        """Starts processing for any tasks that are in 'pending' state."""
        
        # Check constraints or dependencies if needed
        if self.total_subtitle_tasks == 0:
            self.subtitle_barrier_passed = True
            # logger.log("No subtitle tasks in queue. Barrier passed immediately.", level=LogLevel.INFO) # Removing spam

        started_count = 0
        for task_id, state in self.task_states.items():
            # Find the first stage that is NOT 'success' and NOT 'warning'
            first_incomplete_stage = None
            for stage in state.stages:
                status = state.status.get(stage)
                if status not in ['success', 'warning']:
                    first_incomplete_stage = stage
                    break
            
            if not first_incomplete_stage:
                continue

            # If the first incomplete stage is 'pending', we start it
            if state.status.get(first_incomplete_stage) == 'pending':
                started_count += 1
                if first_incomplete_stage == 'stage_download':
                    self._start_download(task_id)
                elif first_incomplete_stage == 'stage_translation':
                    self._start_translation(task_id)
                    time.sleep(0.1)
                else:
                    if not state.text_for_processing:
                        state.text_for_processing = state.original_text
                    self._on_text_ready(task_id)
        
        if started_count > 0:
            logger.log(f"Started processing for {started_count} pending tasks.", level=LogLevel.INFO)
    
    def _start_worker(self, worker_class, task_id, stage_key, config, on_finish_slot, on_error_slot):
        state = self.task_states[task_id]
        pre_found_files = state.lang_data.get('pre_found_files', {})

        if stage_key in pre_found_files or state.status.get(stage_key) in ['success', 'warning']:
            file_path = pre_found_files.get(stage_key)
            
            should_skip = True
            result = None
            
            # If we don't have a file path but have a successful status, we should still skip
            # but we might need dummy results for some workers.
            
            if worker_class.__name__ == 'VideoGenerationWorker':
                should_skip = False
            elif stage_key == 'stage_preview':
                if worker_class.__name__ == 'PreviewWorker':
                    prompts_file = os.path.join(file_path, "preview_prompts.txt") if file_path and os.path.isdir(file_path) else file_path
                    if prompts_file and os.path.isfile(prompts_file):
                        try:
                            with open(prompts_file, 'r', encoding='utf-8') as f:
                                result = f.read()
                                logger.log(f"[{task_id}] Skipping PreviewWorker: Using existing data.", level=LogLevel.INFO)
                                state.skipped_stages.add(stage_key)
                        except:
                            should_skip = False if state.status.get(stage_key) == 'pending' else True
                    else:
                        # If status is warning/success but file missing, we still skip but with empty result
                        should_skip = True if state.status.get(stage_key) in ['success', 'warning'] else False
                        result = ""

                elif worker_class.__name__ == 'ImageGenerationWorker':
                    search_path = file_path
                    if file_path and os.path.isdir(os.path.join(file_path, "images")):
                        search_path = os.path.join(file_path, "images")
                    
                    if search_path and os.path.isdir(search_path):
                        image_exts = ('.png', '.jpg', '.jpeg', '.webp')
                        images = sorted([os.path.join(search_path, f) for f in os.listdir(search_path) if f.lower().endswith(image_exts)])
                        if images:
                            result = {'paths': images, 'total_prompts': len(images)}
                            logger.log(f"[{task_id}] Skipping ImageGenerationWorker (Preview): Found {len(images)} images.", level=LogLevel.INFO)
                            state.skipped_stages.add(stage_key)
                        else:
                            should_skip = True if state.status.get(stage_key) in ['success', 'warning'] else False
                    else:
                        should_skip = True if state.status.get(stage_key) in ['success', 'warning'] else False
            
            else:
                # Standard stage skipping
                if file_path:
                    logger.log(f"[{task_id}] Skipping stage '{stage_key}' using existing files.", level=LogLevel.INFO)
                state.skipped_stages.add(stage_key)
                
                try:
                    is_custom = stage_key.startswith("custom_")
                    is_text_stage = is_custom or stage_key in ['stage_translation', 'stage_rewrite', 'stage_img_prompts', 'stage_transcription']
                    
                    if file_path and os.path.exists(file_path):
                        if is_text_stage:
                            with open(file_path, 'r', encoding='utf-8') as f:
                                result = f.read()
                        elif stage_key == 'stage_images':
                            valid_exts = ('.jpg', '.jpeg', '.png', '.webp', '.bmp', '.mp4', '.mkv', '.mov', '.avi', '.webm')
                            image_paths = sorted(
                                [os.path.join(file_path, f) for f in os.listdir(file_path) if f.lower().endswith(valid_exts)],
                                key=lambda x: int(os.path.splitext(os.path.basename(x))[0]) if os.path.splitext(os.path.basename(x))[0].isdigit() else -1
                            )
                            result = {'paths': image_paths, 'total_prompts': len(image_paths)}
                        else:
                            result = file_path
                    
                    if is_custom:
                        sn = stage_key.replace("custom_", "")
                        result = {'path': file_path, 'stage_name': sn, 'content': result}
                except:
                    pass # Keep result as None

            if should_skip:
                on_finish_slot(task_id, result)
                return
            else:
                 logger.log(f"[{task_id}] Found partial data for '{stage_key}', but proceeding with {worker_class.__name__} to ensure completeness.", level=LogLevel.INFO)

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
        worker.signals.balance_updated.connect(self.balance_updated.emit)
        worker.signals.progress_log.connect(self._on_worker_progress_log)
        self.threadpool.start(worker)

    @Slot(str, str)
    def _on_worker_progress_log(self, task_id, message):
        state = self.task_states.get(task_id)
        if state:
            self.task_progress_log.emit(state.job_id, message)

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
                self._check_if_image_review_ready()

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
        
        if status == 'success' or status == 'error':
            if stage_key == state.stages[-1]:
                state.end_time = datetime.now()
                history_manager.add_entry(state)

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

    def retry_job(self, job_id):
        """Reset failed stages of a job to 'pending' and restart processing."""
        logger.log(f"Received request to retry job: {job_id}", level=LogLevel.INFO)
        found = False
        for task_id, state in self.task_states.items():
            if state.job_id == job_id:
                found = True
                logger.log(f"[{task_id}] Resetting failed stages for retry...", level=LogLevel.INFO)
                for stage_key in state.stages:
                    if state.status.get(stage_key) == 'error':
                        state.status[stage_key] = 'pending'
                        self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, 'pending')
                    
                # Reset montage failure state for this task
                if task_id in self.failed_montage_tasks_ids:
                    self.failed_montage_tasks_ids.discard(task_id)
                    logger.log(f"[{task_id}] Removed from failed montage list.", level=LogLevel.INFO)
        
        if found:
            self.start_processing()
        else:
            logger.log(f"Job {job_id} not found in active task states.", level=LogLevel.WARNING)

    def check_if_all_finished(self):
        if self.is_finished:
            return

        all_current_done = True
        for state in self.task_states.values():
            for stage_key in state.stages:
                status = state.status.get(stage_key)
                if status == 'pending' or status == 'processing' or status == 'review_required':
                    all_current_done = False
                    break
            if not all_current_done:
                break
        
        if all_current_done:
            # CHECK FOR NEW TASKS!
            new_count = self._load_new_tasks_from_queue()
            if new_count > 0:
                logger.log(f"Found {new_count} new tasks in queue. Continuing processing loop.", level=LogLevel.INFO)
                self._start_pending_tasks()
                return # Do not finish, loop continues

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

        if hasattr(self, 'elevenlabs_executor'):
            try:
                self.elevenlabs_executor.shutdown(wait=False, cancel_futures=True)
                logger.log("ElevenLabs executor shut down successfully.", level=LogLevel.INFO)
            except Exception as e:
                logger.log(f"Error shutting down elevenlabs_executor: {e}", level=LogLevel.WARNING)
        
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
