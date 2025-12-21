import os
import subprocess
import re

class YouTubeDownloader:
    @staticmethod
    def download_audio(url, output_dir, yt_dlp_path="yt-dlp", progress_callback=None):
        """
        Downloads audio from a YouTube URL to the specified directory using yt-dlp.
        
        Args:
            url (str): The YouTube URL.
            output_dir (str): The directory to save the audio.
            yt_dlp_path (str): Path to the yt-dlp executable.
            progress_callback (callable, optional): function(str) to report progress (e.g. "45.0%").
            
        Returns:
            str: The full path to the downloaded audio file.
            
        Raises:
            Exception: If yt-dlp fails or the file is not found.
        """
        
        # Output template: always 'downloaded_audio' to avoid path issues
        output_template = os.path.join(output_dir, "downloaded_audio.%(ext)s")
        
        cmd = [
            yt_dlp_path,
            "-x", # Extract audio
            "--audio-format", "mp3",
            "--audio-quality", "0", # Best quality
            "-o", output_template,
            "--no-playlist",
            # Enable progress output
            "--progress", 
            "--newline", # Ensure line-by-line output
            url
        ]
        
        # On Windows, prevent console window popping up
        startupinfo = None
        if os.name == 'nt':
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        
        # Execute yt-dlp with Popen to capture stdout in real-time
        process = subprocess.Popen(
            cmd, 
            startupinfo=startupinfo, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE, # Capture stderr for error reporting
            text=True,
            bufsize=1,            # Line buffered
            universal_newlines=True
        )
        
        # Regex to extract percentage
        progress_pattern = re.compile(r'\[download\]\s+(\d+\.\d+)%')
        
        # Read stdout line by line
        for line in process.stdout:
            if progress_callback:
                match = progress_pattern.search(line)
                if match:
                    percent = match.group(1)
                    progress_callback(f"{percent}%")
        
        # Wait for process to finish
        stdout, stderr = process.communicate()
        
        if process.returncode != 0:
            raise Exception(f"yt-dlp failed: {stderr}")
        
        # Find the file
        for file in os.listdir(output_dir):
            if file.startswith("downloaded_audio."):
                return os.path.join(output_dir, file)
        
        raise Exception("Download finished but file not found.")
