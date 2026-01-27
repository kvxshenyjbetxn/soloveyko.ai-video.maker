import os
import sys
import json
import time
from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from core.workers import VoiceoverWorker, SubtitleWorker, TranscriptionWorker

class SubtitleMixin:
    """
    Mixin for TaskProcessor to handle Voiceover, Subtitles, and Transcription.
    Requires: self.task_states, self.settings,              self.elevenlabs_queue, self.elevenlabs_active_count,
              self.elevenlabs_unlim_queue, self.elevenlabs_unlim_active_count,
              self.edgetts_queue, self.edgetts_active_count,
              self.whisper_queue, self.subtitle_semaphore, self.subtitle_lock,
              self.completed_subtitle_tasks, self.total_subtitle_tasks, self.subtitle_barrier_passed,
              self._start_worker, self._set_stage_status, self.stage_metadata_updated,
              self.check_if_all_finished, self._check_and_start_montages
    """

    def _get_base_path(self):
        if getattr(sys, 'frozen', False):
            return sys._MEIPASS
        else:
            # core/mixins/subtitle_mixin.py -> core/mixins -> core -> root
            return os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

    def _load_voicemaker_voices(self):
        try:
            base_path = self._get_base_path()
            path = os.path.join(base_path, "assets", "voicemaker_voices.json")
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            logger.log(f"Error loading voicemaker voices: {e}", level=LogLevel.ERROR)
            return []

    def _get_voicemaker_language_code(self, voice_id):
        if not hasattr(self, 'voicemaker_voices') or not self.voicemaker_voices:
             self.voicemaker_voices = self._load_voicemaker_voices()
             
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

    def _start_voiceover(self, task_id):
        state = self.task_states[task_id]
        task_settings = state.settings
        lang_config = task_settings.get("languages_config", {}).get(state.lang_id, {})
        config = {
            'text': state.text_for_processing,
            'dir_path': state.dir_path,
            'lang_config': lang_config,
            'voicemaker_api_key': task_settings.get('voicemaker_api_key'),
            'voicemaker_api_key': task_settings.get('voicemaker_api_key'),
            'elevenlabs_api_key': task_settings.get('elevenlabs_api_key'),
            'elevenlabs_unlim_api_key': task_settings.get('elevenlabs_unlim_api_key'),
            'gemini_tts_api_key': task_settings.get('gemini_tts_api_key'),
            'voicemaker_lang_code': self._get_voicemaker_language_code(lang_config.get('voicemaker_voice_id')),
            'job_name': state.job_name,
            'lang_name': state.lang_name
        }

        tts_provider = lang_config.get('tts_provider', 'ElevenLabs')
        if tts_provider == 'ElevenLabs':
            self.elevenlabs_queue.append((task_id, config))
            self._process_elevenlabs_queue()
        elif tts_provider == 'ElevenLabsUnlim':
            self.elevenlabs_unlim_queue.append((task_id, config))
            self._process_elevenlabs_unlim_queue()
        elif tts_provider == 'EdgeTTS':
            self.edgetts_queue.append((task_id, config))
            self._process_edgetts_queue()
        else:
            self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)

    def _process_elevenlabs_queue(self):
        max_threads = self.settings.get("elevenlabs_max_threads", 5)
        while self.elevenlabs_queue and self.elevenlabs_active_count < max_threads:
            task_id, config = self.elevenlabs_queue.popleft()
            self.elevenlabs_active_count += 1
            self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)

    def _process_elevenlabs_unlim_queue(self):
        max_threads = 5
        while self.elevenlabs_unlim_queue and self.elevenlabs_unlim_active_count < max_threads:
            task_id, config = self.elevenlabs_unlim_queue.popleft()
            self.elevenlabs_unlim_active_count += 1
            self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)

        
    def _process_edgetts_queue(self):
        max_threads = 5 # Fixed limit as per requirement
        while self.edgetts_queue and self.edgetts_active_count < max_threads:
            task_id, config = self.edgetts_queue.popleft()
            self.edgetts_active_count += 1
            self._start_worker(VoiceoverWorker, task_id, 'stage_voiceover', config, self._on_voiceover_finished, self._on_voiceover_error)
        
    @Slot(str, object)
    def _on_voiceover_finished(self, task_id, audio_path):
        state = self.task_states[task_id]
        
        tts_provider = state.lang_data.get('tts_provider', 'ElevenLabs')
        if tts_provider == 'ElevenLabs':
            self.elevenlabs_active_count -= 1
            self._process_elevenlabs_queue()
        elif tts_provider == 'ElevenLabsUnlim':
            self.elevenlabs_unlim_active_count -= 1
            self._process_elevenlabs_unlim_queue()
        elif tts_provider == 'EdgeTTS':
            self.edgetts_active_count -= 1
            self._process_edgetts_queue()

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
            if self.subtitle_barrier_passed:
                self._check_and_start_montages()
            self.check_if_all_finished()

    @Slot(str, str)
    def _on_voiceover_error(self, task_id, error):
        state = self.task_states.get(task_id)
        if state:
            tts_provider = state.lang_data.get('tts_provider', 'ElevenLabs')
            if tts_provider == 'ElevenLabs':
                self.elevenlabs_active_count -= 1
                self._process_elevenlabs_queue()
            elif tts_provider == 'ElevenLabsUnlim':
                self.elevenlabs_unlim_active_count -= 1
                self._process_elevenlabs_unlim_queue()
            elif tts_provider == 'EdgeTTS':
                self.edgetts_active_count -= 1
                self._process_edgetts_queue()

        self._set_stage_status(task_id, 'stage_voiceover', 'error', error)
        # CRITICAL FIX: If voiceover fails, we MUST fail or skip subtitles so the job doesn't hang forever in 'pending'
        if 'stage_subtitles' in self.task_states[task_id].stages:
            logger.log(f"[{task_id}] Voiceover failed, marking dependent stage 'stage_subtitles' as error.", level=LogLevel.WARNING)
            self._set_stage_status(task_id, 'stage_subtitles', 'error', "Dependency (Voiceover) failed")
            # We still increment counter so the barrier can pass
            self._increment_subtitle_counter()

    def _start_subtitles(self, task_id):
        self.whisper_queue.append((task_id, 'subtitles'))
        self._process_whisper_queue()

    def _process_whisper_queue(self):
        # --- Global Concurrency Check ---
        allow_simultaneous = self.settings.get("simultaneous_montage_and_subs", False)

        while self.whisper_queue:
            # Note: We peek/pop in loop, so we check condition inside
            
            # If not allowed simultaneous, checking if montages are running
            if not allow_simultaneous and self._are_montages_running():
                # Cannot start new subtitles yet.
                # We stop the loop. The queue remains populated.
                # processing will resume when _process_whisper_queue is called again (e.g. from montage finished)
                break

            task_id, worker_type = self.whisper_queue.popleft()
            state = self.task_states[task_id]
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')

            if whisper_type == 'assemblyai':
                if worker_type == 'subtitles':
                    self._launch_subtitle_worker(task_id)
                else:
                    self._launch_transcription_worker(task_id)
            else:
                if self.subtitle_semaphore.tryAcquire():
                    if worker_type == 'subtitles':
                        self._launch_subtitle_worker(task_id)
                    else:
                        self._launch_transcription_worker(task_id)
                else:
                    self.whisper_queue.appendleft((task_id, worker_type))
                    break

    def _launch_subtitle_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            model_name = sub_settings.get('whisper_model', 'base')
            
            whisper_exe = None; whisper_model_path = model_name
            if whisper_type == 'amd':
                base_path = self._get_base_path()
                # Assuming whisper-cli-amd is in root dir.
                # If frozen, sys.executable dir; if not, root.
                if getattr(sys, 'frozen', False):
                    whisper_base_path = os.path.join(os.path.dirname(sys.executable), "whisper-cli-amd")
                else:
                    whisper_base_path = os.path.join(base_path, "whisper-cli-amd")
                
                whisper_exe = os.path.join(whisper_base_path, "main.exe")
                whisper_model_path = os.path.join(whisper_base_path, model_name)
            
            config = {
                'audio_path': state.audio_path, 'dir_path': state.dir_path,
                'sub_settings': sub_settings, 
                'full_settings': state.settings, # Pass full settings for resolution detection
                'lang_code': state.lang_id.split('-')[0].lower(),
                'whisper_exe': whisper_exe, 'whisper_model_path': whisper_model_path
            }
            self._start_worker(SubtitleWorker, task_id, 'stage_subtitles', config,
                                         self._on_subtitles_finished, self._on_subtitles_error)
        except Exception as e:
            self._on_subtitles_error(task_id, f"Failed to start subtitle worker: {e}")
        
    @Slot(str, object)
    def _on_subtitles_finished(self, task_id, subtitle_path):
        state = self.task_states.get(task_id)
        if state:
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            if whisper_type != 'assemblyai':
                self.subtitle_semaphore.release()
                time.sleep(2.0)
                self._process_whisper_queue()
        
        # Check if we can unblock montages now (if setting dependent)
        if not self.settings.get("simultaneous_montage_and_subs", False):
            # We call this via task_processor (self)
            # Assuming task_processor has _process_montage_queue mixed in (it does from VideoMixin)
            if hasattr(self, '_process_montage_queue'):
                 self._process_montage_queue()
            
        self.task_states[task_id].subtitle_path = subtitle_path
        self._set_stage_status(task_id, 'stage_subtitles', 'success')
        self._increment_subtitle_counter()

    @Slot(str, str)
    def _on_subtitles_error(self, task_id, error):
        state = self.task_states.get(task_id)
        if state:
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            if whisper_type != 'assemblyai':
                self.subtitle_semaphore.release()
                time.sleep(2.0)
                self._process_whisper_queue()

        # Check if we can unblock montages now
        if not self.settings.get("simultaneous_montage_and_subs", False):
            if hasattr(self, '_process_montage_queue'):
                 self._process_montage_queue()

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

    def _start_transcription(self, task_id):
        self.whisper_queue.append((task_id, 'transcription'))
        self._process_whisper_queue()

    def _launch_transcription_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Prepare helper for whisper path
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            model_name = sub_settings.get('whisper_model', 'base')
            
            whisper_exe = None; whisper_model_path = model_name
            if whisper_type == 'amd':
                base_path = self._get_base_path()
                if getattr(sys, 'frozen', False):
                    whisper_base_path = os.path.join(os.path.dirname(sys.executable), "whisper-cli-amd")
                else:
                    whisper_base_path = os.path.join(base_path, "whisper-cli-amd")
                
                whisper_exe = os.path.join(whisper_base_path, "main.exe")
                whisper_model_path = os.path.join(whisper_base_path, model_name)

            # Determine language code: 
            # 1. Forced source language (e.g. for AMD Whisper manual override)
            # 2. 'auto' for rewrite jobs (standard behavior)
            # 3. Target language code (default fallback)
            forced_lang = state.lang_data.get('source_language')
            if forced_lang:
                lang_code = forced_lang
            elif state.job_type == 'rewrite':
                lang_code = 'auto'
            else:
                lang_code = state.lang_id.split('-')[0].lower()

            config = {
                'audio_path': state.audio_path,
                'sub_settings': sub_settings,
                'lang_code': lang_code,
                'whisper_exe': whisper_exe,
                'whisper_model_path': whisper_model_path
            }
            self._start_worker(TranscriptionWorker, task_id, 'stage_transcription', config, self._on_transcription_finished, self._on_transcription_error)
        except Exception as e:
            self._on_transcription_error(task_id, f"Failed to start transcription: {e}")

    @Slot(str, str)
    def _on_transcription_error(self, task_id, error):
        state = self.task_states.get(task_id)
        if state:
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            if whisper_type != 'assemblyai':
                self.subtitle_semaphore.release()
                time.sleep(2.0)
                self._process_whisper_queue()
        
        self._set_stage_status(task_id, 'stage_transcription', 'error', error)
        
        # If transcription fails, we should probably fail dependent stages like rewrite/translation
        # For now just marking this stage as error solves the crash.

    @Slot(str, object)
    def _on_transcription_finished(self, task_id, text):
        state = self.task_states[task_id]
        # Store intermediate text if needed, or pass to rewrite
        # Maybe save to file
        with open(os.path.join(state.dir_path, "transcription.txt"), 'w', encoding='utf-8') as f:
            f.write(text)
            
        if state:
            sub_settings = state.settings.get('subtitles', {})
            whisper_type = sub_settings.get('whisper_type', 'standard')
            if whisper_type != 'assemblyai':
                self.subtitle_semaphore.release()
                time.sleep(2.0)
                self._process_whisper_queue()
            
            # Update stage.text_for_processing or original_text depending on logic
            # Usually transcription provides the "original text" for further processing
            state.original_text = text
            state.text_for_processing = text

            self._set_stage_status(task_id, 'stage_transcription', 'success')
            
            # Trigger next stage
            if 'stage_rewrite' in state.stages:
                self._start_rewrite(task_id, text)
            elif 'stage_translation' in state.stages:
                self._start_translation(task_id)
            else:
                self._on_text_ready(task_id)
