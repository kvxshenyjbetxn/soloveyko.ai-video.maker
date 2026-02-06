import os
import re
import subprocess
import json
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
            
        # --- Skip check: If all target files are already videos, we don't need the worker ---
        VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
        if all(os.path.splitext(p)[1].lower() in VIDEO_EXTS for p in paths_to_animate):
            logger.log(f"[{task_id}] All requested animations already exist as videos. Skipping worker.", level=LogLevel.INFO)
            state.video_animation_count = len(paths_to_animate)
            state.videos_total_count = len(paths_to_animate)
            self._on_video_generation_finished(task_id, {'paths': paths_to_animate})
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
            'aspect_ratio': googler_settings.get("aspect_ratio", "IMAGE_ASPECT_RATIO_LANDSCAPE"),
            'video_semaphore': self.video_semaphore,
            'max_threads': googler_settings.get("max_video_threads", 1)
        }
        self._start_worker(VideoGenerationWorker, task_id, 'stage_images', config, self._on_video_generation_finished, self._on_video_generation_error)

    @Slot(str, object)
    def _on_video_generation_finished(self, task_id, result_dict):
        state = self.task_states[task_id]
        generated_videos = result_dict.get('paths', [])
        
        logger.log(f"[{task_id}] Video generation finished. Generated {len(generated_videos)} videos.", level=LogLevel.SUCCESS)

        # Since we changed VideoGenerationWorker to return ALL results (as a list with None for failures),
        # generated_videos is now a list of length 'video_count_animated'

        # CHECK FOR "SKIPPED" WORKER RESULT (Full Directory Scan from TaskProcessor)
        if 'total_prompts' in result_dict:
            logger.log(f"[{task_id}] Video generation skipped (existing files used). Update image_paths and ensure no images in video section.", level=LogLevel.INFO)
            
            # generated_videos here is state.image_paths before start_worker was called
            # We must ensure that if "Video at the beginning" is enabled, we don't have images in the prefix
            # actually, if TaskProcessor skipped it, it means it just scanned the directory.
            # But we want to re-sort so all videos are first.
            
            VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
            vids = [p for p in generated_videos if os.path.splitext(p)[1].lower() in VIDEO_EXTS]
            imgs = [p for p in generated_videos if os.path.splitext(p)[1].lower() not in VIDEO_EXTS]
            state.image_paths = vids + imgs

            # Since we have full valid list, status is success
            self._set_stage_status(task_id, 'stage_images', 'success')
            self._check_if_image_review_ready()
            if self.subtitle_barrier_passed:
                self._check_and_start_montages()
            return

        video_count_animated = getattr(state, 'video_animation_count', 0)
        
        # New merge logic: successful videos first, then rest (failed animations are discarded)
        success_videos = [p for p in generated_videos if p is not None]
        
        logger.log(
            f"[{task_id}] Processing video results: {len(success_videos)} videos success. "
            f"Total images present before merge: {len(state.image_paths)}. "
            f"Animation count target: {video_count_animated}.",
            level=LogLevel.DEBUG
        )
        
        remaining_images = state.image_paths[video_count_animated:]
        
        if success_videos:
            # Об'єднуємо та прибираємо дублікати (ретельно перевіряємо шлях, ігноруючи регістр на Windows)
            new_paths = []
            seen_abs = set()
            
            # success_videos - це нові (або знайдені) відео
            # remaining_images - це решта картинок
            for p in success_videos + remaining_images:
                abs_p = os.path.abspath(p).lower()
                if abs_p not in seen_abs:
                    new_paths.append(p)
                    seen_abs.add(abs_p)
            
            state.image_paths = new_paths
        else:
            # If 0 videos were produced, we keep the original image_paths as they are
            # so they can be used for the 'Quick Show' fallback.
            pass
        
        # Determine video generation status
        video_gen_status = 'success'
        if len(success_videos) < video_count_animated:
            video_gen_status = 'warning'
        if len(success_videos) == 0 and video_count_animated > 0:
            # Instead of marking as error, we'll fallback to Quick Show
            # So we use 'warning' status to allow montage to proceed
            video_gen_status = 'warning'

        if len(success_videos) == 0 and video_count_animated > 0: 
            logger.log(f"[{task_id}] Video animation produced 0 videos. Fallback to 'Quick Show' mode.", level=LogLevel.WARNING)
            state.fallback_to_quick_show = True

        # Combine statuses
        final_status = state.image_gen_status
        if final_status == 'success' and video_gen_status != 'success':
            final_status = video_gen_status 
        elif final_status == 'warning' and video_gen_status == 'warning':
            final_status = 'warning'

        error_message = None
        if final_status != 'success':
            if state.fallback_to_quick_show:
                error_message = f"Video animation failed (generated {len(success_videos)}/{video_count_animated}), using Quick Show mode for missing ones."
            else:
                error_message = f"Failed to generate all images and/or videos ({len(success_videos)}/{video_count_animated})."
            
        self._set_stage_status(task_id, 'stage_images', final_status, error_message)
        self._check_if_image_review_ready()
        
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
        self._check_if_image_review_ready()
        
        # Continue to montage
        if self.subtitle_barrier_passed:
            self._check_and_start_montages()

    def _check_and_start_montages(self):
        for task_id, state in self.task_states.items():
            if 'stage_montage' not in state.stages or state.status.get('stage_montage') != 'pending':
                continue
            
            # Find all stages that come before 'stage_montage'
            try:
                montage_idx = state.stages.index('stage_montage')
                prerequisite_stages = state.stages[:montage_idx]
            except ValueError:
                prerequisite_stages = []

            prerequisites_are_done = True
            failed_prerequisite = None
            
            for stage in prerequisite_stages:
                status = state.status.get(stage)
                if status in ['pending', 'processing', 'processing_video']:
                    prerequisites_are_done = False
                    break
                if status == 'error':
                    failed_prerequisite = stage
                    # We don't break here to ensure we caught any 'processing' ones first, 
                    # but actually if one is error and none are processing, we are "done" but failed.

            if prerequisites_are_done:
                if failed_prerequisite:
                    error_msg = f"Prerequisite stage '{failed_prerequisite}' failed."
                    self._set_stage_status(task_id, 'stage_montage', 'error', error_msg)
                    continue

                if not state.audio_path or not os.path.exists(state.audio_path):
                    # This might happen if 'stage_voiceover' was skipped but file doesn't exist
                    # or if the logic above somehow missed a processing stage.
                    # We'll log it as error only if the stage was supposed to be there.
                    if 'stage_voiceover' in state.stages:
                        # If we think prerequisites are done but audio is missing, it's a real error
                        self._set_stage_status(task_id, 'stage_montage', 'error', "Audio file missing")
                    else:
                        # Maybe it's a silent video? But usually montage needs audio.
                        # For now, keep the error as it was.
                        self._set_stage_status(task_id, 'stage_montage', 'error', "Audio file missing")
                    continue
                
                if not state.image_paths or len(state.image_paths) == 0:
                    self._set_stage_status(task_id, 'stage_montage', 'error', "No images available")
                    continue
                
                # Use per-task settings for review
                should_review = state.settings.get('image_review_enabled')
                if 'stage_images' not in state.stages or 'stage_images' in state.skipped_stages:
                    should_review = False
                
                if should_review and not state.is_image_reviewed:
                    # Still waiting for image review approval
                    continue
                
                self._start_montage(task_id)
        
        self._check_if_all_are_ready_or_failed()

    def _check_if_image_review_ready(self):
        # We only care about active tasks (those in self.task_states)
        tasks_with_images = [t for t in self.task_states.values() if 'stage_images' in t.stages]
        if not tasks_with_images:
            return

        all_images_done = True
        any_needs_review = False
        
        for state in tasks_with_images:
            # Check if stage_images is finished (not pending or processing)
            status = state.status.get('stage_images')
            if status in ['pending', 'processing', 'processing_video']:
                # Note: if it's 'error', it's still considered "done" for the sake of starting others
                all_images_done = False
                break
            
            if state.settings.get('image_review_enabled') and 'stage_images' not in state.skipped_stages:
                if not state.is_image_reviewed:
                    any_needs_review = True

        if all_images_done and any_needs_review and not self.image_review_notification_emitted:
            logger.log("All image generation tasks finished. Requesting user review.", level=LogLevel.SUCCESS)
            self.image_review_notification_emitted = True
            
            title = translator.translate('notification_image_review_title')
            body = translator.translate('notification_image_review_body')
            notification_manager.send_notification(f"{title}\n{body}")
            
            self.image_review_required.emit()

    def _check_if_all_are_ready_or_failed(self):
        # This now only checks if we need to emit notification about ALL tasks finishing or similar
        # (Actually TaskProcessor.check_if_all_finished does most of this)
        pass

    @Slot()
    def resume_all_montages(self):
        logger.log("User approved image review. Resuming/allowing montages.", level=LogLevel.INFO)
        for state in self.task_states.values():
            if 'stage_images' in state.stages:
                state.is_image_reviewed = True
        
        # Now that review is approved, try to start any montages that were waiting
        self._check_and_start_montages()

    def _start_montage(self, task_id):
        # --- Global Concurrency Check ---
        # Prevent duplicate queueing
        if task_id in self.pending_montages:
            logger.log(f"[{task_id}] Montage already in queue. Skipping duplicate request.", level=LogLevel.DEBUG)
            return

        state = self.task_states.get(task_id)
        if state and state.status.get('stage_montage') in ['processing', 'success']:
             logger.log(f"[{task_id}] Montage already {state.status.get('stage_montage')}. Skipping request.", level=LogLevel.DEBUG)
             return

        allow_simultaneous = self.settings.get("simultaneous_montage_and_subs", False)
        if not allow_simultaneous:
            if self._are_subtitles_running():
                logger.log(f"[{task_id}] Montage deferred. Subtitles are running and simultaneous execution is disabled.", level=LogLevel.INFO)
                self.pending_montages.append(task_id)
                return 

        self.pending_montages.append(task_id)
        self._process_montage_queue()

    def _process_montage_queue(self):
        # --- Global Concurrency Check ---
        # Note: We check again here because this method is called when montages finish too.
        allow_simultaneous = self.settings.get("simultaneous_montage_and_subs", False)
        
        while self.pending_montages:
            if not allow_simultaneous and self._are_subtitles_running():
                 # Cannot start new montages yet
                 break

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
                
            # --- IMPROVED SYNCHRONIZATION LOGIC ---
            text_chunks = state.lang_data.get('text_chunks')
            
            if text_chunks and state.subtitle_path and os.path.exists(state.subtitle_path):
                logger.log(f"[{task_id}] Starting synchronized montage with {len(text_chunks)} segments.", level=LogLevel.INFO)
                
                # Step 1: Get segment timings from subtitles
                segment_timings = self.align_segments_to_subtitles(
                    text_chunks, 
                    state.subtitle_path, 
                    task_id=task_id
                )
                
                if not segment_timings:
                    logger.log(f"[{task_id}] Failed to align segments to subtitles. Using fallback.", level=LogLevel.WARNING)
                else:
                    # Step 2: Determine which images exist for each segment
                    # Check expected extension from existing files
                    ext = '.png'
                    VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
                    
                    if final_image_paths:
                        first_ext = os.path.splitext(final_image_paths[0])[1].lower()
                        if first_ext in ['.jpg', '.jpeg']:
                            ext = first_ext
                        elif first_ext in VIDEO_EXTS:
                            ext = first_ext  # Could be video at beginning mode
                    
                    images_dir = os.path.dirname(final_image_paths[0]) if final_image_paths else state.dir_path
                    
                    # Step 3: Build final image list with gap-filling
                    # Structure: [(image_path, accumulated_duration), ...]
                    final_visuals = []
                    final_durations = []
                    accumulated_duration = 0.0
                    last_valid_image = None
                    pending_segments = []  # Segments waiting for an image
                    sync_offset = 0.0      # Time debt from short videos
                    
                    for i, timing in enumerate(segment_timings):
                        segment_duration = timing['duration']
                        segment_index = timing['index']
                        
                        # Try to find the image for this segment
                        # Check multiple possible formats: 0001.png, 1.png, 0001.jpg, 1.mp4, etc.
                        candidate_paths = []
                        
                        # Always check standard image extensions
                        for img_ext in ['.png', '.jpg', '.jpeg']:
                            candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1:04d}{img_ext}"))
                            candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1}{img_ext}"))
                            
                        # Also check for video versions
                        for v_ext in VIDEO_EXTS:
                            candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1:04d}{v_ext}"))
                            candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1}{v_ext}"))
                        
                        # Check specific 'ext' from detection just in case (e.g. specialized format)
                        if ext not in ['.png', '.jpg', '.jpeg'] and ext not in VIDEO_EXTS:
                             candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1:04d}{ext}"))
                             candidate_paths.append(os.path.join(images_dir, f"{segment_index + 1}{ext}"))

                        found_image = None
                        for cp in candidate_paths:
                            if os.path.exists(cp):
                                found_image = cp
                                break
                        
                        if found_image:
                            # Image/Video exists for this segment
                            
                            # --- GAP FILLING LOGIC ---
                            if pending_segments:
                                # We had pending segments waiting for an image
                                
                                # Check if the last item was a video or image
                                last_was_video = False
                                if last_valid_image:
                                    ext_last = os.path.splitext(last_valid_image)[1].lower()
                                    if ext_last in VIDEO_EXTS:
                                        last_was_video = True
                                
                                if last_valid_image and final_durations:
                                    if last_was_video:
                                        # USER REQUIREMENT: Do NOT freeze/extend video files to fill gaps.
                                        # Instead, accumulate this time as "debt" (sync_offset) 
                                        # which will be paid by the next static image.
                                        sync_offset += accumulated_duration
                                        logger.log(
                                            f"[{task_id}] Missing segments after video {os.path.basename(last_valid_image)}. "
                                            f"Adding {accumulated_duration:.2f}s to sync debt (Total: {sync_offset:.2f}s) to avoid freezing video.",
                                            level=LogLevel.DEBUG
                                        )
                                    else:
                                        # Last item was an image - safe to extend (freeze)
                                        final_durations[-1] += accumulated_duration
                                        logger.log(
                                            f"[{task_id}] Extended {os.path.basename(last_valid_image)} "
                                            f"to cover {len(pending_segments)} missing segment(s) "
                                            f"(+{accumulated_duration:.2f}s)",
                                            level=LogLevel.INFO
                                        )
                                elif not last_valid_image:
                                    # First image(s) were missing, this found_image covers them
                                    # If found_image is video, we might have issue if we try to stretch it.
                                    # But here we are setting 'segment_duration' for the CURRENT image.
                                    # We just add it to the current segment duration.
                                    # If current is video, logic below will handle debt calculation.
                                    segment_duration += accumulated_duration
                                    logger.log(
                                        f"[{task_id}] First {len(pending_segments)} segment(s) had no images. "
                                        f"Using {os.path.basename(found_image)} from start "
                                        f"(duration: {segment_duration:.2f}s)",
                                        level=LogLevel.INFO
                                    )
                                
                                pending_segments = []
                                accumulated_duration = 0.0
                            
                            
                            # --- VIDEO SYNC RIPPLE LOGIC (RESTORED) ---
                            is_video_file = os.path.splitext(found_image)[1].lower() in VIDEO_EXTS
                            
                            if is_video_file:
                                try:
                                    real_dur = self._get_video_file_duration(found_image)
                                    if real_dur > 0:
                                        # FORCE REAL DURATION (User requirement: no freezing, no trimming)
                                        # Calculate diff relative to the PLANNED duration
                                        # If Video (8s) > Text (2s) -> diff = 2 - 8 = -6s (Negative Offset)
                                        # If Video (5s) < Text (10s) -> diff = 10 - 5 = +5s (Positive Offset)
                                        
                                        diff = segment_duration - real_dur
                                        sync_offset += diff
                                        
                                        logger.log(f"[{task_id}] Video {os.path.basename(found_image)}: {real_dur:.2f}s (Text was {segment_duration:.2f}s). Offset change: {diff:+.2f}s. Accum Offset: {sync_offset:.2f}s", level=LogLevel.INFO)
                                        
                                        segment_duration = real_dur
                                except Exception as e:
                                    logger.log(f"[{task_id}] Failed to get duration for {os.path.basename(found_image)}: {e}", level=LogLevel.WARNING)
                            
                            elif sync_offset != 0:
                                # This is an image, and we have sync offset (positive or negative)
                                original = segment_duration
                                segment_duration += sync_offset
                                
                                # Protect against negative duration if we are trying to catch up too fast
                                if segment_duration < 0.1:
                                    # We can't speed up enough here. Swallow this image (min duration)
                                    # and carry over the remaining negative debt.
                                    remainder = segment_duration - 0.1 # e.g. -5 - 0.1 = -5.1
                                    segment_duration = 0.1
                                    sync_offset = remainder
                                    logger.log(f"[{task_id}] Image compressed to min 0.1s to catch up. Remaining debt: {sync_offset:.2f}s", level=LogLevel.WARNING)
                                else:
                                    # Debt fully paid (or surplus consumed)
                                    logger.log(f"[{task_id}] Sync Correction: Adjusted image {os.path.basename(found_image)} by {sync_offset:+.2f}s ({original:.2f}->{segment_duration:.2f}) to restore sync.", level=LogLevel.INFO)
                                    sync_offset = 0.0

                            final_visuals.append(found_image)
                            final_durations.append(segment_duration)
                            last_valid_image = found_image
                            
                        else:
                            # Image missing for this segment
                            pending_segments.append(segment_index)
                            accumulated_duration += segment_duration
                            logger.log(
                                f"[{task_id}] Segment {segment_index + 1} image missing. "
                                f"Accumulating {segment_duration:.2f}s for gap-fill.",
                                level=LogLevel.DEBUG
                            )
                    
                    # Handle any remaining pending segments at the end
                    if pending_segments and last_valid_image and final_durations:
                        final_durations[-1] += accumulated_duration
                        logger.log(
                            f"[{task_id}] Extended last image to cover {len(pending_segments)} "
                            f"trailing missing segment(s) (+{accumulated_duration:.2f}s)",
                            level=LogLevel.INFO
                        )
                    
                    
                    # --- FINAL DURATION CHECK ---
                    # Ensure total visual duration matches total audio duration strictly.
                    # This prevents the last image from freezing if audio has trailing silence/music.
                    audio_path = state.audio_path
                    
                    
                    # Fallback lookup for audio path if not directly in state
                    audio_candidates = []
                    if audio_path and os.path.exists(audio_path):
                        audio_candidates.append(audio_path)
                    
                    # Add standard voiceover path
                    audio_candidates.append(os.path.join(state.dir_path, f"voice_{state.lang_id}.mp3"))
                    # Add generic audio paths
                    audio_candidates.append(os.path.join(state.dir_path, "audio.mp3"))
                    audio_candidates.append(os.path.join(state.dir_path, "voice.mp3"))
                    
                    # Find first valid audio
                    valid_audio_path = None
                    for cand in audio_candidates:
                        if os.path.exists(cand):
                            valid_audio_path = cand
                            break
                    
                    # Last resort: find ANY audio file in folder
                    if not valid_audio_path:
                        try:
                            files = os.listdir(state.dir_path)
                            for f in files:
                                if f.lower().endswith(('.mp3', '.wav', '.m4a')):
                                    valid_audio_path = os.path.join(state.dir_path, f)
                                    logger.log(f"[{task_id}] Sync: Guessing audio path: {valid_audio_path}", level=LogLevel.WARNING)
                                    break
                        except: pass

                    if valid_audio_path and final_durations:
                         try:
                             total_audio_dur = self._get_video_file_duration(valid_audio_path)
                             
                             # Calculate effective visual duration accounting for transitions
                             # Each transition overlaps and consumes 'trans_dur' of time from the total sequence length
                             montage_settings = state.settings.get("montage", {})
                             trans_dur = float(montage_settings.get("transition_duration", 1.0))
                             enable_trans = montage_settings.get("enable_transitions", True)
                             num_files = len(final_durations)
                             
                             total_visual_dur_raw = sum(final_durations)
                             loss_per_trans = trans_dur
                             
                             # Total loss = (N-1) * trans_dur
                             # IF transitions are enabled and we have > 1 file
                             total_loss = 0.0
                             if enable_trans and num_files > 1:
                                 total_loss = (num_files - 1) * loss_per_trans
                                 
                             effective_visual_dur = total_visual_dur_raw - total_loss
                             
                             logger.log(f"[{task_id}] Sync Check (Audio: {os.path.basename(valid_audio_path)}): Audio={total_audio_dur:.2f}s, Visuals(Eff)={effective_visual_dur:.2f}s (Loss: {total_loss:.2f}s)", level=LogLevel.DEBUG)
                             
                             if total_audio_dur > effective_visual_dur:
                                 diff = total_audio_dur - effective_visual_dur
                                 # Allow a tiny tolerance, but generally extend
                                 if diff > 0.05:
                                     final_durations[-1] += diff
                                     logger.log(f"[{task_id}] FIXED: Extended last visual by {diff:.2f}s (incl. transition compensation) to match total audio duration.", level=LogLevel.INFO)
                                 else:
                                     logger.log(f"[{task_id}] Sync Perfect (diff {diff:.3f}s)", level=LogLevel.DEBUG)
                         except Exception as e:
                             logger.log(f"[{task_id}] Warning: Could not sync total duration: {e}", level=LogLevel.WARNING)
                    
                    # Step 4: Update paths and durations
                    if final_visuals:
                        final_image_paths = final_visuals
                        montage_settings['override_durations'] = final_durations
                        
                        # Log summary
                        total_duration = sum(final_durations)
                        avg_confidence = sum(t['confidence'] for t in segment_timings) / len(segment_timings) if segment_timings else 0
                        logger.log(
                            f"[{task_id}] Synchronized montage ready: "
                            f"{len(final_visuals)} visuals, "
                            f"total {total_duration:.2f}s, "
                            f"avg confidence: {avg_confidence:.0%}",
                            level=LogLevel.INFO
                        )
                        
                        # === GENERATE DEBUG FILE ===
                        try:
                            self._generate_sync_debug_file(
                                state.dir_path,
                                text_chunks,
                                segment_timings,
                                final_visuals,
                                final_durations,
                                task_id
                            )
                        except Exception as e:
                            logger.log(f"[{task_id}] Failed to generate sync debug file: {e}", level=LogLevel.WARNING)
                        
                    else:
                        logger.log(f"[{task_id}] No valid images found after gap-filling!", level=LogLevel.ERROR)
            
            # --- Legacy gap-filling for non-sync mode (preserve old behavior) ---
            elif text_chunks:
                # Old logic for backward compatibility
                full_image_list = []
                last_valid_image = None
                
                ext = '.png'
                if final_image_paths and final_image_paths[0].lower().endswith('.jpg'):
                    ext = '.jpg'
                elif final_image_paths and final_image_paths[0].lower().endswith('.jpeg'):
                    ext = '.jpeg'
                    
                images_dir = os.path.dirname(final_image_paths[0]) if final_image_paths else state.dir_path
                
                for i in range(len(text_chunks)):
                    candidate_name = f"{i+1:04d}{ext}"
                    candidate_path = os.path.join(images_dir, candidate_name)
                    
                    if os.path.exists(candidate_path):
                        full_image_list.append(candidate_path)
                        last_valid_image = candidate_path
                    else:
                        if last_valid_image:
                            full_image_list.append(last_valid_image)
                        else:
                            # Look for first valid
                            for k in range(len(text_chunks)):
                                fpath = os.path.join(images_dir, f"{k+1:04d}{ext}")
                                if os.path.exists(fpath):
                                    full_image_list.append(fpath)
                                    last_valid_image = fpath
                                    break
                
                if full_image_list:
                    final_image_paths = full_image_list

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

            # --- Initial Video Config ---
            initial_video_path = lang_config.get("initial_video_path")
            if initial_video_path and os.path.exists(initial_video_path):
                config['initial_video_path'] = initial_video_path
                logger.log(f"[{task_id}] Using initial video: {os.path.basename(initial_video_path)}", level=LogLevel.INFO)
            # --- End Initial Video Config ---

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
        
        # Check if we can unblock subtitles now
        if not self.settings.get("simultaneous_montage_and_subs", False):
            self._process_whisper_queue()
        
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
        
        # Check if we can unblock subtitles now
        if not self.settings.get("simultaneous_montage_and_subs", False):
            self._process_whisper_queue()

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
    
    def _generate_sync_debug_file(self, dir_path, text_chunks, segment_timings, final_visuals, final_durations, task_id):
        """
        Generates a debug file with synchronization details.
        
        Creates sync_debug.txt with a table showing:
        - Image number
        - Text segment assigned to this image
        - Display time range
        - Matched subtitle time range
        - Confidence score
        """
        from datetime import datetime
        
        debug_path = os.path.join(dir_path, "sync_debug.txt")
        
        def format_time(seconds):
            """Convert seconds to MM:SS.ms format"""
            mins = int(seconds // 60)
            secs = seconds % 60
            return f"{mins:02d}:{secs:05.2f}"
        
        with open(debug_path, 'w', encoding='utf-8') as f:
            # Header
            f.write("=" * 100 + "\n")
            f.write(f"SYNCHRONIZATION DEBUG REPORT\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"Task: {task_id}\n")
            f.write("=" * 100 + "\n\n")
            
            # Summary
            total_duration = sum(final_durations) if final_durations else 0
            avg_confidence = sum(t['confidence'] for t in segment_timings) / len(segment_timings) if segment_timings else 0
            
            f.write("SUMMARY\n")
            f.write("-" * 50 + "\n")
            f.write(f"Total Segments: {len(text_chunks)}\n")
            f.write(f"Final Visuals:  {len(final_visuals)}\n")
            f.write(f"Total Duration: {format_time(total_duration)} ({total_duration:.2f}s)\n")
            f.write(f"Avg Confidence: {avg_confidence:.0%}\n")
            f.write("\n")
            
            # Detailed table
            f.write("DETAILED SYNCHRONIZATION TABLE\n")
            f.write("=" * 100 + "\n")
            f.write(f"{'#':<4} {'Image':<20} {'Display Time':<20} {'Subtitle Match':<20} {'Conf':<8} {'Text Segment'}\n")
            f.write("-" * 100 + "\n")
            
            # Track cumulative display time
            current_display_time = 0.0
            visual_index = 0
            
            for i, timing in enumerate(segment_timings):
                segment_index = timing['index']
                segment_text = text_chunks[segment_index] if segment_index < len(text_chunks) else "[N/A]"
                
                # Truncate text for display
                text_preview = segment_text[:60] + "..." if len(segment_text) > 60 else segment_text
                text_preview = text_preview.replace('\n', ' ').replace('\r', '')
                
                # Subtitle match time
                sub_start = timing['start']
                sub_end = timing['end']
                sub_range = f"{format_time(sub_start)} - {format_time(sub_end)}"
                
                # Confidence
                confidence = timing['confidence']
                conf_str = f"{confidence:.0%}" if confidence > 0 else "EST"
                
                # Check if this segment has an image
                # (we need to match segment to visual)
                if visual_index < len(final_visuals):
                    image_name = os.path.basename(final_visuals[visual_index])
                    display_duration = final_durations[visual_index] if visual_index < len(final_durations) else timing['duration']
                    display_end = current_display_time + display_duration
                    display_range = f"{format_time(current_display_time)} - {format_time(display_end)}"
                    
                    # Write row
                    f.write(f"{segment_index + 1:<4} {image_name:<20} {display_range:<20} {sub_range:<20} {conf_str:<8} {text_preview}\n")
                    
                    current_display_time = display_end
                    visual_index += 1
                else:
                    # No image for this segment (was gap-filled)
                    f.write(f"{segment_index + 1:<4} {'[GAP-FILLED]':<20} {'(merged)':<20} {sub_range:<20} {conf_str:<8} {text_preview}\n")
            
            f.write("-" * 100 + "\n")
            f.write(f"\nLEGEND:\n")
            f.write(f"  #           - Segment number\n")
            f.write(f"  Image       - Image/video file used\n")
            f.write(f"  Display Time - When image is shown in final video\n")
            f.write(f"  Subtitle Match - Time range in subtitles where text was found\n")
            f.write(f"  Conf        - Confidence of text match (EST = estimated, no match found)\n")
            f.write(f"  Text Segment - Original text assigned to this segment\n")
            f.write("\n")
            
            # Full text segments
            f.write("\n" + "=" * 100 + "\n")
            f.write("FULL TEXT SEGMENTS\n")
            f.write("=" * 100 + "\n\n")
            
            for i, chunk in enumerate(text_chunks):
                timing = segment_timings[i] if i < len(segment_timings) else None
                if timing:
                    f.write(f"[Segment {i + 1}] ({format_time(timing['start'])} - {format_time(timing['end'])}, confidence: {timing['confidence']:.0%})\n")
                else:
                    f.write(f"[Segment {i + 1}]\n")
                f.write(f"{chunk}\n")
                f.write("-" * 50 + "\n\n")
        
        logger.log(f"[{task_id}] Sync debug file saved: {debug_path}", level=LogLevel.DEBUG)

    def _get_video_file_duration(self, file_path):
        """
        Helper to get video duration using ffprobe.
        """
        try:
            # Try getting json metadata
            cmd = [
                'ffprobe', 
                '-v', 'error', 
                '-show_entries', 'format=duration', 
                '-of', 'default=noprint_wrappers=1:nokey=1', 
                file_path
            ]
            
            # Start process without opening window on Windows
            startupinfo = None
            if os.name == 'nt':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                
            result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, startupinfo=startupinfo, text=True)
            
            if result.returncode == 0:
                duration = float(result.stdout.strip())
                return duration
        except Exception as e:
            logger.log(f"Error checking video duration for {os.path.basename(file_path)}: {e}", level=LogLevel.WARNING)
        return 0.0
