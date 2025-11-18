import datetime

class _Logger:
    def __init__(self):
        self.log_widget = None

    def set_log_widget(self, log_widget):
        self.log_widget = log_widget

    def log(self, message):
        timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_message = f"[{timestamp}] {message}"
        
        if self.log_widget:
            self.log_widget.add_log_message(log_message)
        
        # Also print to console for debugging
        print(log_message)

logger = _Logger()
