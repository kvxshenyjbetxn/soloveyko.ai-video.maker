import os
import re
from PySide6.QtCore import Slot
from utils.logger import logger, LogLevel
from core.workers import VideoGenerationWorker, MontageWorker
from utils.translator import translator
from core.notification_manager import notification_manager

class VideoMixin:
    """
    Mixin for TaskProcessor to handle Video Generation and Montage.
    Requires: self.task_states, self.settings, self.video_semaphore, self.montage_semaphore,
              self.pending_montages, self.montage_tasks_ids, self.failed_montage_tasks_ids,
              self.tasks_awaiting_review, self.subtitle_barrier_passed,
              self._start_worker, self._set_stage_status, self.stage_metadata_updated,
              self.stage_status_changed, self.task_progress_log, self.image_review_required
    """

    def _start_video_generation(self, task_id):
        state = self.task_states[task_id]
        montage_settings = state.settings.get("montage", {})
        googler_settings = state.settings.get("googler", {})

        video_count = montage_settings.get("special_processing_video_count", 1)
        check_sequence = montage_settings.get("special_processing_check_sequence", False)
        
        all_image_paths = state.image_paths
        paths_to_animate = []

        if not all_image_paths:
            logger.log(f"[{task_id}] No images to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return

        if not check_sequence:
            paths_to_animate = all_image_paths[:video_count]
        else:
            # --- Sequence Check Logic ---
            first_img_basename = os.path.splitext(os.path.basename(all_image_paths[0]))[0]
            if first_img_basename != '1':
                logger.log(f"[{task_id}] First image is '{first_img_basename}', not '1'. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
                state.fallback_to_quick_show = True
                # Fix: Update status to success (or warning) so the flow continues, instead of stalling in 'processing_video'
                self._set_stage_status(task_id, 'stage_images', state.image_gen_status) 
                if self.subtitle_barrier_passed: self._check_and_start_montages()
                return

            sequential_count = 0
            for i in range(min(video_count, len(all_image_paths))):
                expected_name = str(i + 1)
                actual_name = os.path.splitext(os.path.basename(all_image_paths[i]))[0]
                if actual_name == expected_name:
                    sequential_count += 1
                else:
                    break

            if sequential_count > 0:
                logger.log(f"[{task_id}] Found a sequence of {sequential_count} images to animate.", level=LogLevel.INFO)
                paths_to_animate = all_image_paths[:sequential_count]

        if not paths_to_animate:
            logger.log(f"[{task_id}] No sequential images found to animate, skipping video generation.", level=LogLevel.INFO)
            if self.subtitle_barrier_passed: self._check_and_start_montages()
            return
            
        state.video_animation_count = len(paths_to_animate)
        state.videos_total_count = len(paths_to_animate)
        state.videos_generated_count = 0
        logger.log(f"[{task_id}] Starting video generation for {len(paths_to_animate)} images.", level=LogLevel.INFO)

        # Update metadata to show video progress
        img_meta = f"{state.images_generated_count}/{state.images_total_count}"
        vid_meta = f"0/{state.videos_total_count}"
        metadata_text = f"IMG: {img_meta}, VID: {vid_meta}"
        self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_images', metadata_text)

        config = {
            'image_paths': paths_to_animate,
            'prompt': googler_settings.get("video_prompt", "Animate this scene, cinematic movement, 4k"),
            'video_semaphore': self.video_semaphore,
            'max_threads': googler_settings.get("max_video_threads", 1)
        }
        self._start_worker(VideoGenerationWorker, task_id, 'stage_images', config, self._on_video_generation_finished, self._on_video_generation_error)

    @Slot(str, object)
    def _on_video_generation_finished(self, task_id, result_dict):
        state = self.task_states[task_id]
        generated_videos = result_dict.get('paths', [])
        
        logger.log(f"[{task_id}] Video generation finished. Generated {len(generated_videos)} videos.", level=LogLevel.SUCCESS)

        video_count_animated = getattr(state, 'video_animation_count', 0)
        
        # Determine video generation status
        video_gen_status = 'success'
        if len(generated_videos) < video_count_animated:
            video_gen_status = 'warning'
        if len(generated_videos) == 0 and video_count_animated > 0:
            # Instead of marking as error, we'll fallback to Quick Show
            # So we use 'warning' status to allow montage to proceed
            video_gen_status = 'warning'

        if generated_videos:
            remaining_images = state.image_paths[video_count_animated:]
            state.image_paths = generated_videos + remaining_images
        elif video_count_animated > 0: # This means video gen failed for all
            logger.log(f"[{task_id}] Video animation produced 0 videos. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
            state.fallback_to_quick_show = True

        # Combine statuses: warning is the "lowest" priority besides success
        final_status = state.image_gen_status
        if final_status == 'success' and video_gen_status != 'success':
            final_status = video_gen_status # 'warning' only now (not 'error')
        elif final_status == 'warning' and video_gen_status == 'warning':
            final_status = 'warning'
        # If image_gen_status was 'error', it remains 'error' (real failure in image generation)

        error_message = None
        if final_status != 'success':
            if state.fallback_to_quick_show:
                error_message = "Video animation failed, using Quick Show mode."
            else:
                error_message = "Failed to generate all images and/or videos."
            
        self._set_stage_status(task_id, 'stage_images', final_status, error_message)
        
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    @Slot(str, str)
    def _on_video_generation_error(self, task_id, error):
        logger.log(f"[{task_id}] Video generation failed: {error}", level=LogLevel.ERROR)
        
        # Instead of blocking montage, fallback to Quick Show
        state = self.task_states[task_id]
        state.fallback_to_quick_show = True
        logger.log(f"[{task_id}] Video generation error. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
        
        # Use warning status instead of error to allow montage to proceed
        # Only set to error if image generation also failed
        final_status = state.image_gen_status if state.image_gen_status == 'error' else 'warning'
        error_message = "Video animation failed, using Quick Show mode."
        
        self._set_stage_status(task_id, 'stage_images', final_status, error_message)
        
        # Continue to montage
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    def _check_and_start_montages(self):
        for task_id, state in self.task_states.items():
            if 'stage_montage' not in state.stages or state.status.get('stage_montage') != 'pending':
                continue
            
            sub_status = state.status.get('stage_subtitles') if 'stage_subtitles' in state.stages else 'success'
            img_status = state.status.get('stage_images') if 'stage_images' in state.stages else 'success'

            prerequisites_are_done = sub_status not in ['pending', 'processing'] and img_status not in ['pending', 'processing', 'processing_video']

            if prerequisites_are_done:
                if sub_status == 'error' or img_status == 'error':
                    error_msg = f"Prerequisite failed. Subtitles: {sub_status}, Images: {img_status}"
                    self._set_stage_status(task_id, 'stage_montage', 'error', error_msg)
                    continue

                if not state.audio_path or not os.path.exists(state.audio_path):
                    self._set_stage_status(task_id, 'stage_montage', 'error', "Audio file missing")
                    continue
                
                if not state.image_paths or len(state.image_paths) == 0:
                    self._set_stage_status(task_id, 'stage_montage', 'error', "No images available")
                    continue
                
                # Use per-task settings for review
                if state.settings.get('image_review_enabled'):
                    if task_id not in self.tasks_awaiting_review:
                        logger.log(f"[{task_id}] Ready for image review.", level=LogLevel.INFO)
                        self.tasks_awaiting_review.append(task_id)
                else:
                    self._start_montage(task_id)
        
        self._check_if_all_are_ready_or_failed()

    def _check_if_all_are_ready_or_failed(self):
        total_montage_tasks = len(self.montage_tasks_ids)
        if total_montage_tasks == 0:
            return

        ready_or_processed_count = 0
        
        for task_id in self.montage_tasks_ids:
            state = self.task_states.get(task_id)
            if not state: continue 

            is_waiting = task_id in self.tasks_awaiting_review
            is_failed = task_id in self.failed_montage_tasks_ids
            # If stage_montage is NOT pending, it means it started processing (skipped review)
            is_processed = state.status.get('stage_montage') != 'pending'
            
            if is_waiting or is_failed or is_processed:
                ready_or_processed_count += 1

        if ready_or_processed_count >= total_montage_tasks:
            # Only trigger if there are actually tasks waiting for review
            if len(self.tasks_awaiting_review) > 0:
                logger.log(f"All {total_montage_tasks} tasks accounted for. Requesting review for {len(self.tasks_awaiting_review)} tasks.", level=LogLevel.SUCCESS)
                
                title = translator.translate('notification_image_review_title')
                body = translator.translate('notification_image_review_body')
                notification_manager.send_notification(f"{title}\n{body}")
                
                self.image_review_required.emit()

    @Slot()
    def resume_all_montages(self):
        logger.log(f"Resuming montage for {len(self.tasks_awaiting_review)} tasks.", level=LogLevel.INFO)
        tasks_to_start = list(self.tasks_awaiting_review)
        self.tasks_awaiting_review.clear()
        
        for task_id in tasks_to_start:
            state = self.task_states.get(task_id)
            if state:
                self._start_montage(task_id)

    def _start_montage(self, task_id):
        self.pending_montages.append(task_id)
        self._process_montage_queue()

    def _process_montage_queue(self):
        while self.pending_montages:
            if self.montage_semaphore.tryAcquire():
                task_id = self.pending_montages.popleft()
                self._launch_montage_worker(task_id)
            else:
                break
    
    def _launch_montage_worker(self, task_id):
        try:
            state = self.task_states[task_id]
            
            # Set status to 'processing'
            self.stage_status_changed.emit(state.job_id, state.lang_id, 'stage_montage', 'processing')
            state.status['stage_montage'] = 'processing'
            
            final_image_paths = state.image_paths
                
            if not final_image_paths:
                self._on_montage_error(task_id, "No visual files found for montage.")
                return
            
            safe_task_name = ("".join(c for c in state.job_name if c.isalnum() or c in (' ', '_')).strip())[:100]
            safe_lang_name = "".join(c for c in state.lang_name if c.isalnum() or c in (' ', '_')).strip()
            output_filename = f"{safe_task_name}_{safe_lang_name}.mp4"
            output_path = os.path.join(state.dir_path, output_filename)
            
            montage_settings = state.settings.get("montage", {}).copy()
            if getattr(state, 'fallback_to_quick_show', False):
                logger.log(f"[{task_id}] Fallback to 'Quick Show' mode for montage.", level=LogLevel.WARNING)
                montage_settings['special_processing_mode'] = "Quick show"
                
            config = {
                'visual_files': final_image_paths, 'audio_path': state.audio_path,
                'output_path': output_path, 'ass_path': state.subtitle_path,
                'settings': montage_settings
            }

            # --- Add Background Music Config ---
            background_music_path = None
            background_music_volume = 100
            
            all_languages_config = state.settings.get("languages_config", {})
            lang_config = all_languages_config.get(state.lang_id, {})
            
            user_files = state.lang_data.get('user_provided_files', {})

            if 'background_music' in user_files:
                override_path = user_files['background_music']
                if os.path.exists(override_path):
                    background_music_path = override_path
                    background_music_volume = user_files.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using user-provided background music: {os.path.basename(background_music_path)} with volume {background_music_volume}%", level=LogLevel.INFO)
                else:
                    logger.log(f"[{task_id}] User-provided background music not found at {override_path}. Skipping.", level=LogLevel.WARNING)

            if not background_music_path:
                default_path = lang_config.get("background_music_path")
                if default_path and os.path.exists(default_path):
                    background_music_path = default_path
                    background_music_volume = lang_config.get("background_music_volume", 100)
                    logger.log(f"[{task_id}] Using default background music: {os.path.basename(background_music_path)}", level=LogLevel.INFO)
            
            if background_music_path:
                config['background_music_path'] = background_music_path
                config['background_music_volume'] = background_music_volume
            # --- End Background Music Config ---

            worker = MontageWorker(task_id, config)
            worker.signals.finished.connect(self._on_montage_finished)
            worker.signals.error.connect(self._on_montage_error)
            worker.signals.progress_log.connect(self._on_montage_progress)
            self.threadpool.start(worker)
        except Exception as e:
            self._on_montage_error(task_id, f"Failed to start montage worker: {e}")

    @Slot(str, object)
    def _on_montage_finished(self, task_id, video_path):
        self.montage_semaphore.release()
        self._process_montage_queue()
        self.task_states[task_id].final_video_path = video_path
        self._set_stage_status(task_id, 'stage_montage', 'success')
        
        # Get file size and emit metadata
        try:
            state = self.task_states[task_id]
            file_size_bytes = os.path.getsize(video_path)
            file_size_gb = file_size_bytes / (1024 * 1024 * 1024)  # Convert to GB
            metadata_text = f"{file_size_gb:.2f} GB"
            self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_montage', metadata_text)
        except Exception as e:
            logger.log(f"[{task_id}] Failed to get video file size: {e}", level=LogLevel.WARNING)

        self._check_if_all_are_ready_or_failed()

    @Slot(str, str)
    def _on_montage_error(self, task_id, error):
        self.montage_semaphore.release()
        self._process_montage_queue()
        self._set_stage_status(task_id, 'stage_montage', 'error', error)
        self._check_if_all_are_ready_or_failed()
    
    @Slot(str, str)
    def _on_montage_progress(self, task_id, message):
        job_id = task_id.split('_')[0] if '_' in task_id else task_id
        self.task_progress_log.emit(job_id, message)
        
        # Parse percentage from message, e.g., "progress=45.20%"
        if "progress=" in message:
            try:
                parts = dict(re.findall(r'(\w+)=([^ |]+)', message))
                progress_str = parts.get('progress')
                if progress_str:
                    state = self.task_states[task_id]
                    self.stage_metadata_updated.emit(state.job_id, state.lang_id, 'stage_montage', progress_str)
            except Exception:
                pass
