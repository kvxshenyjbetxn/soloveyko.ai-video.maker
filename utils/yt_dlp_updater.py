import os
import sys
import subprocess
import platform
from PySide6.QtCore import QThread
from utils.logger import logger, LogLevel

class YtDlpUpdater(QThread):
    """
    Background worker to check for and apply updates to yt-dlp.
    Works on Windows and macOS.
    """
    def __init__(self, parent=None):
        super().__init__(parent)
        self.yt_dlp_path = self._find_yt_dlp()

    def _find_yt_dlp(self):
        """
        Locates the yt-dlp executable in the assets directory or system PATH.
        """
        if getattr(sys, 'frozen', False):
            # If running as compiled bundle
            base_dir = sys._MEIPASS
        else:
            # If running as script
            base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

        assets_dir = os.path.join(base_dir, "assets")
        
        # Determine executable name based on OS
        if platform.system() == "Windows":
            exe_name = "yt-dlp.exe"
        else:
            exe_name = "yt-dlp"

        # 1. Check in Common Data path (where user will manually put it)
        from utils.settings import settings_manager
        path_in_base = os.path.join(settings_manager.base_path, exe_name)
        if os.path.exists(path_in_base):
            return path_in_base

        # 2. Check in assets (internal bundle)
        path_in_assets = os.path.join(assets_dir, exe_name)
        if os.path.exists(path_in_assets):
            return path_in_assets

        # 3. Fallback to system path
        return exe_name

    def run(self):
        """
        Main update logic executed in background thread.
        """
        logger.log(f"Checking for yt-dlp updates (Path: {self.yt_dlp_path})...", level=LogLevel.INFO)
        
        try:
            # Command to update yt-dlp
            # --update-to stable ensures we stay on stable versions
            # --update-to is preferred over -U in newer versions, but -U is more compatible
            cmd = [self.yt_dlp_path, "-U"]
            
            # Check if file exists and is executable
            if not os.path.exists(self.yt_dlp_path):
                # If path is just the name, it might be in PATH
                if "/" not in self.yt_dlp_path and "\\" not in self.yt_dlp_path:
                    pass # Trust result from shutil.which or similar (handled by system)
                else:
                    logger.log(f"yt-dlp not found at {self.yt_dlp_path}. Skipping update.", level=LogLevel.WARNING)
                    return

            # On macOS, ensure execution permissions if we have a direct path
            if platform.system() == "Darwin" and os.path.exists(self.yt_dlp_path):
                try:
                    import stat
                    st = os.stat(self.yt_dlp_path)
                    os.chmod(self.yt_dlp_path, st.st_mode | stat.S_IEXEC)
                except:
                    pass
            
            # On Windows, prevent console window popping up
            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            process = subprocess.Popen(
                cmd,
                startupinfo=startupinfo,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                universal_newlines=True
            )
            
            stdout, stderr = process.communicate()
            
            if process.returncode == 0:
                if "yt-dlp is up to date" in stdout or "yt-dlp is up-to-date" in stdout:
                    logger.log("yt-dlp is already up to date.", level=LogLevel.INFO)
                else:
                    logger.log(f"yt-dlp update successful: {stdout.strip()}", level=LogLevel.SUCCESS)
            else:
                # Some versions might return non-zero if no update is found or for other reasons
                if "up to date" in stdout.lower() or "up-to-date" in stdout.lower():
                    logger.log("yt-dlp is already up to date.", level=LogLevel.INFO)
                else:
                    logger.log(f"yt-dlp update check finished with issues. Output: {stdout.strip()}", level=LogLevel.WARNING)
                    if stderr:
                        logger.log(f"yt-dlp stderr: {stderr.strip()}", level=LogLevel.DEBUG)
                        
        except Exception as e:
            logger.log(f"Error checking for yt-dlp updates: {str(e)}", level=LogLevel.ERROR)
