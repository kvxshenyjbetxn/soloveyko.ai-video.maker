import subprocess
import os
import math
import sys
import tempfile
import re
import platform
import random
from utils.logger import logger, LogLevel

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
                
            if os.path.exists(abs_path):
                valid_files.append(abs_path)
            else:
                logger.log(f"{prefix}[Warning] File not found, skipping: {f}", level=LogLevel.WARNING)
        
        visual_files = valid_files
        
        audio_dur = self._get_duration(audio_path)
        if audio_dur == 0: raise Exception("Аудіо пусте або не читається.")
        if not visual_files: raise Exception("Немає файлів (або жоден файл не знайдено).")

        num_files = len(visual_files)
        
        # Налаштування переходів
        enable_trans = settings.get('enable_transitions', True)
        trans_dur = settings.get('transition_duration', 0.5) if enable_trans else 0
        enable_trans = settings.get('enable_transitions', True)
        trans_dur = settings.get('transition_duration', 0.5) if enable_trans else 0
        transition_effect = settings.get('transition_effect', 'random')

        valid_transitions = [
            "fade", "wipeleft", "wiperight", "wipeup", "wipedown", 
            "slideleft", "slideright", "slideup", "slidedown", "circlecrop", 
            "rectcrop", "distance", "fadeblack", "fadewhite", "radial", 
            "smoothleft", "smoothright", "smoothup", "smoothdown", 
            "circleopen", "circleclose", "vertopen", "vertclose", 
            "horzopen", "horzclose", "dissolve", "pixelize", "diagtl", 
            "diagtr", "diagbl", "diagbr", "hlslice", "hrslice", "vu"
        ]
        codec = settings.get('codec', 'libx264')
        preset = settings.get('preset', 'medium')
        bitrate = settings.get('bitrate_mbps', 15)
        
        # --- НАЛАШТУВАННЯ ЕФЕКТІВ (НОВІ) ---
        enable_zoom = settings.get('enable_zoom', True)
        z_spd = settings.get('zoom_speed_factor', 1.0)
        z_int = settings.get('zoom_intensity', 0.15)

        enable_sway = settings.get('enable_sway', False)
        s_spd = settings.get('sway_speed_factor', 1.0)

        up_factor = settings.get('upscale_factor', 3.0)
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

        for i in range(start_index, num_images):
            img_idx = image_indices[i]
            final_clip_durations[img_idx] = img_duration
        
        # Log compact montage info (single line)
        log_msg = f"{prefix}[FFmpeg] Starting montage | Audio: {audio_dur:.2f}s, Images: {num_images}"
        if special_mode == "Quick show":
            log_msg += f" (Special: {min(num_images, special_count)}x{special_dur:.2f}s)"
        log_msg += f", Other Images: {num_normal_images}x{img_duration:.2f}s"
        logger.log(log_msg, level=LogLevel.INFO)
        
        # 3. ГЕНЕРАЦІЯ FFmpeg КОМАНДИ
        inputs = []
        filter_parts = []
        fps = 30
        
        def fmt(val): return f"{val:.6f}".replace(",", ".")
        
        for i, f in enumerate(visual_files):
            ext = os.path.splitext(f)[1].lower()
            is_video = ext in VIDEO_EXTS
            this_dur = final_clip_durations[i]
            this_dur_str = fmt(this_dur)
            
            abs_path = os.path.abspath(f).replace("\\", "/")
            if not os.path.exists(abs_path):
                 logger.log(f"{prefix}[Error] Input file not found: {abs_path}", level=LogLevel.ERROR)
                 raise Exception(f"Input file missing: {abs_path}")

            inputs.append("-thread_queue_size"); inputs.append("4096")
            inputs.append("-i"); inputs.append(abs_path)
            v_in = f"[{i}:v]"; v_out = f"v{i}_final"
            
            if is_video:
                vf = (
                    f"{v_in}scale=1920:1080:force_original_aspect_ratio=decrease,"
                    f"pad=1920:1080:(ow-iw)/2:(oh-ih)/2,"
                    f"format=yuv420p,setsar=1,fps={fps},"
                    f"setpts=PTS-STARTPTS[{v_out}]"
                )
                filter_parts.append(vf)
            else:
                v_up = f"v{i}_up"
                scale_cmd = (
                    f"{v_in}scale={up_w}:{up_h}:force_original_aspect_ratio=increase,"
                    f"crop={up_w}:{up_h},"
                    f"format=yuv420p,setsar=1[{v_up}]"
                )
                filter_parts.append(scale_cmd)

                # --- МАТЕМАТИКА ЕФЕКТІВ ---
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
                    f"d={d_frames}:s=1920x1080:fps={fps},"
                    f"setpts=PTS-STARTPTS[{v_out}]"
                )
                filter_parts.append(zoom_cmd)

        # 4. TRANSITIONS
        if enable_trans and num_files > 1:
            curr = "[v0_final]"
            current_offset = final_clip_durations[0] - trans_dur
            trans_dur_str_filt = fmt(trans_dur)
            for i in range(1, num_files):
                next_stream = f"[v{i}_final]"; target = f"[v_m{i}]"
                off_str = fmt(current_offset)
                
                current_trans = transition_effect
                if current_trans == "random" or current_trans not in valid_transitions:
                     current_trans = random.choice(valid_transitions)

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
            ass_clean = ass_path.replace("\\", "/").replace(":", "\\:")
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
            inputs.extend(["-stream_loop", "-1", "-thread_queue_size", "4096", "-i", overlay_effect_path.replace("\\", "/")])
            effect_index = current_input_count
            current_input_count += 1
            
            
            eff_v = f"[v_eff_scaled]"
            # Force yuva420p to ensure alpha channel is preserved/respected if present
            # Застосовуємо effect_speed для зміни швидкості відтворення ефекту
            scale_eff = f"[{effect_index}:v]format=yuva420p,scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080{eff_v}"
            filter_parts.append(scale_eff)
            
            v_overlaid = f"[v_overlaid]"
            # Use 'overlay' filter. shortest=1 ensures it stops when the main video stops (though we use -shortest on output too)
            overlay_cmd = f"{final_v}{eff_v}overlay=0:0:shortest=1{v_overlaid}"
            filter_parts.append(overlay_cmd)
            final_v = v_overlaid

        # 7. WATERMARK
        if watermark_path and os.path.exists(watermark_path):
            logger.log(f"{prefix}[FFmpeg] Adding watermark: {os.path.basename(watermark_path)}", level=LogLevel.INFO)
            inputs.extend(["-thread_queue_size", "4096", "-i", watermark_path.replace("\\", "/")])
            wm_index = current_input_count
            current_input_count += 1
            
            
            wm_v = f"[v_wm]"
            # Обчислюємо розмір вотермарки: watermark_size відсотків від 1920px
            wm_width = int(1920 * (float(watermark_size) / 100.0))
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
        
        # --- AUDIO INPUTS AND FILTERS ---
        inputs.append("-thread_queue_size"); inputs.append("4096")
        inputs.append("-i"); inputs.append(audio_path.replace("\\", "/"))
        voiceover_input_index = audio_input_index # Use the tracked index


        audio_filter_chains = []
        final_audio_map = f"[{voiceover_input_index}:a]"

        if background_music_path and os.path.exists(background_music_path):
            logger.log(f"{prefix}[FFmpeg] Adding background music.", level=LogLevel.INFO)
            # Use -stream_loop on the input
            inputs.extend(["-stream_loop", "-1", "-thread_queue_size", "4096", "-i", background_music_path.replace("\\", "/")])
            
            bg_music_input_index = voiceover_input_index + 1

            
            vol_multiplier = (background_music_volume or 100) / 100.0
            
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

        filter_script_path = None
        try:
            # Створюємо тимчасовий файл для filter_complex
            with tempfile.NamedTemporaryFile(mode='w+', delete=False, suffix=".txt", encoding='utf-8') as filter_file:
                filter_file.write(full_graph)
                filter_script_path = filter_file.name

            cmd = ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-stats"]
            cmd.extend(inputs)
            cmd.extend(["-filter_complex_script", filter_script_path.replace("\\", "/"), "-map", output_v_stream, "-map", final_audio_map, "-c:v", codec])

            bitrate_str = f"{bitrate}M"
            if codec == "h264_amf":
                if preset in ["ultrafast", "superfast", "veryfast", "faster", "fast"]: u="speed"
                elif preset == "medium": u="balanced"
                else: u="quality"
                cmd.extend(["-quality", u, "-b:v", bitrate_str, "-pix_fmt", "yuv420p"])
            elif codec == "h264_nvenc":
                cmd.extend(["-preset", "p4", "-b:v", bitrate_str, "-pix_fmt", "yuv420p"])
            else:
                cmd.extend(["-preset", preset, "-b:v", bitrate_str, "-maxrate", bitrate_str, "-bufsize", f"{bitrate*2}M", "-pix_fmt", "yuv420p"])
            
            cmd.extend(["-shortest", "-max_muxing_queue_size", "9999", output_path.replace("\\", "/")])

            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            process = subprocess.Popen(
                cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, 
                stdin=subprocess.DEVNULL,
                text=True, encoding='utf-8', errors='replace', startupinfo=startupinfo
            )

            full_log = []
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

                        progress = (time_sec / audio_dur) * 100 if audio_dur > 0 else 0
                        
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
        finally:
            if filter_script_path and os.path.exists(filter_script_path):
                os.remove(filter_script_path)
        # Success log is handled by MontageWorker

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