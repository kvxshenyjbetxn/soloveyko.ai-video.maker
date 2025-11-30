import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class ElevenLabsAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("elevenlabs_api_key")
        self.base_url = "https://voiceapi.csv666.ru"

    def _make_request(self, method, endpoint, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {"X-API-Key": self.api_key}
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

    def check_connection(self):
        logger.log("Checking ElevenLabs API connection...", level=LogLevel.INFO)
        _, status = self.get_balance()
        if status == "connected":
            logger.log("ElevenLabs API connection successful.", level=LogLevel.SUCCESS)
        else:
            logger.log("ElevenLabs API connection failed.", level=LogLevel.ERROR)
        return status

    def get_balance(self):
        logger.log("Requesting ElevenLabs account balance...", level=LogLevel.INFO)
        data, status = self._make_request("get", "balance")
        if status == "connected" and data:
            balance = data.get("balance", 0)
            logger.log(f"Successfully retrieved balance: {balance}", level=LogLevel.SUCCESS)
            return balance, status
        logger.log("Failed to retrieve ElevenLabs balance.", level=LogLevel.ERROR)
        return None, status
