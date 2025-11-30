import datetime
from enum import Enum

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
    def __init__(self):
        self.log_widget = None

    def set_log_widget(self, log_widget):
        self.log_widget = log_widget

    def log(self, message, level=LogLevel.INFO):
        timestamp = datetime.datetime.now().strftime("%H:%M:%S")
        
        # Prepare data for the widget
        log_data = {
            "timestamp": timestamp,
            "level": level,
            "message": message
        }

        if self.log_widget:
            # The widget will handle the formatting
            try:
                self.log_widget.add_log_message(log_data)
            except RuntimeError:
                # This can happen if the widget is being deleted
                pass
        
        # Console output remains simple
        print(f"[{timestamp}] [{level.name}] {message}")

logger = _Logger()