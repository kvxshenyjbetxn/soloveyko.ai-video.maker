import os
import sys
import re
import time
import platform
import base64
import json
import traceback
import threading
import shutil
import collections
from concurrent.futures import ThreadPoolExecutor
from PySide6.QtCore import QObject, Signal, QRunnable, QThreadPool, QElapsedTimer, QSemaphore, Slot, QMutex, Qt

from utils.youtube_downloader import YouTubeDownloader
from utils.logger import logger, LogLevel
from api.openrouter import OpenRouterAPI
from api.pollinations import PollinationsAPI
from api.googler import GooglerAPI
from api.elevenlabs import ElevenLabsAPI
from api.elevenlabs_unlim import ElevenLabsUnlimAPI
from api.voicemaker import VoicemakerAPI
from api.gemini_tts import GeminiTTSAPI
from api.edge_tts_api import EdgeTTSAPI
from api.elevenlabs_image import ElevenLabsImageAPI
from core.subtitle_engine import SubtitleEngine
from core.montage_engine import MontageEngine
from core.statistics_manager import statistics_manager

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
    metadata_updated = Signal(str, str, str) # task_id, stage_key, metadata_text

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
        api_key = self.config.get('openrouter_api_key')
        api = OpenRouterAPI(api_key=api_key)
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
        api_key = self.config.get('openrouter_api_key')
        api = OpenRouterAPI(api_key=api_key)
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
        if response and response.get('choices') and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            # Count prompts
            prompts = re.findall(r"^\d+\.\s*(.*)", result, re.MULTILINE)
            prompts_count = len(prompts)
            logger.log(f"[{self.task_id}] [{model}] Image prompts generated ({prompts_count} prompts)", level=LogLevel.SUCCESS)
            return result
        else:
            raise Exception(f"Empty or invalid response from image prompt API. Response: {response}")

class PreviewWorker(BaseWorker):
    def do_work(self):
        api_key = self.config.get('openrouter_api_key')
        api = OpenRouterAPI(api_key=api_key)
        preview_settings = self.config['preview_settings']
        model = preview_settings.get('model', 'unknown')
        temp = preview_settings.get('temperature', 1.0)
        max_tokens = preview_settings.get('max_tokens', 10000)
        
        logger.log(f"[{self.task_id}] [{model}] Starting preview prompts generation (temp: {temp}, max_tokens: {max_tokens})", level=LogLevel.INFO)
        
        template = preview_settings.get('prompt', '')
        full_prompt = template.replace('{story}', self.config.get('story', '')).replace('{title}', self.config.get('title', ''))
        
        response = api.get_chat_completion(
            model=model,
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=max_tokens,
            temperature=temp
        )
        if response and response.get('choices') and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            # Count prompts
            prompts = re.findall(r"^\d+\.\s*(.*)", result, re.MULTILINE)
            prompts_count = len(prompts)
            logger.log(f"[{self.task_id}] [{model}] Preview prompts generated ({prompts_count} prompts)", level=LogLevel.SUCCESS)
            return result
        else:
            raise Exception(f"Empty or invalid response from preview prompt API. Response: {response}")

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
            
            def progress_callback(msg):
                self.signals.progress_log.emit(self.task_id, msg)

            audio_content, status = api.generate_audio(text, voice_id, language_code, temp_dir=dir_path, progress_callback=progress_callback)
            if status == 'success' and audio_content:
                return self.save_audio(audio_content, "voice.mp3")
            else:
                raise Exception(f"VoiceMaker generation failed: {status}")

        elif tts_provider == 'GeminiTTS':
            api_key = self.config.get('gemini_tts_api_key')
            api = GeminiTTSAPI(api_key=api_key)
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
            
        elif tts_provider == 'EdgeTTS':
            api = EdgeTTSAPI()
            voice = lang_config.get('edgetts_voice')
            if not voice:
                voice = "en-US-ChristopherNeural" # Fallback
            
            rate = lang_config.get('edgetts_rate', 0)
            pitch = lang_config.get('edgetts_pitch', 0)
            
            filename = "voice.mp3"
            output_path = os.path.join(dir_path, filename)
            
            success, msg = api.generate_audio(text, voice, rate, pitch, output_path)
            if success:
                logger.log(f"[{self.task_id}] [EdgeTTS] Voiceover saved", level=LogLevel.SUCCESS)
                return output_path
            else:
                raise Exception(f"EdgeTTS generation failed: {msg}")



        elif tts_provider == 'ElevenLabsUnlim':
            api_key = self.config.get('elevenlabs_unlim_api_key')
            api = ElevenLabsUnlimAPI(api_key=api_key)
            unlim_settings = lang_config.get('eleven_unlim_settings', {})

            # Retry logic for task creation
            task_id = None
            last_error = "Unknown error"
            
            for attempt in range(3):
                try:
                    task_id, status = api.create_task(text, unlim_settings)
                    if status == 'connected' and task_id:
                        break
                    else:
                        last_error = "Failed to obtain valid task_id"
                        logger.log(f"[{self.task_id}] Attempt {attempt+1}/3 failed to create ElevenLabsUnlim task. Retrying...", level=LogLevel.WARNING)
                except Exception as e:
                    last_error = str(e)
                    logger.log(f"[{self.task_id}] Attempt {attempt+1}/3 raised exception: {e}", level=LogLevel.WARNING)
                time.sleep(5)
                
            if not task_id:
                raise Exception(f"Failed to create ElevenLabsUnlim task after 3 attempts. Last error: {last_error}")
            
            while True:
                task_status, status = api.get_task_status(task_id)
                if status != 'connected':
                    logger.log(f"[{self.task_id}] Weak connection getting status for {task_id}, retrying...", level=LogLevel.WARNING)
                    time.sleep(5)
                    continue

                if task_status == 'completed':
                    audio_content, status = api.get_task_result(task_id)
                    if status == 'connected' and audio_content:
                        return self.save_audio(audio_content, "voice.mp3")
                    else:
                         raise Exception("Failed to download ElevenLabsUnlim audio.")
                elif task_status == 'failed' or task_status == 'error': 
                    raise Exception(f"ElevenLabsUnlim task failed (Status: {task_status}).")
                
                time.sleep(10)

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
                elif task_status in ['error', 'error_handled', 'error_handling']:
                    raise Exception(f"ElevenLabs task processing resulted in an error (Status: {task_status}).")
                
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
        
        api_key = self.config.get('openrouter_api_key')
        api = OpenRouterAPI(api_key=api_key)
        
        prompt = self.config['prompt']
        text = self.config['text']
        model = self.config.get('model', 'unknown') 
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
        # Try to find numbered prompts first (1. ..., 2. ...)
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        if not prompts:
            # If no numbers, take each non-empty line as a prompt
            prompts = [line.strip() for line in prompts_text.split('\n') if line.strip()]
            if prompts:
                logger.log(f"[{self.task_id}] No numbered prompts found. Parsing {len(prompts)} prompts by lines.", level=LogLevel.INFO)

        if not prompts:
            raise Exception("No prompts found in the generated text.")

        provider = self.config['provider']
        images_dir = os.path.join(self.config['dir_path'], "images")
        os.makedirs(images_dir, exist_ok=True)
        
        if provider == 'googler':
            file_extension = 'jpg'
            api_key = self.config.get('api_key')
            shared_api = GooglerAPI(api_key=api_key)
            api_kwargs = self.config['api_kwargs']
        elif provider == 'elevenlabs_image':
            file_extension = 'jpg' # Assuming jpg
            api_key = self.config.get('api_key')
            shared_api = ElevenLabsImageAPI(api_key=api_key)
            api_kwargs = self.config['api_kwargs']
        else: # pollinations
            file_extension = 'png'
            shared_api = PollinationsAPI()
            api_kwargs = self.config['api_kwargs'] # Pass settings to generate_image
        
        service_name = provider.capitalize()
        generated_paths = [None] * len(prompts)
        
        def generate_single_image(index, prompt):
            """Generate a single image and return its data"""
            semaphore = self.config.get('semaphore')
            try:
                if semaphore:
                    semaphore.acquire()
                
                logger.log(f"[{self.task_id}] [{service_name}] Generating image {index + 1}/{len(prompts)}", level=LogLevel.INFO)
                image_data = shared_api.generate_image(prompt, **api_kwargs)

                if not image_data:
                    logger.log(f"[{self.task_id}] [{service_name}] Failed to generate image {index + 1}/{len(prompts)} (no data)", level=LogLevel.WARNING)
                    return None
                
                return (index, image_data, prompt)
            except Exception as e:
                logger.log(f"[{self.task_id}] [{service_name}] Error generating image {index + 1}: {e}", level=LogLevel.ERROR)
                return None
            finally:
                if semaphore:
                    semaphore.release()
        
        if provider == 'pollinations':
            # Sequential processing for Pollinations to avoid rate limits and 429 errors
            for i, prompt in enumerate(prompts):
                result = generate_single_image(i, prompt)
                if result:
                    index_from_result, image_data, prompt_from_result = result
                    image_path = os.path.join(images_dir, f"{index_from_result + 1}.{file_extension}")
                    
                    try:
                        with open(image_path, 'wb') as f:
                            f.write(image_data)
                        
                        logger.log(f"[{self.task_id}] [{service_name}] Image {index_from_result + 1}/{len(prompts)} saved", level=LogLevel.SUCCESS)
                        generated_paths[index_from_result] = image_path
                        
                        self.signals.status_changed.emit(self.task_id, image_path, prompt_from_result, image_path)
                    except IOError as e:
                        logger.log(f"[{self.task_id}] [{service_name}] Error saving image {index_from_result + 1}: {e}", level=LogLevel.ERROR)
                
                # Small delay between sequential requests for safety
                time.sleep(0.5)
        else:
            # Parallel processing for Googler
            max_workers = self.config.get('max_threads', 8)
            prompts_iterator = iter(enumerate(prompts))
            futures = {}
            prompts_exhausted = False

            while True:
                # Check if any tasks completed
                if futures:
                    from concurrent.futures import wait, FIRST_COMPLETED
                    done_set, _ = wait(futures.keys(), timeout=0, return_when=FIRST_COMPLETED)
                    
                    for done_future in done_set:
                        index = futures.pop(done_future)
                        result = done_future.result()

                        if result:
                            index_from_result, image_data, prompt_from_result = result
                            image_path = os.path.join(images_dir, f"{index_from_result + 1}.{file_extension}")
                            
                            try:
                                # Googler provides base64 as string, we need to decode it
                                if isinstance(image_data, str):
                                    data_to_write = base64.b64decode(image_data.split(",", 1)[1] if "," in image_data else image_data)
                                else:
                                    data_to_write = image_data

                                with open(image_path, 'wb') as f:
                                    f.write(data_to_write)
                                
                                logger.log(f"[{self.task_id}] [{service_name}] Image {index_from_result + 1}/{len(prompts)} saved", level=LogLevel.SUCCESS)
                                generated_paths[index_from_result] = image_path
                                
                                self.signals.status_changed.emit(self.task_id, image_path, prompt_from_result, image_path)
                            except Exception as e:
                                logger.log(f"[{self.task_id}] [{service_name}] Error processing/saving image {index_from_result + 1}: {e}", level=LogLevel.ERROR)
                
                # Submit new tasks if slots available
                if len(futures) < max_workers and not prompts_exhausted:
                    try:
                        i, prompt = next(prompts_iterator)
                        future = executor.submit(generate_single_image, i, prompt)
                        futures[future] = i
                        time.sleep(0.5)
                    except StopIteration:
                        prompts_exhausted = True
                
                if prompts_exhausted and not futures:
                    break
                
                if len(futures) >= max_workers or (prompts_exhausted and futures):
                    time.sleep(0.1)


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

class DownloadWorker(BaseWorker):
    def do_work(self):
        url = self.config['url']
        dir_path = self.config['dir_path']
        yt_dlp_path = self.config['yt_dlp_path']
        download_semaphore = self.config['download_semaphore']
        
        logger.log(f"[{self.task_id}] Queuing download for {url}", level=LogLevel.INFO)
        
        try:
            download_semaphore.acquire()
            logger.log(f"[{self.task_id}] Starting download...", level=LogLevel.INFO)
            
            def report_progress(percent_str):
                 self.signals.metadata_updated.emit(self.task_id, 'stage_download', percent_str)

            return YouTubeDownloader.download_audio(url, dir_path, yt_dlp_path, progress_callback=report_progress)

        finally:
            download_semaphore.release()

class TranscriptionWorker(BaseWorker):
    def do_work(self):
        audio_path = self.config['audio_path']
        sub_settings = self.config['sub_settings']
        lang_code = self.config['lang_code']
        whisper_exe = self.config['whisper_exe']
        whisper_model_path = self.config['whisper_model_path']

        logger.log(f"[{self.task_id}] Starting transcription for rewrite...", level=LogLevel.INFO)
        
        engine = SubtitleEngine(whisper_exe, whisper_model_path)
        text = engine.transcribe_text(audio_path, sub_settings, language=lang_code)
        
        if not text:
            raise Exception("Transcription yielded empty text.")
            
        return text

class RewriteWorker(BaseWorker):
    def do_work(self):
        api_key = self.config.get('openrouter_api_key')
        api = OpenRouterAPI(api_key=api_key)
        
        prompt = self.config['prompt']
        text = self.config['text']
        model = self.config.get('model', 'unknown')
        max_tokens = self.config.get('max_tokens', 4096)
        temperature = self.config.get('temperature', 0.7)
        
        logger.log(f"[{self.task_id}] [{model}] Starting rewrite (temp: {temperature}, tokens: {max_tokens})", level=LogLevel.INFO)
        
        full_prompt = f"{prompt}\n\n{text}"
        response = api.get_chat_completion(
            model=model,
            messages=[{"role": "user", "content": full_prompt}],
            max_tokens=max_tokens,
            temperature=temperature
        )
        
        if response and response['choices'][0]['message']['content']:
            result = response['choices'][0]['message']['content']
            logger.log(f"[{self.task_id}] [{model}] Rewrite completed", level=LogLevel.SUCCESS)
            return result
        else:
            raise Exception(f"Empty or invalid response from API for model '{model}'.")
