import os
import re
from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from utils.translator import translator
from core.workers import ImagePromptWorker, ImageGenerationWorker

class ImageMixin:
    """
    Mixin for TaskProcessor to handle Image Prompts and Image Generation.
    Requires: self.task_states, self.settings, self.openrouter_queue, self._process_openrouter_queue,
              self.image_gen_executor, self._start_worker, self._set_stage_status, self.stage_metadata_updated,
              self.check_if_all_finished, self._start_video_generation, self._check_and_start_montages,
              self.subtitle_barrier_passed
    """

    def _start_image_prompts(self, task_id):
        self.openrouter_queue.append((task_id, 'image_prompts', None))
        self._process_openrouter_queue()

    def _launch_image_prompts_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Smart settings merging for image prompts:
            img_settings = state.settings.get("image_prompt_settings", {}).copy()
            
            # Hierarchy for model: 
            model = state.settings.get('model')
            if not model: model = img_settings.get('model')
            if not model: model = state.lang_data.get('model')
            if not model:
                models = state.settings.get('openrouter_models', [])
                model = models[0] if models else 'unknown'
            
            img_settings['model'] = model
            
            config = {
                'text': state.text_for_processing,
                'img_prompt_settings': img_settings,
                'openrouter_api_key': state.settings.get('openrouter_api_key')
            }
            self._start_worker(ImagePromptWorker, task_id, 'stage_img_prompts', config, self._on_img_prompts_finished, self._on_img_prompts_error)
        except Exception as e:
            self._on_img_prompts_error(task_id, f"Failed to start image prompt worker: {e}")

    @Slot(str, object)
    def _on_img_prompts_finished(self, task_id, prompts_text):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        state = self.task_states[task_id]
        
        # Count prompts
        prompts = re.findall(r"^\d+\.\s*(.*)", prompts_text, re.MULTILINE)
        prompts_count = len(prompts)

        # Check if prompt count control is enabled
        is_check_enabled = state.settings.get('prompt_count_control_enabled', False)
        desired_count = state.settings.get('prompt_count', 50)

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
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        self._set_stage_status(task_id, 'stage_img_prompts', 'error', error)

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
        

        provider = state.settings.get('image_generation_provider', 'pollinations')
        
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
            # api_key is passed in config
            
        elif provider == 'pollinations':
            pollinations_settings = state.settings.get('pollinations', {})
            # Filter kwargs to only include valid arguments for the generate_image method
            # The 'token' is handled internally by the PollinationsAPI class.
            valid_keys = ['model', 'width', 'height', 'nologo', 'enhance']
            api_kwargs = {k: v for k, v in pollinations_settings.items() if k in valid_keys}

        # Determine effective max_threads for the current provider
        if provider == 'googler':
             current_max_threads = googler_settings.get("max_threads", 8)
             current_api_key = googler_settings.get('api_key')
        elif provider == 'elevenlabs_image':
             elevenlabs_image_settings = state.settings.get('elevenlabs_image', {})
             current_max_threads = elevenlabs_image_settings.get("max_threads", 5)
             current_api_key = elevenlabs_image_settings.get('api_key')
        else:
             current_max_threads = 8 # Default for pollinations (sequential anyway)
             current_api_key = None

        config = {
            'prompts_text': state.image_prompts,
            'dir_path': state.dir_path,
            'provider': provider,
            'api_kwargs': api_kwargs,
            'api_key': current_api_key, 
            'executor': self.image_gen_executor,
            'max_threads': current_max_threads
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
            logger.log(f"[{task_id}] Image gen finished. Status: {status}.", level=LogLevel.INFO)
            
            self._set_stage_status(task_id, 'stage_images', status, "Failed to generate all images." if status != 'success' else None)
            if getattr(self, 'subtitle_barrier_passed', False):
                self._check_and_start_montages()

    @Slot(str, str)
    def _on_img_generation_error(self, task_id, error):
        self._set_stage_status(task_id, 'stage_images', 'error', error)
        if getattr(self, 'subtitle_barrier_passed', False):
            self._check_and_start_montages()
