import requests
import base64
import time
import os
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

import threading


# Use thread-local storage at module level to persist sessions across API instances
thread_local_storage = threading.local()

class GooglerAPI:
    def __init__(self, api_key=None):
        self.settings = settings_manager.get("googler", {})
        self.api_key = api_key or self.settings.get("api_key")
        self.base_url = "https://app.recrafter.fun/api/v3"

    def _get_session(self):
        if not hasattr(thread_local_storage, "session"):
            thread_local_storage.session = requests.Session()
        return thread_local_storage.session

    def _make_request(self, method, endpoint, **kwargs):
        if not self.api_key:
            return None, "not_configured"
        
        headers = {
            "X-API-Key": self.api_key,
            "Content-Type": "application/json",
            "Accept": "application/json"
        }
        if "headers" in kwargs:
            kwargs["headers"].update(headers)
        else:
            kwargs["headers"] = headers
        
        try:
            url = f"{self.base_url}/{endpoint}"
            session = self._get_session()
            response = session.request(method, url, **kwargs)
            # No special 429 handling for now, just log it.
            if response.status_code not in [200, 201]:
                 logger.log(f"API request to {endpoint} failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                 return None, "error"
            
            # For 200, we might get empty body on success (e.g. status checks)
            # so we handle json decoding carefully
            try:
                return response.json(), "connected"
            except requests.exceptions.JSONDecodeError:
                return {}, "connected" # Return empty dict for empty body

        except requests.exceptions.RequestException as e:
            logger.log(f"API request to {endpoint} failed: {e}", level=LogLevel.ERROR)
            return None, "error"

    def get_usage(self):
        """Fetches detailed account usage from v3 endpoint."""
        logger.log("Requesting Googler account usage (v3)...", level=LogLevel.INFO)
        # Construct full URL for v3 endpoint
        url = "https://app.recrafter.fun/api/v3/account/usage"
        
        headers = {
            "X-API-Key": self.api_key,
            "Accept": "application/json"
        }
        
        try:
            session = self._get_session()
            response = session.get(url, headers=headers)
            if response.status_code == 200:
                data = response.json()
                logger.log(f"Successfully retrieved Googler usage stats.", level=LogLevel.SUCCESS)
                return data
            else:
                logger.log(f"Googler usage request failed with status {response.status_code}: {response.text}", level=LogLevel.ERROR)
                return None
        except Exception as e:
            logger.log(f"Error fetching Googler usage: {e}", level=LogLevel.ERROR)
            return None

    def generate_image(self, prompt, aspect_ratio="IMAGE_ASPECT_RATIO_LANDSCAPE", seed=None, negative_prompt=None):
        logger.log(f"Requesting image generation from Googler for prompt: {prompt}", level=LogLevel.INFO)
        
        parameters = {
            "prompt": prompt,
            "aspect_ratio": aspect_ratio,
        }
        if seed:
            try:
                parameters["seed"] = int(seed)
            except (ValueError, TypeError):
                logger.log(f"Invalid seed value: {seed}. It will be ignored.", level=LogLevel.WARNING)

        if negative_prompt:
            parameters["negative_prompt"] = negative_prompt

        parameters["provider"] = "google_fx"
        data, status = self._make_request("post", "image/from-text", json=parameters)

        if status == "connected" and data and data.get("success"):
            result = data.get("result")
            if result and isinstance(result, str):
                logger.log(f"Successfully generated image for prompt: {prompt}", level=LogLevel.SUCCESS)
                return result
            else:
                logger.log(f"API call successful but got empty or invalid result for prompt: {prompt}. Result: {result}", level=LogLevel.WARNING)
                return None
        
        error_message = data.get("error") if data else "Unknown error"
        logger.log(f"Failed to generate image for prompt: {prompt}. Error: {error_message}", level=LogLevel.ERROR)
        return None

    def generate_video(self, image_path, prompt, aspect_ratio="IMAGE_ASPECT_RATIO_LANDSCAPE"):
        
        def encode_image(path):
            if not os.path.exists(path):
                logger.log(f"Image file not found for video generation: {path}", level=LogLevel.ERROR)
                return None
            with open(path, "rb") as image_file:
                encoded_string = base64.b64encode(image_file.read()).decode('utf-8')
                # Guess extension
                ext = os.path.splitext(path)[1].lower().replace('.', '')
                if ext not in ['jpeg', 'jpg', 'png']:
                    ext = 'jpeg' # Default
                return f"data:image/{ext};base64,{encoded_string}"

        image_data_uri = encode_image(image_path)
        if not image_data_uri:
            return None

        payload = {
            "provider": "google_fx",
            "prompt": prompt,
            "input_image": image_data_uri,
            "aspect_ratio": aspect_ratio
        }
        
        logger.log(f"Requesting video generation for image: {os.path.basename(image_path)}", level=LogLevel.INFO)
        
        start_data, start_status = self._make_request("post", "video/from-image-legacy", json=payload)
        
        if start_status != "connected" or not start_data or "operation_id" not in start_data:
            logger.log(f"Failed to start video generation for {os.path.basename(image_path)}. Response: {start_data}", level=LogLevel.ERROR)
            return None
            
        operation_id = start_data["operation_id"]
        logger.log(f"Video generation task created for {os.path.basename(image_path)}. Operation ID: {operation_id}", level=LogLevel.INFO)

        # Polling for result
        for i in range(60): # 5 minute timeout
            time.sleep(5)
            status_data, status_status = self._make_request("get", f"video/status/{operation_id}")

            if status_status != "connected":
                logger.log(f"Failed to get status for operation {operation_id}", level=LogLevel.WARNING)
                continue

            status = status_data.get("status")
            logger.log(f"Polling for {operation_id}: status is '{status}'", level=LogLevel.INFO)
            
            if status == "success":
                result = status_data.get("result")
                if result:
                    logger.log(f"Successfully generated video for {os.path.basename(image_path)}", level=LogLevel.SUCCESS)
                    return result
                else:
                    logger.log(f"Video generation status is 'success' but no result/output field found for {operation_id}", level=LogLevel.ERROR)
                    return None
            
            elif status == "error":
                logger.log(f"Video generation failed for {operation_id}. Details: {status_data}", level=LogLevel.ERROR)
                return None
            
            elif status not in ["pending", "processing"]:
                logger.log(f"Unknown status '{status}' for operation {operation_id}. Aborting.", level=LogLevel.ERROR)
                return None
        
        logger.log(f"Timeout waiting for video generation result for operation {operation_id}", level=LogLevel.ERROR)
        return None
