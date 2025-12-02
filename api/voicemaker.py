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

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        # Виправлено VoiceID на ai3-Jony, який точно є в документації
        payload = {
            "Engine": "neural",
            "VoiceId": "ai3-Jony", 
            "LanguageCode": "en-US",
            "Text": "Connection check",
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
                    remaining = data.get("remainChars", "Unknown")
                    logger.log(f"Voicemaker connection successful. Remaining chars: {remaining}", level=LogLevel.SUCCESS)
                    return "connected"
                else:
                    logger.log(f"Voicemaker API error: {data.get('message')}", level=LogLevel.ERROR)
                    return "error"
            elif response.status_code == 401:
                logger.log("Voicemaker unauthorized (401). Check API Key.", level=LogLevel.ERROR)
                return "error"
            else:
                logger.log(f"Voicemaker HTTP error: {response.status_code} - {response.text}", level=LogLevel.ERROR)
                return "error"

        except requests.exceptions.RequestException as e:
            logger.log(f"Voicemaker connection check failed: {e}", level=LogLevel.ERROR)
            return "error"