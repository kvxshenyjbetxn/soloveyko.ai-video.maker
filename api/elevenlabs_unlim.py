import requests
import time
import threading
from functools import wraps
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

def retry(tries=3, delay=5, backoff=2):
    """
    A decorator for retrying a function or method if it fails.
    """
    def deco_retry(f):
        @wraps(f)
        def f_retry(*args, **kwargs):
            mtries, mdelay = tries, delay
            while mtries > 1:
                try:
                    return f(*args, **kwargs)
                except Exception as e:
                    msg = f"'{f.__name__}' failed with exception: {e}. Retrying in {mdelay} seconds..."
                    logger.log(msg, level=LogLevel.WARNING)
                    time.sleep(mdelay)
                    mtries -= 1
                    mdelay *= backoff
            return f(*args, **kwargs)
        return f_retry
    return deco_retry


# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class ElevenLabsUnlimAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("elevenlabs_unlim_api_key")
        self.base_url = "https://elevenlabs-unlimited.net/api/v1"

    def _get_session(self):
        if not hasattr(thread_local_storage, "session"):
            thread_local_storage.session = requests.Session()
        return thread_local_storage.session

    def _make_request(self, method, endpoint, json=None, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        try:
            session = self._get_session()
            response = session.request(method, f"{self.base_url}/{endpoint}", headers=headers, json=json, timeout=30, **kwargs)
            
            # Special handling for 4xx errors to return status/message properly
            if response.status_code >= 400:
                try:
                    err_data = response.json()
                    detail = err_data.get('detail', response.text)
                    logger.log(f"API request to {endpoint} failed (Status {response.status_code}): {detail}", level=LogLevel.ERROR)
                    return err_data, "error"
                except:
                    logger.log(f"API request to {endpoint} failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                    return None, "error"
            
            if response.status_code == 200:
                if "audio/mpeg" in response.headers.get("Content-Type", ""):
                    return response.content, "connected"
                return response.json(), "connected"
            return response, "connected"

        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def check_connection(self):
        logger.log("Checking ElevenLabsUnlim API connection...", level=LogLevel.INFO)
        _, status = self.get_balance()
        if status == "connected":
            logger.log("ElevenLabsUnlim API connection successful.", level=LogLevel.SUCCESS)
        else:
            logger.log("ElevenLabsUnlim API connection failed.", level=LogLevel.ERROR)
        return status

    @retry(tries=3, delay=5, backoff=2)
    def get_balance(self):
        # User stats endpoint: /api/v1/user/stats
        logger.log("Requesting ElevenLabsUnlim account stats...", level=LogLevel.INFO)
        data, status = self._make_request("get", "user/stats")
        if status == "connected" and data:
            # {
            #   "user_id": "user_123456",
            #   "subscription_type": "unlimited",
            #   "total_characters": 100000,
            #   "used_characters": 25000,
            #   "remaining_characters": 75000
            # }
            remaining = data.get("remaining_characters", 0)
            logger.log(f"Successfully retrieved balance: {remaining} chars", level=LogLevel.SUCCESS)
            return remaining, status
        
        if status == "not_configured":
             return None, status

        logger.log("Failed to retrieve ElevenLabsUnlim balance.", level=LogLevel.ERROR)
        return None, status
        
    def create_task(self, text, settings):
        """
        Create a voiceover task.
        settings dict should contain:
        - voice_id
        - model_id (optional, default 'eleven_multilingual_v2')
        - stability, similarity_boost, style, use_speaker_boost inside 'voice_settings'
        """
        logger.log("Creating ElevenLabsUnlim voiceover task...", level=LogLevel.INFO)
        
        payload = {
            "text": text,
            "voice_id": settings.get("voice_id", "21m00Tcm4TlvDq8ikWAM"), # default to Rachel
            "model_id": settings.get("model_id", "eleven_multilingual_v2"),
            "voice_settings": {
                "stability": settings.get("stability", 0.5),
                "similarity_boost": settings.get("similarity_boost", 0.75),
                "style": settings.get("style", 0.0),
                "use_speaker_boost": settings.get("use_speaker_boost", True)
            }
        }
        
        data, status = self._make_request("post", "voice/synthesize", json=payload)
        # Expected: {"task_id": "...", "status": "pending", ...}
        
        if status == "connected" and data and "task_id" in data:
            task_id = data.get("task_id")
            logger.log(f"Successfully created task with ID: {task_id}", level=LogLevel.SUCCESS)
            return task_id, status
            
        logger.log("Failed to create ElevenLabsUnlim task.", level=LogLevel.ERROR)
        return None, "error"

    def get_task_status(self, task_id):
        # GET /api/v1/voice/status/{task_id}
        # Returns 200 OK with status string or json? Doc says response value schema "string". Assume it returns JSON with status inside?
        # Actually doc example for status endpoint response is "string", but typically it's json.
        # Let's inspect create_task response: "status": "pending".
        # Let's assume it returns a JSON object with status. If not, we handle string.
        # Wait, doc says Response Schema: "string". Let's assume the body IS the status string.
        
        data, status = self._make_request("get", f"voice/status/{task_id}")
        
        # If the API returns a raw string for status, _make_request might parse it as json if it's quoted?
        # Or if it's just a string, requests.json() might fail if not valid json.
        # However, _make_request returns response.json() if 200.
        
        if status == "connected":
            # If data is a dict, look for status. If it's a string, it IS the status.
            task_status = data.get("status") if isinstance(data, dict) else data
            return task_status, status
            
        return None, status

    def get_task_result(self, task_id):
        # GET /api/v1/voice/download/{task_id}
        logger.log(f"Downloading audio for task {task_id}...", level=LogLevel.INFO)
        
        # We need the raw content here
        if not self.api_key:
            return None, "not_configured"
            
        headers = {"Authorization": f"Bearer {self.api_key}"}
        url = f"{self.base_url}/voice/download/{task_id}"
        
        try:
            session = self._get_session()
            response = session.get(url, headers=headers, timeout=60)
            
            if response.status_code == 200:
                logger.log(f"Successfully downloaded audio for task {task_id}", level=LogLevel.SUCCESS)
                return response.content, "connected"
            else:
                 logger.log(f"Failed to download audio for task {task_id}. Status: {response.status_code}", level=LogLevel.ERROR)
                 return None, "error"
                 
        except requests.exceptions.RequestException as e:
            logger.log(f"Request failed for task {task_id} download: {e}", level=LogLevel.ERROR)
            return None, "error"
