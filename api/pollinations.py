import requests
import threading
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class PollinationsAPI:
    def __init__(self):
        self.base_url = "https://image.pollinations.ai"
        self.load_credentials()
        logger.log("PollinationsAPI initialized", LogLevel.INFO)

    def load_credentials(self):
        self.settings = settings_manager.get("pollinations", {})
        self.token = self.settings.get("token")
        self.model = self.settings.get("model", "flux")
        self.width = self.settings.get("width", 1024)
        self.height = self.settings.get("height", 1024)
        self.nologo = self.settings.get("nologo", False)
        self.enhance = self.settings.get("enhance", False)

    def _get_session(self):
        if not hasattr(thread_local_storage, "session"):
            thread_local_storage.session = requests.Session()
        return thread_local_storage.session

    def generate_image(self, prompt, model=None, width=None, height=None, nologo=None, enhance=None):
        self.load_credentials() 
        
        # Override with provided arguments if they exist
        current_model = model if model is not None else self.model
        current_width = width if width is not None else self.width
        current_height = height if height is not None else self.height
        current_nologo = nologo if nologo is not None else self.nologo
        current_enhance = enhance if enhance is not None else self.enhance

        # Normalize parameters for Pollinations API
        params = {
            "model": current_model,
            "width": current_width,
            "height": current_height,
            "nologo": str(current_nologo).lower(),
            "enhance": str(current_enhance).lower(),
        }
        
        # Use 'key' parameter for auth instead of 'token' if possible, 
        # but keep compatibility with both just in case.
        if self.token:
            params["token"] = self.token
            params["key"] = self.token
            
        max_retries = 3
        retry_delay = 5 # seconds
        
        for attempt in range(max_retries):
            try:
                url_prompt = requests.utils.quote(prompt)
                request_url = f"{self.base_url}/prompt/{url_prompt}"
                
                logger.log(f"      - Generating image (Model: {current_model}, Size: {current_width}x{current_height}) for prompt: '{prompt[:50]}...'", LogLevel.INFO)
                
                session = self._get_session()
                response = session.get(request_url, params=params, timeout=60)
                
                if response.status_code == 429:
                    if attempt < max_retries - 1:
                        logger.log(f"      - [429 Error] Rate limit exceeded. Retrying in {retry_delay}s... (Attempt {attempt + 1}/{max_retries})", LogLevel.WARNING)
                        time.sleep(retry_delay)
                        retry_delay *= 2 # Exponential backoff
                        continue
                    else:
                        logger.log(f"      - [429 Error] Rate limit exceeded after {max_retries} attempts.", LogLevel.ERROR)
                        return None

                response.raise_for_status()
                return response.content
            except requests.exceptions.RequestException as e:
                logger.log(f"      - Error generating image for prompt: '{prompt[:50]}...': {e}", LogLevel.ERROR)
                if attempt < max_retries - 1:
                    time.sleep(2)
                    continue
                return None
        return None
