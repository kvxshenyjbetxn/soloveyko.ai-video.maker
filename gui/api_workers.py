
from PySide6.QtCore import QObject, Signal, QRunnable
import requests
from utils.logger import logger, LogLevel

# Windows COM handling for threads
try:
    import pythoncom
except ImportError:
    pythoncom = None

class ApiKeyCheckSignals(QObject):
    finished = Signal(bool, str, int, object) # is_valid, expires_at, subscription_level, telegram_id

class ApiKeyCheckWorker(QRunnable):
    def __init__(self, api_key, server_url):
        super().__init__()
        self.signals = ApiKeyCheckSignals()
        self.api_key = api_key
        self.server_url = server_url

    def run(self):
        if pythoncom:
             pythoncom.CoInitialize()
        try:
            is_valid = False
            expires_at = None
            subscription_level = 1 # Default to base level
            telegram_id = None
            if not self.api_key or not self.server_url:
                self.signals.finished.emit(is_valid, expires_at, subscription_level, telegram_id)
                return
            try:
                from utils.hardware_id import get_hardware_id
                hardware_id = get_hardware_id()
                
                response = requests.post(
                    f"{self.server_url}/validate_key/",
                    json={"key": self.api_key, "hardware_id": hardware_id},
                    timeout=5
                )
                if response.status_code == 200:
                    data = response.json()
                    is_valid = data.get("valid", False)
                    expires_at = data.get("expires_at")
                    subscription_level = data.get("subscription_level", 1)
                    telegram_id = data.get("telegram_id")
                else:
                    is_valid = False
                    expires_at = None
                    subscription_level = 1
                    telegram_id = None
            except requests.RequestException:
                # Network error, etc.
                pass
            self.signals.finished.emit(is_valid, expires_at, subscription_level, telegram_id)
        finally:
            if pythoncom:
                pythoncom.CoUninitialize()
