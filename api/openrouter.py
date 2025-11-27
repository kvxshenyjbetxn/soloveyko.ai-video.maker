import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class OpenRouterAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("openrouter_api_key")
        self.base_url = "https://openrouter.ai/api/v1"

    def _make_request(self, endpoint):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {"Authorization": f"Bearer {self.api_key}"}
        try:
            response = requests.get(f"{self.base_url}/{endpoint}", headers=headers)
            if response.status_code == 200:
                return response.json(), "connected"
            else:
                logger.log(f"API request to {endpoint} failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                return None, "error"
        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def check_connection(self):
        logger.log("Checking OpenRouter API connection...", level=LogLevel.INFO)
        _, status = self._make_request("auth/key")
        if status == "connected":
            logger.log("OpenRouter API connection successful.", level=LogLevel.SUCCESS)
        else:
            logger.log("OpenRouter API connection failed.", level=LogLevel.ERROR)
        return status

    def get_balance(self):
        logger.log("Requesting OpenRouter account balance...", level=LogLevel.INFO)
        data, status = self._make_request("auth/key")
        if status == "connected" and data:
            usage = data.get("data", {}).get("usage")
            logger.log(f"Successfully retrieved balance: {usage:.4f}$", level=LogLevel.SUCCESS)
            return usage
        logger.log("Failed to retrieve OpenRouter balance.", level=LogLevel.ERROR)
        return None

    def get_chat_completion(self, model, messages, max_tokens=4096):
        if not self.api_key:
            error_msg = "API key is not configured."
            logger.log(error_msg, level=LogLevel.ERROR)
            raise ValueError(error_msg)

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        data = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens
        }
        
        logger.log(f"Requesting chat completion from model: {model}", level=LogLevel.INFO)
        try:
            response = requests.post(f"{self.base_url}/chat/completions", headers=headers, json=data)
            response.raise_for_status() # Raises an HTTPError for bad responses (4xx or 5xx)
            logger.log(f"Chat completion from {model} successful.", level=LogLevel.SUCCESS)
            return response.json()
        except requests.exceptions.RequestException as e:
            error_msg = f"An error occurred during chat completion request: {e}"
            logger.log(error_msg, level=LogLevel.ERROR)
            # You might want to log this error or handle it more gracefully
            print(error_msg)
            return None
