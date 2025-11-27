
import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

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

    def generate_image(self, prompt):
        self.load_credentials() 
        
        params = {
            "model": self.model,
            "width": self.width,
            "height": self.height,
            "nologo": self.nologo,
            "enhance": self.enhance,
        }
        if self.token:
            params["token"] = self.token
            
        try:
            url_prompt = requests.utils.quote(prompt)
            request_url = f"{self.base_url}/prompt/{url_prompt}"
            
            # Using the custom logger here
            logger.log(f"      - Generating image (Model: {self.model}, Size: {self.width}x{self.height}) for prompt: '{prompt}'", LogLevel.INFO)
            
            response = requests.get(request_url, params=params)
            response.raise_for_status()
            return response.content
        except requests.exceptions.RequestException as e:
            logger.log(f"      - Error generating image for prompt: '{prompt}': {e}", LogLevel.ERROR)
            return None
