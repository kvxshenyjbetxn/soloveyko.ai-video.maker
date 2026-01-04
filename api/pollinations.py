import requests
import threading
import time
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class PollinationsAPI:
    def __init__(self):
        self.base_url = "https://gen.pollinations.ai"
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

    def get_models(self):
        """
        Fetches the list of available image models dynamically from the API.
        Returns a list of model names.
        """
        try:
            url = f"{self.base_url}/image/models"
            logger.log(f"Fetching Pollinations models from {url}", LogLevel.INFO)
            
            session = self._get_session()
            response = session.get(url, timeout=10)
            response.raise_for_status()
            
            models_data = response.json()
            # models_data is a list of objects with 'name' and 'aliases'
            # We want to collect all names
            model_names = []
            for model in models_data:
                if "name" in model:
                    model_names.append(model["name"])
            
            if not model_names:
                logger.log("Warning: Pollinations API returned empty model list.", LogLevel.WARNING)
                return []
                
            return model_names
        except Exception as e:
            logger.log(f"Error fetching Pollinations models: {e}", LogLevel.ERROR)
            return []

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
        
        headers = {}
        # Determine which API to use based on token presence
        if self.token and self.token.strip():
            # Authenticated: Use new API (gen.pollinations.ai)
            base_url = "https://gen.pollinations.ai"
            endpoint = "/image"
            headers["Authorization"] = f"Bearer {self.token.strip()}"
        else:
            # Unauthenticated: Use legacy API (image.pollinations.ai)
            base_url = "https://image.pollinations.ai"
            endpoint = "/prompt"
            
        max_retries = 3
        retry_delay = 5 # seconds
        
        for attempt in range(max_retries):
            try:
                url_prompt = requests.utils.quote(prompt)
                request_url = f"{base_url}{endpoint}/{url_prompt}"
                
                logger.log(f"      - Generating image (Model: {current_model}, Base: {base_url}) for prompt: '{prompt[:50]}...'", LogLevel.INFO)
                
                session = self._get_session()
                # Pass params and headers
                response = session.get(request_url, params=params, headers=headers, timeout=60)
                
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
                
                # Don't retry client errors (4xx) except 429 which is handled above
                # But if we get a 401 on the new API, we might want to fail fast or maybe fallback? 
                # For now let's failing fast on 401 is correct as it means invalid token if we tried new API.
                if response.status_code and 400 <= response.status_code < 500 and response.status_code != 429:
                     return None

                if attempt < 2:
                     time.sleep(2)
                     continue
                
                return None
        return None
