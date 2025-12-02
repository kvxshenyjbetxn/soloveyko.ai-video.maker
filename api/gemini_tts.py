import requests
import time
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class GeminiTTSAPI:
    def __init__(self):
        self.api_key = settings_manager.get("gemini_tts_api_key")
        self.base_url = "https://gemini-tts-server-beta-production.up.railway.app"

    def _make_request(self, method, endpoint, json=None, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {"x-api-key": self.api_key}
        if json:
            headers["Content-Type"] = "application/json"
        
        url = f"{self.base_url.rstrip('/')}/{endpoint.lstrip('/')}"
        
        try:
            response = requests.request(method, url, headers=headers, json=json, **kwargs)
            response.raise_for_status()
            if response.status_code == 200:
                # Check content type for binary data (audio)
                if "audio" in response.headers.get("Content-Type", ""):
                     return response.content, "connected"
                return response.json(), "connected"
            return response, "connected"
        except requests.exceptions.HTTPError as e:
            logger.log(f"API request to {endpoint} failed with status {e.response.status_code}: {e.response.text}", level=LogLevel.ERROR)
            return e.response, "error"
        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def check_connection(self):
        logger.log("Checking GeminiTTS API connection...", level=LogLevel.INFO)
        _, status = self.get_balance()
        if status == "connected":
            logger.log("GeminiTTS API connection successful.", level=LogLevel.SUCCESS)
        else:
            logger.log("GeminiTTS API connection failed.", level=LogLevel.ERROR)
        return status

    def get_balance(self):
        # Endpoint: /api/v1/me
        logger.log("Requesting GeminiTTS account balance...", level=LogLevel.INFO)
        data, status = self._make_request("get", "api/v1/me")
        if status == "connected" and data:
            balance = data.get("balance", 0.0)
            logger.log(f"Successfully retrieved balance: {balance}", level=LogLevel.SUCCESS)
            return balance, status
        logger.log("Failed to retrieve GeminiTTS balance.", level=LogLevel.ERROR)
        return None, status

    def create_task(self, text, voice="Puck", tone=None):
        # Endpoint: /api/v1/tts/task
        logger.log("Creating GeminiTTS task...", level=LogLevel.INFO)
        payload = {
            "text": text,
            "voice": voice or "Puck"
        }
        if tone:
            payload["tone"] = tone
            
        data, status = self._make_request("post", "api/v1/tts/task", json=payload)
        if status == "connected" and data:
            task_id = data.get("task_id")
            logger.log(f"Successfully created task with ID: {task_id}", level=LogLevel.SUCCESS)
            return task_id, status
        logger.log("Failed to create GeminiTTS task.", level=LogLevel.ERROR)
        return None, status

    def get_task_status(self, task_id):
        # Endpoint: /api/v1/tts/tasks/{task_id}
        data, status = self._make_request("get", f"api/v1/tts/tasks/{task_id}")
        if status == "connected" and data:
            task_status = data.get("status")
            progress = data.get("progress", {})
            current = progress.get('current', '?')
            total = progress.get('total', '?')
            logger.log(f"Task {task_id} status: {task_status} ({current}/{total} chunks)", level=LogLevel.INFO)
            return task_status, status
        logger.log(f"Failed to get status for task {task_id}.", level=LogLevel.ERROR)
        return None, status

    def download_audio(self, task_id, context_info=""):
        # Endpoint: /api/v1/tts/tasks/{task_id}/download
        logger.log(f"Downloading audio for task {task_id}...", level=LogLevel.INFO)
        data, status = self._make_request("get", f"api/v1/tts/tasks/{task_id}/download")
        if status == "connected" and data:
            msg = f"Successfully downloaded audio for task {task_id}"
            if context_info:
                msg += f" ({context_info})"
            logger.log(msg, level=LogLevel.SUCCESS)
            return data, status
        logger.log(f"Failed to download audio for task {task_id}.", level=LogLevel.ERROR)
        return None, status
