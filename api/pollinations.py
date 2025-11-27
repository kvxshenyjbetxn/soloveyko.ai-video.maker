
import logging
import requests
from utils.settings import settings_manager

class PollinationsAPI:
    def __init__(self):
        self.base_url = "https://image.pollinations.ai"
        self.load_credentials()
        logging.info("PollinationsAPI initialized")

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
        logging.info(f"Generating image with prompt: '{prompt}' using model: {self.model}")
        
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
            # The prompt is part of the URL path, not a query parameter
            url_prompt = requests.utils.quote(prompt)
            response = requests.get(f"{self.base_url}/prompt/{url_prompt}", params=params)
            response.raise_for_status()
            logging.info(f"Successfully generated image for prompt: '{prompt}'")
            return response.content
        except requests.exceptions.RequestException as e:
            logging.error(f"Error generating image for prompt: '{prompt}': {e}")
            return None
