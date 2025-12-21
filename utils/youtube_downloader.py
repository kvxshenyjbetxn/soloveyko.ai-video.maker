import os
import subprocess
import logging

class YouTubeDownloader:
    @staticmethod
    def download_audio(url, output_dir, yt_dlp_path="yt-dlp"):
        """
        Downloads audio from a YouTube URL to the specified directory using yt-dlp.
        
        Args:
            url (str): The YouTube URL.
            output_dir (str): The directory to save the audio.
            yt_dlp_path (str): Path to the yt-dlp executable.
            
        Returns:
            str: The full path to the downloaded audio file.
            
        Raises:
            Exception: If yt-dlp fails or the file is not found.
        """
        
        # Output template: always 'downloaded_audio' to avoid path issues
        # We let yt-dlp determine extension
        output_template = os.path.join(output_dir, "downloaded_audio.%(ext)s")
        
        cmd = [
            yt_dlp_path,
            "-x", # Extract audio
            "--audio-format", "mp3",
            "--audio-quality", "0", # Best quality
            "-o", output_template,
            "--no-playlist",
            url
        ]
        
        # On Windows, prevent console window popping up
        startupinfo = None
        if os.name == 'nt':
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        
        # Execute yt-dlp
        process = subprocess.run(cmd, startupinfo=startupinfo, capture_output=True, text=True)
        
        if process.returncode != 0:
            raise Exception(f"yt-dlp failed: {process.stderr}")
        
        # Find the file
        for file in os.listdir(output_dir):
            if file.startswith("downloaded_audio."):
                return os.path.join(output_dir, file)
        
        raise Exception("Download finished but file not found.")
