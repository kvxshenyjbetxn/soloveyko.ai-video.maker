import datetime
from enum import Enum
import os
import threading
import sys
from PySide6.QtCore import QObject, Signal
from utils.settings import settings_manager

class LogLevel(Enum):
    INFO = "INFO"
    SUCCESS = "SUCCESS"
    WARNING = "WARNING"
    ERROR = "ERROR"
    DEBUG = "DEBUG"

    def to_color(self):
        theme = settings_manager.get('theme', 'light')
        if theme == 'light':
            info_color = "#000000"  # Black for light theme
        else:
            info_color = "#ffffff"  # White for dark themes

        return {
            LogLevel.INFO: info_color,
            LogLevel.SUCCESS: "#28a745",  # Green
            LogLevel.WARNING: "#ffa500",  # Orange
            LogLevel.ERROR: "#dc3545",    # Red
        }.get(self, info_color)

    def to_icon(self):
        return {
            LogLevel.INFO: "ℹ️",
            LogLevel.SUCCESS: "✅",
            LogLevel.WARNING: "⚠️",
            LogLevel.ERROR: "❌",
        }.get(self, "➡️")

class _Logger(QObject):
    log_message_signal = Signal(dict)

    def __init__(self):
        super().__init__()
        self.log_file = None
        self.lock = threading.Lock()
        self.debug_file = None
        self.reconfigure()

    def reconfigure(self):
        with self.lock:
            if settings_manager.get('detailed_logging_enabled', False):
                log_dir = "logs"
                os.makedirs(log_dir, exist_ok=True)
                timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
                self.log_file = os.path.join(log_dir, f"app_{timestamp}.log")
            else:
                self.log_file = None

    def log(self, message, level=LogLevel.INFO):
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        log_data = {
            "timestamp": datetime.datetime.now().strftime("%H:%M:%S"),
            "level": level,
            "message": message
        }

        # Emit signal for the UI to update safely
        self.log_message_signal.emit(log_data)

        with self.lock:
            # Write to conditional log file
            if self.log_file:
                try:
                    with open(self.log_file, "a", encoding="utf-8") as f:
                        f.write(f"[{timestamp}] [{level.name}] {message}\n")
                except IOError as e:
                    print(f"Failed to write to log file: {e}")
            
            # Write to Console
            try:
                print(f"[{log_data['timestamp']}] [{level.name}] {message}")
                sys.stdout.flush()
            except:
                pass

    def cleanup_old_logs(self, max_days=7):
        """
        Deletes log files in the 'logs' directory that are older than max_days.
        Also rotates 'crash.log' if it's too old.
        """
        log_dir = "logs"
        if not os.path.exists(log_dir):
            return

        now = datetime.datetime.now()
        cutoff = now - datetime.timedelta(days=max_days)

        self.log(f"Cleaning up logs older than {max_days} days...", level=LogLevel.INFO)
        count = 0
        
        # 1. Clean up "logs/" directory
        try:
            for filename in os.listdir(log_dir):
                file_path = os.path.join(log_dir, filename)
                if os.path.isfile(file_path):
                    try:
                        file_mtime = datetime.datetime.fromtimestamp(os.path.getmtime(file_path))
                        if file_mtime < cutoff:
                            os.remove(file_path)
                            count += 1
                    except Exception as e:
                        print(f"Failed to delete old log {filename}: {e}")
        except Exception as e:
             self.log(f"Error during log cleanup: {e}", level=LogLevel.ERROR)

        if count > 0:
            self.log(f"Cleaned up {count} old log files.", level=LogLevel.SUCCESS)
        else:
            self.log("No old logs found to clean up.", level=LogLevel.INFO)

        # 2. Check "crash.log" in the root directory
        # Since crash.log is appended to, we might want to rename/archive it if it gets too old/large
        # For now, let's just rotate it if it hasn't been modified in a long time (unlikely if active)
        # or maybe we just leave it as the user asked for "folder logs". 
        # The prompt said "there are 2 types of logs", likely referring to `logs/` and `crash.log`.
        # I will check crash.log's mtime too.
        crash_log_path = "crash.log"
        if os.path.exists(crash_log_path):
             try:
                mod_time = datetime.datetime.fromtimestamp(os.path.getmtime(crash_log_path))
                if mod_time < cutoff:
                    # If crash.log hasn't been touched in 7 days, it's probably stale or from a previous install
                    # But since it's a single file we keep appending to, deleting it might lose history.
                    # However, if it's really old (last modified > 7 days ago), it means NO crashes happened for 7 days.
                    # Let's delete it to keep it clean.
                    os.remove(crash_log_path)
                    self.log("Removed old crash.log", level=LogLevel.SUCCESS)
             except Exception as e:
                print(f"Failed to check/delete crash.log: {e}")

logger = _Logger()