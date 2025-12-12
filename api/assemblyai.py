import assemblyai as aai
from utils.settings import settings_manager
from utils.logger import logger, LogLevel

class AssemblyAIAPI:
    def __init__(self):
        self.api_key = settings_manager.get('assemblyai_api_key')
        if self.api_key:
            aai.settings.api_key = self.api_key

    def transcribe(self, audio_path, lang='auto'):
        if not self.api_key:
            logger.log("AssemblyAI API key is not set.", LogLevel.ERROR)
            return None

        try:
            logger.log(f"Starting transcription for {audio_path} with AssemblyAI", LogLevel.INFO)
            
            if lang == 'auto':
                config = aai.TranscriptionConfig(language_detection=True)
            else:
                config = aai.TranscriptionConfig(language_code=lang)
            
            transcriber = aai.Transcriber(config=config)
            transcript = transcriber.transcribe(audio_path)

            if transcript.status == aai.TranscriptStatus.error:
                logger.log(f"AssemblyAI transcription failed: {transcript.error}", LogLevel.ERROR)
                return None
            
            logger.log(f"AssemblyAI transcription successful for {audio_path}", LogLevel.INFO)
            return transcript

        except Exception as e:
            logger.log(f"An error occurred during AssemblyAI transcription: {e}", LogLevel.ERROR)
            return None

    def get_srt(self, transcript, chars_per_caption=40):
        if not transcript:
            return None
        return transcript.export_subtitles_srt(chars_per_caption=chars_per_caption)

assembly_ai_api = AssemblyAIAPI()
