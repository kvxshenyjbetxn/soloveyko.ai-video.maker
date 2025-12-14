import datetime
from enum import Enum
import os
import threading
import sys
from utils.settings import settings_manager

class LogLevel(Enum):
    INFO = "INFO"
    SUCCESS = "SUCCESS"
    WARNING = "WARNING"
    ERROR = "ERROR"

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

class _Logger:
    def __init__(self):
        self.log_widget = None
        self.log_file = None
        self.lock = threading.Lock()
        
        # Always create a debug trace file
        # try:
        #     self.debug_file = open("debug_trace.log", "a", encoding="utf-8")
        # except:
        #     self.debug_file = None
        self.debug_file = None
            
        self.reconfigure()

    def reconfigure(self):
        with self.lock:
            # We keep the debug_file open always
            if settings_manager.get('detailed_logging_enabled', False):
                log_dir = "logs"
                os.makedirs(log_dir, exist_ok=True)
                timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
                self.log_file = os.path.join(log_dir, f"app_{timestamp}.log")
            else:
                self.log_file = None

    def set_log_widget(self, log_widget):
        with self.lock:
            self.log_widget = log_widget

    def log(self, message, level=LogLevel.INFO):
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        # Prepare data for the widget
        log_data = {
            "timestamp": datetime.datetime.now().strftime("%H:%M:%S"),
            "level": level,
            "message": message
        }

        with self.lock:
            # 1. Write to debug trace (Primary debugging tool)
            # if self.debug_file:
            #     try:
            #         self.debug_file.write(f"[{timestamp}] [{level.name}] {message}\n")
            #         self.debug_file.flush()
            #         os.fsync(self.debug_file.fileno())
            #     except:
            #         pass

            # 2. Write to conditional log file
            if self.log_file:
                try:
                    with open(self.log_file, "a", encoding="utf-8") as f:
                        f.write(f"[{timestamp}] [{level.name}] {message}\n")
                except IOError as e:
                    print(f"Failed to write to log file: {e}")

            # 3. Write to UI
            if self.log_widget:
                try:
                    self.log_widget.add_log_message(log_data)
                except RuntimeError:
                    pass
            
            # 4. Write to Console
            try:
                print(f"[{log_data['timestamp']}] [{level.name}] {message}")
                sys.stdout.flush()
            except:
                pass

logger = _Logger()