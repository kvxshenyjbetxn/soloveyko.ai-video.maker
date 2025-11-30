
import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class GooglerAPI:
    def __init__(self, api_key=None):
        self.settings = settings_manager.get("googler", {})
        self.api_key = api_key or self.settings.get("api_key")
        self.base_url = "https://app.recrafter.fun/api/v2"

    def _make_request(self, method, endpoint, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {"X-API-Key": self.api_key}
        if "headers" in kwargs:
            kwargs["headers"].update(headers)
        else:
            kwargs["headers"] = headers
        
        try:
            response = requests.request(method, f"{self.base_url}/{endpoint}", **kwargs)
            if response.status_code == 200:
                return response.json(), "connected"
            else:
                logger.log(f"API request to {endpoint} failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                return None, "error"
        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def get_usage(self):
        logger.log("Requesting Googler account usage...", level=LogLevel.INFO)
        data, status = self._make_request("get", "usage")
        if status == "connected" and data:
            logger.log(f"Successfully retrieved Googler usage.", level=LogLevel.SUCCESS)
            return data
        logger.log("Failed to retrieve Googler usage.", level=LogLevel.ERROR)
        return None

    def generate_image(self, prompt, aspect_ratio="IMAGE_ASPECT_RATIO_LANDSCAPE", seed=None, negative_prompt=None):
        logger.log(f"Requesting image generation from Googler for prompt: {prompt}", level=LogLevel.INFO)
        
        parameters = {
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }
        if seed:
            try:
                parameters["seed"] = int(seed)
            except (ValueError, TypeError):
                logger.log(f"Invalid seed value: {seed}. It will be ignored.", level=LogLevel.WARNING)

        if negative_prompt:
            parameters["negative_prompt"] = negative_prompt

        json_data = {
            "provider": "google_fx",
            "operation": "generate",
            "parameters": parameters
        }

        data, status = self._make_request("post", "images", json=json_data)

        if status == "connected" and data and data.get("success"):
            result = data.get("result")
            if result and isinstance(result, str):
                logger.log(f"Successfully generated image for prompt: {prompt}", level=LogLevel.SUCCESS)
                return result
            else:
                logger.log(f"API call successful but got empty or invalid result for prompt: {prompt}. Result: {result}", level=LogLevel.WARNING)
                return None
        
        error_message = data.get("error") if data else "Unknown error"
        logger.log(f"Failed to generate image for prompt: {prompt}. Error: {error_message}", level=LogLevel.ERROR)
        return None
