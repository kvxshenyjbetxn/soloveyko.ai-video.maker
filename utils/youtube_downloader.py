import os
import subprocess
import re
import sys
from utils.settings import settings_manager

class YouTubeDownloader:
    @staticmethod
    def download_audio(url, output_dir, yt_dlp_path=None, ffmpeg_path=None, progress_callback=None):
        """
        Downloads audio using yt-dlp subprocess.
        Ensures Deno is in PATH for compliance with new YouTube changes.
        """
        
        # Determine paths
        base_path = settings_manager.base_path
        assets_path = os.path.join(base_path, "assets")
        
        # If yt_dlp_path is not provided, look in assets path
        if not yt_dlp_path or not os.path.exists(yt_dlp_path):
             exe_name = "yt-dlp.exe" if os.name == 'nt' else "yt-dlp"
             yt_dlp_path = os.path.join(assets_path, exe_name)
             
        # Output template
        output_template = os.path.join(output_dir, "downloaded_audio.%(ext)s")
        
        # Prepare environment
        # CRITICAL: Add assets_path to PATH so yt-dlp can find deno
        env = os.environ.copy()
        env["PATH"] = assets_path + os.pathsep + env["PATH"]
        
        cmd = [
            yt_dlp_path,
            "-x", # Extract audio
            "--audio-format", "mp3",
            "--audio-quality", "0",
            "-o", output_template,
            "--no-playlist",
            "--progress", 
            "--newline",
            # New Anti-Bot options
            "--extractor-args", "youtube:player_client=android", # Often helps
            url
        ]
        
        if ffmpeg_path:
             cmd.extend(["--ffmpeg-location", ffmpeg_path])

        # On Windows, prevent console window popping up
        startupinfo = None
        if os.name == 'nt':
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        
        process = subprocess.Popen(
            cmd, 
            startupinfo=startupinfo, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE, 
            text=True,
            bufsize=1,
            universal_newlines=True,
            env=env # Pass modified environment with Deno path
        )
        
        progress_pattern = re.compile(r'\[download\]\s+(\d+\.\d+)%')
        
        for line in process.stdout:
            if progress_callback:
                match = progress_pattern.search(line)
                if match:
                    percent = match.group(1)
                    progress_callback(f"{percent}%")
        
        stdout, stderr = process.communicate()
        
        if process.returncode != 0:
            raise Exception(f"yt-dlp failed: {stderr}")
        
        for file in os.listdir(output_dir):
            if file.startswith("downloaded_audio."):
                return os.path.join(output_dir, file)
        
        raise Exception("Download finished but file not found.")

    @staticmethod
    def get_video_title(url, yt_dlp_path=None):
        """
        Fetches the title of the video using yt-dlp.
        """
        # Determine paths
        base_path = settings_manager.base_path
        assets_path = os.path.join(base_path, "assets")
        
        if not yt_dlp_path or not os.path.exists(yt_dlp_path):
             exe_name = "yt-dlp.exe" if os.name == 'nt' else "yt-dlp"
             yt_dlp_path = os.path.join(assets_path, exe_name)
        
        env = os.environ.copy()
        env["PATH"] = assets_path + os.pathsep + env["PATH"]
        
        cmd = [
            yt_dlp_path,
            "--get-title",
            "--no-playlist",
            "--skip-download",
            # New Anti-Bot options
            "--extractor-args", "youtube:player_client=android",
            url
        ]
        
        startupinfo = None
        if os.name == 'nt':
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
        try:
            result = subprocess.run(
                cmd,
                startupinfo=startupinfo,
                capture_output=True,
                text=True,
                env=env,
                check=True
            )
            title = result.stdout.strip()
            if title:
                # Sanitize title immediately to prevent issues downstream
                # Allow unicode letters (\w), numbers, spaces and hyphens
                # This explicitly removes quotes, colons, dots, emojis etc.
                clean_title = re.sub(r'[^\w\s\-]', '', title)
                # Collapse multiple spaces and strip trailing spaces/dots (Windows safety)
                clean_title = re.sub(r'\s+', ' ', clean_title).strip('. ')
                if clean_title:
                    return clean_title
                # If sanitization removed everything (e.g. only emojis), return original or fallback?
                # Best to fallback to None so it gets "Task N" name, rather than " " or empty.
                return None
        except Exception as e:
            pass
            
        return None

