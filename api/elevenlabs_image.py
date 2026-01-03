import requests
import time
from utils.settings import settings_manager
from utils.logger import logger, LogLevel
import threading

# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class ElevenLabsImageAPI:
    def __init__(self, api_key=None):
        self.settings = settings_manager.get("elevenlabs_image", {})
        self.api_key = api_key or self.settings.get("api_key")
        self.base_url = "https://voiceapi.csv666.ru/api/v1"

    def _get_session(self):
        if not hasattr(thread_local_storage, "session"):
            thread_local_storage.session = requests.Session()
        return thread_local_storage.session

    def _make_request(self, method, endpoint, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {
            "X-API-Key": self.api_key,
            "Content-Type": "application/json",
            "Accept": "application/json"
        }
        if "headers" in kwargs:
            kwargs["headers"].update(headers)
        else:
            kwargs["headers"] = headers
        
        try:
            url = f"{self.base_url}/{endpoint}"
            session = self._get_session()
            response = session.request(method, url, **kwargs)
            
            if response.status_code not in [200, 201]:
                 logger.log(f"API request to {endpoint} failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                 return None, "error"
            
            try:
                return response.json(), "connected"
            except requests.exceptions.JSONDecodeError:
                return {}, "connected" 

        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def generate_image(self, prompt, aspect_ratio="3:2", **kwargs):
        # ElevenLabsImage might support aspect_ratio as string like "3:2"
        # The documentation says: "aspect_ratio": "3:2"
        
        logger.log(f"Requesting image generation from ElevenLabsImage for prompt: {prompt}", level=LogLevel.INFO)
        
        # Googler uses keys like "IMAGE_ASPECT_RATIO_LANDSCAPE", VoiceAPI expects "3:2"
        # We might need to map them if the UI passes the long enum-like strings.
        # But for now let's assume the UI will be adapted or we handle mapping here.
        # Given the task description, "create separate tab... separate file for api and tab", I can define the options in the tab.
        
        payload = {
            "prompt": prompt,
            "aspect_ratio": aspect_ratio
        }

        data, status = self._make_request("post", "image/create", json=payload)

        if status == "connected" and data:
            image_b64 = data.get("image_b64")
            if image_b64:
                return image_b64
            else:
                 logger.log(f"API call successful but got empty image_b64 for prompt: {prompt}. Response: {data}", level=LogLevel.WARNING)
                 return None
        
        return None
