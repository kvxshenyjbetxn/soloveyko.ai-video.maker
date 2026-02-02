import os
import re
import shutil
from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from utils.translator import translator
from core.workers import PreviewWorker, ImageGenerationWorker

class PreviewMixin:
    """
    Mixin for TaskProcessor to handle Preview Stage.
    Requires: self.task_states, self.settings, self.openrouter_queue, self._process_openrouter_queue,
              self.image_gen_executor, self._start_worker, self._set_stage_status, self.stage_metadata_updated,
              self.check_if_all_finished, self._start_image_prompts
    """

    def _start_preview(self, task_id):
        self.openrouter_queue.append((task_id, 'preview', None))
        self._process_openrouter_queue()

    def _launch_preview_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            preview_settings = state.settings.get("preview_settings", {}).copy()
            
            # Use global settings if not in task settings (though apply_settings should have handled this)
            if not preview_settings:
                 preview_settings = self.settings.get("preview_settings", {}).copy()

            # Merge with task specific overrides if any (unlikely for preview but good practice)
            
            config = {
                'story': state.text_for_processing,
                'title': state.job_name,
                'preview_settings': preview_settings,
                'openrouter_api_key': state.settings.get('openrouter_api_key')
            }
            self._start_worker(PreviewWorker, task_id, 'stage_preview', config, self._on_preview_prompts_finished, self._on_preview_error)
        except Exception as e:
            self._on_preview_error(task_id, f"Failed to start preview worker: {e}")

    @Slot(str, object)
    def _on_preview_prompts_finished(self, task_id, prompts_text):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        state = self.task_states[task_id]
        
        # Count prompts
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        if not prompts:
             prompts = [line.strip() for line in prompts_text.split('\n') if line.strip()]
        prompts_count = len(prompts)

        # Save prompts
        preview_dir = os.path.join(state.dir_path, "preview")
        os.makedirs(preview_dir, exist_ok=True)
        
        prompts_path = os.path.join(preview_dir, "preview_prompts.txt")
        with open(prompts_path, 'w', encoding='utf-8') as f:
            f.write(prompts_text)
            
        logger.log(f"[{task_id}] Preview prompts ready ({prompts_count}). Starting image generation.", level=LogLevel.INFO)
        
        # Start Image Generation for Preview
        self._start_preview_image_generation(task_id, prompts_text, preview_dir)

    def _start_preview_image_generation(self, task_id, prompts_text, preview_dir):
        state = self.task_states[task_id]
        
        # We need to setup config for ImageGenerationWorker
        # It expects 'prompts_text', 'dir_path', 'provider', etc.
        
        preview_settings = state.settings.get("preview_settings", {})
        # Provider selection logic: prioritize preview settings, fallback to global, default to pollinations
        provider = preview_settings.get('image_provider')
        if not provider:
            provider = state.settings.get('image_generation_provider', 'pollinations')
        
        googler_settings = state.settings.get('googler', {})
        
        api_kwargs = {}
        if provider == 'googler':
             api_kwargs = {
                'aspect_ratio': googler_settings.get('aspect_ratio', 'IMAGE_ASPECT_RATIO_LANDSCAPE'),
                'seed': googler_settings.get('seed'),
                'negative_prompt': googler_settings.get('negative_prompt')
            }
        elif provider == 'elevenlabs_image':
            elevenlabs_image_settings = state.settings.get('elevenlabs_image', {})
            api_kwargs = {
                'aspect_ratio': elevenlabs_image_settings.get('aspect_ratio', '16:9')
            }
            
        elif provider == 'pollinations':
            pollinations_settings = state.settings.get('pollinations', {}).copy()
            
            # Override model for preview specifically
            preview_poll_model = preview_settings.get('pollinations_model')
            if preview_poll_model:
                pollinations_settings['model'] = preview_poll_model
            elif 'image_provider' not in preview_settings:
                # Old template fallback as requested
                pollinations_settings['model'] = 'zimage'
            
            valid_keys = ['model', 'width', 'height', 'nologo', 'enhance']
            api_kwargs = {k: v for k, v in pollinations_settings.items() if k in valid_keys}

        if provider == 'googler':
             current_max_threads = googler_settings.get("max_threads", 8)
             current_api_key = googler_settings.get('api_key')
             executor = self.image_gen_executor
             current_semaphore = getattr(self, 'googler_semaphore', None)
        elif provider == 'elevenlabs_image':
             elevenlabs_image_settings = state.settings.get('elevenlabs_image', {})
             current_max_threads = elevenlabs_image_settings.get("max_threads", 5)
             current_api_key = elevenlabs_image_settings.get('api_key')
             executor = self.elevenlabs_executor
             current_semaphore = getattr(self, 'elevenlabs_image_semaphore', None)
        else:
             current_max_threads = 8 
             current_api_key = None
             executor = self.image_gen_executor
             current_semaphore = None

        image_count = preview_settings.get('image_count', 1)

        config = {
            'prompts_text': prompts_text,
            'dir_path': preview_dir, # Write images to preview folder
            'provider': provider,
            'api_kwargs': api_kwargs,
            'image_count': image_count,
            'api_key': current_api_key, 
            'executor': executor,
            'max_threads': current_max_threads,
            'semaphore': current_semaphore
        }
        
        # We use 'stage_preview' as stage name.
        self._start_worker(ImageGenerationWorker, task_id, 'stage_preview', config, self._on_preview_images_finished, self._on_preview_images_error)

    @Slot(str, object)
    def _on_preview_images_finished(self, task_id, result_dict):
        generated_paths = result_dict.get('paths', [])
        total_prompts = result_dict.get('total_prompts', 0)
        
        status = 'error'
        if total_prompts > 0 and len(generated_paths) == total_prompts:
            status = 'success'
        elif len(generated_paths) > 0:
            status = 'warning'

        logger.log(f"[{task_id}] Preview image gen finished. Status: {status}.", level=LogLevel.INFO)
            
        self._set_stage_status(task_id, 'stage_preview', status, "Failed to generate all preview images." if status != 'success' else None)
        
        state = self.task_states[task_id]
        
        # Proceed to next stages if any
        # if 'stage_img_prompts' in state.stages:
        #    self._start_image_prompts(task_id)
        # else:
        self.check_if_all_finished()

    @Slot(str, str)
    def _on_preview_error(self, task_id, error):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        self._set_stage_status(task_id, 'stage_preview', 'error', error)

    @Slot(str, str)
    def _on_preview_images_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_preview', 'error', error)
        # Even if preview fails, we might want to continue? 
        # Usually error stops the flow for that branch.
        # But if it's just preview... maybe?
        # For now, treat as error.
