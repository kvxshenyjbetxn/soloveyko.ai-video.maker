import os
import re
import time
import base64
import json
import traceback
import threading
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer, QSemaphore, Qt, Slot

from utils.logger import logger, LogLevel
from utils.settings import settings_manager
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
    status_changed = Signal(str, str) # task_id, status_message
    progress_log = Signal(str, str)  # task_id, log_message (for card-only logs)

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
        full_prompt = f"{lang_config.get('prompt', '')}\n\n{self.config['text']}"
        response = api.get_chat_completion(
            model=lang_config.get('model'),
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=lang_config.get('max_tokens', 4096)
        )
        if response and response['choices'][0]['message']['content']:
            return response['choices'][0]['message']['content']
        else:
            raise Exception("Empty or invalid response from translation API.")

class ImagePromptWorker(BaseWorker):
    def do_work(self):
        api = OpenRouterAPI()
        img_prompt_settings = self.config['img_prompt_settings']
        full_prompt = f"{img_prompt_settings.get('prompt', '')}\n\n{self.config['text']}"
        response = api.get_chat_completion(
            model=img_prompt_settings.get('model'),
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=img_prompt_settings.get('max_tokens', 4096)
        )
        if response and response['choices'][0]['message']['content']:
            return response['choices'][0]['message']['content']
        else:
            raise Exception("Empty or invalid response from image prompt API.")

class VoiceoverWorker(BaseWorker):
    def do_work(self):
        text = self.config['text']
        dir_path = self.config['dir_path']
        lang_config = self.config['lang_config']
        tts_provider = lang_config.get('tts_provider', 'ElevenLabs')

        if tts_provider == 'VoiceMaker':
            api = VoicemakerAPI()
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
            api = ElevenLabsAPI()
            task_id, status = api.create_task(text, lang_config['elevenlabs_template_uuid'])
            if status != 'connected' or not task_id:
                raise Exception("Failed to create ElevenLabs task.")
            
            for _ in range(30): # 5 min timeout
                task_status, status = api.get_task_status(task_id)
                if status != 'connected': raise Exception("Failed to get ElevenLabs task status.")
                if task_status in ['ending', 'ending_processed']:
                    audio_content, status = api.get_task_result(task_id)
                    if status == 'connected' and audio_content:
                        return self.save_audio(audio_content, "voice.mp3")
                    elif status == 'not_ready': time.sleep(10); continue
                    else: raise Exception("Failed to download ElevenLabs audio.")
                elif task_status in ['error', 'error_handled']:
                    raise Exception("ElevenLabs task processing resulted in an error.")
                time.sleep(10)
            raise Exception("Timeout waiting for ElevenLabs result.")

    def save_audio(self, content, filename):
        path = os.path.join(self.config['dir_path'], filename)
        with open(path, 'wb') as f: f.write(content)
        return path

class SubtitleWorker(BaseWorker):
    def do_work(self):
        logger.log(f"[{self.task_id}] Starting subtitle generation", level=LogLevel.INFO)
        engine = SubtitleEngine(self.config['whisper_exe'], self.config['whisper_model_path'])
        output_filename = os.path.splitext(os.path.basename(self.config['audio_path']))[0] + ".ass"
        output_path = os.path.join(self.config['dir_path'], output_filename)
        engine.generate_ass(self.config['audio_path'], output_path, self.config['sub_settings'], language=self.config['lang_code'])
        # Success log is handled by _set_stage_status
        return output_path

class ImageGenerationWorker(BaseWorker):
    def do_work(self):
        prompts_text = self.config['prompts_text']
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        if not prompts:
            raise Exception("No prompts found in the generated text.")

        provider = self.config['provider']
        images_dir = os.path.join(self.config['dir_path'], "images")
        os.makedirs(images_dir, exist_ok=True)
        
        if provider == 'googler':
            api = GooglerAPI()
            file_extension = 'jpg'
            api_kwargs = self.config['api_kwargs']
        else: # pollinations
            api = PollinationsAPI()
            file_extension = 'png'
            api_kwargs = {}
        
        generated_paths = []
        for i, prompt in enumerate(prompts):
            image_data = api.generate_image(prompt, **api_kwargs)
            if not image_data:
                logger.log(f"[{self.task_id}] Failed to generate image {i + 1} for prompt: '{prompt}' (no data).", level=LogLevel.WARNING)
                continue

            image_path = os.path.join(images_dir, f"{i + 1}.{file_extension}")
            data_to_write = image_data
            if provider == 'googler' and isinstance(image_data, str):
                data_to_write = base64.b64decode(image_data.split(",", 1)[1] if "," in image_data else image_data)
            
            with open(image_path, 'wb') as f: f.write(data_to_write)
            
            logger.log(f"[{self.task_id}] Saved image {i + 1} to {image_path}", level=LogLevel.SUCCESS)
            generated_paths.append(image_path)
            # Emit signal for gallery update
            self.signals.status_changed.emit(self.task_id, image_path)

        
        if len(generated_paths) == 0 and len(prompts) > 0:
            raise Exception("Failed to generate any images.")

        return {'paths': generated_paths, 'total_prompts': len(prompts)}

class MontageWorker(BaseWorker):
    def do_work(self):
        logger.log(f"[{self.task_id}] Starting video montage", level=LogLevel.INFO)
        engine = MontageEngine()
        # Pass task_id and progress callback to the engine
        self.config['task_id'] = self.task_id
        self.config['progress_callback'] = lambda msg: self.signals.progress_log.emit(self.task_id, msg)
        engine.create_video(**self.config)
        # Success log is handled by _set_stage_status
        return self.config['output_path']

# endregion
# =================================================================================================================
# region TASK PROCESSOR
# =================================================================================================================

class TaskState:
    """Holds the state and data for a single language within a single job."""
    def __init__(self, job, lang_id, lang_data, base_save_path):
        self.job_id = job['id']
        self.lang_id = lang_id
        self.task_id = f"{self.job_id}_{self.lang_id}"

        self.job_name = job['name']
        self.lang_name = lang_data['display_name']
        self.stages = lang_data['stages']
        self.original_text = job['text']

        self.dir_path = self._get_save_path(base_save_path, self.job_name, self.lang_name)

        self.text_for_processing = None
        self.image_prompts = None
        self.audio_path = None
        self.subtitle_path = None
        self.image_paths = None
        self.final_video_path = None

        self.status = {stage: 'pending' for stage in self.stages}

    def _get_save_path(self, base_path, job_name, lang_name):
        if not base_path: return None
        try:
            safe_job_name = "".join(c for c in job_name if c.isalnum() or c in (' ', '_')).rstrip()
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
            dir_path = os.path.join(base_path, safe_job_name, safe_lang_name)
            os.makedirs(dir_path, exist_ok=True)
            return dir_path
        except Exception as e:
            logger.log(f"[{self.task_id}] Failed to create save directory {dir_path}. Error: {e}", level=LogLevel.ERROR)
            return None

class TaskProcessor(QObject):
    processing_finished = Signal(str)
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status
    image_generated = Signal(str, str, str) # task_name, image_path, prompt (TaskID, image_path, prompt)
    task_progress_log = Signal(str, str) # job_id, log_message (for card-only logs)

    def __init__(self, queue_manager):
        super().__init__()
        self.queue_manager = queue_manager
        self.settings = settings_manager
        self.threadpool = QThreadPool()
        self.timer = QElapsedTimer()
        self.voicemaker_voices = self._load_voicemaker_voices()
        
        self.task_states = {}
        self.total_subtitle_tasks = 0
        self.completed_subtitle_tasks = 0
        self.subtitle_barrier_passed = False
        self.is_finished = False
        self.subtitle_lock = threading.Lock() # Used to sync counter access

        # --- Semaphores for concurrency control ---
        self.subtitle_semaphore = QSemaphore(1)
        montage_settings = self.settings.get("montage", {})
        max_montage = montage_settings.get("max_concurrent_montages", 1)
        self.montage_semaphore = QSemaphore(max_montage)
        logger.log(f"Task Processor initialized. Subtitle concurrency: 1, Montage concurrency: {max_montage}", level=LogLevel.INFO)

    def _load_voicemaker_voices(self):
        try:
            with open("assets/voicemaker_voices.json", "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            logger.log(f"Error loading voicemaker voices: {e}", level=LogLevel.ERROR)
            return []

    def _get_voicemaker_language_code(self, voice_id):
        for lang_data in self.voicemaker_voices:
            if voice_id in lang_data.get("Voices", []):
                return lang_data.get("LanguageCode")
        return "en-US"

    def start_processing(self):
        jobs = self.queue_manager.get_tasks()
        if not jobs:
            logger.log("Queue is empty.", level=LogLevel.INFO)
            return

        self.timer.start()
        self.task_states = {}
        self.total_subtitle_tasks = 0
        self.completed_subtitle_tasks = 0
        
        base_save_path = self.settings.get('results_path')
        all_languages_config = self.settings.get("languages_config", {})

        # 1. Initialize all task states
        for job in jobs:
            for lang_id, lang_data in job['languages'].items():
                state = TaskState(job, lang_id, lang_data, base_save_path)
                self.task_states[state.task_id] = state
                if 'stage_subtitles' in state.stages:
                    self.total_subtitle_tasks += 1

        logger.log(f"Starting processing for {len(self.task_states)} language tasks.", level=LogLevel.INFO)

        # 2. Fire initial workers
        for task_id, state in self.task_states.items():
            if 'stage_translation' in state.stages:
                self._start_translation(task_id, all_languages_config)
                time.sleep(0.5) # Per user request
            else:
                state.text_for_processing = state.original_text
                self._on_text_ready(task_id)
    
    def _start_worker(self, worker_class, task_id, stage_key, config, on_finish_slot, on_error_slot):
        self.stage_status_changed.emit(self.task_states[task_id].job_id, self.task_states[task_id].lang_id, stage_key, 'processing')
        worker = worker_class(task_id, config)
        worker.signals.finished.connect(on_finish_slot)
        worker.signals.error.connect(on_error_slot)
        worker.signals.status_changed.connect(self._on_worker_status_changed) # For gallery updates
        self.threadpool.start(worker)

    @Slot(str, str)
    def _on_worker_status_changed(self, task_id, message):
        """Used for intermediate status updates, like an image being generated."""
        state = self.task_states.get(task_id)
        if state:
            # We assume the message is the image path for now
            self.image_generated.emit(state.job_name, message, "") # task_name, image_path, prompt

    def _set_stage_status(self, task_id, stage_key, status, error_message=None):
        state = self.task_states.get(task_id)
        if not state: return

        state.status[stage_key] = status
        self.stage_status_changed.emit(state.job_id, state.lang_id, stage_key, status)
        
        # Human-readable stage names
        stage_names = {
            'stage_translation': 'Translation',
            'stage_img_prompts': 'Image prompts generation',
            'stage_voiceover': 'Voiceover generation',
            'stage_subtitles': 'Subtitle generation',
            'stage_images': 'Image generation',
            'stage_montage': 'Video montage'
        }
        stage_name = stage_names.get(stage_key, stage_key)
        
        if status in ['success', 'warning']:
             log_level = LogLevel.SUCCESS if status == 'success' else LogLevel.WARNING
             logger.log(f"[{task_id}] ✅ {stage_name} completed", level=log_level)
        else: # error
            logger.log(f"[{task_id}] ❌ {stage_name} failed: {error_message}", level=LogLevel.ERROR)
        
        # Always check if the whole queue is finished after a status change.
        # This simplifies logic in error handlers.
        self.check_if_all_finished()

    # --- Pipeline Logic ---

    def _start_translation(self, task_id, all_languages_config):
        state = self.task_states[task_id]
        config = {
            'text': state.original_text,
            'lang_config': all_languages_config.get(state.lang_id, {})
        }
        self._start_worker(TranslationWorker, task_id, 'stage_translation', config, self._on_translation_finished, self._on_translation_error)

    @Slot(str, object)
    def _on_translation_finished(self, task_id, translated_text):
        state = self.task_states[task_id]
        state.text_for_processing = translated_text
        with open(os.path.join(state.dir_path, "translation.txt"), 'w', encoding='utf-8') as f:
            f.write(translated_text)
        self._set_stage_status(task_id, 'stage_translation', 'success')
        self._on_text_ready(task_id)

    @Slot(str, str)
    def _on_translation_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_translation', 'error', error)

    def _on_text_ready(self, task_id):
        state = self.task_states[task_id]
        if 'stage_img_prompts' in state.stages:
            self._start_image_prompts(task_id)
        if 'stage_voiceover' in state.stages:
            self._start_voiceover(task_id)
        # If neither of the above, check if finished
        if 'stage_img_prompts' not in state.stages and 'stage_voiceover' not in state.stages:
            self.check_if_all_finished()

    def _start_image_prompts(self, task_id):
        state = self.task_states[task_id]
        config = {
            'text': state.text_for_processing,
            'img_prompt_settings': self.settings.get("image_prompt_settings", {})
        }
        self._start_worker(ImagePromptWorker, task_id, 'stage_img_prompts', config, self._on_img_prompts_finished, self._on_img_prompts_error)

    @Slot(str, object)
    def _on_img_prompts_finished(self, task_id, prompts_text):
        state = self.task_states[task_id]
        state.image_prompts = prompts_text
        with open(os.path.join(state.dir_path, "image_prompts.txt"), 'w', encoding='utf-8') as f:
            f.write(prompts_text)
        self._set_stage_status(task_id, 'stage_img_prompts', 'success')
        
        if 'stage_images' in state.stages:
            self._start_image_generation(task_id)
        else:
            self.check_if_all_finished()

    @Slot(str, str)
    def _on_img_prompts_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_img_prompts', 'error', error)

    def _start_voiceover(self, task_id):
        state = self.task_states[task_id]
        lang_config = self.settings.get("languages_config", {}).get(state.lang_id, {})
        config = {
            'text': state.text_for_processing,
            'dir_path': state.dir_path,
            'lang_config': lang_config,
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
        
        if 'stage_subtitles' in state.stages:
            self._start_subtitles(task_id)
        else:
            self.check_if_all_finished()

    @Slot(str, str)
    def _on_voiceover_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_voiceover', 'error', error)
        if 'stage_subtitles' in self.task_states[task_id].stages:
            self._increment_subtitle_counter()

    def _start_subtitles(self, task_id):
        worker = self._create_subtitle_runnable(task_id)
        # This worker will only run when the semaphore is free
        self.threadpool.start(worker)

    def _create_subtitle_runnable(self, task_id):
        # Create a QRunnable that waits for the semaphore
        class SemaphoreRunnable(QRunnable):
            def __init__(self, processor, task_id):
                super().__init__()
                self.processor = processor
                self.task_id = task_id
            def run(self):
                self.processor.subtitle_semaphore.acquire()
                try:
                    state = self.processor.task_states[self.task_id]
                    sub_settings = self.processor.settings.get('subtitles', {})
                    whisper_type = sub_settings.get('whisper_type', 'amd')
                    model_name = sub_settings.get('whisper_model', 'base.bin')
                    
                    whisper_exe = None; whisper_model_path = model_name
                    if whisper_type == 'amd':
                        current_dir = os.path.dirname(os.path.abspath(__file__))
                        whisper_base_path = os.path.join(current_dir, "whisper-cli-amd")
                        whisper_exe = os.path.join(whisper_base_path, "main.exe")
                        whisper_model_path = os.path.join(whisper_base_path, model_name)
                    
                    config = {
                        'audio_path': state.audio_path, 'dir_path': state.dir_path,
                        'sub_settings': sub_settings, 'lang_code': state.lang_id.split('-')[0].lower(),
                        'whisper_exe': whisper_exe, 'whisper_model_path': whisper_model_path
                    }
                    self.processor._start_worker(SubtitleWorker, self.task_id, 'stage_subtitles', config,
                                                 self.processor._on_subtitles_finished, self.processor._on_subtitles_error)
                except Exception as e:
                    # If creating the worker fails, we must handle the error and release the semaphore
                    self.processor._on_subtitles_error(self.task_id, f"Failed to start subtitle worker: {e}")

        return SemaphoreRunnable(self, task_id)
        
    @Slot(str, object)
    def _on_subtitles_finished(self, task_id, subtitle_path):
        self.subtitle_semaphore.release()
        self.task_states[task_id].subtitle_path = subtitle_path
        self._set_stage_status(task_id, 'stage_subtitles', 'success')
        self._increment_subtitle_counter()

    @Slot(str, str)
    def _on_subtitles_error(self, task_id, error):
        self.subtitle_semaphore.release()
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
        googler_settings = self.settings.get('googler', {})
        config = {
            'prompts_text': state.image_prompts,
            'dir_path': state.dir_path,
            'provider': self.settings.get('image_generation_provider', 'pollinations'),
            'api_kwargs': {
                'aspect_ratio': googler_settings.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE'),
                'seed': googler_settings.get('seed'),
                'negative_prompt': googler_settings.get('negative_prompt')
            }
        }
        self._start_worker(ImageGenerationWorker, task_id, 'stage_images', config, self._on_img_generation_finished, self._on_img_generation_error)

    @Slot(str, object)
    def _on_img_generation_finished(self, task_id, result_dict):
        generated_paths = result_dict.get('paths', [])
        total_prompts = result_dict.get('total_prompts', 0)
        
        status = 'error'
        # Ensure total_prompts is not zero to avoid division by zero or success on empty prompts
        if total_prompts > 0 and len(generated_paths) == total_prompts:
            status = 'success'
        elif len(generated_paths) > 0:
            status = 'warning'

        self.task_states[task_id].image_paths = generated_paths
        self._set_stage_status(task_id, 'stage_images', status, "Failed to generate all images." if status != 'success' else None)

        # After image status is set, check if the subtitle barrier has passed and if we can start montages.
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    @Slot(str, str)
    def _on_img_generation_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_images', 'error', error)

    def _check_and_start_montages(self):
        for task_id, state in self.task_states.items():
            if 'stage_montage' in state.stages and state.status.get('stage_montage') == 'pending':
                # Check if all prerequisites are met
                subtitles_done = 'stage_subtitles' not in state.stages or state.status.get('stage_subtitles') in ('success', 'warning')
                images_done = 'stage_images' not in state.stages or state.status.get('stage_images') in ('success', 'warning')
                
                if subtitles_done and images_done:
                    # Montage can proceed if audio and subtitles are OK, and there's at least one image
                    if state.status.get('stage_subtitles') == 'success' and state.status.get('stage_images') in ('success', 'warning'):
                        # SET PROCESSING STATUS SYNCHRONOUSLY TO PREVENT DOUBLE START
                        self.stage_status_changed.emit(state.job_id, state.lang_id, 'stage_montage', 'processing')
                        state.status['stage_montage'] = 'processing'
                        self._start_montage(task_id)
                    else:
                        self._set_stage_status(task_id, 'stage_montage', 'error', "Prerequisites failed.")
                        self.check_if_all_finished()


    def _start_montage(self, task_id):
        # Wrapper to handle semaphore
        class MontageRunnable(QRunnable):
            def __init__(self, processor, task_id):
                super().__init__()
                self.processor = processor
                self.task_id = task_id
            def run(self):
                self.processor.montage_semaphore.acquire()
                try:
                    state = self.processor.task_states[self.task_id]
                    safe_task_name = "".join(c for c in state.job_name if c.isalnum() or c in (' ', '_')).strip()
                    safe_lang_name = "".join(c for c in state.lang_name if c.isalnum() or c in (' ', '_')).strip()
                    output_filename = f"{safe_task_name}_{safe_lang_name}.mp4"
                    output_path = os.path.join(state.dir_path, output_filename)
                    
                    config = {
                        'visual_files': state.image_paths, 'audio_path': state.audio_path,
                        'output_path': output_path, 'ass_path': state.subtitle_path,
                        'settings': self.processor.settings.get("montage", {})
                    }
                    worker = MontageWorker(self.task_id, config)
                    worker.signals.finished.connect(self.processor._on_montage_finished)
                    worker.signals.error.connect(self.processor._on_montage_error)
                    # Connect progress_log to emit job_id based signal
                    worker.signals.progress_log.connect(self.processor._on_montage_progress)
                    self.processor.stage_status_changed.emit(state.job_id, state.lang_id, 'stage_montage', 'processing')
                    self.processor.threadpool.start(worker)
                except Exception as e:
                    self.processor._on_montage_error(self.task_id, f"Failed to start montage worker: {e}")

        self.threadpool.start(MontageRunnable(self, task_id))


    @Slot(str, object)
    def _on_montage_finished(self, task_id, video_path):
        self.montage_semaphore.release()
        self.task_states[task_id].final_video_path = video_path
        self._set_stage_status(task_id, 'stage_montage', 'success')

    @Slot(str, str)
    def _on_montage_error(self, task_id, error):
        self.montage_semaphore.release()
        self._set_stage_status(task_id, 'stage_montage', 'error', error)
    
    @Slot(str, str)
    def _on_montage_progress(self, task_id, message):
        """Handle FFmpeg progress messages (card-only logs)"""
        # Extract job_id from task_id (Task-1_uk-UK -> Task-1)
        job_id = task_id.split('_')[0] if '_' in task_id else task_id
        self.task_progress_log.emit(job_id, message)

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
