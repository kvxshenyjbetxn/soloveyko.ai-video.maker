import os
import re
from utils.logger import logger, LogLevel

class TaskState:
    """Holds the state and data for a single language within a single job."""
    def __init__(self, job, lang_id, lang_data, base_save_path, settings):
        self.job_id = job['id']
        self.lang_id = lang_id
        self.task_id = f"{self.job_id}_{self.lang_id}"

        self.job_name = job['name']
        self.lang_name = lang_data['display_name']
        self.stages = lang_data['stages']
        self.original_text = job.get('text', '')
        self.input_source = job.get('input_source', '')
        self.job_type = job.get('type', 'text')
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
        self.rewrite_review_dialog_shown = False
        self.prompt_regeneration_attempts = 0
        self.image_gen_status = 'pending'
        
        # Metadata counters
        self.images_generated_count = 0
        self.images_total_count = 0
        self.videos_generated_count = 0
        self.videos_total_count = 0
        
        self.fallback_to_quick_show = False

    def _get_save_path(self, base_path, job_name, lang_name):
        if not base_path: return None
        try:
            safe_job_name = job_name.replace('…', '').replace('...', '')
            safe_job_name = re.sub(r'[<>:"/\\|?*]', '', safe_job_name).strip()
            safe_job_name = safe_job_name[:100].strip()
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).rstrip()
            dir_path = os.path.join(base_path, safe_job_name, safe_lang_name)
            os.makedirs(dir_path, exist_ok=True)
            return dir_path
        except Exception as e:
            logger.log(f"[{self.task_id}] Failed to create save directory {dir_path}. Error: {e}", level=LogLevel.ERROR)
            return None
