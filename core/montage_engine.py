import subprocess
import os
import math
import sys
import tempfile
import re
import platform
import random
import difflib
import shutil
from datetime import datetime, timezone
from utils.logger import logger, LogLevel
from utils.settings import settings_manager

class MontageEngine:
    def create_video(self, visual_files, audio_path, output_path, ass_path, settings, task_id=None, progress_callback=None, start_time=None, background_music_path=None, background_music_volume=None, **kwargs):
        prefix = f"[{task_id}] " if task_id else ""
        
        def log_progress(msg):
            """Log to card only (not to main log)"""
            if progress_callback:
                progress_callback(msg)
        
        # 1. ОТРИМУЄМО ДАНІ
        VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
        IMAGE_EXTS = ['.jpg', '.jpeg', '.png', '.webp', '.bmp']
        ALLOWED_EXTS = set(VIDEO_EXTS + IMAGE_EXTS)

        valid_files = []
        for f in visual_files:
            abs_path = os.path.abspath(f)
            ext = os.path.splitext(abs_path)[1].lower()
            
            # Strict extension check
            if ext not in ALLOWED_EXTS:
                # Silently skip unsupported/system files
                continue
            
            if os.path.basename(abs_path).startswith('.'):
                continue
                
            if os.path.exists(abs_path):
                valid_files.append(abs_path)
            else:
                logger.log(f"{prefix}[Warning] File not found, skipping: {f}", level=LogLevel.WARNING)
        
        visual_files = valid_files
        
        # --- PATH OPTIMIZATION ---
        # Determine a common root directory to set as CWD for FFmpeg
        # This allows using relative paths for inputs, significantly shortening the command line
        base_cwd = None
        if visual_files:
            # Use directory of the first file as the base
            base_cwd = os.path.dirname(os.path.abspath(visual_files[0]))
            # On Windows, path casing might vary, so we rely on what OS gives us
            logger.log(f"{prefix}[Montage] Optimization: base CWD set to: {base_cwd}", level=LogLevel.DEBUG)

        def get_opt_path(p):
            """Returns relative path if inside base_cwd, else absolute."""
            if not p: return p
            abs_p = os.path.abspath(p)
            if base_cwd:
                try:
                    # Check if file is on same drive (Windows)
                    if platform.system() == "Windows" and os.path.splitdrive(abs_p)[0].lower() != os.path.splitdrive(base_cwd)[0].lower():
                        return abs_p.replace("\\", "/")
                    
                    rel = os.path.relpath(abs_p, base_cwd)
                    # If relative path does not start with .. (meaning it is inside), use it
                    if not rel.startswith(".."):
                        return rel.replace("\\", "/")
                except ValueError:
                    pass
            return abs_p.replace("\\", "/")
        # -------------------------
        
        audio_dur = self._get_duration(audio_path)
        if audio_dur == 0: raise Exception("Аудіо пусте або не читається.")
        if not visual_files: raise Exception("Немає файлів (або жоден файл не знайдено).")

        num_files = len(visual_files)
        
        # Налаштування переходів
        enable_trans = settings.get('enable_transitions', True)
        trans_dur = settings.get('transition_duration', 0.5) if enable_trans else 0
        transition_effect = settings.get('transition_effect', 'fade')

        valid_transitions = [
            "fade", "wipeleft", "wiperight", "wipeup", "wipedown", 
            "slideleft", "slideright", "slideup", "slidedown", "circlecrop", 
            "rectcrop", "distance", "fadeblack", "fadewhite", "radial", 
            "smoothleft", "smoothright", "smoothup", "smoothdown", 
            "circleopen", "circleclose", "vertopen", "vertclose", 
            "horzopen", "horzclose", "dissolve", "pixelize", "diagtl", 
            "diagtr", "diagbl", "diagbr"
        ]
        codec = settings.get('codec', 'libx264')
        preset = settings.get('preset', 'medium')
        bitrate = settings.get('bitrate_mbps', 15)
        
        # --- НАЛАШТУВАННЯ ЕФЕКТІВ (НОВІ) ---
        enable_zoom = settings.get('enable_zoom', True)
        z_spd = settings.get('zoom_speed_factor', 1.0)
        z_int = settings.get('zoom_intensity', 0.15)

        enable_sway = settings.get('enable_sway', False)
        enable_sway = settings.get('enable_sway', False)
        enable_sway = settings.get('enable_sway', False)
        s_spd = settings.get('sway_speed_factor', 1.0)
        
        # New: GPU Acceleration for Effects
        # Re-enabled: Trying 'libplacebo' filter instead of 'glsl'
        use_gpu_shaders = settings.get('use_gpu_shaders', True)

        # 1.1 Detection of Aspect Ratio
        is_portrait = False
        
        # Priority 1: Check settings for explicit hints
        pollinations = settings.get('pollinations', {})
        googler = settings.get('googler', {})
        elevenlabs = settings.get('elevenlabs_image', {})
        
        if (pollinations.get('height', 0) > pollinations.get('width', 0) or 
            googler.get('aspect_ratio') == 'IMAGE_ASPECT_RATIO_PORTRAIT' or 
            elevenlabs.get('aspect_ratio') == '9:16'):
            is_portrait = True
            logger.log(f"{prefix}[Montage] Portrait mode detected from settings hint", level=LogLevel.INFO)

        # Priority 2: Check actual files (especially if settings were generic or missing)
        if not is_portrait and visual_files:
            portrait_count = 0
            check_count = min(20, len(visual_files)) # Check up to 20 files
            for i in range(check_count):
                w, h = self._get_dimensions(visual_files[i])
                if h > w:
                    portrait_count += 1
            
            # If at least 30% of checked files are portrait, we treat it as a portrait project
            # (To account for landscape intro/outro videos)
            if portrait_count >= (check_count * 0.3) and portrait_count > 0:
                is_portrait = True
                logger.log(f"{prefix}[Montage] Portrait mode detected from files ({portrait_count}/{check_count} portrait)", level=LogLevel.INFO)

        up_factor = settings.get('upscale_factor', 3.0)
        if is_portrait:
            base_w, base_h = 1080, 1920
        else:
            base_w, base_h = 1920, 1080
            
        up_w, up_h = int(base_w * up_factor), int(base_h * up_factor)

        # Effects & Watermark Settings
        overlay_effect_path = settings.get('overlay_effect_path')
        watermark_path = settings.get('watermark_path')
        watermark_size = settings.get('watermark_size', 20)  # % від ширини
        watermark_position = settings.get('watermark_position', 8)  # індекс позиції


        # 2. МАТЕМАТИКА ЧАСУ (Аудіо - головне)
        VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
        total_video_time = 0.0
        image_indices = []
        final_clip_durations = [0] * num_files
        
        for i, f in enumerate(visual_files):
            ext = os.path.splitext(f)[1].lower()
            if ext in VIDEO_EXTS:
                d = self._get_duration(f)
                if d == 0: d = 5.0 # fallback
                final_clip_durations[i] = d
                total_video_time += d
            else:
                image_indices.append(i)

        num_images = len(image_indices)
        total_trans_loss = (num_files - 1) * trans_dur
        required_raw_time = audio_dur + total_trans_loss
        time_for_all_images = required_raw_time - total_video_time

        # --- Нова логіка для спецобробки ---
        special_mode = settings.get("special_processing_mode", "Disabled")
        special_count = settings.get("special_processing_image_count", 5)
        special_dur = settings.get("special_processing_duration_per_image", 2.0)

        time_for_normal_images = time_for_all_images
        num_normal_images = num_images

        if special_mode == "Quick show" and num_images > 0:
            num_special_images = min(num_images, special_count)
            num_normal_images = num_images - num_special_images

            if num_normal_images == 0:
                # Всі зображення - "спеціальні", тому розтягуємо їх на весь доступний час
                img_duration = time_for_all_images / num_images if num_images > 0 else 0
                for i in range(num_images):
                    final_clip_durations[image_indices[i]] = img_duration
                time_for_normal_images = 0 # Весь час розподілено
            else:
                # Є і спеціальні, і звичайні зображення
                special_time_total = 0
                for i in range(num_special_images):
                    img_idx = image_indices[i]
                    final_clip_durations[img_idx] = special_dur
                    special_time_total += special_dur
                time_for_normal_images = time_for_all_images - special_time_total
        
        # For "Video at the beginning", no special time math is needed here.
        # The videos are already in visual_files and their durations are accounted for.

        # --- Кінець нової логіки ---

        img_duration = 0
        if num_normal_images > 0:
            if time_for_normal_images <= 0:
                logger.log("⚠️ УВАГА: Відео та спец. картинки довші за аудіо. Звичайні картинки будуть миттєві.", level=LogLevel.WARNING)
                img_duration = 0.1 
            else:
                img_duration = time_for_normal_images / num_normal_images

        # Призначаємо тривалість для "звичайних" картинок
        start_index = 0
        if special_mode == "Quick show":
            start_index = min(num_images, special_count)

        # Check for externally provided durations (e.g. from subtitle synchronization)
        # NOTE: override_durations is passed inside 'settings' dict from video_mixin
        override_durations = settings.get('override_durations') or kwargs.get('override_durations')
        
        # NEW: override_durations now applies to ALL visual files (videos + images)
        if override_durations:
            # Determine how many items we are overriding
            override_count = len(override_durations)
            
            logger.log(f"{prefix}[Montage] Applying synchronized timings. Transitions enabled: {enable_trans} ({trans_dur}s)", level=LogLevel.INFO)

            if override_count == num_files:
                # Scenario A: We have timings for EVERY file (Images + Videos)
                for i in range(num_files):
                    base_dur = override_durations[i]
                    # COMPENSATION FOR TRANSITIONS:
                    # If transitions are on (CPU MODE), every clip loses 'trans_dur' to the overlap.
                    # In GPU mode (Concat), clips are sequential, so NO overlap compensation is needed.
                    if enable_trans and not use_gpu_shaders:
                        base_dur += trans_dur
                    
                    final_clip_durations[i] = base_dur
                    
            elif override_count == (num_images - start_index):
                # Scenario B: We have timings strictly for the "Normal Images" part
                # (e.g. video files kept their natural duration, or Quick Show took first N images)
                
                # 1. First, handle the ones found in valid_files/visual_files that are NOT covered by override
                # (This loop might be redundant if we initialized correctly, but let's be safe)
                # Video files and special-mode images ALREADY have durations set above (lines 136 and 169).
                # However, they ALSO need transition compensation if they participate in the montage!
                
                if enable_trans:
                    for i in range(num_files):
                        # If it's already set (video or special img), add compensation ONLY if CPU mode
                        if final_clip_durations[i] > 0 and not use_gpu_shaders:
                            final_clip_durations[i] += trans_dur

                # 2. Now apply overrides to the specific images
                for i in range(start_index, num_images):
                    img_idx = image_indices[i]
                    # i - start_index gives the 0-based index in the overrides list
                    base_dur = override_durations[i - start_index]
                    
                    if enable_trans and not use_gpu_shaders:
                        base_dur += trans_dur
                        
                    final_clip_durations[img_idx] = base_dur
            else:
                logger.log(f"{prefix}[Montage] Warning: Override count ({override_count}) mismatch. Fallback to even.", level=LogLevel.WARNING)
                # Fallback
                for i in range(start_index, num_images):
                    img_idx = image_indices[i]
                    final_clip_durations[img_idx] = img_duration 
                    if enable_trans and not use_gpu_shaders: final_clip_durations[img_idx] += trans_dur
        else:
            # No overrides - Standard Even Distribution
            # Apply calculated duration + transition compensation
            for i in range(start_index, num_images):
                img_idx = image_indices[i]
                final_clip_durations[img_idx] = img_duration
            
            # If standard mode, we should also compensation for transitions for consistency?
            # Yes, because 'time_for_normal_images' calculation (line 144) ALREADY accounted for lost time:
            # required_raw_time = audio_dur + total_trans_loss
            # So 'img_duration' calculated there is ALREADY the "Raw Duration including Transition".
            # So we DO NOT add it again here for the standard even.
            pass
        
        # Log compact montage info (single line)
        log_msg = f"{prefix}[FFmpeg] Starting montage | Audio: {audio_dur:.2f}s, Files: {num_files} (Videos: {num_files - num_images}, Images: {num_images})"
        if special_mode == "Quick show":
            log_msg += f" (Special: {min(num_images, special_count)}x{special_dur:.2f}s)"
        if not override_durations:
            log_msg += f", Img duration: {img_duration:.2f}s"
        logger.log(log_msg, level=LogLevel.INFO)
        
        # 3. ГЕНЕРАЦІЯ FFmpeg КОМАНДИ
        inputs = []
        filter_parts = []
        fps = 30
        
        # LOG FFmpeg PATH for debugging
        ffmpeg_path = shutil.which("ffmpeg")
        logger.log(f"{prefix}[Debug] Using FFmpeg binary from: {ffmpeg_path}", level=LogLevel.INFO)
        # Check if assets ffmpeg is prioritized or system
        
        def fmt(val): return f"{val:.6f}".replace(",", ".")
        
        ffmpeg_cmd = [ffmpeg_path, "-y", "-hide_banner", "-loglevel", "error"]
        
        # --- NEW GPU PIPELINE (Sequential Clip Rendering) ---
        if use_gpu_shaders:
            logger.log(f"{prefix}[Montage] Using GPU Pipeline (Sequential Render + Concat)", level=LogLevel.INFO)

            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            # Save clips near the output file for easier debugging
            output_dir = os.path.dirname(output_path)
            gpu_temp_dir = os.path.join(output_dir, "clips", f"task_{task_id}")
            os.makedirs(gpu_temp_dir, exist_ok=True)
            logger.log(f"{prefix}[Montage] GPU Clips Dir: {gpu_temp_dir}", level=LogLevel.INFO)
            
            rendered_clips = []
            
            # 1. Render each clip individually
            for i, f in enumerate(visual_files):
                clip_out = os.path.join(gpu_temp_dir, f"clip_{i:04d}.mp4")
                
                # Setup duration and params
                this_dur = final_clip_durations[i]
                
                # Shader generation (Local relative to GPU temp dir)
                # Store shaders in a subfolder of gpu_temp_dir to keep things organized and clean project root
                shader_dir = os.path.join(gpu_temp_dir, 'shaders')
                os.makedirs(shader_dir, exist_ok=True)
                shader_filename = f"shader_{task_id}_{i}.frag"
                shader_abs_path = os.path.join(shader_dir, shader_filename)
                
                # Params matching CPU logic
                # FORCE ZOOM slightly only if EFFECTS are enabled to ensure it's visible even if sway is small
                # If both disabled -> 1.0 (no crop)
                if not enable_zoom and not enable_sway:
                    base_zoom_val = 1.0
                else:
                    base_zoom_val = 1.1 if enable_sway else 1.05
                z_amp_val = z_int if enable_zoom else 0.0
                z_spd_val = z_spd if enable_zoom else 0.0
                dur_val = this_dur if this_dur > 0 else 0.1
                s_spd_val = s_spd if enable_sway else 0.0
                
                amp_x = 0.026 * (up_factor / 2.0)
                if amp_x < 0.02: amp_x = 0.02
                amp_y = amp_x * 0.5
                
                # Check settings
                c_zoom = "1.0" if enable_zoom else "0.0"
                c_sway = "1.0" if enable_sway else "0.0"

                # Log params for first clip
                if i == 0:
                     logger.log(f"{prefix}[Debug] Shader Params: Zoom={enable_zoom} (Base={base_zoom_val}, Amp={z_amp_val}), Sway={enable_sway}, Dur={dur_val}", level=LogLevel.DEBUG)

                # FFmpeg Command for ONE CLIP
                # Check if input is image or video
                ext = os.path.splitext(f)[1].lower()
                is_vid = ext in VIDEO_EXTS
                
                # SHADER PARAMS CORRECTION FOR VIDEOS
                # If it's a video, we DO NOT want it to sway or breathe (zoom in/out).
                # We typically just want it static or slightly cropped.
                if is_vid:
                    # Force disable dynamic effects in shader
                    c_zoom = "0.0"
                    c_sway = "0.0"
                    # We can keep base_zoom_val (1.05 or 1.1) to ensure edge coverage
                else:
                    # Normal logic for images
                    c_zoom = "1.0" if enable_zoom else "0.0"
                    c_sway = "1.0" if enable_sway else "0.0"

                # Update the shader file with specific params for this clip
                # (We already generated the file above with potentially generic params, 
                #  but we passed params via Format String. Wait, the shader code WAS generated above at line 403.
                #  We need to Regenerate or Move the generation down.
                #  Actually, let's just move the Shader Generation code DOWN here, or update variables before generating.)
                
                # BUG PREVIOUSLY: Shader was generated BEFORE checking is_vid for params override.
                # RE-GENERATING SHADER CODE WITH CORRECT PARAMS

                shader_code = f"""//!HOOK MAIN
//!BIND HOOKED
//!DESC AI_Montage_Clip_{i}

vec4 hook() {{
    vec2 pos = HOOKED_pos;
    float FPS = {fmt(fps)};
    float DURATION = {fmt(dur_val)};
    
    // Use raw frame count. 
    float time = float(frame) / FPS;
    
    // Zoom
    float zoom_val = {fmt(base_zoom_val)};
    if ({c_zoom} > 0.5) {{
        float cycle = (time / DURATION) * {fmt(z_spd_val)};
        zoom_val = {fmt(base_zoom_val)} + {fmt(z_amp_val)} * (1.0 - cos(6.28318 * cycle)) / 2.0;
    }}
    
    // Sway
    vec2 offset = vec2(0.0);
    if ({c_sway} > 0.5) {{
        float sa = {fmt(s_spd_val)};
        float ax = {fmt(amp_x)}; 
        float ay = {fmt(amp_y)};
        offset.x = sin(time * 2.0 * sa) * ax + cos(time * 1.3 * sa) * (ax * 0.5);
        offset.y = cos(time * 1.7 * sa) * ay + sin(time * 0.9 * sa) * (ay * 0.5);
    }}
    
    vec2 center = vec2(0.5);
    pos = (pos - center) / zoom_val + center;
    pos = pos - offset;
    return HOOKED_tex(pos);
}}
"""
                with open(shader_abs_path, 'w', encoding='utf-8') as sf:
                    sf.write(shader_code)


                input_args = ["-i", f]
                pre_filters = [] # Filters before the shader
                
                if not is_vid:
                    input_args = ["-loop", "1"] + input_args
                
                # Filter chain construction
                # fade=t=in:st=0:d=0.5,fade=t=out:st={dur-0.5}:d=0.5
                fade_in = "fade=t=in:st=0:d=0.5"
                # Need to calculate fade out start
                fade_out_st = this_dur - 0.5
                if fade_out_st < 0: fade_out_st = 0
                fade_out = f"fade=t=out:st={fmt(fade_out_st)}:d=0.5"
                
                # Combine filters
                # [Input] -> [PreFilters (tpad)] -> [Format] -> [Libplacebo] -> [Fades] -> [Out]
                
                vf_chain = ["format=yuv420p"]
                if pre_filters:
                    vf_chain.extend(pre_filters)
                
                vf_chain.append(f"libplacebo=w={base_w}:h={base_h}:custom_shader_path='{shader_abs_path.replace(os.sep, '/').replace(':', '\\:')}'")
                vf_chain.append(fade_in)
                vf_chain.append(fade_out)
                
                vf = ",".join(vf_chain)
                
                clip_cmd = [ffmpeg_path, "-y", "-hide_banner", "-loglevel", "error"] + input_args + [
                    "-t", fmt(this_dur),
                    "-vf", vf,
                    "-c:v", "h264_amf", "-usage", "transcoding", "-rc", "cqp", "-qp_p", "23", "-qp_i", "23", "-quality", "speed",
                    "-r", str(fps), "-pix_fmt", "yuv420p",
                    clip_out
                ]
                
                # Optimization: Skip rendering if clip exists and has size
                if os.path.exists(clip_out) and os.path.getsize(clip_out) > 0:
                    logger.log(f"{prefix}[Montage] Clip {i} exists in cache, skipping render.", level=LogLevel.INFO)
                    rendered_clips.append(clip_out)
                    if progress_callback:
                        progress = int((i / len(visual_files)) * 80)
                        progress_callback(f"Skipping Clip {i+1}/{len(visual_files)} (Cached)...", progress)
                    continue

                # Run clip render
                try:
                    subprocess.run(clip_cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, startupinfo=startupinfo)
                    rendered_clips.append(clip_out)
                    if progress_callback:
                         # 0-80% for clip rendering
                         progress = int((i / len(visual_files)) * 80)
                         progress_callback(f"Rendering Clip {i+1}/{len(visual_files)}...", progress)
                except subprocess.CalledProcessError as e:
                    logger.log(f"{prefix}[Error] Failed to render clip {i}: {e.stderr}", level=LogLevel.ERROR)
                    # Use fallback? Or just fail? Let's fail for now as user wants shaders.
                    raise e

            # 2. Concat
            concat_list = os.path.join(gpu_temp_dir, "concat.txt")
            with open(concat_list, 'w', encoding='utf-8') as cf:
                for c in rendered_clips:
                    cf.write(f"file '{c.replace(os.sep, '/')}'\n")
            
            temp_joined = os.path.join(gpu_temp_dir, "joined.mp4")
            concat_cmd = [ffmpeg_path, "-y", "-f", "concat", "-safe", "0", "-i", concat_list, "-c", "copy", temp_joined]
            subprocess.run(concat_cmd, check=True, startupinfo=startupinfo)
            
            # 3. Final Assembly (Audio + Subs + Overlay)
            
            # Copy subs to temp folder to avoid path problems (Cyrillic etc)
            temp_ass_path = None
            if ass_path and os.path.exists(ass_path):
                temp_ass_path = os.path.join(gpu_temp_dir, "temp_subs.ass")
                try:
                    shutil.copy2(ass_path, temp_ass_path)
                    logger.log(f"{prefix}[Montage] Copied subtitles to temp (safe path): {temp_ass_path}", level=LogLevel.INFO)
                except Exception as e:
                    logger.log(f"{prefix}[Error] Failed to copy subs: {e}", level=LogLevel.ERROR)
                    temp_ass_path = ass_path # Fallback

            # Construct final command based on temp_joined
            # Use safer queue size (512) for inputs to prevent memory issues
            final_inp_args = ["-thread_queue_size", "512", "-i", temp_joined]
            
            # Audio with improved settings
            # Force stereo/44100 to avoid "Reconfiguring filter graph" spam which eats resources
            final_inp_args.extend(["-thread_queue_size", "512", "-i", audio_path])
            
            # Filters (Subs + Overlay)
            final_filters = []
            
            # --- OVERLAY TRIGGERS LOGIC ---
            # Retrieve triggers from settings
            triggers = settings.get('overlay_triggers', [])
            if isinstance(triggers, str):
                try:
                    import json
                    triggers = json.loads(triggers)
                except:
                    triggers = []

            # Current video stream label to chain filters
            # Initially it is [0:v] (the joined video)
            # But we are building a filtergraph string, so we will use implicit input [0:v] for first filter
            # or explicit if needed. Let's use chaining naming.
            current_v_label = "[0:v]" 
            
            overlay_index = 2 # 0 is video, 1 is audio. Next inputs start from 2.
            
            if triggers and isinstance(triggers, list):
                for trig in triggers:
                    if not isinstance(trig, dict): continue
                    
                    phrase = trig.get('value', '')
                    path = trig.get('path', '')
                    if not phrase or not path or not os.path.exists(path):
                        continue
                        
                    # Find timing
                    timing = self._get_text_timing(ass_path, phrase, prefix=prefix)
                    if not timing:
                        continue
                        
                    start_t = timing[0]
                    
                    # USER REQUEST: Overlay duration must depend on the FILE duration, not the phrase duration!
                    # Calculate duration of the trigger file
                    trigger_duration = 5.0 # Default for images
                    try:
                        f_dur = self._get_duration(path)
                        if f_dur and f_dur > 0:
                            trigger_duration = f_dur
                    except:
                        pass
                        
                    end_t = start_t + trigger_duration
                    logger.log(f"{prefix}[Montage] Trigger '{phrase}' starts at {start_t}s, file duration {trigger_duration}s -> ends at {end_t}s", level=LogLevel.INFO)
                    
                    # Add input with LOOPING to ensure it exists at the trigger time
                    trig_ext = os.path.splitext(path)[1].lower()
                    if trig_ext in IMAGE_EXTS:
                        final_inp_args.extend(["-thread_queue_size", "512", "-loop", "1", "-i", path])
                    else:
                        # Video/GIF - loop it so it's playing when enabled
                        # Note: If we just loop, it plays continuously. 'enable' visibility handles the show/hide.
                        # Ideally for video we'd want it to restart at start_t, but overlay filter doesn't support seeking the overlay input easily without complex setpts.
                        # Looping ensures frames exist.
                        final_inp_args.extend(["-thread_queue_size", "512", "-stream_loop", "-1", "-i", path])
                    
                    # Prepare overlay filter
                    # Position logic (default center) with optional offsets
                    off_x = int(trig.get('x', 0))
                    off_y = int(trig.get('y', 0))
                    
                    # (main_w-overlay_w)/2 + offset
                    # Ensure we handle negative/positive offsets correctly in the string
                    # FFmpeg expressions: X and Y
                    pos_x = f"(main_w-overlay_w)/2+{off_x}"
                    pos_y = f"(main_h-overlay_h)/2+{off_y}"
                    
                    # Define unique label for this step
                    next_v_label = f"[v_ov{overlay_index}]"
                    
                    # Filter string: [prev][new_input] overlay=... [next]
                    # REMOVED shortest=1 because it kills the main video if trigger is short!
                    ov_cmd = f"{current_v_label}[{overlay_index}:v]overlay=x={pos_x}:y={pos_y}:enable='between(t,{start_t},{end_t})'{next_v_label}"
                    
                    final_filters.append(ov_cmd)
                    
                    # Update current label for next filter
                    current_v_label = next_v_label
                    overlay_index += 1

            # If we added overlays, the last label is the input for subtitles.
            # If not, current_v_label is [0:v]. 
            # FFmpeg filtergraph: if first filter starts with [0:v], it consumes it.
            # If no filters, we map 0:v direct.
            
            # --- END OVERLAY TRIGGERS ---
            
            # Force audio format stabilization
            # aformat=channel_layouts=stereo:sample_rates=44100
            final_audio_filter = "aformat=channel_layouts=stereo:sample_rates=44100"
            
            # Subs
            if temp_ass_path:
                 # Ensure forward slashes and escaped colon even for temp path
                 # For filter_complex, we need consistent escaping.
                 # If we used overlay, we must use current_v_label as input
                 
                 # Escape path for subtitles filter:
                 # Windows: D\:/path -> D\\:/path
                 # And for filter string: 'filename'
                 
                 # Simple slash replacement
                 ass_path_fwd = temp_ass_path.replace("\\", "/")
                 # Escape colon in drive letter
                 ass_path_esc = ass_path_fwd.replace(":", "\\:")
                 
                 # Subtitles filter
                 # subtitles='path'
                 subs_filter = f"subtitles='{ass_path_esc}'"
                 
                 if current_v_label != "[0:v]":
                     # We have a chain from overlays
                     # [last_ov]subtitles=...[v_out]
                     final_filters.append(f"{current_v_label}{subs_filter}[v_final]")
                     current_v_label = "[v_final]"
                 else:
                     # No overlays, direct subs on input
                     # subtitles=...
                     # We MUST use explicit naming to map it correctly
                     # [0:v]subtitles=...[v_final]
                     final_filters.append(f"[0:v]{subs_filter}[v_final]")
                     current_v_label = "[v_final]"
            else:
                # No subs. If we had overlays, current_v_label is [v_ovN] or similar.
                pass

            # (Removed redundant re-definition of inputs)

            final_cmd = [ffmpeg_path, "-y", "-hide_banner"] + final_inp_args

            # Join filters
            if final_filters:
                # If we have a chain, join with semicolon
                vf_string = ";".join(final_filters)
                final_cmd.extend(["-filter_complex", vf_string])
                
                # If we used valid labels, we need to map the RESULT.
                if current_v_label != "[0:v]" and current_v_label != "":
                     # Map the final label WITH BRACKETS to avoid parsing errors
                     final_cmd.extend(["-map", current_v_label])
                else:
                     pass 
            else:
                 # No filters at all
                 final_cmd.extend(["-map", "0:v"])

            # Map audio (always 1:a)
            final_cmd.extend(["-map", "1:a"])
            
            # Encoding props for final output
            # User wants FULL GPU PIPELINE. Reverting to h264_amf but with SAFER buffer settings.
            
            # Allow user to override bitrate and quality via settings
            bitrate_val = settings.get('bitrate_mbps', 5)
            bitrate = f"{bitrate_val}M"
            
            # For AMF, buffer size should be ~2x bitrate for safety
            buf_size = f"{bitrate_val * 2}M"

            # UI key is 'video_quality' (choice: speed, balanced, quality)
            quality = settings.get('video_quality', 'speed') 
            
            logger.log(f"{prefix}[Montage] Final Encode: {bitrate} (buf: {buf_size}), Quality: {quality}", level=LogLevel.INFO)

            # Add -stats to show progress line (frame=... speed=...)
            new_args = ["-stats", "-max_interleave_delta", "0"]
            
            # Use COPY for audio to reduce CPU/buffer usage
            audio_codec = ["-c:a", "copy"] 

            # --- DYNAMIC CODEC SELECTION ---
            codec_choice = settings.get('codec', 'h264_amf')
            preset_choice = settings.get('preset', 'medium')
            
            video_args = []
            
            if codec_choice == 'h264_amf':
                # AMD AMF
                amf_quality = settings.get('video_quality', 'speed')
                video_args = [
                    "-c:v", "h264_amf",
                    "-usage", "transcoding",
                    "-b:v", bitrate, "-maxrate", bitrate, "-bufsize", buf_size,
                    "-quality", amf_quality,
                    "-pix_fmt", "yuv420p",
                ]
                
            elif codec_choice == 'h264_nvenc':
                # NVIDIA NVENC
                # Map standard presets to NVENC p-presets
                nvenc_preset_map = {
                    'ultrafast': 'p1', 'superfast': 'p1', 'veryfast': 'p2',
                    'faster': 'p3', 'fast': 'p3', 'medium': 'p4',
                    'slow': 'p5', 'slower': 'p6', 'veryslow': 'p7'
                }
                nv_preset = nvenc_preset_map.get(preset_choice, 'p4')
                video_args = [
                    "-c:v", "h264_nvenc",
                    "-preset", nv_preset,
                    "-b:v", bitrate, "-maxrate", bitrate, "-bufsize", buf_size,
                    "-pix_fmt", "yuv420p",
                ]
                
            elif codec_choice == 'libx264':
                # CPU H.264
                video_args = [
                    "-c:v", "libx264",
                    "-preset", preset_choice,
                    "-b:v", bitrate, "-maxrate", bitrate, "-bufsize", buf_size,
                    "-pix_fmt", "yuv420p",
                ]
                
            elif codec_choice == 'libx265':
                # CPU H.265
                video_args = [
                    "-c:v", "libx265",
                    "-preset", preset_choice,
                    "-b:v", bitrate, "-maxrate", bitrate, "-bufsize", buf_size,
                    "-pix_fmt", "yuv420p",
                    "-tag:v", "hvc1"
                ]
                
            elif codec_choice == 'h264_videotoolbox':
                # Mac Hardware
                video_args = [
                    "-c:v", "h264_videotoolbox",
                    "-b:v", bitrate, 
                    "-pix_fmt", "yuv420p",
                ]
                
            else:
                # Fallback
                logger.log(f"{prefix}[Montage] Unknown codec '{codec_choice}', using libx264 fallback.", level=LogLevel.WARNING)
                video_args = [
                    "-c:v", "libx264",
                    "-preset", "superfast",
                    "-b:v", bitrate, "-maxrate", bitrate, "-bufsize", buf_size,
                    "-pix_fmt", "yuv420p",
                ]

            final_cmd.extend(new_args)
            final_cmd.extend(video_args)
            final_cmd.extend(audio_codec)
            final_cmd.extend([
                "-max_muxing_queue_size", "2048",
                "-shortest",
                output_path
            ])
            
            logger.log(f"{prefix}[Montage] Assembling final video on GPU...", level=LogLevel.INFO)
            
            # NO CPU FALLBACK - RAW EXECUTION AS REQUESTED
            subprocess.run(final_cmd, check=True, startupinfo=startupinfo)
            
            if progress_callback: progress_callback("Done!", 100)
            
            logger.log(f"{prefix}[Montage] GPU Pipeline Complete: {output_path}", level=LogLevel.INFO)
            
            # Cleanup temporary clips and shaders AFTER successful render
            try:
                if os.path.exists(gpu_temp_dir):
                    shutil.rmtree(gpu_temp_dir, ignore_errors=True)
                    logger.log(f"{prefix}[Montage] Cleaned up temp clips: {gpu_temp_dir}", level=LogLevel.DEBUG)
            except Exception as e:
                 logger.log(f"{prefix}[Montage] Cleanup warning: {e}", level=LogLevel.WARNING)

            return output_path

        # --- END GPU PIPELINE ---
        
        # --- OLD CPU/FILTER GRAPH PIPELINE (Fallback) ---
        for i, f in enumerate(visual_files):
            # Initialize loop variables
            abs_path = os.path.abspath(f)
            ext = os.path.splitext(f)[1].lower()
            is_video = ext in VIDEO_EXTS
            this_dur = final_clip_durations[i]
            this_dur_str = fmt(this_dur)

            if not os.path.exists(abs_path):
                 logger.log(f"{prefix}[Error] Input file not found: {abs_path}", level=LogLevel.ERROR)
                 raise Exception(f"Input file missing: {abs_path}")

            # Check if this is a video that needs looping BEFORE adding to inputs
            # Add inputs with optional stream_loop
            inputs.append("-thread_queue_size"); inputs.append("4096")
            inputs.append("-i"); inputs.append(get_opt_path(f))
            v_in = f"[{i}:v]"; v_out = f"v{i}_final"
            
            if is_video:
                # Target duration is set in final_clip_durations (could be from override_durations)
                target_dur = this_dur
                actual_dur = self._get_duration(abs_path) or 5.0
                # Determine if we need to extend (freeze last frame) or trim
                # We prioritize simple extension using tpad instead of complex looping
                
                if target_dur > actual_dur + 0.02: # Allow small tolerance
                    # Video is shorter than target. We MUST pad it (freeze last frame) 
                    # so that transitions (xfade) have enough material to overlap.
                    pad_dur = target_dur - actual_dur
                    logger.log(f"{prefix}[Montage] Video {i+1}: extending by {pad_dur:.2f}s (tpad) to match target {target_dur:.2f}s", level=LogLevel.INFO)
                    
                    vf = (
                        f"{v_in}tpad=stop_mode=clone:stop_duration={pad_dur:.3f},"
                        f"scale={base_w}:{base_h},"
                        f"scale=1.08*iw:-1,crop={base_w}:{base_h}:0:0,"
                        f"format=yuv420p,setsar=1,fps={fps},"
                        f"setpts=PTS-STARTPTS[{v_out}]"
                    )
                elif target_dur < actual_dur * 0.95:
                    logger.log(f"{prefix}[Montage] Video {i+1}: trimming to {target_dur:.2f}s (original: {actual_dur:.2f}s)", level=LogLevel.DEBUG)
                    # Trim video to target duration
                    vf = (
                        f"{v_in}trim=duration={this_dur_str},"
                        f"scale={base_w}:{base_h},"
                        f"scale=1.08*iw:-1,crop={base_w}:{base_h}:0:0,"
                        f"format=yuv420p,setsar=1,fps={fps},"
                        f"setpts=PTS-STARTPTS[{v_out}]"
                    )
                else:
                    # Use video as-is (within tolerance)
                    vf = (
                        f"{v_in}scale={base_w}:{base_h},"
                        f"scale=1.08*iw:-1,crop={base_w}:{base_h}:0:0,"
                        f"format=yuv420p,setsar=1,fps={fps},"
                        f"setpts=PTS-STARTPTS[{v_out}]"
                    )
                filter_parts.append(vf)
            else:
                v_up = f"v{i}_up"
                # For GPU shaders, we want a stream, so we'll handle scaling/looping differently below if enabled
                if not use_gpu_shaders:
                    # ORIGINAL CPU (Zoompan) PATH
                    scale_cmd = (
                        f"{v_in}scale={up_w}:{up_h}:force_original_aspect_ratio=increase,"
                        f"crop={up_w}:{up_h},"
                        f"format=yuv420p,setsar=1[{v_up}]"
                    )
                    filter_parts.append(scale_cmd)

                    # --- МАТЕМАТИКА ЕФЕКТІВ (CPU) ---
                    base_zoom = 1.1 if enable_sway else 1.0
                    
                    if enable_zoom:
                        z_amp = z_int 
                        cycle = f"((on/{fps})/{this_dur_str}) * {fmt(z_spd)}"
                        z_expr = f"{fmt(base_zoom)}+{fmt(z_amp)}*(1-cos(6.283*{cycle}))/2"
                    else:
                        z_expr = fmt(base_zoom)

                    if enable_sway:
                        base_amp_x = 50 * up_factor
                        base_amp_y = 25 * up_factor
                        
                        freq_x1 = fmt(0.02 * s_spd)
                        freq_x2 = fmt(0.05 * s_spd)
                        freq_y1 = fmt(0.025 * s_spd)
                        freq_y2 = fmt(0.06 * s_spd)

                        val_x = f"sin(on*{freq_x1})*{base_amp_x} + cos(on*{freq_x2})*{base_amp_x/2}"
                        val_y = f"cos(on*{freq_y1})*{base_amp_y} + sin(on*{freq_y2})*{base_amp_y/2}"
                        x_expr = f"iw/2-(iw/zoom/2)+{val_x}"
                        y_expr = f"ih/2-(ih/zoom/2)+{val_y}"
                    else:
                        x_expr = "iw/2-(iw/zoom/2)"
                        y_expr = "ih/2-(ih/zoom/2)"

                    d_frames = int(this_dur * fps) + 5
                    zoom_cmd = (
                        f"[{v_up}]zoompan=z='{z_expr}':x='{x_expr}':y='{y_expr}':"
                        f"d={d_frames}:s={base_w}x{base_h}:fps={fps},"
                        f"setpts=PTS-STARTPTS[{v_out}]"
                    )
                    filter_parts.append(zoom_cmd)
                
                else:
                    # NEW GPU (GLSL) PATH
                    # 1. Scale input to upscaled resolution
                    v_scaled = f"v{i}_scaled"
                    scale_cmd = (
                        f"{v_in}scale={up_w}:{up_h}:force_original_aspect_ratio=increase,"
                        f"crop={up_w}:{up_h},"
                        f"format=yuv420p,setsar=1[{v_scaled}]"
                    )
                    filter_parts.append(scale_cmd)
                    
                    # 2. Loop and Trim to create a video stream of correct duration
                    v_looped = f"v{i}_looped"
                    # loop=-1 makes it infinite, trim cuts it to duration. 
                    # setpts=PTS-STARTPTS makes timestamps start at 0 for the shader 'time' variable.
                    loop_trim_cmd = (
                        f"[{v_scaled}]loop=loop=-1:size=1:start=0,"
                        f"trim=duration={this_dur_str},"
                        f"setpts=PTS-STARTPTS[{v_looped}]"
                    )
                    filter_parts.append(loop_trim_cmd)
                    
                    # 3. Apply GLSL Shader
                    try:
                        # Direct shader generation (matching user's working example perfectly)
                        
                        # Prepare values
                        base_zoom_val = 1.1 if enable_sway else 1.0
                        z_amp_val = z_int if enable_zoom else 0.0
                        z_spd_val = z_spd if enable_zoom else 0.0
                        dur_val = this_dur if this_dur > 0 else 0.1
                        s_spd_val = s_spd if enable_sway else 0.0
                        
                        # Sway amplitudes
                        amp_x = 0.026 * (up_factor / 2.0)
                        if amp_x < 0.02: amp_x = 0.02
                        amp_y = amp_x * 0.5
                        
                        # Conditions for shader
                        c_zoom = "1.0" if enable_zoom else "0.0"
                        c_sway = "1.0" if enable_sway else "0.0"

                        # EXACT SHADER FROM USER EXAMPLE (with dynamic values inserted)
                        shader_code = f"""//!HOOK MAIN
//!BIND HOOKED
//!DESC AI_Montage_Clip_{i}

vec4 hook() {{
    vec2 pos = HOOKED_pos;
    
    float FPS = {fmt(fps)};
    float DURATION = {fmt(dur_val)};
    
    float time = float(frame) / FPS;
    
    // Zoom
    float zoom_val = {fmt(base_zoom_val)};
    if ({c_zoom} > 0.5) {{
        float cycle = (time / DURATION) * {fmt(z_spd_val)};
        zoom_val = {fmt(base_zoom_val)} + {fmt(z_amp_val)} * (1.0 - cos(6.28318 * cycle)) / 2.0;
    }}
    
    // Sway
    vec2 offset = vec2(0.0);
    if ({c_sway} > 0.5) {{
        float sa = {fmt(s_spd_val)};
        float ax = {fmt(amp_x)}; 
        float ay = {fmt(amp_y)};
        offset.x = sin(time * 2.0 * sa) * ax + cos(time * 1.3 * sa) * (ax * 0.5);
        offset.y = cos(time * 1.7 * sa) * ay + sin(time * 0.9 * sa) * (ay * 0.5);
    }}
    
    // Transform
    vec2 center = vec2(0.5);
    pos = (pos - center) / zoom_val + center;
    pos = pos - offset;
    
    return HOOKED_tex(pos);
}}
"""
                        # Save temp shader file LOCAL (to be safer with windows paths)
                        # We use a folder 'temp_shaders' inside the current working directory or execution dir
                        shader_dir = os.path.join(os.getcwd(), 'temp_shaders')
                        if not os.path.exists(shader_dir):
                            os.makedirs(shader_dir, exist_ok=True)

                        shader_filename = f"shader_{task_id}_{i}.frag"
                        shader_abs_path = os.path.join(shader_dir, shader_filename)
                        
                        with open(shader_abs_path, 'w', encoding='utf-8') as f:
                            f.write(shader_code)
                            
                        # Apply GPU filter (libplacebo)
                        v_shaded = f"v{i}_shaded"
                        
                        # Use ABSOLUTE PATH with specific escaping for Windows FFmpeg filters
                        # Colon is a separator in filters, so C:\path becomes C\:/path
                        # Backslashes are escape chars, so we use forward slashes.
                        # Single quotes around the path handle spaces properly.
                        
                        # 1. Absolute path
                        shader_arg = os.path.abspath(shader_abs_path)
                        # 2. Normalize slashes
                        shader_arg = shader_arg.replace('\\', '/')
                        # 3. Escape the drive colon (e.g. C: -> C\:)
                        # CRITICAL: Colon is a filter option separator, even in quotes sometimes on Windows ffmpeg builds
                        shader_arg = shader_arg.replace(':', '\\:')
                        
                        # We enclose in single quotes just in case
                        glsl_cmd = f"[{v_looped}]libplacebo=w={base_w}:h={base_h}:custom_shader_path='{shader_arg}'[{v_shaded}]"
                        filter_parts.append(glsl_cmd)
                        
                        # 4. Final format conversion back to YUV
                        # Convert to yuv420p and set PTS again to be safe
                        conv_cmd = f"[{v_shaded}]format=yuv420p,setpts=PTS-STARTPTS[{v_out}]"
                        filter_parts.append(conv_cmd)
                        
                    except Exception as e:
                        logger.log(f"{prefix}[Error] Failed to apply GPU shader: {e}. Falling back to standard scaling.", level=LogLevel.ERROR)
                        # Fallback: Just scale to base resolution (no zoom/sway)
                        fallback_cmd = (
                            f"[{v_looped}]scale={base_w}:{base_h},"
                            f"format=yuv420p,setpts=PTS-STARTPTS[{v_out}]"
                        )
                        filter_parts.append(fallback_cmd)

        # 4. TRANSITIONS
        if enable_trans and num_files > 1:
            curr = "[v0_final]"
            current_offset = final_clip_durations[0] - trans_dur
            trans_dur_str_filt = fmt(trans_dur)
            for i in range(1, num_files):
                next_stream = f"[v{i}_final]"; target = f"[v_m{i}]"
                off_str = fmt(current_offset)
                
                current_trans = transition_effect
                if current_trans == "random":
                     current_trans = random.choice(valid_transitions)
                elif current_trans not in valid_transitions:
                     # Warn only once or simple fallback
                     current_trans = "fade"
                
                # logger.log(f"{prefix}[FFmpeg] Transition {i}->{i+1}: {current_trans}", level=LogLevel.DEBUG)

                xfade = (
                    f"{curr}{next_stream}xfade=transition={current_trans}:"
                    f"duration={trans_dur_str_filt}:offset={off_str}{target}"
                )
                filter_parts.append(xfade)
                curr = target
                current_offset += (final_clip_durations[i] - trans_dur)
            final_v = curr
        else:
            ins = "".join([f"[v{i}_final]" for i in range(num_files)])
            concat = f"{ins}concat=n={num_files}:v=1:a=0[v_concat]"
            filter_parts.append(concat)
            final_v = "[v_concat]"

        # 5. SUBS & AUDIO
        output_v_stream = final_v 
        if ass_path and os.path.exists(ass_path):
            ass_clean = ass_path.replace("\\", "/").replace(":", "\\:").replace("'", "\\'")
            subs = f"{final_v}subtitles='{ass_clean}'[v_out]"
            filter_parts.append(subs)
            output_v_stream = "[v_out]"
            final_v = output_v_stream # Update final_v for next steps

        # 6. OVERLAY EFFECT
        # To avoid index confusion, let's restructure input additions.
        # Current inputs: [img/video 0] ... [img/video N-1]
        
        # We need to add effect and watermark inputs *before* we finalize the command generation, 
        # but the logic above already constructs the command heavily relying on indices.
        
        # Let's add extra inputs here and track their indices relative to what we have so far.
        current_input_count = len(visual_files)
        
        if overlay_effect_path and os.path.exists(overlay_effect_path):
            logger.log(f"{prefix}[FFmpeg] Adding overlay effect: {os.path.basename(overlay_effect_path)}", level=LogLevel.INFO)
            inputs.extend(["-stream_loop", "-1", "-thread_queue_size", "4096", "-i", get_opt_path(overlay_effect_path)])
            effect_index = current_input_count
            current_input_count += 1
            
            
            eff_v = f"[v_eff_scaled]"
            # Force yuva420p to ensure alpha channel is preserved/respected if present
            # Застосовуємо effect_speed для зміни швидкості відтворення ефекту
            scale_eff = f"[{effect_index}:v]format=yuva420p,scale={base_w}:{base_h}:force_original_aspect_ratio=increase,crop={base_w}:{base_h}{eff_v}"
            filter_parts.append(scale_eff)
            
            v_overlaid = f"[v_overlaid]"
            # Use 'overlay' filter. shortest=1 ensures it stops when the main video stops (though we use -shortest on output too)
            overlay_cmd = f"{final_v}{eff_v}overlay=0:0:shortest=1{v_overlaid}"
            filter_parts.append(overlay_cmd)
            final_v = v_overlaid

        # 6.5. DYNAMIC TRIGGER OVERLAYS
        overlay_triggers = settings.get('overlay_triggers', [])
        if overlay_triggers:
            logger.log(f"{prefix}[FFmpeg] Processing {len(overlay_triggers)} potential overlay triggers.", level=LogLevel.INFO)
            for i, trigger in enumerate(overlay_triggers):
                t_type = trigger.get('type', 'text')
                t_val = trigger.get('value', '')
                t_path = trigger.get('path', '')
                
                if not t_path or not os.path.exists(t_path):
                    continue
                
                start_time = None
                if t_type == 'time':
                    start_time = self._parse_time(t_val)
                else: # text
                    start_time = self._get_text_timing(ass_path, t_val, prefix)
                
                if start_time is None:
                    logger.log(f"{prefix}[FFmpeg] Trigger '{t_val}' NOT applied: Could not determine start time.", level=LogLevel.WARNING)
                    continue

                logger.log(f"{prefix}[FFmpeg] Trigger '{t_val}' found/set at {start_time:.2f}s. Applying effect: {os.path.basename(t_path)}", level=LogLevel.INFO)

                # Add input
                # abs_t_path is for Python checks, get_opt_path(t_path) is for ffmpeg arg
                inputs.extend(["-thread_queue_size", "4096", "-i", get_opt_path(t_path)])
                trig_index = current_input_count
                current_input_count += 1
                
                # Get extension to decide behavior
                ext = os.path.splitext(t_path)[1].lower()
                is_image = ext in IMAGE_EXTS
                
                trig_v_ready = f"[v_trig_{i}_ready]"
                s_str = fmt(start_time)
                
                # Apply format, scale and pts
                scale_trig = (
                    f"[{trig_index}:v]format=yuva420p,scale={base_w}:{base_h}:force_original_aspect_ratio=increase,"
                    f"crop={base_w}:{base_h},setpts=PTS-STARTPTS+{s_str}/TB{trig_v_ready}"
                )
                filter_parts.append(scale_trig)
                
                v_trig_out = f"[v_trig_{i}_out]"
                off_x = trigger.get('x', 0)
                off_y = trigger.get('y', 0)
                
                if is_image:
                    # For images, we use a fixed 5s duration
                    e_str = fmt(start_time + 5.0)
                    overlay_trig = f"{final_v}{trig_v_ready}overlay={off_x}:{off_y}:enable='between(t,{s_str},{e_str})'{v_trig_out}"
                else:
                    # For videos, play until end of file (natural duration)
                    # eof_action=pass ensures it disappears when animation ends
                    overlay_trig = f"{final_v}{trig_v_ready}overlay={off_x}:{off_y}:eof_action=pass:enable='gte(t,{s_str})'{v_trig_out}"
                
                filter_parts.append(overlay_trig)
                final_v = v_trig_out

        # 7. WATERMARK
        if watermark_path and os.path.exists(watermark_path):
            logger.log(f"{prefix}[FFmpeg] Adding watermark: {os.path.basename(watermark_path)}", level=LogLevel.INFO)
            inputs.extend(["-thread_queue_size", "4096", "-i", get_opt_path(watermark_path)])
            wm_index = current_input_count
            current_input_count += 1
            
            
            wm_v = f"[v_wm]"
            # Обчислюємо розмір вотермарки: watermark_size відсотків від 1920px
            wm_width = int(base_w * (float(watermark_size) / 100.0))
            logger.log(f"{prefix}[FFmpeg] Watermark size: {watermark_size}% = {wm_width}px", level=LogLevel.INFO)
            scale_wm = f"[{wm_index}:v]scale={wm_width}:-1{wm_v}"
            filter_parts.append(scale_wm)
            
            v_wm_out = f"[v_wm_out]"
            
            # Position mapping:
            # 0: top-left, 1: top-center, 2: top-right
            # 3: center-left, 4: center, 5: center-right
            # 6: bottom-left, 7: bottom-center, 8: bottom-right
            padding = 30
            position_map = {
                0: f"{padding}:{padding}",  # top-left
                1: f"(main_w-overlay_w)/2:{padding}",  # top-center
                2: f"main_w-overlay_w-{padding}:{padding}",  # top-right
                3: f"{padding}:(main_h-overlay_h)/2",  # center-left
                4: f"(main_w-overlay_w)/2:(main_h-overlay_h)/2",  # center
                5: f"main_w-overlay_w-{padding}:(main_h-overlay_h)/2",  # center-right
                6: f"{padding}:main_h-overlay_h-{padding}",  # bottom-left
                7: f"(main_w-overlay_w)/2:main_h-overlay_h-{padding}",  # bottom-center
                8: f"main_w-overlay_w-{padding}:main_h-overlay_h-{padding}"  # bottom-right
            }
            
            overlay_position = position_map.get(watermark_position, position_map[8])
            overlay_wm = f"{final_v}{wm_v}overlay={overlay_position}{v_wm_out}"
            filter_parts.append(overlay_wm)
            final_v = v_wm_out

        output_v_stream = final_v
        
        # Audio is next input
        audio_input_index = current_input_count

        full_graph = ";".join(filter_parts)


        full_graph = ";".join(filter_parts)
        
        # --- AUDIO INPUTS AND FILTERS ---
        inputs.append("-thread_queue_size"); inputs.append("4096")
        inputs.append("-i"); inputs.append(get_opt_path(audio_path))
        voiceover_input_index = audio_input_index # Use the tracked index


        audio_filter_chains = []
        final_audio_map = f"[{voiceover_input_index}:a]"

        if background_music_path and os.path.exists(background_music_path):
            logger.log(f"{prefix}[FFmpeg] Adding background music.", level=LogLevel.INFO)
            # Use -stream_loop on the input
            inputs.extend(["-stream_loop", "-1", "-thread_queue_size", "4096", "-i", get_opt_path(background_music_path)])
            
            bg_music_input_index = voiceover_input_index + 1

            
            vol_multiplier = (background_music_volume if background_music_volume is not None else 100) / 100.0
            
            fade_duration = 5
            # Ensure fade out doesn't start before the audio begins
            fade_start_time = max(0, audio_dur - fade_duration)

            # Filter chain for background music: set volume, trim to voiceover length, fade out
            bg_music_filter = (
                f"[{bg_music_input_index}:a]volume={vol_multiplier:.2f},"
                f"atrim=duration={audio_dur:.3f},"
                f"afade=t=out:st={fade_start_time:.3f}:d={fade_duration}[bg_audio]"
            )
            audio_filter_chains.append(bg_music_filter)
            
            # Filter chain for mixing voiceover and background music
            mix_filter = f"[{voiceover_input_index}:a][bg_audio]amix=inputs=2:duration=first:dropout_transition=2[a_out]"
            audio_filter_chains.append(mix_filter)

            final_audio_map = "[a_out]"
        else:
             # Direct input mapping (no brackets for raw stream map in some contexts, 
             # but standard notation like "0:a" works reliably)
             final_audio_map = f"{voiceover_input_index}:a"
        
        if audio_filter_chains:
            full_graph += ";" + ";".join(audio_filter_chains)
        
        # --- END AUDIO ---

        # 8. INITIAL VIDEO (PREPEND)
        initial_video_path = kwargs.get('initial_video_path')
        if initial_video_path and os.path.exists(initial_video_path):
            logger.log(f"{prefix}[FFmpeg] Prepending initial video: {os.path.basename(initial_video_path)}", level=LogLevel.INFO)
            
            inputs.extend(["-thread_queue_size", "4096", "-i", get_opt_path(initial_video_path)])
            # Correctly calculate index based on actual number of inputs added so far
            # (Audio/Music inputs might have been added without updating current_input_count)
            intro_index = inputs.count("-i") - 1
            current_input_count = intro_index + 1 # Align for safety
            
            # --- Intro Video Processing ---
            v_intro = "[v_intro]"
            # Scale and Pad to match base dimensions
            intro_scale = (
                f"[{intro_index}:v]scale={base_w}:{base_h}:force_original_aspect_ratio=decrease,"
                f"pad={base_w}:{base_h}:(ow-iw)/2:(oh-ih)/2,"
                f"format=yuv420p,setsar=1,fps={fps},"
                f"setpts=PTS-STARTPTS{v_intro}"
            )
            full_graph += ";" + intro_scale
            
            # --- Intro Audio Processing ---
            a_intro = "[a_intro]"
            has_intro_audio = self._has_audio(initial_video_path)
            intro_dur = self._get_duration(initial_video_path)
            
            if has_intro_audio:
                # Resample to match common settings (stereo, 44100) to avoid concat issues
                intro_audio_filter = f"[{intro_index}:a]aformat=sample_rates=44100:channel_layouts=stereo{a_intro}"
                full_graph += ";" + intro_audio_filter
            else:
                # Generate silence
                intro_audio_filter = f"anullsrc=channel_layout=stereo:sample_rate=44100,atrim=duration={intro_dur}{a_intro}"
                full_graph += ";" + intro_audio_filter

            # --- Concat/Transition Intro + Main ---
            v_total = "[v_total]"
            a_total = "[a_total]"
            
            # We assume main audio is already at final_audio_map (and might need formatting to match?)
            # Usually main audio (voiceover/music) is robust, but let's ensure it matches format too just in case
            a_main_fmt = "[a_main_fmt]"
            
            # Ensure final_audio_map is wrapped in brackets for filter syntax if it's a raw stream selector
            safe_audio_map = final_audio_map
            if ":" in final_audio_map and not final_audio_map.startswith("["):
                safe_audio_map = f"[{final_audio_map}]"
            
            full_graph += ";" + f"{safe_audio_map}aformat=sample_rates=44100:channel_layouts=stereo{a_main_fmt}"
            
            # --- PAUSE LOGIC (3s delay for voiceover) ---
            # User requested ~3s pause between Intro and Voiceover.
            pause_dur = 1.5
            
            # Delay Main Audio (insert silence at start)
            a_main_delayed = "[a_main_delayed]"
            # adelay expects milliseconds. all=1 applies to all channels (stereo)
            full_graph += ";" + f"{a_main_fmt}adelay={int(pause_dur*1000)}:all=1{a_main_delayed}"
            a_main_fmt = a_main_delayed

            # Delay Main Video (pad start with clone of first frame)
            # This creates a "static pause" effect before the main video starts playing
            v_main_padded = "[v_main_padded]"
            full_graph += ";" + f"{final_v}tpad=start_mode=clone:start_duration={pause_dur}{v_main_padded}"
            final_v = v_main_padded
            
            # Reset PTS for Main Video to ensure clean start for transitions/concat
            final_v_reset = "[final_v_reset]"
            full_graph += ";" + f"{final_v}setpts=PTS-STARTPTS{final_v_reset}"
            final_v = final_v_reset
            
            # Apply transition if enabled
            if enable_trans and intro_dur > trans_dur:
                # XFADE for Video
                offset = intro_dur - trans_dur
                
                # Use configured transition (or default to fade if random/invalid)
                current_trans = transition_effect
                if current_trans == "random": current_trans = random.choice(valid_transitions)
                elif current_trans not in valid_transitions: current_trans = "fade"
                
                xfade_cmd = (
                    f"{v_intro}{final_v}xfade=transition={current_trans}:"
                    f"duration={trans_dur}:offset={offset}{v_total}"
                )
                full_graph += ";" + xfade_cmd
                
                # ACROSSFADE for Audio
                # Note: acrossfade consumes the overlap, so duration math works out similar to xfade
                acrossfade_cmd = f"{a_intro}{a_main_fmt}acrossfade=d={trans_dur}:c1=tri:c2=tri{a_total}"
                full_graph += ";" + acrossfade_cmd
                
            else:
                # Fallback to hard cut (Concat)
                concat_v = f"{v_intro}{final_v}concat=n=2:v=1:a=0{v_total}"
                concat_a = f"{a_intro}{a_main_fmt}concat=n=2:v=0:a=1{a_total}"
                
                full_graph += ";" + concat_v
                full_graph += ";" + concat_a
            
            output_v_stream = v_total
            final_audio_map = a_total
        else:
             intro_dur = 0 # No intro video
             pause_dur = 0


        filter_script_path = None
        try:
            # Створюємо тимчасовий файл для filter_complex
            with tempfile.NamedTemporaryFile(mode='w+', delete=False, suffix=".txt", encoding='utf-8') as filter_file:
                filter_file.write(full_graph)
                filter_script_path = filter_file.name

            # Add buffer settings to prevent overflow
            cmd = ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-stats", "-max_interleave_delta", "0"]
            cmd.extend(inputs)
            
            # Для libx264 НЕ додаємо -c:v тут, бо додамо його пізніше разом з професійними параметрами
            if codec == "libx264":
                cmd.extend(["-filter_complex_script", filter_script_path.replace("\\", "/"), "-map", output_v_stream, "-map", final_audio_map])
            else:
                cmd.extend(["-filter_complex_script", filter_script_path.replace("\\", "/"), "-map", output_v_stream, "-map", final_audio_map, "-c:v", codec])


            # Prevent silent video tail if visual stream drifts
            cmd.append("-shortest")

            bitrate_str = f"{bitrate}M"
            
            # ==================== ПРОФЕСІЙНІ ПАРАМЕТРИ КОДЕКА (DAVINCI RESOLVE) ====================
            if codec == "libx264":
                cmd.extend([
                    '-c:v', 'libx264',
                    '-profile:v', 'main',
                    '-level', '5.0',
                    '-pix_fmt', 'yuvj420p',
                    '-color_range', 'pc',
                    '-colorspace', 'bt709',
                    '-color_trc', 'bt709',
                    '-color_primaries', 'bt709',
                    '-x264-params', 'colorprim=bt709:transfer=bt709:colormatrix=bt709:fullrange=1',
                    '-r', '30',
                    '-c:a', 'aac',
                    '-ar', '48000',
                    '-ac', '2',
                    '-preset', preset,
                    '-b:v', bitrate_str,
                    '-maxrate', bitrate_str,
                    '-bufsize', f"{bitrate*2}M"
                ])
            elif codec == "h264_amf":
                if preset in ["ultrafast", "superfast", "veryfast", "faster", "fast"]: u="speed"
                elif preset == "medium": u="balanced"
                else: u="quality"
                # FORCE -r 30 to match filter graph generation rate (prevent slow motion sync drift)
                cmd.extend(["-quality", u, "-b:v", bitrate_str, "-pix_fmt", "yuv420p", "-r", "30"])
            elif codec == "h264_nvenc":
                # FORCE -r 30 to match filter graph generation rate (prevent slow motion sync drift)
                cmd.extend(["-preset", "p4", "-b:v", bitrate_str, "-pix_fmt", "yuv420p", "-r", "30"])
            else:
                # FORCE -r 30 to match filter graph generation rate (prevent slow motion sync drift)
                cmd.extend(["-preset", preset, "-b:v", bitrate_str, "-maxrate", bitrate_str, "-bufsize", f"{bitrate*2}M", "-pix_fmt", "yuv420p", "-r", "30"])
            # ==================== КІНЕЦЬ ПАРАМЕТРІВ КОДЕКА ====================
            
            # Генеруємо timestamp для метаданих (додамо пізніше)
            current_time = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%f')[:-3] + 'Z'
            
            # Метадані будуть застосовані частково через FFmpeg (stream tags) та mutagen (format tags)
            
            # КРИТИЧНО: Глобальний creation_time (для format.tags.creation_time)
            cmd.extend(['-metadata', f'creation_time={current_time}'])
            
            # Stream-specific metadata (FFmpeg під час рендерингу)
            cmd.extend(['-metadata:s:v:0', f'creation_time={current_time}'])
            cmd.extend(['-metadata:s:v:0', 'handler_name=VideoHandler'])
            cmd.extend(['-metadata:s:v:0', 'timecode=01:00:00:00'])
            cmd.extend(['-metadata:s:v:0', 'encoder=H.264 AMD'])
            cmd.extend(['-metadata:s:v:0', 'vendor_id=[0][0][0][0]'])
            cmd.extend(['-metadata:s:v:0', 'language=und'])
            
            cmd.extend(['-metadata:s:a:0', f'creation_time={current_time}'])
            cmd.extend(['-metadata:s:a:0', 'handler_name=SoundHandler'])
            cmd.extend(['-metadata:s:a:0', 'vendor_id=[0][0][0][0]'])
            cmd.extend(['-metadata:s:a:0', 'language=und'])
            
            # КРИТИЧНО: Timecode data stream (як у DaVinci)
            # -write_tmcd створює streams.2, але треба задати метадані окремо
            cmd.extend(['-write_tmcd', '1'])
            cmd.extend(['-metadata:s:d:0', f'creation_time={current_time}'])  # d = data stream (timecode)
            cmd.extend(['-metadata:s:d:0', 'handler_name=TimeCodeHandler'])  # ВАЖЛИВО: TimeCodeHandler
            cmd.extend(['-metadata:s:d:0', 'language=eng'])  # ВАЖЛИВО: eng для timecode
            
            # Clean output path for FFmpeg (remove \\?\\ prefix which can break when slashes are flipped)
            clean_out_path = output_path.replace("\\\\?\\", "").replace("//?/", "")
            # Для libx264 -shortest вже додано в секції кодека, для інших додаємо тут
            if codec == "libx264":
                cmd.extend(["-max_muxing_queue_size", "9999", clean_out_path.replace("\\", "/")])
            else:
                cmd.extend(["-shortest", "-max_muxing_queue_size", "9999", clean_out_path.replace("\\", "/")])
            
            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            with subprocess.Popen(
                cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, 
                stdin=subprocess.DEVNULL, cwd=base_cwd, # PASS CWD HERE
                text=True, encoding='utf-8', errors='replace', startupinfo=startupinfo
            ) as process:

                full_log = []
                
                # Calculate total expected duration for progress bar
                # Total = (Intro Video) + (Main Audio + Pause) - (Overlap if transition used)
                total_expected_duration = audio_dur + intro_dur + pause_dur
                if enable_trans and intro_dur > 0:
                     total_expected_duration -= trans_dur
    
                while True:
                    line = process.stderr.readline()
                    if not line and process.poll() is not None: break
                    if line:
                        c = line.strip()
                        full_log.append(c)
                        
                        if "frame=" in c or "time=" in c:
                            parts = dict(re.findall(r'(\w+)=\s*([^ ]+)', c))
                            time_str = parts.get('time', '00:00:00.00')
                            
                            try:
                                # Конвертація часу в секунди
                                time_parts = time_str.split(':')
                                h = int(time_parts[0])
                                m = int(time_parts[1])
                                s = float(time_parts[2])
                                time_sec = h * 3600 + m * 60 + s
                            except (ValueError, IndexError):
                                time_sec = 0.0
    
                            # Calculate progress based on NEW total duration
                            denom = total_expected_duration if total_expected_duration > 0.1 else 1.0
                            progress = min(max((time_sec / denom) * 100, 0.0), 100.0)
                            
                            fps = parts.get('fps', '0')
                            bitrate = parts.get('bitrate', 'N/A')
                            
                            log_line = (
                                f"time={time_str} | "
                                f"fps={fps} | "
                                f"bit={bitrate} | "
                                f"progress={progress:.2f}%"
                            )
                            log_progress(log_line)
                            
                        elif "Error" in c and "Error submitting packet to decoder" not in c:
                            logger.log(f"{prefix}[FFmpeg] {c}", level=LogLevel.ERROR)
                            log_progress(f"[FFmpeg] Error: {c}")
    
                if process.returncode != 0:
                    err = "\n".join(full_log[-20:])
                    logger.log(f"{prefix}[FFmpeg] Rendering failed:\n{err}", level=LogLevel.ERROR)
                    raise Exception("FFmpeg failed.")
            
            # ==================== ПОСТОБРОБКА МЕТАДАНИХ (MUTAGEN) ====================
            # ==================== METADATA POST-PROCESSING (MUTAGEN) ====================
            sim_target = settings_manager.get('simulation_target', 'DaVinci Resolve Studio')
            
            if sim_target == 'DaVinci Resolve Studio':
                logger.log(f"{prefix}[Metadata] Applying metadata...", level=LogLevel.INFO)
                
                try:
                    from mutagen.mp4 import MP4, MP4FreeForm, MP4Tags
                    
                    # Open the newly created file
                    video = MP4(clean_out_path)
                    
                    # Format-level tags ONLY (like DaVinci Resolve)
                    video["\xa9enc"] = "Blackmagic Design DaVinci Resolve Studio"  # Encoder (main tag)
                    video["----:com.apple.iTunes:encoder"] = MP4FreeForm("Blackmagic Design DaVinci Resolve Studio".encode('utf-8'))
                    
                    # Brands (like DaVinci)
                    video["----:com.apple.iTunes:major_brand"] = MP4FreeForm(b"isom")
                    video["----:com.apple.iTunes:minor_version"] = MP4FreeForm(b"512")
                    video["----:com.apple.iTunes:compatible_brands"] = MP4FreeForm(b"isomiso2avc1mp41")
                    
                    # Save changes
                    video.save()
                    
                    logger.log(f"{prefix}[Metadata] ✅ Metadata applied successfully!", level=LogLevel.INFO)
                    
                except ImportError:
                    logger.log(f"{prefix}[Metadata] ⚠️ Please install mutagen: pip install mutagen", level=LogLevel.WARNING)
                except Exception as e:
                    logger.log(f"{prefix}[Metadata] ⚠️ Metadata application failed: {str(e)}", level=LogLevel.WARNING)
            else:
                logger.log(f"{prefix}[Metadata] Metadata simulation disabled or other profile selected ({sim_target}).", level=LogLevel.INFO)
            # ==================== END METADATA POST-PROCESSING ====================
            
        finally:
            if filter_script_path and os.path.exists(filter_script_path):
                os.remove(filter_script_path)
        # Success log is handled by MontageWorker

    def _has_audio(self, path):
        """Checks if the file has an audio stream."""
        path = path.replace("\\", "/")
        cmd = [
            "ffprobe", 
            "-v", "error", 
            "-select_streams", "a:0", 
            "-show_entries", "stream=codec_type", 
            "-of", "csv=p=0", 
            path
        ]
        try:
            # Use subprocess directly to avoid overhead? No, keep consistent style if possible but simple here
            import subprocess
            # startupinfo needed for non-blocking console on Windows
            startupinfo = None
            if platform.system() == 'Windows':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                
            output = subprocess.check_output(cmd, stderr=subprocess.STDOUT, startupinfo=startupinfo).decode().strip()
            return bool(output)
        except Exception:
            return False

    def _get_dimensions(self, path):
        normalized_path = path.replace("\\", "/")
        cmd = ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", 
               "-of", "csv=s=x:p=0", normalized_path]
        try:
            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            res = subprocess.run(cmd, capture_output=True, text=True, startupinfo=startupinfo)
            val = res.stdout.strip().split('x')
            if len(val) == 2:
                return int(val[0]), int(val[1])
            return 0, 0
        except Exception:
            return 0, 0

    def _get_duration(self, path):
        normalized_path = path.replace("\\", "/")
        cmd = ["ffprobe", "-v", "error", "-show_entries", "format=duration", 
               "-of", "default=noprint_wrappers=1:nokey=1", normalized_path]
        try:
            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            res = subprocess.run(cmd, capture_output=True, text=True, startupinfo=startupinfo)
            val = res.stdout.strip()
            return float(val) if val else 0
        except Exception:
            return 0
        except:
            return 0
            
    def _get_text_timing(self, ass_path, phrase, prefix=""):
        if not ass_path or not os.path.exists(ass_path) or not phrase:
            logger.log(f"{prefix}[FFmpeg] Cannot search for phrase: ASS path missing or empty phrase.", level=LogLevel.WARNING)
            return None
        
        # Helper for cleaning text
        def clean(s):
            s = s.lower()
            s = s.replace('ё', 'е')
            s = re.sub(r'[^\w\s]', '', s)
            return re.sub(r'\s+', ' ', s).strip()

        cleaned_phrase = clean(phrase)
        if not cleaned_phrase:
            return None

        # Just verify length to not search for "a"
        if len(cleaned_phrase) < 3:
            logger.log(f"{prefix}[FFmpeg] Phrase too short to search.", level=LogLevel.WARNING)
            return None

        logger.log(f"{prefix}[FFmpeg] Searching for full phrase: '{cleaned_phrase}'", level=LogLevel.DEBUG)

        try:
            with open(ass_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # 1. Parse all subtitle events strictly
            # We store: start_time (seconds), clean_text, original_line_index
            parsed_segments = []
            
            full_text_buffer = ""
            # Map character index in full_text_buffer to the segment it belongs to
            # We'll use this to trace back the start time
            char_to_segment_map = [] 

            for line in lines:
                if line.startswith('Dialogue:'):
                    parts = line.split(',', 9)
                    if len(parts) >= 10:
                        start_str = parts[1].strip()
                        raw_text = parts[9]
                        sub_text = re.sub(r'\{.*?\}', '', raw_text) # remove tags
                        cleaned_sub = clean(sub_text)
                        
                        if cleaned_sub:
                            start_time = self._parse_ass_time(start_str)
                            
                            current_buffer_len = len(full_text_buffer)
                            
                            # Add space if not start
                            prefix_space = " " if full_text_buffer else ""
                            full_text_buffer += prefix_space + cleaned_sub
                            
                            # Map the new characters to this segment
                            # Because we added a space, the first char maps to "prev" segment conceptually? 
                            # Or we can just map all new chars to this segment index
                            # Let's map strictly.
                            
                            # If we added a space, that space technically bridges previous and current.
                            # Let's assign it to current for simplicity of "start of phrase".
                            
                            segment_idx = len(parsed_segments)
                            added_len = len(prefix_space) + len(cleaned_sub)
                            
                            char_to_segment_map.extend([segment_idx] * added_len)
                            
                            parsed_segments.append({
                                'start': start_time,
                                'text': cleaned_sub,
                                'buffer_start_idx': current_buffer_len
                            })

            if not full_text_buffer:
                return None

            # 2. Search in the full content
            # Try exact search first
            start_index = full_text_buffer.find(cleaned_phrase)
            
            if start_index == -1:
                # 3. Fuzzy search if exact failed
                # Use SequenceMatcher on the WHOLE buffer vs phrase
                # Note: This might be slow if buffer is huge (hour long video), but for standard clips it's fine.
                matcher = difflib.SequenceMatcher(None, full_text_buffer, cleaned_phrase)
                match = matcher.find_longest_match(0, len(full_text_buffer), 0, len(cleaned_phrase))
                
                # We want the match to cover significantly the phrase
                if match.size > len(cleaned_phrase) * 0.6: # 60% coverage
                     logger.log(f"{prefix}[FFmpeg] Fuzzy match found! Ratio: {match.size/len(cleaned_phrase):.2f}", level=LogLevel.INFO)
                     start_index = match.a
                else: 
                    # Try finding "anchor" (just the start)
                    # If the phrase is super long, maybe we only matched the beginning?
                    pass

            if start_index != -1:
                # We found the start char index. Map it back to segment.
                # start_index matches the index in full_text_buffer
                if start_index < len(char_to_segment_map):
                    seg_idx = char_to_segment_map[start_index]
                    found_segment = parsed_segments[seg_idx]
                    
                    found_time = found_segment['start']
                    
                    # Calculate reasonable end time
                    # We can try to finding where the phrase ends in the buffer
                    end_char_index = start_index + len(cleaned_phrase)
                    if end_char_index < len(char_to_segment_map):
                         end_seg_idx = char_to_segment_map[end_char_index]
                         if end_seg_idx < len(parsed_segments):
                             # Use the start time of the NEXT segment as the end of this current phrase block?
                             # Or just add some duration. 
                             # Let's try to find the start time of the segment corresponding to the END of the phrase.
                             # But segments only have start times.
                             
                             # Let's estimate:
                             # If phrase spans multiple segments, end time is start of segment AFTER the last one spanned.
                             
                             if end_seg_idx + 1 < len(parsed_segments):
                                 end_time = parsed_segments[end_seg_idx + 1]['start']
                             else:
                                 # Last segment. Add 5s?
                                 end_time = found_time + 5.0
                         else:
                             end_time = found_time + 5.0
                    else:
                         end_time = found_time + 5.0

                    logger.log(f"{prefix}[FFmpeg] Trigger found at char {start_index} -> Segment {seg_idx} -> Time {found_time}s - {end_time}s", level=LogLevel.INFO)
                    return (found_time, end_time)
            
            logger.log(f"{prefix}[FFmpeg] Phrase not found in subtitles.", level=LogLevel.WARNING)
            return None

        except Exception as e:
            logger.log(f"Error searching text timing: {e}", level=LogLevel.ERROR)
            return None       
    def _parse_ass_time(self, time_str):
        # Format usually H:MM:SS.cc
        try:
            parts = time_str.split(':')
            if len(parts) == 3:
                h, m, s = parts
                return int(h) * 3600 + int(m) * 60 + float(s)
            return 0.0
        except:
            return 0.0

    def _parse_time(self, time_str):
        if not time_str: return 0.0
        try:
            # Handle float seconds
            if ':' not in str(time_str):
                return float(time_str)
            
            parts = str(time_str).split(':')
            if len(parts) == 2: # MM:SS
                return int(parts[0]) * 60 + float(parts[1])
            elif len(parts) == 3: # HH:MM:SS
                return int(parts[0]) * 3600 + int(parts[1]) * 60 + float(parts[2])
            return 0.0
        except:
            return 0.0