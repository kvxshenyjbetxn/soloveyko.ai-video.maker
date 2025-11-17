import requests

class OpenRouterAPI:
    def __init__(self, api_key):
        self.api_key = api_key
        self.base_url = "https://openrouter.ai/api/v1"
        self.headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }

    def check_connection_and_get_balance(self):
        """Checks the API key and retrieves the total usage."""
        try:
            # Note: The endpoint was corrected from /auth/key to /key
            response = requests.get(f"{self.base_url}/key", headers=self.headers)
            if response.status_code == 200:
                data = response.json().get("data", {})
                usage = data.get("usage")
                if usage is not None:
                    return True, usage # Returns usage in USD
                return True, 0.0 # Valid key but no usage info
            else:
                return False, response.json().get("error", {}).get("message", "Unknown error")
        except requests.RequestException as e:
            return False, str(e)

    def get_models(self):
        """Retrieves the list of available models."""
        try:
            response = requests.get(f"{self.base_url}/models", headers=self.headers)
            if response.status_code == 200:
                models = response.json().get("data", [])
                return True, models
            else:
                return False, response.json().get("error", {}).get("message", "Unknown error")
        except requests.RequestException as e:
            return False, str(e)