import os
from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from utils.translator import translator
from core.workers import TranslationWorker, RewriteWorker, CustomStageWorker

class TranslationMixin:
    """
    Mixin for TaskProcessor to handle Translation, Rewrite, and Custom Text Stages.
    Requires: self.task_states, self.settings, self.openrouter_queue, self._process_openrouter_queue,
              self._start_worker, self._set_stage_status, self.stage_metadata_updated,
              self.translation_regenerated, self.translation_review_required,
              self.rewrite_regenerated, self.rewrite_review_required,
              self._start_image_prompts, self._start_voiceover, self._start_image_generation,
              self.check_if_all_finished
    """

    def _start_rewrite(self, task_id, text):
        self.openrouter_queue.append((task_id, 'rewrite', text))
        self._process_openrouter_queue()

    def _launch_rewrite_worker(self, task_id, text):
        try:
            state = self.task_states[task_id]
            
            # Smart model selection for Rewrite:
            model = state.settings.get('rewrite_model')
            if not model: model = state.settings.get('model')
            if not model: model = state.lang_data.get('rewrite_model')
            if not model: model = state.lang_data.get('model')
            if not model: model = state.settings.get('languages_config', {}).get(state.lang_id, {}).get('rewrite_model')
            if not model: model = state.settings.get('languages_config', {}).get(state.lang_id, {}).get('model')
            if not model:
                models = state.settings.get('openrouter_models', [])
                model = models[0] if models else 'unknown'
            
            config = {
                'text': text,
                'prompt': state.lang_data.get('rewrite_prompt') or 'Rewrite this text:',
                'model': model,
                'max_tokens': state.lang_data.get('rewrite_max_tokens') or 4096,
                'temperature': state.lang_data.get('rewrite_temperature') if state.lang_data.get('rewrite_temperature') is not None else 0.7,
                'openrouter_api_key': state.settings.get('openrouter_api_key')
            }
            self._start_worker(RewriteWorker, task_id, 'stage_rewrite', config, self._on_rewrite_finished, self._on_rewrite_error)
        except Exception as e:
            self._on_rewrite_error(task_id, f"Failed to start rewrite: {e}")

    @Slot(str, object)
    def _on_rewrite_finished(self, task_id, rewritten_text):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        state = self.task_states[task_id]
        state.text_for_processing = rewritten_text
        if state.dir_path:
            # Save original rewrite for reference
            with open(os.path.join(state.dir_path, "translation_orig.txt"), 'w', encoding='utf-8') as f:
                f.write(rewritten_text)
            # Save working copy
            with open(os.path.join(state.dir_path, "translation.txt"), 'w', encoding='utf-8') as f:
                f.write(rewritten_text)
            
        # Update metadata with character count
        char_count = len(rewritten_text)
        metadata_text = f"{char_count} {translator.translate('characters_count')}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_rewrite', metadata_text)

        is_review_enabled = state.settings.get('rewrite_review_enabled', False)

        if is_review_enabled:
            self._set_stage_status(task_id, 'stage_rewrite', 'success')
            self.rewrite_regenerated.emit(task_id, rewritten_text) # Update dialog if open

            if not state.rewrite_review_dialog_shown:
                state.rewrite_review_dialog_shown = True
                self.rewrite_review_required.emit(task_id, rewritten_text)
        else:
            # No review, proceed as normal
            self._set_stage_status(task_id, 'stage_rewrite', 'success')
            self._on_text_ready(task_id)

    @Slot(str, str)
    def _on_rewrite_error(self, task_id, error):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        self._set_stage_status(task_id, 'stage_rewrite', 'error', error)
        # Fail dependencies
        for stage in ['stage_img_prompts', 'stage_images', 'stage_voiceover', 'stage_subtitles', 'stage_montage']:
            if stage in self.task_states[task_id].stages:
                self._set_stage_status(task_id, stage, 'error', "Dependency (Rewrite) failed")

    def _start_translation(self, task_id):
        self.openrouter_queue.append((task_id, 'translation', None))
        self._process_openrouter_queue()

    def _launch_translation_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Smart model selection for Translation:
            model = state.settings.get('model')
            if not model: model = state.lang_data.get('model')
            if not model: model = state.settings.get('languages_config', {}).get(state.lang_id, {}).get('model')
            if not model:
                models = state.settings.get('openrouter_models', [])
                model = models[0] if models else 'unknown'
                
            config = {
                'text': state.original_text,
                'lang_config': {
                    'prompt': state.lang_data.get('prompt', ''),
                    'model': model,
                    'temperature': state.lang_data.get('temperature') if state.lang_data.get('temperature') is not None else 0.7,
                    'max_tokens': state.lang_data.get('max_tokens') or 4096
                },
                'openrouter_api_key': state.settings.get('openrouter_api_key')
            }
            self._start_worker(TranslationWorker, task_id, 'stage_translation', config, self._on_translation_finished, self._on_translation_error)
        except Exception as e:
            self._on_translation_error(task_id, f"Failed to start translation: {e}")

    @Slot(str, object)
    def _on_translation_finished(self, task_id, translated_text):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        state = self.task_states[task_id]
        state.text_for_processing = translated_text
        if state.dir_path:
            # Save the original translation before review for reference
            with open(os.path.join(state.dir_path, "translation_orig.txt"), 'w', encoding='utf-8') as f:
                f.write(translated_text)
            # Save working copy
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
        else:
            # No review, proceed as normal
            self._set_stage_status(task_id, 'stage_translation', 'success')
            self._on_text_ready(task_id)

    @Slot(str, str)
    def _on_translation_error(self, task_id, error):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        self._set_stage_status(task_id, 'stage_translation', 'error', error)
        # Fail dependencies
        state = self.task_states[task_id]
        for stage in ['stage_voiceover', 'stage_subtitles', 'stage_img_prompts', 'stage_images', 'stage_montage']:
             if stage in state.stages:
                self._set_stage_status(task_id, stage, 'error', "Dependency (Translation) failed")

        # Check if we can start montages for other tasks
        if getattr(self, 'subtitle_barrier_passed', False):
            self._check_and_start_montages()

    def regenerate_translation(self, task_id):
        logger.log(f"[{task_id}] User requested translation regeneration.", level=LogLevel.INFO)
        if task_id in self.task_states:
            self.task_states[task_id].translation_review_dialog_shown = False
        self._start_translation(task_id)

    def regenerate_rewrite(self, task_id):
        logger.log(f"[{task_id}] User requested rewrite regeneration.", level=LogLevel.INFO)
        if task_id in self.task_states:
            self.task_states[task_id].rewrite_review_dialog_shown = False
        state = self.task_states[task_id]
        self._start_rewrite(task_id, state.original_text)

    def _on_text_ready(self, task_id):
        state = self.task_states[task_id]
        
        # If translation was not used, emit metadata for original text
        if 'stage_translation' not in state.stages and 'stage_rewrite' not in state.stages and state.text_for_processing:
            char_count = len(state.text_for_processing)
            metadata_text = f"{char_count} {translator.translate('characters_count')}"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'original_text', metadata_text)
        
        if 'stage_img_prompts' in state.stages:
            self._start_image_prompts(task_id)
        if 'stage_voiceover' in state.stages:
            self._start_voiceover(task_id)
        if 'stage_img_prompts' not in state.stages and 'stage_images' in state.stages:
             self._start_image_generation(task_id)
        if 'stage_img_prompts' not in state.stages and 'stage_voiceover' not in state.stages and 'stage_images' not in state.stages:
            self.check_if_all_finished()

        # --- Custom Stages ---
        custom_stages = self.settings.get("custom_stages", [])
        if custom_stages:
            for stage in custom_stages:
                stage_name = stage.get("name")
                prompt = stage.get("prompt")
                model = stage.get("model")
                max_tokens = stage.get("max_tokens")
                temperature = stage.get("temperature")
                
                stage_key = f"custom_{stage_name}"
                if stage_key in state.stages:
                    if stage_name and prompt:
                        self._start_custom_stage(task_id, stage_name, prompt, model, max_tokens, temperature)

    def _start_custom_stage(self, task_id, stage_name, prompt, model=None, max_tokens=None, temperature=None):
        extra_data = (stage_name, prompt, model, max_tokens, temperature)
        self.openrouter_queue.append((task_id, 'custom_stage', extra_data))
        self._process_openrouter_queue()

    def _launch_custom_stage_worker(self, task_id, stage_name, prompt, model=None, max_tokens=None, temperature=None):
        try:
            state = self.task_states[task_id]
            if not model:
                model = state.settings.get('model') or state.lang_data.get('model') or \
                        state.settings.get("image_prompt_settings", {}).get("model") or \
                        (state.settings.get('openrouter_models', [])[0] if state.settings.get('openrouter_models') else 'unknown')
            
            config = {
                'text': state.text_for_processing,
                'dir_path': state.dir_path,
                'stage_name': stage_name,
                'prompt': prompt,
                'model': model,
                'max_tokens': int(max_tokens or 4096),
                'temperature': float(temperature) if temperature is not None else 0.7,
                'openrouter_api_key': state.settings.get('openrouter_api_key')
            }
            self._start_worker(CustomStageWorker, task_id, f"custom_{stage_name}", config, 
                               self._on_custom_stage_finished, self._on_custom_stage_error_slot)
        except Exception as e:
            self._on_custom_stage_error_slot(task_id, f"Failed to start custom stage '{stage_name}': {e}")

    @Slot(str, object)
    def _on_custom_stage_finished(self, task_id, result_data):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        stage_name = result_data.get('stage_name')
        stage_key = f"custom_{stage_name}"
        logger.log(f"[{task_id}] Custom stage '{stage_name}' finished.", level=LogLevel.INFO)
        self._set_stage_status(task_id, stage_key, 'success')
        self.check_if_all_finished()

    @Slot(str, str)
    def _on_custom_stage_error_slot(self, task_id, error):
        self.openrouter_active_count -= 1
        self._process_openrouter_queue()
        
        worker = self.sender()
        stage_name = getattr(worker, 'config', {}).get('stage_name', 'unknown') if worker else 'unknown'
        
        stage_key = f"custom_{stage_name}"
        logger.log(f"[{task_id}] [Custom Stage: {stage_name}] Failed: {error}", level=LogLevel.ERROR)
        self._set_stage_status(task_id, stage_key, 'error', error)
        self.check_if_all_finished()

    def _process_openrouter_queue(self):
        max_openrouter = self.settings.get("openrouter_max_threads", 5)
        while self.openrouter_queue and self.openrouter_active_count < max_openrouter:
            task_id, worker_type, extra_data = self.openrouter_queue.popleft()
            self.openrouter_active_count += 1
            
            if worker_type == 'rewrite':
                self._launch_rewrite_worker(task_id, extra_data)
            elif worker_type == 'translation':
                self._launch_translation_worker(task_id)
            elif worker_type == 'image_prompts':
                self._launch_image_prompts_worker(task_id)
            elif worker_type == 'custom_stage':
                self._launch_custom_stage_worker(task_id, *extra_data)

