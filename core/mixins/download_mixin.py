from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from core.workers import DownloadWorker

class DownloadMixin:
    """
    Mixin for TaskProcessor to handle downloading resources (like YouTube videos/audio).
    Requires: self.task_states, self.yt_dlp_path, self.download_semaphore, self._start_worker,
              self._set_stage_status, self._start_transcription (if logical flow demands)
    """

    def _start_download(self, task_id):
        # We assume self.task_states is present in the main class
        state = self.task_states[task_id]
        config = {
            'url': state.input_source, # For rewrite tasks, input_source is the URL
            'dir_path': state.dir_path,
            'yt_dlp_path': self.yt_dlp_path,
            'download_semaphore': self.download_semaphore
        }
        self._start_worker(DownloadWorker, task_id, 'stage_download', config, self._on_download_finished, self._on_download_error)

    @Slot(str, object)
    def _on_download_finished(self, task_id, audio_path):
        state = self.task_states[task_id]
        state.audio_path = audio_path # Temporary path for transcription
        self._set_stage_status(task_id, 'stage_download', 'success')
        
        # Check dependency for next stage (Transcription)
        if 'stage_transcription' in state.stages:
            self._start_transcription(task_id)
        else:
            # Should not happen in normal flow, but just in case
            logger.log(f"[{task_id}] Download finished but no transcription stage found.", level=LogLevel.WARNING)

    @Slot(str, str)
    def _on_download_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_download', 'error', error)
        # Fail dependencies
        for stage in ['stage_transcription', 'stage_rewrite', 'stage_img_prompts', 'stage_images', 'stage_voiceover', 'stage_subtitles', 'stage_montage']:
            if stage in self.task_states[task_id].stages:
                self._set_stage_status(task_id, stage, 'error', "Dependency (Download) failed")
