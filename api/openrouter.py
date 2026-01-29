import requests
import time
from functools import wraps
from utils.settings import settings_manager
from utils.logger import logger, LogLevel


import threading

# Use thread-local storage to keep sessions isolated per thread
thread_local = threading.local()

def get_session():
    if not hasattr(thread_local, "session"):
        thread_local.session = requests.Session()
    return thread_local.session

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

class OpenRouterAPI:
    def __init__(self, api_key=None):
        self.api_key = api_key or settings_manager.get("openrouter_api_key")
        self.base_url = "https://openrouter.ai/api/v1"

    def _make_request(self, method, endpoint, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {"Authorization": f"Bearer {self.api_key}"}
        kwargs["headers"] = headers
        
        try:
            # Use thread-local session
            session = get_session()
            response = session.request(method, f"{self.base_url}/{endpoint}", **kwargs)
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
        _, status = self._make_request("get", "auth/key")
        if status == "connected":
            logger.log("OpenRouter API connection successful.", level=LogLevel.SUCCESS)
        else:
            logger.log("OpenRouter API connection failed.", level=LogLevel.ERROR)
        return status

    def get_balance(self):
        logger.log("Requesting OpenRouter account balance...", level=LogLevel.INFO)
        data, status = self._make_request("get", "credits")
        if status == "connected" and data:
            credits = data.get("data", {}).get("total_credits", 0.0)
            usage = data.get("data", {}).get("total_usage", 0.0)
            balance = credits - usage
            logger.log(f"Successfully retrieved balance: {balance:.4f}$", level=LogLevel.SUCCESS)
            return balance
        
        if status == "not_configured":
             return None

        logger.log("Failed to retrieve OpenRouter balance.", level=LogLevel.ERROR)
        return None



    @retry(tries=2, delay=5, backoff=2)
    def get_chat_completion(self, model, messages, max_tokens=None, temperature=None):
        if not self.api_key:
            error_msg = "API key is not configured."
            logger.log(error_msg, level=LogLevel.ERROR)
            raise ValueError(error_msg)

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://soloveyko-ai.kherson.ua", # Required for some free models/policies
            "X-Title": "Soloveyko.AI-Video.Maker"
        }
        
        # Prepare payload
        data = {
            "model": model,
            "messages": messages
        }

        if max_tokens and max_tokens > 0:
            data["max_tokens"] = max_tokens

        # Add optional parameters if provided
        if temperature is not None:
            data["temperature"] = temperature

        # Apply Google-specific safety settings to disable content filtering
        if model.lower().startswith("google/"):
             data["safety_settings"] = [
                {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE"}
            ]

        logger.log(f"Requesting chat completion from model: {model}", level=LogLevel.INFO)
        
        try:
            # Use thread-local session
            session = get_session()
            response = session.post(f"{self.base_url}/chat/completions", headers=headers, json=data)
            
            if response.status_code != 200:
                error_body = response.text
                try:
                    error_json = response.json()
                    if "error" in error_json:
                        error_body = error_json["error"].get("message", error_body)
                except:
                    pass
                
                error_msg = f"OpenRouter Error {response.status_code}: {error_body}"
                # If we exhausted retries or it's a fatal error, the decorator will eventually let this bubble up
                raise Exception(error_msg)

            logger.log(f"Chat completion from {model} successful.", level=LogLevel.SUCCESS)
            return response.json()
        except requests.exceptions.RequestException as e:
            error_msg = f"An error occurred during chat completion request: {e}"
            if hasattr(e, 'response') and e.response is not None:
                 error_msg += f"\nResponse status: {e.response.status_code}"
                 error_msg += f"\nBody: {e.response.text}"
            logger.log(error_msg, level=LogLevel.ERROR)
            raise Exception(error_msg)
