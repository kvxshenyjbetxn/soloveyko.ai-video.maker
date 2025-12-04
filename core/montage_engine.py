import subprocess
import os
import math
import sys
from utils.logger import logger, LogLevel

class MontageEngine:
    def create_video(self, visual_files, audio_path, output_path, ass_path, settings, task_id=None, progress_callback=None, start_time=None):
        prefix = f"[{task_id}] " if task_id else ""
        
        def log_progress(msg):
            """Log to card only (not to main log)"""
            if progress_callback:
                progress_callback(msg)
        
        # 1. ОТРИМУЄМО ДАНІ
        audio_dur = self._get_duration(audio_path)
        if audio_dur == 0: raise Exception("Аудіо пусте або не читається.")
        if not visual_files: raise Exception("Немає файлів.")

        num_files = len(visual_files)
        
        # Налаштування переходів
        enable_trans = settings.get('enable_transitions', True)
        trans_dur = settings.get('transition_duration', 0.5) if enable_trans else 0
        
        # Налаштування рендеру
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

        # 2. МАТЕМАТИКА ЧАСУ (Аудіо - головне)
        VIDEO_EXTS = ['.mp4', '.mkv', '.mov', '.avi', '.webm']
        total_video_time = 0.0
        num_images = 0
        final_clip_durations = [0] * num_files
        
        for i, f in enumerate(visual_files):
            ext = os.path.splitext(f)[1].lower()
            if ext in VIDEO_EXTS:
                d = self._get_duration(f)
                if d == 0: d = 5.0
                final_clip_durations[i] = d
                total_video_time += d
            else:
                num_images += 1
        
        total_trans_loss = (num_files - 1) * trans_dur
        required_raw_time = audio_dur + total_trans_loss
        time_for_images = required_raw_time - total_video_time
        
        img_duration = 0
        if num_images > 0:
            if time_for_images <= 0:
                logger.log("⚠️ УВАГА: Відео довші за аудіо. Картинки будуть миттєві.", level=LogLevel.WARNING)
                img_duration = 0.1 
            else:
                img_duration = time_for_images / num_images
        
        for i, f in enumerate(visual_files):
            ext = os.path.splitext(f)[1].lower()
            if ext not in VIDEO_EXTS:
                final_clip_durations[i] = img_duration

        # Log compact montage info (single line)
        logger.log(f"{prefix}[FFmpeg] Starting montage | Audio: {audio_dur:.2f}s, Images: {num_images} ({img_duration:.2f}s each)", level=LogLevel.INFO)
        
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
            
            inputs.append("-i"); inputs.append(f.replace("\\", "/"))
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
                xfade = (
                    f"{curr}{next_stream}xfade=transition=fade:"
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

        full_graph = ";".join(filter_parts)
        inputs.append("-i"); inputs.append(audio_path.replace("\\", "/"))
        
        cmd = ["ffmpeg", "-y"]
        cmd.extend(inputs)
        cmd.extend(["-filter_complex", full_graph, "-map", output_v_stream, "-map", f"{num_files}:a", "-c:v", codec])
        
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
        
        cmd.extend(["-shortest", output_path.replace("\\", "/")])

        # # --- LOG COMMAND FOR DEBUGGING ---
        # # Create a printable version of the command
        # cmd_str = " ".join(f'"{c}"' if " " in c or ":" in c else c for c in cmd)
        # logger.log(f"{prefix}[FFmpeg] Executing command: {cmd_str}", level=LogLevel.INFO)
        # # --- END LOGGING ---

        startupinfo = subprocess.STARTUPINFO()
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        
        process = subprocess.Popen(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
            text=True, encoding='utf-8', errors='replace', startupinfo=startupinfo
        )

        full_log = []
        while True:
            line = process.stderr.readline()
            if not line and process.poll() is not None: break
            if line:
                c = line.strip()
                full_log.append(c)
                
                # Send FFmpeg progress to card only (not to main log)
                if "frame=" in c or "time=" in c:
                    log_progress(c)
                # Log errors to both main log and card (but ignore mp3 decoder errors - they're non-fatal)
                elif "Error" in c and "Error submitting packet to decoder" not in c:
                    logger.log(f"{prefix}[FFmpeg] {c}", level=LogLevel.ERROR)
                    log_progress(f"[FFmpeg] Error: {c}")

        if process.returncode != 0:
            err = "\n".join(full_log[-20:])
            logger.log(f"{prefix}[FFmpeg] Rendering failed:\n{err}", level=LogLevel.ERROR)
            raise Exception("FFmpeg failed.")
        # Success log is handled by MontageWorker

    def _get_duration(self, path):
        normalized_path = path.replace("\\", "/")
        cmd = ["ffprobe", "-v", "error", "-show_entries", "format=duration", 
               "-of", "default=noprint_wrappers=1:nokey=1", normalized_path]
        try:
            si = subprocess.STARTUPINFO()
            si.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            res = subprocess.run(cmd, capture_output=True, text=True, startupinfo=si)
            val = res.stdout.strip()
            return float(val) if val else 0
        except:
            return 0