import os
import sys
import subprocess
import requests
import zipfile
import shutil
import platform
import stat
from PySide6.QtCore import QThread
from utils.logger import logger, LogLevel
from utils.settings import settings_manager

class YtDlpUpdater(QThread):
    """
    Background worker to check for and download/update yt-dlp and Deno binaries.
    Ensures they are present in the settings.base_path (UserData folder).
    """

    def __init__(self, parent=None):
        super().__init__(parent)
        self.base_path = settings_manager.base_path
        # Use 'assets' subdirectory for binaries
        self.assets_path = os.path.join(self.base_path, "assets")
        self._ensure_assets_path()
        
        # Define binary names based on OS
        self.is_windows = platform.system() == "Windows"
        self.yt_dlp_name = "yt-dlp.exe" if self.is_windows else "yt-dlp"
        self.deno_name = "deno.exe" if self.is_windows else "deno"
        
        self.yt_dlp_path = os.path.join(self.assets_path, self.yt_dlp_name)
        self.deno_path = os.path.join(self.assets_path, self.deno_name)

    def _ensure_assets_path(self):
        if not os.path.exists(self.assets_path):
            try:
                os.makedirs(self.assets_path)
            except Exception as e:
                logger.log(f"Error creating assets path {self.assets_path}: {e}", level=LogLevel.ERROR)

    def run(self):
        """
        Main logic: check Deno, then check yt-dlp.
        """
        try:
            self._check_and_update_deno()
            self._check_and_update_ytdlp()
            
            # Ensure PATH is updated for the current process so subsequent calls find them
            if self.assets_path not in os.environ["PATH"]:
                os.environ["PATH"] = self.assets_path + os.pathsep + os.environ["PATH"]

                
        except Exception as e:
            logger.log(f"Dependency check failed: {e}", level=LogLevel.ERROR)

    def _check_and_update_deno(self):
        if os.path.exists(self.deno_path):
            # Optimistic check: if exists, assume it's okay for now to save startup time.
            # Real Deno updates are rare compared to yt-dlp.
            # Verify it runs
            try:
                kwargs = {}
                if self.is_windows:
                     kwargs['creationflags'] = subprocess.CREATE_NO_WINDOW
                
                subprocess.run([self.deno_path, "--version"], capture_output=True, check=True, **kwargs)
                return
            except Exception:
                logger.log("Deno binary exists but seems broken. Re-downloading...", level=LogLevel.WARNING)

        logger.log("Deno is missing or broken. Downloading...", level=LogLevel.INFO)
        self._download_deno()

    def _download_deno(self):
        # determine URL
        if self.is_windows:
            url = "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip"
        elif platform.system() == "Darwin":
            # Taking a safe bet on x86_64 for compatibility or aarch64 if we detect Apple Silicon?
            # Universal binary isn't a simple zip usually. Let's check machine.
            machine = platform.machine()
            if machine == 'arm64':
                url = "https://github.com/denoland/deno/releases/latest/download/deno-aarch64-apple-darwin.zip"
            else:
                url = "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-apple-darwin.zip"
        else:
            logger.log("Unsupported OS for auto-Deno download.", level=LogLevel.ERROR)
            return

        zip_path = os.path.join(self.assets_path, "deno.zip")
        try:
            self._download_file(url, zip_path)
            
            # Extract
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                zip_ref.extractall(self.assets_path)
            
            # Cleanup zip
            os.remove(zip_path)
            
            # Chmod on Unix
            if not self.is_windows:
                self._make_executable(self.deno_path)
                
            logger.log("Deno downloaded and installed successfully.", level=LogLevel.SUCCESS)
        except Exception as e:
            logger.log(f"Failed to download Deno: {e}", level=LogLevel.ERROR)

    def _check_and_update_ytdlp(self):
        # Always try to update yt-dlp, or at least check if it exists
        if not os.path.exists(self.yt_dlp_path):
            logger.log("yt-dlp is missing. Downloading...", level=LogLevel.INFO)
            self._download_ytdlp()
        else:
            # Check for updates by running it with -U
            # Note: This updates the binary IN PLACE.
            logger.log("Checking for yt-dlp updates...", level=LogLevel.INFO)
            try:
                cmd = [self.yt_dlp_path, "-U"]
                kwargs = {}
                if self.is_windows:
                     kwargs['creationflags'] = subprocess.CREATE_NO_WINDOW
                
                subprocess.run(cmd, capture_output=True, text=True, **kwargs)
                # We trust it updated itself if needed
                logger.log("yt-dlp update check completed.", level=LogLevel.SUCCESS)
            except Exception as e:
                logger.log(f"Failed to auto-update yt-dlp: {e}", level=LogLevel.WARNING)
                # If update fails (e.g. permissions), maybe try re-downloading?
                # For now, let's stick to in-place update.

    def _download_ytdlp(self):
        base_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/"
        if self.is_windows:
            url = base_url + "yt-dlp.exe"
        elif platform.system() == "Darwin":
            url = base_url + "yt-dlp_macos"
        else:
             url = base_url + "yt-dlp"

        try:
            self._download_file(url, self.yt_dlp_path)
            if not self.is_windows:
                self._make_executable(self.yt_dlp_path)
            logger.log("yt-dlp downloaded successfully.", level=LogLevel.SUCCESS)
        except Exception as e:
            logger.log(f"Failed to download yt-dlp: {e}", level=LogLevel.ERROR)

    def _download_file(self, url, dest_path):
        with requests.get(url, stream=True) as r:
            r.raise_for_status()
            with open(dest_path, 'wb') as f:
                for chunk in r.iter_content(chunk_size=8192):
                    f.write(chunk)

    def _make_executable(self, path):
        try:
            st = os.stat(path)
            os.chmod(path, st.st_mode | stat.S_IEXEC)
            # Remove quarantine attribute on macOS
            if platform.system() == "Darwin":
                 subprocess.run(["xattr", "-d", "com.apple.quarantine", path], stderr=subprocess.DEVNULL)
        except Exception:
            pass


