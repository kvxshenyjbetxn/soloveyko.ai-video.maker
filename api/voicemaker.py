import requests
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class VoicemakerAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("voicemaker_api_key")
        self.base_url = "https://developer.voicemaker.in/voice/api"

    def get_balance(self):
        if not self.api_key:
            return None, "not_configured"

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        # Мінімальний запит для перевірки балансу (1 символ)
        payload = {
            "Engine": "neural",
            "VoiceId": "ai3-Jony", 
            "LanguageCode": "en-US",
            "Text": ".",
            "OutputFormat": "mp3",
            "SampleRate": "48000",
            "Effect": "default",
            "MasterSpeed": "0",
            "MasterVolume": "0",
            "MasterPitch": "0"
        }

        try:
            response = requests.post(self.base_url, headers=headers, json=payload)
            
            if response.status_code == 200:
                data = response.json()
                if data.get("success"):
                    remaining = data.get("remainChars", 0)
                    return remaining, "connected"
                else:
                    logger.log(f"Voicemaker API error: {data.get('message')}", level=LogLevel.ERROR)
                    return None, "error"
            elif response.status_code == 401:
                return None, "error"
            else:
                return None, "error"

        except Exception as e:
            logger.log(f"Voicemaker balance check failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def check_connection(self):
        logger.log("Checking Voicemaker API connection...", level=LogLevel.INFO)
        balance, status = self.get_balance()
        
        if status == "connected":
            logger.log(f"Voicemaker connection successful. Remaining chars: {balance}", level=LogLevel.SUCCESS)
            return "connected"
        elif status == "not_configured":
            return "not_configured"
        else:
            return "error"