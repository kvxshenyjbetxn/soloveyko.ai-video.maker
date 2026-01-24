import os
import re
import platform
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
        self.is_image_reviewed = False
        self.skipped_stages = set()

        # History data
        self.start_time = None
        self.end_time = None
        self.original_text_preview = None
        self.translated_text_preview = None

    def _get_save_path(self, base_path, job_name, lang_name):
        if not base_path: return None
        try:
            # 1. Видаляємо трикрапки
            clean_job_name = job_name.replace('…', '').replace('...', '')
            
            # 2. Видаляємо заборонені символи Windows (навіть для Mac для уніфікації)
            safe_job_name = re.sub(r'[<>:"/\\|?*]', '', clean_job_name)
            
            # 3. Обрізаємо до 100 символів та чистимо кінець (крапки/пробіли)
            safe_job_name = safe_job_name[:100].strip('. ')
            
            if not safe_job_name: safe_job_name = "Untitled_Task"
            
            # Аналогічно для мови
            safe_lang_name = "".join(c for c in lang_name if c.isalnum() or c in (' ', '_')).strip('. ')
            
            # Прямий шлях без префіксів довгих шляхів
            dir_path = os.path.abspath(os.path.join(base_path, safe_job_name, safe_lang_name))
            
            os.makedirs(dir_path, exist_ok=True)
            return dir_path
        except Exception as e:
            logger.log(f"[{self.task_id}] Failed to create save directory. Error: {e}", level=LogLevel.ERROR)
            return None
