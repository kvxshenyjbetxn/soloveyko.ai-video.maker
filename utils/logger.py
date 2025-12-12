import datetime
from enum import Enum
import os

class LogLevel(Enum):
    INFO = "INFO"
    SUCCESS = "SUCCESS"
    WARNING = "WARNING"
    ERROR = "ERROR"

    def to_color(self):
        return {
            LogLevel.INFO: "#ffffff",  # White
            LogLevel.SUCCESS: "#28a745", # Green
            LogLevel.WARNING: "#ffa500", # Orange
            LogLevel.ERROR: "#dc3545",   # Red
        }.get(self, "#ffffff") # Default to white

    def to_icon(self):
        return {
            LogLevel.INFO: "ℹ️",
            LogLevel.SUCCESS: "✅",
            LogLevel.WARNING: "⚠️",
            LogLevel.ERROR: "❌",
        }.get(self, "➡️")


class _Logger:
    def __init__(self, log_file="app.log"):
        self.log_widget = None
        self.log_file = log_file
        # Clear the log file at the start of the session
        try:
            if os.path.exists(self.log_file):
                os.remove(self.log_file)
        except OSError as e:
            print(f"Error removing old log file: {e}")

    def set_log_widget(self, log_widget):
        self.log_widget = log_widget

    def log(self, message, level=LogLevel.INFO):
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        # Prepare data for the widget
        log_data = {
            # Use a simpler timestamp for the UI
            "timestamp": datetime.datetime.now().strftime("%H:%M:%S"),
            "level": level,
            "message": message
        }

        # Log to file
        try:
            with open(self.log_file, "a", encoding="utf-8") as f:
                f.write(f"[{timestamp}] [{level.name}] {message}\n")
        except IOError as e:
            # Fallback to console if file logging fails
            print(f"Failed to write to log file: {e}")

        if self.log_widget:
            # The widget will handle the formatting
            try:
                self.log_widget.add_log_message(log_data)
            except RuntimeError:
                # This can happen if the widget is being deleted
                pass
        
        # Console output remains simple
        print(f"[{log_data['timestamp']}] [{level.name}] {message}")

logger = _Logger()