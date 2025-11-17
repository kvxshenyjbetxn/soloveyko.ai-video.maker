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
