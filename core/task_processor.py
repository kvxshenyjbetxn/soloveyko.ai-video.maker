import os
import re
import time
import base64
import json
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer, QSemaphore, Qt
from utils.logger import logger, LogLevel
from utils.settings import settings_manager
from api.openrouter import OpenRouterAPI
from api.pollinations import PollinationsAPI
from api.googler import GooglerAPI
from api.elevenlabs import ElevenLabsAPI
from api.voicemaker import VoicemakerAPI

class TaskProcessor(QObject):
    processing_finished = Signal(str)
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status
    image_generated = Signal(str, str, str) # task_name, image_path, prompt

    def __init__(self, queue_manager):
        super().__init__()
        self.queue_manager = queue_manager
        self.settings = settings_manager
        self.openrouter_api = OpenRouterAPI()
        self.pollinations_api = PollinationsAPI()
        self.googler_api = GooglerAPI()
        self.elevenlabs_api = ElevenLabsAPI()
        self.voicemaker_api = VoicemakerAPI()
        self.threadpool = QThreadPool()
        self.active_jobs = 0
        self.timer = QElapsedTimer()
        self.voicemaker_voices = []
        self.load_voicemaker_voices()

    def load_voicemaker_voices(self):
        try:
            with open("assets/voicemaker_voices.json", "r", encoding="utf-8") as f:
                self.voicemaker_voices = json.load(f)
        except Exception as e:
            logger.log(f"Error loading voicemaker voices: {e}", level=LogLevel.ERROR)
            self.voicemaker_voices = []

    def start_processing(self):
        jobs = self.queue_manager.get_tasks()
        if not jobs:
            logger.log("Queue is empty. Nothing to process.", level=LogLevel.INFO)
            return

        self.active_jobs = len(jobs)
        self.timer.start()
        logger.log(f"Starting to process {self.active_jobs} jobs in the queue.", level=LogLevel.INFO)
        
        for job in jobs:
            worker = JobWorker(self, self.settings, job)
            worker.signals.finished.connect(self.job_finished)
            # Propagate the stage status signal
            worker.signals.stage_status_changed.connect(self.stage_status_changed)
            self.threadpool.start(worker)

    def job_finished(self):
        self.active_jobs -= 1
        if self.active_jobs == 0:
            elapsed_ms = self.timer.elapsed()
            elapsed_str = time.strftime('%H:%M:%S', time.gmtime(elapsed_ms / 1000))
            logger.log(f"Queue processing finished in {elapsed_str}.", level=LogLevel.SUCCESS)
            self.processing_finished.emit(elapsed_str)
            
    def _get_voicemaker_language_code(self, voice_id):
        for lang_data in self.voicemaker_voices:
            if voice_id in lang_data.get("Voices", []):
                return lang_data.get("LanguageCode")
        return "en-US" # Default fallback

class JobWorkerSignals(QObject):
    finished = Signal()
    stage_status_changed = Signal(str, str, str, str) # job_id, lang_id, stage_key, status

class ImageGenerationWorkerSignals(QObject):
    finished = Signal(int, bool) # index, success

class ImageGenerationWorker(QRunnable):
    def __init__(self, processor, task_name, api, prompt, index, images_dir, file_extension, provider, **kwargs):
        super().__init__()
        self.processor = processor
        self.task_name = task_name
        self.api = api
        self.prompt = prompt
        self.index = index
        self.images_dir = images_dir
        self.file_extension = file_extension
        self.provider = provider
        self.kwargs = kwargs
        self.signals = ImageGenerationWorkerSignals()

    def run(self):
        try:
            image_data = self.api.generate_image(self.prompt, **self.kwargs)
            
            if not image_data:
                logger.log(f"      - Failed to generate image {self.index + 1} for prompt: '{self.prompt}' (no data returned).", level=LogLevel.WARNING)
                self.signals.finished.emit(self.index, False)
                return

            # --- Saving Logic moved inside the worker ---
            image_path = os.path.join(self.images_dir, f"{self.index + 1}.{self.file_extension}")
            
            data_to_write = image_data
            if self.provider == 'googler' and isinstance(image_data, str):
                if "," in image_data:
                    _, encoded = image_data.split(",", 1)
                    data_to_write = base64.b64decode(encoded)
                else:
                    data_to_write = base64.b64decode(image_data)
            
            with open(image_path, 'wb') as f:
                f.write(data_to_write)
                
            logger.log(f"      - Successfully saved image {self.index + 1} to {image_path}", level=LogLevel.SUCCESS)
            self.processor.image_generated.emit(self.task_name, image_path, self.prompt) # Emit signal for the gallery
            self.signals.finished.emit(self.index, True)

        except Exception as e:
            logger.log(f"      - Failed to generate or save image {self.index + 1} for prompt '{self.prompt}'. Error: {e}", level=LogLevel.ERROR)
            self.signals.finished.emit(self.index, False)

class JobWorker(QRunnable):
    def __init__(self, processor, settings, job):
        super().__init__()
        self.processor = processor
        self.openrouter_api = processor.openrouter_api
        self.pollinations_api = processor.pollinations_api
        self.googler_api = processor.googler_api
        self.elevenlabs_api = processor.elevenlabs_api
        self.voicemaker_api = processor.voicemaker_api
        self.settings = settings
        self.job = job
        self.signals = JobWorkerSignals()

    def run(self):
        try:
            logger.log(f"Processing job: {self.job['name']}", level=LogLevel.INFO)
            job_id = self.job['id']
            
            base_save_path = self.settings.get('results_path')
            if not base_save_path:
                logger.log("Warning: 'results_path' is not configured in settings. Results will not be saved.", level=LogLevel.WARNING)

            all_languages_config = self.settings.get("languages_config", {})
            
            for lang_id, lang_data in self.job['languages'].items():
                lang_dir_path = self._get_save_path(base_save_path, self.job['name'], lang_data['display_name'])
                
                text_for_processing = self.job['text']
                
                # --- Translation Stage ---
                if 'stage_translation' in lang_data['stages']:
                    translated_text = self._perform_translation(job_id, lang_id, lang_data, all_languages_config)
                    if translated_text:
                        text_for_processing = translated_text
                        if lang_dir_path:
                            self.save_translation(lang_dir_path, translated_text)
                
                # --- Image Prompts Stage ---
                if 'stage_img_prompts' in lang_data['stages']:
                    self.process_image_prompts(job_id, lang_id, lang_data, text_for_processing, lang_dir_path)

                # --- Image Generation Stage ---
                if 'stage_images' in lang_data['stages']:
                    self.process_image_generation(job_id, lang_id, lang_data, lang_dir_path)
                
                # --- Voiceover Stage ---
                if 'stage_voiceover' in lang_data['stages']:
                    self._process_voiceover_stage(job_id, lang_id, lang_data, all_languages_config, text_for_processing, lang_dir_path)

            logger.log(f"Finished processing job: {self.job['name']}", level=LogLevel.INFO)
        finally:
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
            logger.log(f"    - Failed to create save directory. Error: {e}", level=LogLevel.ERROR)
            return None

    def _perform_translation(self, job_id, lang_id, lang_data, all_languages_config):
        stage_key = 'stage_translation'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')
        
        lang_config = all_languages_config.get(lang_id)
        if not lang_config:
            logger.log(f"  - Skipping translation for {lang_data['display_name']}: Configuration not found.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None
        
        logger.log(f"  - Starting translation for: {lang_data['display_name']}", level=LogLevel.INFO)
        
        prompt = lang_config.get('prompt', '')
        model = lang_config.get('model', '')
        max_tokens = lang_config.get('max_tokens', 4096)
        
        if not model:
            logger.log(f"  - Skipping translation for {lang_data['display_name']}: Model not configured.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None
        
        full_prompt = f"{prompt}\n\n{self.job['text']}"
        
        try:
            logger.log(f"    - Calling model '{model}' with max_tokens={max_tokens}.", level=LogLevel.INFO)
            response = self.openrouter_api.get_chat_completion(
                model=model,
                messages=[{"role": "user", "content": full_prompt}],
                max_tokens=max_tokens
            )
            
            if response and response['choices'][0]['message']['content']:
                translated_text = response['choices'][0]['message']['content']
                logger.log(f"    - Translation successful for {lang_data['display_name']}.", level=LogLevel.SUCCESS)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
                return translated_text
            else:
                logger.log(f"    - Translation failed for {lang_data['display_name']}: Empty or invalid response from API.", level=LogLevel.ERROR)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
                return None

        except Exception as e:
            logger.log(f"    - An error occurred during translation for {lang_data['display_name']}: {e}", level=LogLevel.ERROR)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return None

    def _process_voiceover_stage(self, job_id, lang_id, lang_data, all_languages_config, text_to_voice, dir_path):
        stage_key = 'stage_voiceover'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')

        lang_config = all_languages_config.get(lang_id)
        if not lang_config:
            logger.log(f"    - Skipping voiceover for {lang_data['display_name']}: Lang config not found.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        tts_provider = lang_config.get('tts_provider', 'ElevenLabs')
        logger.log(f"  - Starting voiceover for: {lang_data['display_name']} using {tts_provider}", level=LogLevel.INFO)

        if not dir_path:
            logger.log(f"    - Skipping voiceover for {lang_data['display_name']}: Results path not set.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        # tts_provider variable is already set above

        if tts_provider == 'VoiceMaker':
            voice_id = lang_config.get('voicemaker_voice_id')
            if not voice_id:
                logger.log(f"    - Skipping voiceover for {lang_data['display_name']}: VoiceMaker Voice ID not configured.", level=LogLevel.WARNING)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
                return
            
            try:
                language_code = self.processor._get_voicemaker_language_code(voice_id)
                audio_content, status = self.voicemaker_api.generate_audio(text_to_voice, voice_id, language_code, temp_dir=dir_path)
                
                if status == 'success' and audio_content:
                    self.save_voiceover(dir_path, audio_content)
                    self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
                else:
                    logger.log(f"    - VoiceMaker generation failed: {status}", level=LogLevel.ERROR)
                    self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

            except Exception as e:
                logger.log(f"    - An error occurred during VoiceMaker voiceover for {lang_data['display_name']}: {e}", level=LogLevel.ERROR)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

        else: # ElevenLabs
            if not lang_config.get('elevenlabs_template_uuid'):
                logger.log(f"    - Skipping voiceover for {lang_data['display_name']}: ElevenLabs template not configured.", level=LogLevel.WARNING)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
                return

            template_uuid = lang_config['elevenlabs_template_uuid']
            
            try:
                task_id, status = self.elevenlabs_api.create_task(text_to_voice, template_uuid)
                if status != 'connected' or not task_id:
                    raise Exception("Failed to create task.")

                # Polling for result
                for _ in range(30): # 5 minutes timeout (30 * 10 seconds)
                    task_status, status = self.elevenlabs_api.get_task_status(task_id)
                    if status != 'connected':
                        raise Exception("Failed to get task status.")
                    
                    if task_status in ['ending', 'ending_processed']:
                        audio_content, status = self.elevenlabs_api.get_task_result(task_id)
                        if status == 'connected' and audio_content:
                            self.save_voiceover(dir_path, audio_content)
                            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
                            return
                        elif status == 'not_ready':
                            time.sleep(10)
                            continue
                        else:
                            raise Exception("Failed to download audio content.")
                    
                    elif task_status in ['error', 'error_handled']:
                        raise Exception("Task processing resulted in an error.")
                    
                    time.sleep(10)

                raise Exception("Timeout while waiting for voiceover result.")

            except Exception as e:
                logger.log(f"    - An error occurred during voiceover for {lang_data['display_name']}: {e}", level=LogLevel.ERROR)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

    def process_image_prompts(self, job_id, lang_id, lang_data, text_to_use, dir_path):
        stage_key = 'stage_img_prompts'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')
        logger.log(f"  - Starting image prompt generation for: {lang_data['display_name']}", level=LogLevel.INFO)
        
        img_prompt_settings = self.settings.get("image_prompt_settings", {})
        prompt_template = img_prompt_settings.get('prompt', '')
        model = img_prompt_settings.get('model', '')
        max_tokens = img_prompt_settings.get('max_tokens', 4096)

        if not model or not prompt_template:
            logger.log(f"  - Skipping image prompt generation for {lang_data['display_name']}: Model or prompt template not configured.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        full_prompt = f"{prompt_template}\n\n{text_to_use}"

        try:
            logger.log(f"    - Calling model '{model}' for image prompts with max_tokens={max_tokens}.", level=LogLevel.INFO)
            response = self.openrouter_api.get_chat_completion(
                model=model,
                messages=[{"role": "user", "content": full_prompt}],
                max_tokens=max_tokens
            )

            if response and response['choices'][0]['message']['content']:
                image_prompts_text = response['choices'][0]['message']['content']
                logger.log(f"    - Image prompt generation successful for {lang_data['display_name']}.", level=LogLevel.SUCCESS)
                
                if dir_path:
                    self.save_image_prompts(dir_path, image_prompts_text)
                
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
            else:
                logger.log(f"    - Image prompt generation failed for {lang_data['display_name']}: Empty or invalid response from API.", level=LogLevel.ERROR)
                self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

        except Exception as e:
            logger.log(f"    - An error occurred during image prompt generation for {lang_data['display_name']}: {e}", level=LogLevel.ERROR)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')

    def _run_image_generation_batch(self, prompts_to_process, api, thread_pool, images_dir, file_extension, provider, **kwargs):
        results = {}
        if not prompts_to_process:
            return results

        semaphore = QSemaphore(0)
        
        def on_worker_finished(index, success_bool):
            results[index] = success_bool
            semaphore.release()

        for index, prompt in prompts_to_process:
            worker = ImageGenerationWorker(self.processor, self.job['name'], api, prompt, index, images_dir, file_extension, provider, **kwargs)
            worker.signals.finished.connect(on_worker_finished, Qt.DirectConnection)
            thread_pool.start(worker)
        
        semaphore.acquire(len(prompts_to_process))
        return results

    def process_image_generation(self, job_id, lang_id, lang_data, lang_dir_path):
        stage_key = 'stage_images'
        self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'processing')
        logger.log(f"  - Starting image generation for: {lang_data['display_name']}", level=LogLevel.INFO)

        if not lang_dir_path:
            logger.log(f"    - Skipping image generation: No save path defined.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        prompts_file_path = os.path.join(lang_dir_path, "image_prompts.txt")
        if not os.path.exists(prompts_file_path):
            logger.log(f"    - Skipping image generation: 'image_prompts.txt' not found.", level=LogLevel.WARNING)
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            return

        with open(prompts_file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        prompts = re.findall(r"^\d+\.\s*(.*)", content, re.MULTILINE)
        logger.log(f"    - Found {len(prompts)} prompts to process.", level=LogLevel.INFO)

        images_dir = os.path.join(lang_dir_path, "images")
        os.makedirs(images_dir, exist_ok=True)

        provider = self.settings.get('image_generation_provider', 'pollinations')
        api_kwargs = {}
        max_threads = 1
        file_extension = 'png'

        if provider == 'googler':
            api = self.googler_api
            googler_settings = self.settings.get('googler', {})
            max_threads = googler_settings.get('max_threads', 1)
            api_kwargs['aspect_ratio'] = googler_settings.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE')
            api_kwargs['seed'] = googler_settings.get('seed')
            api_kwargs['negative_prompt'] = googler_settings.get('negative_prompt')
            file_extension = 'jpg'
        else: # pollinations
            api = self.pollinations_api
            max_threads = 1
            
        thread_pool = self.processor.threadpool
        all_results = {}
        
        # --- First Pass ---
        prompts_to_process = list(enumerate(prompts))
        for i in range(0, len(prompts_to_process), max_threads):
            batch = prompts_to_process[i:i+max_threads]
            logger.log(f"    - Processing batch of {len(batch)} images...", level=LogLevel.INFO)
            batch_results = self._run_image_generation_batch(batch, api, thread_pool, images_dir, file_extension, provider, **api_kwargs)
            all_results.update(batch_results)

        # --- Retry Pass ---
        failed_prompts = [(idx, prompt) for idx, prompt in prompts_to_process if not all_results.get(idx)]
        if failed_prompts:
            logger.log(f"    - Retrying {len(failed_prompts)} failed prompts...", level=LogLevel.WARNING)
            for i in range(0, len(failed_prompts), max_threads):
                batch = failed_prompts[i:i+max_threads]
                logger.log(f"    - Processing retry batch of {len(batch)} images...", level=LogLevel.INFO)
                retry_results = self._run_image_generation_batch(batch, api, thread_pool, images_dir, file_extension, provider, **api_kwargs)
                all_results.update(retry_results)

        # --- Final Status Check ---
        final_fail_count = len([res for res in all_results.values() if not res])
        
        if final_fail_count == 0:
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'success')
            logger.log(f"  - Image generation completed successfully for: {lang_data['display_name']}", level=LogLevel.SUCCESS)
        else:
            self.signals.stage_status_changed.emit(job_id, lang_id, stage_key, 'error')
            logger.log(f"  - Image generation completed with {final_fail_count} errors for: {lang_data['display_name']}", level=LogLevel.ERROR)

    def save_translation(self, dir_path, content):
        try:
            file_path = os.path.join(dir_path, "translation.txt")
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            logger.log(f"    - Translation result saved to: {file_path}", level=LogLevel.INFO)
        except Exception as e:
            logger.log(f"    - Failed to save translation result. Error: {e}", level=LogLevel.ERROR)

    def save_image_prompts(self, dir_path, content):
        try:
            file_path = os.path.join(dir_path, "image_prompts.txt")
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            logger.log(f"    - Image prompts saved to: {file_path}", level=LogLevel.INFO)
        except Exception as e:
            logger.log(f"    - Failed to save image prompts. Error: {e}", level=LogLevel.ERROR)

    def save_voiceover(self, dir_path, content):
        try:
            file_path = os.path.join(dir_path, "voice.mp3")
            with open(file_path, 'wb') as f:
                f.write(content)
            logger.log(f"    - Voiceover audio saved to: {file_path}", level=LogLevel.SUCCESS)
        except Exception as e:
            logger.log(f"    - Failed to save voiceover audio. Error: {e}", level=LogLevel.ERROR)