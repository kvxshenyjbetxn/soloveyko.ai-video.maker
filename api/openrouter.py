import requests
from utils.settings import settings_manager

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
                return None, "error"
        except requests.exceptions.RequestException:
            return None, "error"

    def check_connection(self):
        _, status = self._make_request("auth/key")
        return status

    def get_balance(self):
        data, status = self._make_request("auth/key")
        if status == "connected" and data:
            usage = data.get("data", {}).get("usage")
            return usage
        return None

    def get_chat_completion(self, model, messages, max_tokens=4096):
        if not self.api_key:
            raise ValueError("API key is not configured.")

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        data = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens
        }
        
        try:
            response = requests.post(f"{self.base_url}/chat/completions", headers=headers, json=data)
            response.raise_for_status() # Raises an HTTPError for bad responses (4xx or 5xx)
            return response.json()
        except requests.exceptions.RequestException as e:
            # You might want to log this error or handle it more gracefully
            print(f"An error occurred: {e}")
            return None
