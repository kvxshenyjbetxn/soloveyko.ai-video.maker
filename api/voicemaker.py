import requests
import re
import os
import time
import concurrent.futures
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

    def _split_text(self, text, limit):
        """Splits text into chunks respecting punctuation and character limit."""
        if len(text) <= limit:
            return [text]
        
        chunks = []
        # Split by sentence endings (. ! ?), keeping the delimiter
        # The lookbehind (?<=[.!?]) splits *after* the delimiter
        # But lookbehind requires fixed width, so we use a simpler regex and reconstruct
        # Or simply split by space and accumulate. 
        # Better: regex to find sentence boundaries.
        
        # Try to split by sentence delimiters
        sentences = re.split(r'(?<=[.!?])\s+', text)
        
        current_chunk = ""
        
        for sentence in sentences:
            if len(current_chunk) + len(sentence) <= limit:
                current_chunk += sentence + " "
            else:
                # If current chunk is not empty, add it to chunks
                if current_chunk:
                    chunks.append(current_chunk.strip())
                    current_chunk = ""
                
                # If the sentence itself is longer than limit (rare, but possible), split by comma
                if len(sentence) > limit:
                    sub_parts = re.split(r'(?<=,)\s+', sentence)
                    for part in sub_parts:
                        if len(current_chunk) + len(part) <= limit:
                            current_chunk += part + " "
                        else:
                            if current_chunk:
                                chunks.append(current_chunk.strip())
                            current_chunk = part + " "
                else:
                    current_chunk = sentence + " "
        
        if current_chunk:
            chunks.append(current_chunk.strip())
            
        return chunks

    def _generate_chunk(self, text, voice_id, language_code):
        """Internal method to generate audio for a single chunk."""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        payload = {
            "Engine": "neural",
            "VoiceId": voice_id,
            "LanguageCode": language_code,
            "Text": text,
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
                    audio_url = data.get("path")
                    if audio_url:
                        audio_response = requests.get(audio_url)
                        if audio_response.status_code == 200:
                             return audio_response.content, None
                        else:
                             return None, f"Download failed: {audio_response.status_code}"
                    else:
                        return None, "No audio path in response"
                else:
                    return None, f"API error: {data.get('message')}"
            elif response.status_code == 401:
                return None, "Unauthorized"
            else:
                return None, f"HTTP error: {response.status_code}"

        except requests.exceptions.RequestException as e:
            return None, str(e)

    def generate_audio(self, text, voice_id, language_code="en-US", temp_dir=None):
        if not self.api_key:
            return None, "not_configured"

        limit = settings_manager.get("voicemaker_char_limit", 3000)
        chunks = self._split_text(text, int(limit))
        
        logger.log(f"Voicemaker: Text length {len(text)}. Split into {len(chunks)} chunks (Limit: {limit}).", level=LogLevel.INFO)

        if len(chunks) == 1:
            # Simple case: just one chunk
            content, error = self._generate_chunk(chunks[0], voice_id, language_code)
            if content:
                return content, "success"
            else:
                logger.log(f"Voicemaker error: {error}", level=LogLevel.ERROR)
                return None, error

        # Multiple chunks: Parallel execution
        combined_audio = b""
        temp_audio_folder = None
        
        if temp_dir:
            temp_audio_folder = os.path.join(temp_dir, "temp_audio")
            os.makedirs(temp_audio_folder, exist_ok=True)

        results = [None] * len(chunks)
        
        # Increased max_workers to allow more parallel processing while throttling submission
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            future_to_index = {}
            for i, chunk in enumerate(chunks):
                future = executor.submit(self._generate_chunk, chunk, voice_id, language_code)
                future_to_index[future] = i
                # Small delay to avoid burst rate limits
                time.sleep(0.5)
            
            for future in concurrent.futures.as_completed(future_to_index):
                index = future_to_index[future]
                try:
                    content, error = future.result()
                    if content:
                        results[index] = content
                        
                        # Save temp chunk if folder exists
                        if temp_audio_folder:
                            chunk_filename = f"chunk_{index:03d}.mp3"
                            chunk_path = os.path.join(temp_audio_folder, chunk_filename)
                            with open(chunk_path, "wb") as f:
                                f.write(content)
                            logger.log(f"Voicemaker: Saved chunk {index+1}/{len(chunks)} to {chunk_path}", level=LogLevel.INFO)
                    else:
                        logger.log(f"Voicemaker: Failed to generate chunk {index}: {error}", level=LogLevel.ERROR)
                        return None, f"Chunk {index} failed: {error}"
                except Exception as e:
                    logger.log(f"Voicemaker: Exception in chunk {index}: {e}", level=LogLevel.ERROR)
                    return None, f"Exception in chunk {index}"

        # Concatenate in order
        for content in results:
            if content:
                combined_audio += content
            else:
                return None, "Missing chunk data"
                
        return combined_audio, "success"