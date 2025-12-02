import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class VoicemakerAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("voicemaker_api_key")
        self.base_url = "https://developer.voicemaker.in/voice/api"

    def check_connection(self):
        logger.log("Checking Voicemaker API connection...", level=LogLevel.INFO)
        if not self.api_key:
            return "not_configured"

        balance, status = self.get_balance()
        return status

    def get_balance(self):
        if not self.api_key:
            return None, "not_configured"

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        # Minimal request to check balance based on documentation
        payload = {
            "Engine": "neural",
            "VoiceId": "ai3-Jony", 
            "LanguageCode": "en-US",
            "Text": "test", # Using "test" instead of "." to avoid potential validation errors
            "OutputFormat": "mp3"
        }

        try:
            response = requests.post(self.base_url, headers=headers, json=payload)
            
            if response.status_code == 200:
                data = response.json()
                if data.get("success"):
                    remaining = data.get("remainChars", 0)
                    logger.log(f"Voicemaker connection successful. Remaining chars: {remaining}", level=LogLevel.SUCCESS)
                    return remaining, "connected"
                else:
                    logger.log(f"Voicemaker API error: {data.get('message')}", level=LogLevel.ERROR)
                    return None, "error"
            elif response.status_code == 401:
                logger.log("Voicemaker unauthorized (401). Check API Key.", level=LogLevel.ERROR)
                return None, "error"
            else:
                logger.log(f"Voicemaker HTTP error: {response.status_code} - {response.text}", level=LogLevel.ERROR)
                return None, "error"

        except requests.exceptions.RequestException as e:
            logger.log(f"Voicemaker connection check failed: {e}", level=LogLevel.ERROR)
            return None, "error"