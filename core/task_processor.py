import os
import sys
import re
import time
import base64
import json
import traceback
import threading
import shutil
import collections
from concurrent.futures import ThreadPoolExecutor
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer, QSemaphore, Slot, QMutex, Qt
from PySide6.QtGui import QPixmap

from utils.logger import logger, LogLevel
from utils.logger import logger, LogLevel
from utils.settings import settings_manager, template_manager
from utils.translator import translator
import copy
from core.statistics_manager import statistics_manager
from api.openrouter import OpenRouterAPI
from api.pollinations import PollinationsAPI
from api.googler import GooglerAPI
from api.elevenlabs import ElevenLabsAPI
from api.voicemaker import VoicemakerAPI
from api.gemini_tts import GeminiTTSAPI
from core.subtitle_engine import SubtitleEngine
from core.montage_engine import MontageEngine

# =================================================================================================================
# region WORKER DEFINITIONS
# =================================================================================================================

class WorkerSignals(QObject):
    finished = Signal(str, object)  # task_id, result
    error = Signal(str, str)        # task_id, error_message
    status_changed = Signal(str, str, str, str) # task_id, image_path, prompt, thumbnail_path
    progress_log = Signal(str, str)  # task_id, log_message (for card-only logs)
    video_generated = Signal(str, str) # old_image_path, new_video_path
    video_progress = Signal(str) # task_id

class BaseWorker(QRunnable):
    def __init__(self, task_id, config):
        super().__init__()
        self.task_id = task_id
        self.config = config
        self.signals = WorkerSignals()

    def run(self):
        try:
            # logger.log(f"[{self.task_id}] Starting worker", level=LogLevel.INFO)
            result = self.do_work()
            self.signals.finished.emit(self.task_id, result)
            # logger.log(f"[{self.task_id}] Finished worker", level=LogLevel.INFO)
        except Exception as e:
            error_msg = f"[{self.task_id}] Error: {e}"
            logger.log(error_msg, level=LogLevel.ERROR)
            logger.log(f"[{self.task_id}] Traceback:\n{traceback.format_exc()}", level=LogLevel.ERROR)
            self.signals.error.emit(self.task_id, str(e))

    def do_work(self):
        raise NotImplementedError

# --- Specific Workers ---

class TranslationWorker(BaseWorker):
    def do_work(self):
        api = OpenRouterAPI()
        lang_config = self.config['lang_config']
        model = lang_config.get('model', 'unknown')
        temp = lang_config.get('temperature', 0.7)
        max_tokens = lang_config.get('max_tokens', 4096)
        
        logger.log(f"[{self.task_id}] [{model}] Starting translation (temp: {temp}, max_tokens: {max_tokens})", level=LogLevel.INFO)
        
        full_prompt = f"{lang_config.get('prompt', '')}\n\n{self.config['text']}"
        response = api.get_chat_completion(
            model=model,
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=max_tokens,
            temperature=temp
        )
        if response and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            logger.log(f"[{self.task_id}] [{model}] Translation completed", level=LogLevel.SUCCESS)
            return result
        else:
            raise Exception(f"Empty or invalid response from translation API for model '{model}'. Response: {response}")

class ImagePromptWorker(BaseWorker):
    def do_work(self):
        api = OpenRouterAPI()
        img_prompt_settings = self.config['img_prompt_settings']
        model = img_prompt_settings.get('model', 'unknown')
        temp = img_prompt_settings.get('temperature', 0.7)
        max_tokens = img_prompt_settings.get('max_tokens', 4096)
        
        logger.log(f"[{self.task_id}] [{model}] Starting image prompts generation (temp: {temp}, max_tokens: {max_tokens})", level=LogLevel.INFO)
        
        full_prompt = f"{img_prompt_settings.get('prompt', '')}\n\n{self.config['text']}"
        response = api.get_chat_completion(
            model=model,
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=max_tokens,
            temperature=temp
        )
        if response and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            # Count prompts
            prompts = re.findall(r"^\d+\.\s*(.*)", result, re.MULTILINE)
            prompts_count = len(prompts)
            logger.log(f"[{self.task_id}] [{model}] Image prompts generated ({prompts_count} prompts)", level=LogLevel.SUCCESS)
            return result
        else:
            raise Exception("Empty or invalid response from image prompt API.")

class VoiceoverWorker(BaseWorker):
    def do_work(self):
        text = self.config['text']
        dir_path = self.config['dir_path']
        lang_config = self.config['lang_config']
        tts_provider = lang_config.get('tts_provider', 'ElevenLabs')
        
        logger.log(f"[{self.task_id}] [{tts_provider}] Starting voiceover generation", level=LogLevel.INFO)

        if tts_provider == 'VoiceMaker':
            api_key = self.config.get('voicemaker_api_key')
            api = VoicemakerAPI(api_key=api_key)
            voice_id = lang_config.get('voicemaker_voice_id')
            language_code = self.config['voicemaker_lang_code']
            audio_content, status = api.generate_audio(text, voice_id, language_code, temp_dir=dir_path)
            if status == 'success' and audio_content:
                return self.save_audio(audio_content, "voice.mp3")
            else:
                raise Exception(f"VoiceMaker generation failed: {status}")

        elif tts_provider == 'GeminiTTS':
            api = GeminiTTSAPI()
            task_id, status = api.create_task(text, lang_config.get('gemini_voice', 'Puck'), lang_config.get('gemini_tone', ''))
            if status != 'connected' or not task_id:
                raise Exception("Failed to create GeminiTTS task.")
            
            for _ in range(60): # 5 min timeout
                task_status, status = api.get_task_status(task_id)
                if status != 'connected': raise Exception("Failed to get GeminiTTS task status.")
                if task_status == 'completed':
                    context = f"Task: {self.config['job_name']}, Lang: {self.config['lang_name']}"
                    audio_content, status = api.download_audio(task_id, context_info=context)
                    if status == 'connected' and audio_content:
                        return self.save_audio(audio_content, "voice.wav")
                    else:
                        raise Exception("Failed to download GeminiTTS audio.")
                elif task_status == 'failed':
                    raise Exception("GeminiTTS task failed on server side.")
                time.sleep(5)
            raise Exception("Timeout waiting for GeminiTTS result.")

        else: # ElevenLabs
            api_key = self.config.get('elevenlabs_api_key')
            api = ElevenLabsAPI(api_key=api_key)
            
            # Retry logic for task creation
            task_id = None
            last_error = "Unknown error"
            
            for attempt in range(3):
                try:
                    task_id, status = api.create_task(text, lang_config['elevenlabs_template_uuid'])
                    if status == 'connected' and task_id:
                        break
                    else:
                        last_error = "Failed to obtain valid task_id"
                        logger.log(f"[{self.task_id}] Attempt {attempt+1}/3 failed to create ElevenLabs task. Retrying...", level=LogLevel.WARNING)
                except Exception as e:
                    last_error = str(e)
                    logger.log(f"[{self.task_id}] Attempt {attempt+1}/3 raised exception: {e}", level=LogLevel.WARNING)
                time.sleep(5)
                
            if not task_id:
                raise Exception(f"Failed to create ElevenLabs task after 3 attempts. Last error: {last_error}")
            
            while True:
                task_status, status = api.get_task_status(task_id)
                # Retry status check within the loop if network blips
                if status != 'connected': 
                    logger.log(f"[{self.task_id}] Weak connection getting status for {task_id}, retrying...", level=LogLevel.WARNING)
                    time.sleep(5)
                    continue

                if task_status in ['ending', 'ending_processed']:
                    audio_content, status = api.get_task_result(task_id)
                    if status == 'connected' and audio_content:
                        return self.save_audio(audio_content, "voice.mp3")
                    elif status == 'not_ready': 
                        time.sleep(10)
                        continue
                    else: 
                        raise Exception("Failed to download ElevenLabs audio.")
                elif task_status in ['error', 'error_handled']:
                    raise Exception("ElevenLabs task processing resulted in an error.")
                
                time.sleep(10)

    def save_audio(self, content, filename):
        path = os.path.join(self.config['dir_path'], filename)
        with open(path, 'wb') as f: f.write(content)
        tts_provider = self.config['lang_config'].get('tts_provider', 'ElevenLabs')
        logger.log(f"[{self.task_id}] [{tts_provider}] Voiceover saved", level=LogLevel.SUCCESS)
        return path

class SubtitleWorker(BaseWorker):
    def do_work(self):
        whisper_type = self.config['sub_settings'].get('whisper_type', 'amd')

        if whisper_type == 'amd':
            whisper_label = 'amd-fork'
        elif whisper_type == 'standard':
            whisper_label = 'whisper'
        else:
            whisper_label = 'assemblyai'
        
        logger.log(f"[{self.task_id}] [{whisper_label}] Starting subtitle generation", level=LogLevel.INFO)
        engine = SubtitleEngine(self.config['whisper_exe'], self.config['whisper_model_path'])
        output_filename = os.path.splitext(os.path.basename(self.config['audio_path']))[0] + ".ass"
        output_path = os.path.join(self.config['dir_path'], output_filename)
        engine.generate_ass(self.config['audio_path'], output_path, self.config['sub_settings'], language=self.config['lang_code'])
        logger.log(f"[{self.task_id}] [{whisper_label}] Subtitles saved", level=LogLevel.SUCCESS)
        return output_path


class CustomStageWorker(BaseWorker):
    def do_work(self):
        stage_name = self.config['stage_name']
        
        api = OpenRouterAPI()
        
        prompt = self.config['prompt']
        text = self.config['text']
        model = self.config.get('model', 'google/gemini-2.0-flash-exp:free') 
        max_tokens = self.config.get('max_tokens', 4096)
        temperature = self.config.get('temperature', 0.7)
        
        logger.log(f"[{self.task_id}] [Custom Stage: {stage_name}] Starting processing (model: {model}, tokens: {max_tokens}, temp: {temperature})...", level=LogLevel.INFO)
        
        full_prompt = f"{prompt}\n\n{text}"
        response = api.get_chat_completion(
            model=model,
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=max_tokens,
            temperature=temperature
        )
        
        if response and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            
            # Save to file
            safe_name = "".join(c for c in stage_name if c.isalnum() or c in (' ', '_')).rstrip()
            filename = f"{safe_name}.txt"
            output_path = os.path.join(self.config['dir_path'], filename)
            
            with open(output_path, "w", encoding="utf-8") as f:
                f.write(result)
                
            logger.log(f"[{self.task_id}] [Custom Stage: {stage_name}] Completed and saved to {filename}", level=LogLevel.SUCCESS)
            return {'path': output_path, 'stage_name': stage_name}
        else:
            raise Exception("Empty or invalid response from API.")


class ImageGenerationWorker(BaseWorker):
    def do_work(self):
        from concurrent.futures import as_completed
        
        executor = self.config.get('executor')
        if not executor:
            raise Exception("Executor not provided to ImageGenerationWorker")

        prompts_text = self.config['prompts_text']
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        if not prompts:
            logger.log(f"[{self.task_id}] No numbered prompts found. Parsing by lines.", level=LogLevel.INFO)
            prompts = [line.strip() for line in prompts_text.split('\n') if line.strip()]

        if not prompts:
            raise Exception("No prompts found in the generated text.")

        provider = self.config['provider']
        images_dir = os.path.join(self.config['dir_path'], "images")
        os.makedirs(images_dir, exist_ok=True)
        
        if provider == 'googler':
            file_extension = 'jpg'
            shared_api = GooglerAPI()
            api_kwargs = self.config['api_kwargs']
        else: # pollinations
            file_extension = 'png'
            shared_api = PollinationsAPI()
            api_kwargs = {}
        
        service_name = provider.capitalize()
        generated_paths = [None] * len(prompts)
        
        def generate_single_image(index, prompt):
            """Generate a single image and return its data"""
            try:
                logger.log(f"[{self.task_id}] [{service_name}] Generating image {index + 1}/{len(prompts)}", level=LogLevel.INFO)
                image_data = shared_api.generate_image(prompt, **api_kwargs)

                if not image_data:
                    logger.log(f"[{self.task_id}] [{service_name}] Failed to generate image {index + 1}/{len(prompts)} (no data)", level=LogLevel.WARNING)
                    return None
                
                return (index, image_data, prompt)
            except Exception as e:
                logger.log(f"[{self.task_id}] [{service_name}] Error generating image {index + 1}: {e}", level=LogLevel.ERROR)
                return None
        
        max_workers = self.config.get('max_threads', 8)
        prompts_iterator = iter(enumerate(prompts))
        futures = {}

        # Start with an initial batch of workers
        for _ in range(max_workers):
            try:
                i, prompt = next(prompts_iterator)
                future = executor.submit(generate_single_image, i, prompt)
                futures[future] = i
                time.sleep(0.5)  # Respect the initial submission delay
            except StopIteration:
                break  # No more prompts to start with

        while futures:
            # Wait for the next future to complete
            done_future = next(as_completed(futures))
            time.sleep(0.1) # Add delay to space out result processing
            
            index = futures.pop(done_future)
            result = done_future.result()

            if result:
                index_from_result, image_data, prompt_from_result = result
                
                image_path = os.path.join(images_dir, f"{index_from_result + 1}.{file_extension}")
                data_to_write = image_data

                if provider == 'googler' and isinstance(image_data, str):
                    try:
                        data_to_write = base64.b64decode(image_data.split(",", 1)[1] if "," in image_data else image_data)
                    except Exception as e:
                        logger.log(f"[{self.task_id}] [{service_name}] Error decoding base64 for image {index_from_result + 1}: {e}", level=LogLevel.ERROR)
                        # Continue to the submission part to keep the worker count stable
                    else:
                        try:
                            time.sleep(0.2)
                            with open(image_path, 'wb') as f:
                                f.write(data_to_write)
                            
                            logger.log(f"[{self.task_id}] [{service_name}] Image {index_from_result + 1}/{len(prompts)} saved", level=LogLevel.SUCCESS)
                            generated_paths[index_from_result] = image_path
                            
                            # --- Thumbnail Generation (Disabled by user request to avoid duplicates) ---
                            thumbnail_path = image_path
                            # --- End Thumbnail Generation ---
                            
                            time.sleep(0.1)
                            self.signals.status_changed.emit(self.task_id, image_path, prompt_from_result, thumbnail_path)
                        except IOError as e:
                            logger.log(f"[{self.task_id}] [{service_name}] Error saving image {index_from_result + 1} to {image_path}: {e}", level=LogLevel.ERROR)
            
            # After processing a completed task, try to submit a new one
            try:
                i, prompt = next(prompts_iterator)
                future = executor.submit(generate_single_image, i, prompt)
                futures[future] = i
                time.sleep(0.5) # Delay for the new submission
            except StopIteration:
                # No more prompts to submit, the while loop will continue until all in-flight futures are done
                pass

        final_paths = [path for path in generated_paths if path is not None]

        if len(final_paths) == 0 and len(prompts) > 0:
            raise Exception("Failed to generate any images.")

        return {'paths': final_paths, 'total_prompts': len(prompts)}

class VideoGenerationWorker(BaseWorker):
    def do_work(self):
        from concurrent.futures import ThreadPoolExecutor, as_completed
        
        image_paths_to_animate = self.config['image_paths']
        video_prompt = self.config['prompt']
        video_semaphore = self.config['video_semaphore']
        
        api = GooglerAPI()
        
        generated_videos = [None] * len(image_paths_to_animate)

        def generate_single_video(index, image_path):
            max_retries = 3
            for attempt in range(max_retries):
                try:
                    video_semaphore.acquire()
                    logger_msg = f"[{self.task_id}] [Googler Video] Animating image {index + 1}/{len(image_paths_to_animate)}: {os.path.basename(image_path)}"
                    if max_retries > 1:
                        logger_msg += f" (Attempt {attempt + 1}/{max_retries})"
                    logger.log(logger_msg, level=LogLevel.INFO)

                    video_data = api.generate_video(image_path, video_prompt)
                    
                    if not video_data:
                        raise Exception("API returned no data.")
                    
                    base_name = os.path.splitext(image_path)[0]
                    video_path = f"{base_name}.mp4"

                    try:
                        data_to_write = base64.b64decode(video_data.split(",", 1)[1] if "," in video_data else video_data)
                        with open(video_path, 'wb') as f:
                            f.write(data_to_write)
                        
                        self.signals.video_generated.emit(image_path, video_path)
                        self.signals.video_progress.emit(self.task_id)
                        
                        try:
                            os.remove(image_path)
                        except OSError as e:
                            logger.log(f"[{self.task_id}] [Googler Video] Could not delete original image {image_path}: {e}", level=LogLevel.WARNING)

                        logger.log(f"[{self.task_id}] [Googler Video] Video {index + 1}/{len(image_paths_to_animate)} saved to {video_path}", level=LogLevel.SUCCESS)
                        return (index, video_path)

                    except Exception as e:
                        raise Exception(f"Error saving video for {os.path.basename(image_path)}: {e}")

                except Exception as e:
                    logger.log(f"[{self.task_id}] [Googler Video] Attempt {attempt + 1} failed for {os.path.basename(image_path)}: {e}", level=LogLevel.WARNING)
                    if attempt < max_retries - 1:
                        logger.log(f"Retrying in 5 seconds...", level=LogLevel.INFO)
                        time.sleep(5)
                    else:
                        logger.log(f"[{self.task_id}] [Googler Video] All retries failed for {os.path.basename(image_path)}.", level=LogLevel.ERROR)
                        # Return None from inside the loop on final failure
                finally:
                    video_semaphore.release()
            return None # This is reached if all retries fail

        with ThreadPoolExecutor(max_workers=self.config.get('max_threads', 1)) as executor:
            future_to_index = {executor.submit(generate_single_video, i, path): i for i, path in enumerate(image_paths_to_animate)}
            
            for future in as_completed(future_to_index):
                result = future.result()
                if result:
                    index, video_path = result
                    generated_videos[index] = video_path
        
        final_paths = [path for path in generated_videos if path is not None]
        
        if len(final_paths) != len(image_paths_to_animate):
            logger.log(f"[{self.task_id}] [Googler Video] Warning: only {len(final_paths)} of {len(image_paths_to_animate)} videos were generated.", level=LogLevel.WARNING)

        return {'paths': final_paths}

class MontageWorker(BaseWorker):
    def do_work(self):
        statistics_manager.record_video_creation()
        import time
        
        start_time = time.time()
        engine = MontageEngine()
        # Pass task_id and progress callback to the engine
        self.config['task_id'] = self.task_id
        self.config['progress_callback'] = lambda msg: self.signals.progress_log.emit(self.task_id, msg)
        self.config['start_time'] = start_time  # Pass start time for logging
        engine.create_video(**self.config)
        
        elapsed = time.time() - start_time
        elapsed_str = time.strftime('%M:%S', time.gmtime(elapsed))
        logger.log(f"[{self.task_id}] [FFmpeg] Video montage completed (duration: {elapsed_str})", level=LogLevel.SUCCESS)
        
        return self.config['output_path']

# endregion
# =================================================================================================================
# region TASK PROCESSOR
# =================================================================================================================

class TaskState:
    """Holds the state and data for a single language within a single job."""
    def __init__(self, job, lang_id, lang_data, base_save_path, settings):
        self.job_id = job['id']
        self.lang_id = lang_id
        self.task_id = f"{self.job_id}_{self.lang_id}"

        self.job_name = job['name']
        self.lang_name = lang_data['display_name']
        self.stages = lang_data['stages']
        self.original_text = job['text']
        self.lang_data = lang_data
        self.settings = settings

        self.dir_path = self._get_save_path(base_save_path, self.job_name, self.lang_name)

        self.text_for_processing = None
        self.image_prompts = None
        self.audio_path = None
        self.subtitle_path = None
        self.image_paths = None
        self.final_video_path = None

        self.status = {stage: 'pending' for stage in self.stages}
        self.translation_review_dialog_shown = False
        self.prompt_regeneration_attempts = 0
        self.image_gen_status = 'pending'
        
        # Metadata counters
        self.images_generated_count = 0
        self.images_total_count = 0
        self.videos_generated_count = 0
        self.videos_total_count = 0

    def _get_save_path(self, base_path, job_name, lang_name):
        if not base_path: return None
        try:
            safe_job_name = ("".join(c for c in job_name if c.isalnum() or c in (' ', '_')).rstrip())[:100]
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
            dir_path = os.path.join(base_path, safe_job_name, safe_lang_name)
            os.makedirs(dir_path, exist_ok=True)
            return dir_path
        except Exception as e:
            logger.log(f"[{self.task_id}] Failed to create save directory {dir_path}. Error: {e}", level=LogLevel.ERROR)
            return None

# Determine the base path for resources, accommodating PyInstaller
if getattr(sys, 'frozen', False):
    BASE_PATH = sys._MEIPASS
else:
    BASE_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

class TaskProcessor(QObject):
    processing_finished = Signal(str)
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status
    image_generated = Signal(str, str, str, str, str) # job_name, lang_name, image_path, prompt, thumbnail_path
    video_generated = Signal(str, str) # old_image_path, new_video_path
    task_progress_log = Signal(str, str) # job_id, log_message (for card-only logs)
    image_review_required = Signal()
    translation_review_required = Signal(str, str) # task_id, translated_text
    translation_regenerated = Signal(str, str) # task_id, new_text
    stage_metadata_updated = Signal(str, str, str, str) # job_id, lang_id, stage_key, metadata_text


    def __init__(self, queue_manager):
        super().__init__()
        self.queue_manager = queue_manager
        self.settings = settings_manager
        self.threadpool = QThreadPool()
        # Limit global threads to prevent resource exhaustion (e.g., sockets, file handles)
        # 35 concurrent tasks might be too much for the system or API limits logic not handled elsewhere.
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
        self.pending_subtitles = collections.deque()
        self.pending_montages = collections.deque()
        
        logger.log(f"Task Processor initialized. Subtitle concurrency: 1, Montage concurrency: {max_montage}, Googler concurrency: {max_googler}, Video concurrency: {max_video}", level=LogLevel.INFO)

    def _load_voicemaker_voices(self):
        try:
            path = os.path.join(BASE_PATH, "assets", "voicemaker_voices.json")
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            logger.log(f"Error loading voicemaker voices: {e}", level=LogLevel.ERROR)
            return []

    def _get_voicemaker_language_code(self, voice_id):
        for lang_data in self.voicemaker_voices:
            if voice_id in lang_data.get("Voices", []):
                return lang_data.get("LanguageCode")
        return "en-US"
    
    def _get_audio_duration(self, audio_path):
        """Get audio duration in seconds using mutagen for accuracy."""
        from mutagen.mp3 import MP3
        from mutagen.wave import WAVE
        
        try:
            ext = os.path.splitext(audio_path)[1].lower()
            if ext == '.mp3':
                audio = MP3(audio_path)
                return audio.info.length
            elif ext == '.wav':
                audio = WAVE(audio_path)
                return audio.info.length
            else:
                logger.log(f"Unsupported audio format for duration check: {ext}. Falling back to estimation.", level=LogLevel.WARNING)
                # Fallback for other types if any
                file_size = os.path.getsize(audio_path)
                duration = (file_size * 8) / (128 * 1000) # Assume 128 kbps
                return duration
        except Exception as e:
            logger.log(f"Could not get audio duration for {audio_path}: {e}. Falling back to estimation.", level=LogLevel.ERROR)
            # Fallback on any error
            file_size = os.path.getsize(audio_path)
            duration = (file_size * 8) / (128 * 1000) # Assume 128 kbps
            return duration

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
                
                # --- Prepare Settings for this Task (Global + Template) ---
                current_settings = copy.deepcopy(self.settings.settings)
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
                    else:
                        logger.log(f"[{job['name']}_{lang_id}] Template '{template_name}' not found. Using global settings.", level=LogLevel.WARNING)

                # Get the save path FROM THE MERGED SETTINGS for this specific task
                base_save_path = current_settings.get('results_path')

                state = TaskState(job, lang_id, lang_data, base_save_path, current_settings)
                
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

        # Get the global language config once, as it's not part of templates
        all_languages_config = self.settings.get("languages_config", {})

        for task_id, state in self.task_states.items():
            if 'stage_translation' in state.stages:
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
                if stage_key == 'stage_translation' or stage_key == 'stage_img_prompts':
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
        worker.signals.finished.connect(on_finish_slot)
        worker.signals.error.connect(on_error_slot)
        worker.signals.status_changed.connect(self._on_worker_status_changed) # For gallery updates
        worker.signals.video_generated.connect(self.video_generated) # For gallery updates
        worker.signals.video_progress.connect(self._on_video_progress)
        self.threadpool.start(worker)

    @Slot(str, str, str, str)
    def _on_worker_status_changed(self, task_id, image_path, prompt, thumbnail_path):
        """Used for intermediate status updates, like an image being generated."""
        state = self.task_states.get(task_id)
        if state:
            self.image_generated.emit(state.job_name, state.lang_name, image_path, prompt, thumbnail_path)
            
            # Update image counter and emit metadata
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

    def _set_stage_status(self, task_id, stage_key, status, error_message=None):
        state = self.task_states.get(task_id)
        if not state: return

        state.status[stage_key] = status
        self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, status)
        
        if status == 'error':
            if task_id in self.montage_tasks_ids and stage_key != 'stage_montage':
                self.failed_montage_tasks_ids.add(task_id)
                self._check_if_all_are_ready_or_failed()

            stage_names = {
                'stage_translation': 'Translation', 'stage_img_prompts': 'Image prompts generation',
                'stage_voiceover': 'Voiceover generation', 'stage_subtitles': 'Subtitle generation',
                'stage_images': 'Image generation', 'stage_montage': 'Video montage'
            }
            stage_name = stage_names.get(stage_key, stage_key)
            logger.log(f"[{task_id}] {stage_name} failed: {error_message}", level=LogLevel.ERROR)
        
        self.check_if_all_finished()

    # --- Pipeline Logic ---

    def _start_translation(self, task_id):
        state = self.task_states[task_id]
        lang_config = state.settings.get("languages_config", {}).get(state.lang_id, {})
        config = {
            'text': state.original_text,
            'lang_config': lang_config
        }
        self._start_worker(TranslationWorker, task_id, 'stage_translation', config, self._on_translation_finished, self._on_translation_error)

    @Slot(str, object)
    def _on_translation_finished(self, task_id, translated_text):
        state = self.task_states[task_id]
        state.text_for_processing = translated_text
        if state.dir_path:
            # Save the original translation before review
            with open(os.path.join(state.dir_path, "translation.txt"), 'w', encoding='utf-8') as f:
                f.write(translated_text)

        # Update metadata with character count
        char_count = len(translated_text)
        metadata_text = f"{char_count} {translator.translate('characters_count')}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_translation', metadata_text)

        is_review_enabled = state.settings.get('translation_review_enabled', False)

        if is_review_enabled:
            self._set_stage_status(task_id, 'stage_translation', 'success')
            self.translation_regenerated.emit(task_id, translated_text) # Update dialog if open

            if not state.translation_review_dialog_shown:
                state.translation_review_dialog_shown = True
                self.translation_review_required.emit(task_id, translated_text)
            # If dialog was already shown, we just updated the text. We do nothing else.
            # The execution flow is paused, waiting for the user to close the dialog.
        else:
            # No review, proceed as normal
            self._set_stage_status(task_id, 'stage_translation', 'success')
            self._on_text_ready(task_id)


    @Slot(str, str)
    def _on_translation_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_translation', 'error', error)
        # If translation fails, subsequent stages that depend on it must also fail.
        state = self.task_states[task_id]
        if 'stage_voiceover' in state.stages:
            self._set_stage_status(task_id, 'stage_voiceover', 'error', "Dependency (Translation) failed")
        if 'stage_subtitles' in state.stages:
            self._set_stage_status(task_id, 'stage_subtitles', 'error', "Dependency (Translation) failed")
            self._increment_subtitle_counter() # Still need to pass the barrier
        if 'stage_img_prompts' in state.stages:
            self._set_stage_status(task_id, 'stage_img_prompts', 'error', "Dependency (Translation) failed")

        # Check if we can start montages for other tasks
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    def regenerate_translation(self, task_id):
        logger.log(f"[{task_id}] User requested translation regeneration.", level=LogLevel.INFO)
        self._start_translation(task_id)

    def _on_text_ready(self, task_id):
        state = self.task_states[task_id]
        
        # If translation was not used, emit metadata for original text
        if 'stage_translation' not in state.stages and state.text_for_processing:
            char_count = len(state.text_for_processing)
            metadata_text = f"{char_count} {translator.translate('characters_count')}"
            # For original text, we use a special key 'original_text'
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'original_text', metadata_text)
        
        if 'stage_img_prompts' in state.stages:
            self._start_image_prompts(task_id)
        if 'stage_voiceover' in state.stages:
            self._start_voiceover(task_id)
        if 'stage_img_prompts' not in state.stages and 'stage_images' in state.stages:
             self._start_image_generation(task_id)
        if 'stage_img_prompts' not in state.stages and 'stage_voiceover' not in state.stages and 'stage_images' not in state.stages:
            self.check_if_all_finished()

        # --- Custom Stages (Parallel) ---
        custom_stages = self.settings.get("custom_stages", [])
        if custom_stages:
            for stage in custom_stages:
                stage_name = stage.get("name")
                prompt = stage.get("prompt")
                model = stage.get("model")
                max_tokens = stage.get("max_tokens")
                temperature = stage.get("temperature")
                
                # Check if this stage was selected for this job
                stage_key = f"custom_{stage_name}"
                if stage_key in state.stages:
                    if stage_name and prompt:
                        self._start_custom_stage(task_id, stage_name, prompt, model, max_tokens, temperature)

    def _start_custom_stage(self, task_id, stage_name, prompt, model=None, max_tokens=None, temperature=None):
        state = self.task_states[task_id]
        
        # Fallback to defaults if not specified or empty
        # If model is not in custom stage settings, use the image prompt model or default
        if not model:
            model = state.settings.get("image_prompt_settings", {}).get("model", "google/gemini-2.0-flash-exp:free")
        if not max_tokens:
             max_tokens = 4096
        if temperature is None:
            temperature = 0.7

        config = {
            'text': state.text_for_processing,
            'dir_path': state.dir_path,
            'stage_name': stage_name,
            'prompt': prompt,
            'model': model,
            'max_tokens': int(max_tokens),
            'temperature': float(temperature)
        }
        
        # We use a unique stage key for each custom stage to track status if we wanted to block, 
        # but the requirement is "parallel and shouldn't block anything".
        # However, it's good to track them in logs.
        # We won't add them to state.stages so they don't prevent "Job Finished" status if they are slow,
        # UNLESS we want them to be part of the job completion.
        # The user said: "result simply saved... process shouldn't disturb anything absolutely"
        # So we might treat them as fire-and-forget from the main pipeline's perspective,
        # BUT usually users want to know when EVERYTHING is done.
        # Let's run them as workers but not add them to the critical path for "voiceover finished" signals etc.
        # But for "Job Finished" dialog, maybe we should wait? 
        # The user said "parallel... result simply saved".
        # Let's just fire them. If the user closes the app, they might be killed.
        # Ideally we should verify.
        
        # Define callbacks with closures to capture stage_name safely
        self._start_worker(CustomStageWorker, task_id, f"custom_{stage_name}", config, 
                           self._on_custom_stage_finished, self._on_custom_stage_error_slot)

    @Slot(str, object)
    def _on_custom_stage_finished(self, task_id, result_data):
        stage_name = result_data.get('stage_name')
        # output_path = result_data.get('path') 
        
        stage_key = f"custom_{stage_name}"
        logger.log(f"[{task_id}] Custom stage '{stage_name}' finished (UI update trigger).", level=LogLevel.INFO)
        self._set_stage_status(task_id, stage_key, 'success')
        # We also need to check if everything is finished, as this might be the last thing running
        self.check_if_all_finished()

    @Slot(str, str)
    def _on_custom_stage_error_slot(self, task_id, error):
         # We need to find which stage failed. 
         # Since error signal is generic (task_id, error_msg), we might not know the stage name easily 
         # if multiple custom stages run for the same task_id (which they do).
         # BUT, BaseWorker emits error with task_id. 
         # Wait, CustomStageWorker is started with task_id.
         # If multiple custom stages run for the same language task, they share task_id!
         # This is a problem in the original design too?
         # No, the original design used a closure `on_custom_error` which captured `stage_name`.
         # To fix this properly without closures, we need to pass stage_name in the error too, 
         # OR rely on the fact that we might parse it from the worker config if we had access, but we don't here.
         # Actually, the simplest way to keep error handling working with stage names 
         # without closures is to arguably keep the closure for error or create a partial.
         # BUT, using partial matches the Slot requirement better than a raw closure?
         # Qt Slots can't easily take extra args unless signal provides them.
         # 
         # Let's look at how I replaced _start_custom_stage. 
         # I need to handle the error case too.
         
         # Re-reading the plan: "Modify CustomStageWorker.do_work ... Modify TaskProcessor._on_custom_stage_finished ... Modify TaskProcessor._start_custom_stage"
         # I didn't explicitly detail error handling in the plan, but I need to make sure I don't break it.
         # 
         # Issue: BaseWorker.error signal is Signal(str, str) -> (task_id, error_message).
         # It doesn't include stage_name. 
         # If I use a single slot `_on_custom_stage_error_slot(self, task_id, error)`, I don't know which stage failed if there are multiple.
         # 
         # However, the user request specifically complained about SUCCESS status not updating (green circle).
         # The closure for success `on_custom_finished` was `lambda tid, res: self._on_custom_stage_finished(tid, res, stage_name)`.
         # This closure was being called from a worker thread (likely), via the signal connection? 
         # Qt signals related across threads are queued. 
         # If `finish` signal is connected to a lambda, the lambda runs. 
         # BUT lambdas are not Slots. If the sender is in a different thread, Qt queues the call. 
         # When the slot is a lambda, does it execute in the receiver's thread? 
         # Standard PySide/PyQt behavior: if receiver is QObject (self), auto-connection tries to run in receiver thread.
         # But a lambda isn't a QObject method so it might default to direct connection or be problematic.
         # 
         # Converting to a proper Slot on TaskProcessor (which is in the main thread) guarantees execution in the main thread.
         # 
         # FOR ERROR HANDLING:
         # I will define a specific wrapper/adaptor or use `sender()`?
         # `sender()` in `_on_custom_stage_error` would give the Worker object.
         # The Worker object has `self.config` which has `stage_name`.
         # PERFECT.
         
        worker = self.sender()
        stage_name = "unknown"
        if worker and hasattr(worker, 'config') and 'stage_name' in worker.config:
            stage_name = worker.config['stage_name']
        
        stage_key = f"custom_{stage_name}"
        logger.log(f"[{task_id}] [Custom Stage: {stage_name}] Failed: {error}", level=LogLevel.ERROR)
        self._set_stage_status(task_id, stage_key, 'error', error)
        self.check_if_all_finished()


    def _start_image_prompts(self, task_id):
        state = self.task_states[task_id]
        config = {
            'text': state.text_for_processing,
            'img_prompt_settings': state.settings.get("image_prompt_settings", {})
        }
        self._start_worker(ImagePromptWorker, task_id, 'stage_img_prompts', config, self._on_img_prompts_finished, self._on_img_prompts_error)

# ... (lines omitted)

    def _start_voiceover(self, task_id):
        state = self.task_states[task_id]
        lang_config = state.settings.get("languages_config", {}).get(state.lang_id, {})
        config = {
            'text': state.text_for_processing,
            'dir_path': state.dir_path,
            'lang_config': lang_config,
            'voicemaker_lang_code': self._get_voicemaker_language_code(lang_config.get('voicemaker_voice_id')),
            'job_name': state.job_name,
            'lang_name': state.lang_name
        }
        self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)
        
# ... (lines omitted)

    def _launch_subtitle_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'amd')
            model_name = sub_settings.get('whisper_model', 'base.bin')
            
            whisper_exe = None; whisper_model_path = model_name
            if whisper_type == 'amd':
                if getattr(sys, 'frozen', False):
                    whisper_base_path = os.path.join(os.path.dirname(sys.executable), "whisper-cli-amd")
                else:
                    current_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
                    whisper_base_path = os.path.join(current_dir, "whisper-cli-amd")
                
                whisper_exe = os.path.join(whisper_base_path, "main.exe")
                whisper_model_path = os.path.join(whisper_base_path, model_name)
            
            config = {
                'audio_path': state.audio_path, 'dir_path': state.dir_path,
                'sub_settings': sub_settings, 'lang_code': state.lang_id.split('-')[0].lower(),
                'whisper_exe': whisper_exe, 'whisper_model_path': whisper_model_path
            }
            self._start_worker(SubtitleWorker, task_id, 'stage_subtitles', config,
                                         self._on_subtitles_finished, self._on_subtitles_error)
        except Exception as e:
            self._on_subtitles_error(task_id, f"Failed to start subtitle worker: {e}")

# ... (lines omitted)

    def _start_image_generation(self, task_id):
        state = self.task_states[task_id]
        googler_settings = state.settings.get('googler', {})
        
        # Calculate total prompts count for metadata
        prompts = re.findall(r"^\d+\.\s*(.*)", state.image_prompts, re.MULTILINE)
        if not prompts:
            prompts = [line.strip() for line in state.image_prompts.split('\n') if line.strip()]
        state.images_total_count = len(prompts)
        state.images_generated_count = 0  # Reset counter
        
        # Emit initial metadata (0/total)
        metadata_text = f"0/{state.images_total_count}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)
        
        config = {
            'prompts_text': state.image_prompts,
            'dir_path': state.dir_path,
            'provider': state.settings.get('image_generation_provider', 'pollinations'),
            'api_kwargs': {
                'aspect_ratio': googler_settings.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE'),
                'seed': googler_settings.get('seed'),
                'negative_prompt': googler_settings.get('negative_prompt')
            },
            'executor': self.image_gen_executor,
            'max_threads': googler_settings.get("max_threads", 8)
        }
        self._start_worker(ImageGenerationWorker, task_id, 'stage_images', config, self._on_img_generation_finished, self._on_img_generation_error)

# ... (lines omitted)

    def _start_video_generation(self, task_id):
        state = self.task_states[task_id]
        montage_settings = state.settings.get("montage", {})
        googler_settings = state.settings.get("googler", {})

        video_count = montage_settings.get("special_processing_video_count", 1)
        check_sequence = montage_settings.get("special_processing_check_sequence", False)
        
        all_image_paths = state.image_paths
        paths_to_animate = []

        if not all_image_paths:
            logger.log(f"[{task_id}] No images to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return

        if not check_sequence:
            paths_to_animate = all_image_paths[:video_count]
        else:
            # --- Sequence Check Logic ---
            first_img_basename = os.path.splitext(os.path.basename(all_image_paths[0]))[0]
            if first_img_basename != '1':
                logger.log(f"[{task_id}] First image is '{first_img_basename}', not '1'. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
                state.fallback_to_quick_show = True
                if self.subtitle_barrier_passed: self._check_and_start_montages()
                return

            sequential_count = 0
            for i in range(min(video_count, len(all_image_paths))):
                expected_name = str(i + 1)
                actual_name = os.path.splitext(os.path.basename(all_image_paths[i]))[0]
                if actual_name == expected_name:
                    sequential_count += 1
                else:
                    break

            if sequential_count > 0:
                logger.log(f"[{task_id}] Found a sequence of {sequential_count} images to animate.", level=LogLevel.INFO)
                paths_to_animate = all_image_paths[:sequential_count]

        if not paths_to_animate:
            logger.log(f"[{task_id}] No sequential images found to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return
            
        state.video_animation_count = len(paths_to_animate)
        logger.log(f"[{task_id}] Starting video generation for {len(paths_to_animate)} images.", level=LogLevel.INFO)

        config = {
            'image_paths': paths_to_animate,
            'prompt': googler_settings.get("video_prompt", "Animate this scene, cinematic movement, 4k"),
            'video_semaphore': self.video_semaphore,
            'max_threads': googler_settings.get("max_video_threads", 1)
        }
        self._start_worker(VideoGenerationWorker, task_id, 'stage_images', config, self._on_video_generation_finished, self._on_video_generation_error)

# ... (lines omitted)

    def _launch_montage_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Set status to 'processing'
            self.stage_status_changed.emit(state.job_id, state.lang_id, 'stage_montage', 'processing')
            state.status['stage_montage'] = 'processing'
            
            final_image_paths = state.image_paths
                
            if not final_image_paths:
                self._on_montage_error(task_id, "No visual files found for montage.")
                return
            
            safe_task_name = ("".join(c for c in state.job_name if c.isalnum() or c in (' ', '_')).strip())[:100]
            safe_lang_name = "".join(c for c in state.lang_name if c.isalnum() or c in (' ', '_')).strip()
            output_filename = f"{safe_task_name}_{safe_lang_name}.mp4"
            output_path = os.path.join(state.dir_path, output_filename)
            
            montage_settings = state.settings.get("montage", {}).copy()
            if getattr(state, 'fallback_to_quick_show', False):
                logger.log(f"[{task_id}] Fallback to 'Quick Show' mode for montage.", level=LogLevel.WARNING)
                montage_settings['special_processing_mode'] = "Quick show"
                
            config = {
                'visual_files': final_image_paths, 'audio_path': state.audio_path,
                'output_path': output_path, 'ass_path': state.subtitle_path,
                'settings': montage_settings
            }

            # --- Add Background Music Config ---
            background_music_path = None
            background_music_volume = 100
            
            # Use state.settings for languages config too
            all_languages_config = state.settings.get("languages_config", {})
            lang_config = all_languages_config.get(state.lang_id, {})
            
            user_files = state.lang_data.get('user_provided_files', {})

            if 'background_music' in user_files:
                # User override from TextTab
                override_path = user_files['background_music']
                if os.path.exists(override_path):
                    background_music_path = override_path
                    # Use volume from user_files if available, otherwise default to 100
                    background_music_volume = user_files.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using user-provided background music: {os.path.basename(background_music_path)} with volume {background_music_volume}%", level=LogLevel.INFO)
                else:
                    logger.log(f"[{task_id}] User-provided background music not found at {override_path}. Skipping.", level=LogLevel.WARNING)

            if not background_music_path:
                # Fallback to default from language settings
                default_path = lang_config.get("background_music_path")
                if default_path and os.path.exists(default_path):
                    background_music_path = default_path
                    background_music_volume = lang_config.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using default background music: {os.path.basename(background_music_path)}", level=LogLevel.INFO)
            
            if background_music_path:
                config['background_music_path'] = background_music_path
                config['background_music_volume'] = background_music_volume
            # --- End Background Music Config ---

            worker = MontageWorker(task_id, config)
            worker.signals.finished.connect(self._on_montage_finished)
            worker.signals.error.connect(self._on_montage_error)
            worker.signals.progress_log.connect(self._on_montage_progress)
            self.threadpool.start(worker)
        except Exception as e:
            self._on_montage_error(task_id, f"Failed to start montage worker: {e}")

    @Slot(str, object)
    def _on_img_prompts_finished(self, task_id, prompts_text):
        state = self.task_states[task_id]
        
        # Count prompts
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        prompts_count = len(prompts)

        # Check if prompt count control is enabled
        is_check_enabled = self.settings.get('image_prompt_count_check_enabled', False)
        desired_count = self.settings.get('image_prompt_count', 50)

        if is_check_enabled and prompts_count != desired_count:
            if state.prompt_regeneration_attempts < 3:
                state.prompt_regeneration_attempts += 1
                logger.log(
                    f"[{task_id}] Image prompt count is {prompts_count}, but {desired_count} is required. "
                    f"Regenerating, attempt {state.prompt_regeneration_attempts}/3.",
                    level=LogLevel.WARNING
                )
                self._start_image_prompts(task_id)
                return  # Stop processing this result and wait for the new one
            else:
                logger.log(
                    f"[{task_id}] Failed to generate the required number of image prompts ({desired_count}) after 3 attempts. "
                    f"Proceeding with {prompts_count} prompts.",
                    level=LogLevel.ERROR
                )

        state.image_prompts = prompts_text
        if state.dir_path:
            with open(os.path.join(state.dir_path, "image_prompts.txt"), 'w', encoding='utf-8') as f:
                f.write(prompts_text)
        self._set_stage_status(task_id, 'stage_img_prompts', 'success')
        
        # Emit metadata
        metadata_text = f"{prompts_count} {translator.translate('prompts_count')}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_img_prompts', metadata_text)
        
        if 'stage_images' in state.stages:
            self._start_image_generation(task_id)
        else:
            self.check_if_all_finished()

    @Slot(str, str)
    def _on_img_prompts_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_img_prompts', 'error', error)

    def _start_voiceover(self, task_id):
        state = self.task_states[task_id]
        task_settings = state.settings
        lang_config = task_settings.get("languages_config", {}).get(state.lang_id, {})
        config = {
            'text': state.text_for_processing,
            'dir_path': state.dir_path,
            'lang_config': lang_config,
            'voicemaker_api_key': task_settings.get('voicemaker_api_key'),
            'elevenlabs_api_key': task_settings.get('elevenlabs_api_key'),
            'voicemaker_lang_code': self._get_voicemaker_language_code(lang_config.get('voicemaker_voice_id')),
            'job_name': state.job_name,
            'lang_name': state.lang_name
        }
        self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)
        
    @Slot(str, object)
    def _on_voiceover_finished(self, task_id, audio_path):
        state = self.task_states[task_id]
        state.audio_path = audio_path
        self._set_stage_status(task_id, 'stage_voiceover', 'success')
        
        # Get audio duration and emit metadata
        try:
            duration_seconds = self._get_audio_duration(audio_path)
            hours = int(duration_seconds // 3600)
            minutes = int((duration_seconds % 3600) // 60)
            seconds = int(duration_seconds % 60)
            if hours > 0:
                metadata_text = f"{hours}:{minutes:02d}:{seconds:02d}"
            else:
                metadata_text = f"{minutes}:{seconds:02d}"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_voiceover', metadata_text)
        except Exception as e:
            logger.log(f"[{task_id}] Failed to get audio duration: {e}", level=LogLevel.WARNING)
        
        if 'stage_subtitles' in state.stages:
            self._start_subtitles(task_id)
        else:
            self.check_if_all_finished()

    @Slot(str, str)
    def _on_voiceover_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_voiceover', 'error', error)
        # CRITICAL FIX: If voiceover fails, we MUST fail or skip subtitles so the job doesn't hang forever in 'pending'
        if 'stage_subtitles' in self.task_states[task_id].stages:
            logger.log(f"[{task_id}] Voiceover failed, marking dependent stage 'stage_subtitles' as error.", level=LogLevel.WARNING)
            self._set_stage_status(task_id, 'stage_subtitles', 'error', "Dependency (Voiceover) failed")
            # We still increment counter so the barrier can pass
            self._increment_subtitle_counter()

    def _start_subtitles(self, task_id):
        self.pending_subtitles.append(task_id)
        self._process_subtitle_queue()

    def _process_subtitle_queue(self):
        sub_settings = self.settings.get('subtitles', {})
        whisper_type = sub_settings.get('whisper_type', 'amd')
        
        # AssemblyAI doesn't need semaphore in this logic (handled by API limits potentially, but treated as unblocked here?)
        # Current logic: "if whisper_type == 'assemblyai': ... run directly"
        # We'll treat AssemblyAI as infinite concurrency or separate logic?
        # The original code: "if whisper_type == 'assemblyai': self._start_the_work()" (no acquire)
        
        while self.pending_subtitles:
            # check what the NEXT task needs
            # But we can't easily peek and check config without popping or complex logic.
            # Simplified: If queue has items, try to run them.
            
            # Since whisper_type is global setting (likely), we assume it applies to all.
            if whisper_type == 'assemblyai':
                task_id = self.pending_subtitles.popleft()
                self._launch_subtitle_worker(task_id)
            else:
                if self.subtitle_semaphore.tryAcquire():
                    task_id = self.pending_subtitles.popleft()
                    self._launch_subtitle_worker(task_id)
                else:
                    break

    def _launch_subtitle_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            sub_settings = self.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'amd')
            model_name = sub_settings.get('whisper_model', 'base.bin')
            
            whisper_exe = None; whisper_model_path = model_name
            if whisper_type == 'amd':
                if getattr(sys, 'frozen', False):
                    whisper_base_path = os.path.join(os.path.dirname(sys.executable), "whisper-cli-amd")
                else:
                    current_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
                    whisper_base_path = os.path.join(current_dir, "whisper-cli-amd")
                
                whisper_exe = os.path.join(whisper_base_path, "main.exe")
                whisper_model_path = os.path.join(whisper_base_path, model_name)
            
            config = {
                'audio_path': state.audio_path, 'dir_path': state.dir_path,
                'sub_settings': sub_settings, 'lang_code': state.lang_id.split('-')[0].lower(),
                'whisper_exe': whisper_exe, 'whisper_model_path': whisper_model_path
            }
            self._start_worker(SubtitleWorker, task_id, 'stage_subtitles', config,
                                         self._on_subtitles_finished, self._on_subtitles_error)
        except Exception as e:
            self._on_subtitles_error(task_id, f"Failed to start subtitle worker: {e}")
        
    @Slot(str, object)
    def _on_subtitles_finished(self, task_id, subtitle_path):
        sub_settings = self.settings.get('subtitles', {})
        whisper_type = sub_settings.get('whisper_type', 'amd')
        if whisper_type != 'assemblyai':
            self.subtitle_semaphore.release()
            self._process_subtitle_queue()
            
        self.task_states[task_id].subtitle_path = subtitle_path
        self._set_stage_status(task_id, 'stage_subtitles', 'success')
        self._increment_subtitle_counter()

    @Slot(str, str)
    def _on_subtitles_error(self, task_id, error):
        sub_settings = self.settings.get('subtitles', {})
        whisper_type = sub_settings.get('whisper_type', 'amd')
        if whisper_type != 'assemblyai':
            self.subtitle_semaphore.release()
            self._process_subtitle_queue()

        self._set_stage_status(task_id, 'stage_subtitles', 'error', error)
        self._increment_subtitle_counter()

    def _increment_subtitle_counter(self):
        with self.subtitle_lock:
            self.completed_subtitle_tasks += 1
            logger.log(f"Subtitle task progress: {self.completed_subtitle_tasks}/{self.total_subtitle_tasks}", level=LogLevel.INFO)
        
        if not self.subtitle_barrier_passed and self.completed_subtitle_tasks >= self.total_subtitle_tasks:
            self.subtitle_barrier_passed = True
            logger.log("All subtitle tasks completed. Barrier passed. Checking for pending montages.", level=LogLevel.SUCCESS)
            self._check_and_start_montages()

    def _start_image_generation(self, task_id):
        state = self.task_states[task_id]

        # If user has provided their own images, the worker will see this and skip generation.
        if 'stage_images' in state.lang_data.get('pre_found_files', {}):
            config = {} # Dummy config, not used by skipping logic in worker
            self._start_worker(ImageGenerationWorker, task_id, 'stage_images', config, self._on_img_generation_finished, self._on_img_generation_error)
            return

        if not state.image_prompts:
            self._on_img_generation_error(task_id, "Cannot generate images because image prompts text is missing.")
            return
            
        googler_settings = self.settings.get('googler', {})
        
        # Calculate total prompts count for metadata
        prompts = re.findall(r"^\d+\.\s*(.*)", state.image_prompts, re.MULTILINE)
        if not prompts:
            prompts = [line.strip() for line in state.image_prompts.split('\n') if line.strip()]
        state.images_total_count = len(prompts)
        state.images_generated_count = 0  # Reset counter
        
        # Emit initial metadata (0/total)
        metadata_text = f"0/{state.images_total_count}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)
        
        config = {
            'prompts_text': state.image_prompts,
            'dir_path': state.dir_path,
            'provider': state.settings.get('image_generation_provider', 'pollinations'),
            'api_kwargs': {
                'aspect_ratio': googler_settings.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE'),
                'seed': googler_settings.get('seed'),
                'negative_prompt': googler_settings.get('negative_prompt')
            },
            'executor': self.image_gen_executor,
            'max_threads': googler_settings.get("max_threads", 8)
        }
        self._start_worker(ImageGenerationWorker, task_id, 'stage_images', config, self._on_img_generation_finished, self._on_img_generation_error)

    @Slot(str, object)
    def _on_img_generation_finished(self, task_id, result_dict):
        generated_paths = result_dict.get('paths', [])
        total_prompts = result_dict.get('total_prompts', 0)
        
        status = 'error'
        if total_prompts > 0 and len(generated_paths) == total_prompts:
            status = 'success'
        elif len(generated_paths) > 0:
            status = 'warning'

        state = self.task_states[task_id]
        state.image_paths = generated_paths
        state.image_gen_status = status # Store the status
        
        montage_settings = state.settings.get("montage", {})
        special_mode = montage_settings.get("special_processing_mode", "Disabled")
        
        if special_mode == "Video at the beginning" and generated_paths:
            self._set_stage_status(task_id, 'stage_images', 'processing_video')
            self._start_video_generation(task_id)
        else:
            self._set_stage_status(task_id, 'stage_images', status, "Failed to generate all images." if status != 'success' else None)
            if self.subtitle_barrier_passed:
                self._check_and_start_montages()

    @Slot(str, str)
    def _on_img_generation_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_images', 'error', error)
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    def _start_video_generation(self, task_id):
        state = self.task_states[task_id]
        montage_settings = self.settings.get("montage", {})
        googler_settings = self.settings.get("googler", {})

        video_count = montage_settings.get("special_processing_video_count", 1)
        check_sequence = montage_settings.get("special_processing_check_sequence", False)
        
        all_image_paths = state.image_paths
        paths_to_animate = []

        if not all_image_paths:
            logger.log(f"[{task_id}] No images to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return

        if not check_sequence:
            paths_to_animate = all_image_paths[:video_count]
        else:
            # --- Sequence Check Logic ---
            first_img_basename = os.path.splitext(os.path.basename(all_image_paths[0]))[0]
            if first_img_basename != '1':
                logger.log(f"[{task_id}] First image is '{first_img_basename}', not '1'. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
                state.fallback_to_quick_show = True
                if self.subtitle_barrier_passed: self._check_and_start_montages()
                return

            sequential_count = 0
            for i in range(min(video_count, len(all_image_paths))):
                expected_name = str(i + 1)
                actual_name = os.path.splitext(os.path.basename(all_image_paths[i]))[0]
                if actual_name == expected_name:
                    sequential_count += 1
                else:
                    break

            if sequential_count > 0:
                logger.log(f"[{task_id}] Found a sequence of {sequential_count} images to animate.", level=LogLevel.INFO)
                paths_to_animate = all_image_paths[:sequential_count]

        if not paths_to_animate:
            logger.log(f"[{task_id}] No sequential images found to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return
            
        state.video_animation_count = len(paths_to_animate)
        state.videos_total_count = len(paths_to_animate)
        state.videos_generated_count = 0
        logger.log(f"[{task_id}] Starting video generation for {len(paths_to_animate)} images.", level=LogLevel.INFO)

        # Update metadata to show video progress
        img_meta = f"{state.images_generated_count}/{state.images_total_count}"
        vid_meta = f"0/{state.videos_total_count}"
        metadata_text = f"IMG: {img_meta}, VID: {vid_meta}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)

        config = {
            'image_paths': paths_to_animate,
            'prompt': googler_settings.get("video_prompt", "Animate this scene, cinematic movement, 4k"),
            'video_semaphore': self.video_semaphore,
            'max_threads': googler_settings.get("max_video_threads", 1)
        }
        self._start_worker(VideoGenerationWorker, task_id, 'stage_images', config, self._on_video_generation_finished, self._on_video_generation_error)

    @Slot(str, object)
    def _on_video_generation_finished(self, task_id, result_dict):
        state = self.task_states[task_id]
        generated_videos = result_dict.get('paths', [])
        
        logger.log(f"[{task_id}] Video generation finished. Generated {len(generated_videos)} videos.", level=LogLevel.SUCCESS)

        video_count_animated = getattr(state, 'video_animation_count', 0)
        
        # Determine video generation status
        video_gen_status = 'success'
        if len(generated_videos) < video_count_animated:
            video_gen_status = 'warning'
        if len(generated_videos) == 0 and video_count_animated > 0:
            video_gen_status = 'error'

        if generated_videos:
            remaining_images = state.image_paths[video_count_animated:]
            state.image_paths = generated_videos + remaining_images
        elif video_count_animated > 0: # This means video gen failed for all
            logger.log(f"[{task_id}] Video animation produced 0 videos. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
            state.fallback_to_quick_show = True

        # Combine statuses: warning is the "lowest" priority besides success
        final_status = state.image_gen_status
        if final_status == 'success' and video_gen_status != 'success':
            final_status = video_gen_status # 'warning' or 'error'
        elif final_status == 'warning' and video_gen_status == 'error':
            final_status = 'error'
        # If image_gen_status was 'error', it remains 'error'

        error_message = None
        if final_status != 'success':
            error_message = "Failed to generate all images and/or videos."
            
        self._set_stage_status(task_id, 'stage_images', final_status, error_message)
        
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    @Slot(str, str)
    def _on_video_generation_error(self, task_id, error):
        logger.log(f"[{task_id}] Video generation failed: {error}", level=LogLevel.ERROR)
        self._set_stage_status(task_id, 'stage_images', 'error', error)
        # We don't stop the whole process, just log the error and proceed with the images we have
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()


    def _check_and_start_montages(self):
        for task_id, state in self.task_states.items():
            if 'stage_montage' not in state.stages or state.status.get('stage_montage') != 'pending':
                continue
            
            sub_status = state.status.get('stage_subtitles') if 'stage_subtitles' in state.stages else 'success'
            img_status = state.status.get('stage_images') if 'stage_images' in state.stages else 'success'

            prerequisites_are_done = sub_status not in ['pending', 'processing'] and img_status not in ['pending', 'processing', 'processing_video']

            if prerequisites_are_done:
                if sub_status == 'error' or img_status == 'error':
                    error_msg = f"Prerequisite failed. Subtitles: {sub_status}, Images: {img_status}"
                    self._set_stage_status(task_id, 'stage_montage', 'error', error_msg)
                    continue

                if not state.audio_path or not os.path.exists(state.audio_path):
                    self._set_stage_status(task_id, 'stage_montage', 'error', "Audio file missing")
                    continue
                
                if not state.image_paths or len(state.image_paths) == 0:
                    self._set_stage_status(task_id, 'stage_montage', 'error', "No images available")
                    continue
                
                # Use per-task settings for review
                if state.settings.get('image_review_enabled'):
                    if task_id not in self.tasks_awaiting_review:
                        logger.log(f"[{task_id}] Ready for image review.", level=LogLevel.INFO)
                        self.tasks_awaiting_review.append(task_id)
                else:
                    self._start_montage(task_id)
        
        self._check_if_all_are_ready_or_failed()

    def _check_if_all_are_ready_or_failed(self):
        # Determine if we should trigger the review dialog.
        # Logic: All tasks scheduled for montage must be either:
        # 1. Waiting for review
        # 2. Failed
        # 3. Already started/processed (meaning they skipped review due to settings)
        
        total_montage_tasks = len(self.montage_tasks_ids)
        if total_montage_tasks == 0:
            return

        ready_or_processed_count = 0
        
        for task_id in self.montage_tasks_ids:
            state = self.task_states.get(task_id)
            if not state: continue # Should not happen

            is_waiting = task_id in self.tasks_awaiting_review
            is_failed = task_id in self.failed_montage_tasks_ids
            # If stage_montage is NOT pending, it means it started processing (skipped review)
            is_processed = state.status.get('stage_montage') != 'pending'
            
            if is_waiting or is_failed or is_processed:
                ready_or_processed_count += 1

        if ready_or_processed_count >= total_montage_tasks:
            # Only trigger if there are actually tasks waiting for review
            if len(self.tasks_awaiting_review) > 0:
                logger.log(f"All {total_montage_tasks} tasks accounted for. Requesting review for {len(self.tasks_awaiting_review)} tasks.", level=LogLevel.SUCCESS)
                self.image_review_required.emit()

    @Slot()
    def resume_all_montages(self):
        logger.log(f"Resuming montage for {len(self.tasks_awaiting_review)} tasks.", level=LogLevel.INFO)
        tasks_to_start = list(self.tasks_awaiting_review)
        self.tasks_awaiting_review.clear()
        
        for task_id in tasks_to_start:
            state = self.task_states.get(task_id)
            if state:
                self._start_montage(task_id)

    def _start_montage(self, task_id):
        self.pending_montages.append(task_id)
        self._process_montage_queue()

    def _process_montage_queue(self):
        while self.pending_montages:
            if self.montage_semaphore.tryAcquire():
                task_id = self.pending_montages.popleft()
                self._launch_montage_worker(task_id)
            else:
                break
    
    def _launch_montage_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Set status to 'processing'
            self.stage_status_changed.emit(state.job_id, state.lang_id, 'stage_montage', 'processing')
            state.status['stage_montage'] = 'processing'
            
            final_image_paths = state.image_paths
                
            if not final_image_paths:
                self._on_montage_error(task_id, "No visual files found for montage.")
                return
            
            safe_task_name = ("".join(c for c in state.job_name if c.isalnum() or c in (' ', '_')).strip())[:100]
            safe_lang_name = "".join(c for c in state.lang_name if c.isalnum() or c in (' ', '_')).strip()
            output_filename = f"{safe_task_name}_{safe_lang_name}.mp4"
            output_path = os.path.join(state.dir_path, output_filename)
            
            montage_settings = self.settings.get("montage", {}).copy()
            if getattr(state, 'fallback_to_quick_show', False):
                logger.log(f"[{task_id}] Fallback to 'Quick Show' mode for montage.", level=LogLevel.WARNING)
                montage_settings['special_processing_mode'] = "Quick show"
                
            config = {
                'visual_files': final_image_paths, 'audio_path': state.audio_path,
                'output_path': output_path, 'ass_path': state.subtitle_path,
                'settings': montage_settings
            }

            # --- Add Background Music Config ---
            background_music_path = None
            background_music_volume = 100
            
            all_languages_config = self.settings.get("languages_config", {})
            lang_config = all_languages_config.get(state.lang_id, {})
            
            user_files = state.lang_data.get('user_provided_files', {})

            if 'background_music' in user_files:
                # User override from TextTab
                override_path = user_files['background_music']
                if os.path.exists(override_path):
                    background_music_path = override_path
                    # Use volume from user_files if available, otherwise default to 100
                    background_music_volume = user_files.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using user-provided background music: {os.path.basename(background_music_path)} with volume {background_music_volume}%", level=LogLevel.INFO)
                else:
                    logger.log(f"[{task_id}] User-provided background music not found at {override_path}. Skipping.", level=LogLevel.WARNING)

            if not background_music_path:
                # Fallback to default from language settings
                default_path = lang_config.get("background_music_path")
                if default_path and os.path.exists(default_path):
                    background_music_path = default_path
                    background_music_volume = lang_config.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using default background music: {os.path.basename(background_music_path)}", level=LogLevel.INFO)
            
            if background_music_path:
                config['background_music_path'] = background_music_path
                config['background_music_volume'] = background_music_volume
            # --- End Background Music Config ---

            worker = MontageWorker(task_id, config)
            worker.signals.finished.connect(self._on_montage_finished)
            worker.signals.error.connect(self._on_montage_error)
            worker.signals.progress_log.connect(self._on_montage_progress)
            self.threadpool.start(worker)
        except Exception as e:
            self._on_montage_error(task_id, f"Failed to start montage worker: {e}")


    @Slot(str, object)
    def _on_montage_finished(self, task_id, video_path):
        self.montage_semaphore.release()
        self._process_montage_queue()
        self.task_states[task_id].final_video_path = video_path
        self._set_stage_status(task_id, 'stage_montage', 'success')
        
        # Get file size and emit metadata
        try:
            state = self.task_states[task_id]
            file_size_bytes = os.path.getsize(video_path)
            file_size_gb = file_size_bytes / (1024 * 1024 * 1024)  # Convert to GB
            metadata_text = f"{file_size_gb:.2f} GB"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_montage', metadata_text)
        except Exception as e:
            logger.log(f"[{task_id}] Failed to get video file size: {e}", level=LogLevel.WARNING)

    @Slot(str, str)
    def _on_montage_error(self, task_id, error):
        self.montage_semaphore.release()
        self._process_montage_queue()
        self._set_stage_status(task_id, 'stage_montage', 'error', error)
    
    @Slot(str, str)
    def _on_montage_progress(self, task_id, message):
        job_id = task_id.split('_')[0] if '_' in task_id else task_id
        self.task_progress_log.emit(job_id, message)

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
            self.processing_finished.emit(elapsed_str)

# endregion