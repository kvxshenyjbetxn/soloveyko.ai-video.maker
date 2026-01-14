import requests
import time
import threading
from functools import wraps
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

def retry(tries=3, delay=5, backoff=2):
    """
    A decorator for retrying a function or method if it fails.
    """
    def deco_retry(f):
        @wraps(f)
        def f_retry(*args, **kwargs):
            mtries, mdelay = tries, delay
            while mtries > 1:
                try:
                    return f(*args, **kwargs)
                except Exception as e:
                    msg = f"'{f.__name__}' failed with exception: {e}. Retrying in {mdelay} seconds..."
                    logger.log(msg, level=LogLevel.WARNING)
                    time.sleep(mdelay)
                    mtries -= 1
                    mdelay *= backoff
            return f(*args, **kwargs)
        return f_retry
    return deco_retry


# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class ElevenLabsAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("elevenlabs_api_key")
        self.base_url = "https://voiceapi.csv666.ru"

    def _get_session(self):
        if not hasattr(thread_local_storage, "session"):
            thread_local_storage.session = requests.Session()
        return thread_local_storage.session

    def _make_request(self, method, endpoint, json=None, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {
            "X-API-Key": self.api_key,
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        }
        if json:
            headers["Content-Type"] = "application/json"

        # Proxy configuration
        proxies = {}
        if settings_manager.get("elevenlabs_proxy_enabled", False):
            proxy_url = settings_manager.get("elevenlabs_proxy_url", "").strip()
            if proxy_url:
                proxies = {"http": proxy_url, "https": proxy_url}
        
        try:
            session = self._get_session()
            response = session.request(
                method, 
                f"{self.base_url}/{endpoint}", 
                headers=headers, 
                json=json, 
                timeout=10, 
                proxies=proxies, # Add proxies here
                **kwargs
            )
            response.raise_for_status() 
            if response.status_code == 200:
                if "audio/mpeg" in response.headers.get("Content-Type", ""):
                    return response.content, "connected"
                return response.json(), "connected"
            return response, "connected"
        except requests.exceptions.HTTPError as e:
            logger.log(f"API request to {endpoint} failed with status {e.response.status_code}: {e.response.text}", level=LogLevel.ERROR)
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

    @retry(tries=3, delay=5, backoff=2)
    def get_balance(self):
        logger.log("Requesting ElevenLabs account balance...", level=LogLevel.INFO)
        data, status = self._make_request("get", "balance")
        if status == "connected" and data:
            balance = data.get("balance", 0)
            logger.log(f"Successfully retrieved balance: {balance}", level=LogLevel.SUCCESS)
            return balance, status
        
        if status == "not_configured":
             return None, status

        logger.log("Failed to retrieve ElevenLabs balance.", level=LogLevel.ERROR)
        return None, status
        
    def get_templates(self):
        logger.log("Requesting ElevenLabs templates...", level=LogLevel.INFO)
        data, status = self._make_request("get", "templates")
        if status == "connected" and data:
            logger.log("Successfully retrieved templates.", level=LogLevel.SUCCESS)
            return data, status
        
        if status == "not_configured":
            return None, status

        logger.log("Failed to retrieve ElevenLabs templates.", level=LogLevel.ERROR)
        return None, status

    def create_task(self, text, template_uuid=None):
        logger.log("Creating ElevenLabs voiceover task...", level=LogLevel.INFO)
        payload = {"text": text}
        if template_uuid:
            payload["template_uuid"] = template_uuid
        
        data, status = self._make_request("post", "tasks", json=payload)
        if status == "connected" and data:
            task_id = data.get("task_id")
            logger.log(f"Successfully created task with ID: {task_id}", level=LogLevel.SUCCESS)
            return task_id, status
        logger.log("Failed to create ElevenLabs task.", level=LogLevel.ERROR)
        return None, status

    def get_task_status(self, task_id):
        logger.log(f"Checking status for task {task_id}...", level=LogLevel.INFO)
        data, status = self._make_request("get", f"tasks/{task_id}/status")
        if status == "connected" and data:
            task_status = data.get("status")
            logger.log(f"Task {task_id} status: {task_status}", level=LogLevel.INFO)
            return task_status, status
        logger.log(f"Failed to get status for task {task_id}.", level=LogLevel.ERROR)
        return None, status

    def get_task_result(self, task_id):
        logger.log(f"Getting result for task {task_id}...", level=LogLevel.INFO)
        
        headers = {"X-API-Key": self.api_key}
        url = f"{self.base_url}/tasks/{task_id}/result"

        # Proxy configuration
        proxies = {}
        if settings_manager.get("elevenlabs_proxy_enabled", False):
            proxy_url = settings_manager.get("elevenlabs_proxy_url", "").strip()
            if proxy_url:
                proxies = {"http": proxy_url, "https": proxy_url}
        
        try:
            session = self._get_session()
            response = session.get(url, headers=headers, proxies=proxies)
            if response.status_code == 200:
                logger.log(f"Successfully downloaded audio for task {task_id}", level=LogLevel.SUCCESS)
                return response.content, "connected"
            elif response.status_code == 202:
                logger.log(f"Audio for task {task_id} is not ready yet.", level=LogLevel.INFO)
                return None, "not_ready"
            else:
                logger.log(f"Failed to download audio for task {task_id}. Status: {response.status_code}", level=LogLevel.ERROR)
                return None, "error"
        except requests.exceptions.RequestException as e:
            logger.log(f"Request failed for task {task_id} result: {e}", level=LogLevel.ERROR)
            return None, "error"
