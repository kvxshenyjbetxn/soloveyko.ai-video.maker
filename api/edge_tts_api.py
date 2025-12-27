import asyncio
import os
import edge_tts
from utils.logger import logger, LogLevel

class EdgeTTSAPI:
    def __init__(self):
        pass

    async def _generate_audio_async(self, text, voice, rate, pitch, output_file):
        """
        Generate audio using edge-tts.
        rate: str, e.g. "+0%", "-10%"
        pitch: str, e.g. "+0Hz", "+10Hz"
        """
        communicate = edge_tts.Communicate(text, voice, rate=rate, pitch=pitch)
        await communicate.save(output_file)

    def generate_audio(self, text, voice, rate, pitch, output_path):
        try:
            # properly format rate and pitch if they are just numbers or raw strings
            # Assuming rate is passed as int (percentage) or string with %
            if isinstance(rate, (int, float)):
                rate_str = f"{int(rate):+d}%"
            elif isinstance(rate, str) and not rate.endswith('%') and (rate.startswith('+') or rate.startswith('-')):
                 rate_str = f"{rate}%"
            elif isinstance(rate, str) and not rate:
                rate_str = "+0%"
            else:
                 rate_str = rate

            # Assuming pitch is passed as int (Hz) or string
            if isinstance(pitch, (int, float)):
                pitch_str = f"{int(pitch):+d}Hz"
            elif isinstance(pitch, str) and not pitch:
                pitch_str = "+0Hz"
            else:
                pitch_str = pitch

            logger.log(f"EdgeTTS generating with voice={voice}, rate={rate_str}, pitch={pitch_str}", level=LogLevel.INFO)
            asyncio.run(self._generate_audio_async(text, voice, rate_str, pitch_str, output_path))
            return True, "success"
        except Exception as e:
            logger.log(f"EdgeTTS generation failed: {e}", level=LogLevel.ERROR)
            return False, str(e)

    async def _get_voices_async(self):
        return await edge_tts.list_voices()

    def get_voices(self):
        try:
            voices = asyncio.run(self._get_voices_async())
            return voices
        except Exception as e:
            logger.log(f"Failed to list EdgeTTS voices: {e}", level=LogLevel.ERROR)
            return []
