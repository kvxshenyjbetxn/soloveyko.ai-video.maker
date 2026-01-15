import os
import subprocess
import re
import datetime
import time
import platform
from api.assemblyai import assembly_ai_api
from utils.logger import logger, LogLevel

class SubtitleEngine:
    def __init__(self, exe_path=None, model_path=None):
        self.exe_path = exe_path
        self.model_path = model_path

    def generate_ass(self, audio_path, output_path, settings, language='en'):
        segments = self._get_segments(audio_path, settings, language)
        
        if not segments:
            raise Exception("No subtitles generated (segments list empty).")

        # For AssemblyAI, splitting is already handled by `chars_per_caption`
        engine_type = settings.get('whisper_type', 'amd')
        if engine_type != 'assemblyai':
            processed_segments = self._split_long_lines(segments, settings.get('max_words', 10))
        else:
            processed_segments = segments

        self._write_ass_file(processed_segments, output_path, settings)

    def transcribe_text(self, audio_path, settings, language='en'):
        """Generates a plain text transcription of the audio."""
        segments = self._get_segments(audio_path, settings, language)
        if not segments:
            return ""
        return " ".join([seg['text'] for seg in segments])

    def _get_segments(self, audio_path, settings, language='en'):
        engine_type = settings.get('whisper_type', 'amd')
        logger.log(f"SubtitleEngine: Generating segments using '{engine_type}' for {language}", LogLevel.DEBUG)
        
        segments = []

        # --- Handle 'auto' language detection for AMD (doesn't support it natively) ---
        if language == 'auto' and engine_type == 'amd':
            logger.log("AMD fork doesn't support 'auto'. Detecting language via standard whisper library...", LogLevel.INFO)
            try:
                import whisper
                import torch
                
                # Use a fast model for detection.
                model_type = "base"
                if self.model_path and "medium" in self.model_path.lower():
                    model_type = "medium"
                elif self.model_path and "small" in self.model_path.lower():
                    model_type = "small"
                
                model = whisper.load_model(model_type, device="cpu") 
                audio = whisper.load_audio(audio_path)
                audio = whisper.pad_or_trim(audio)
                mel = whisper.log_mel_spectrogram(audio).to(model.device)
                
                _, probs = model.detect_language(mel)
                language = max(probs, key=probs.get)
                logger.log(f"Detected language for AMD: {language}", LogLevel.SUCCESS)
            except Exception as e:
                logger.log(f"Language detection failed: {e}. Defaulting to 'en'", LogLevel.WARNING)
                language = 'en'

        # --- Main Engine Routing ---
        if engine_type == 'assemblyai':
            logger.log(f"Running AssemblyAI Transcription: Lang={language}", LogLevel.INFO)
            transcript = assembly_ai_api.transcribe(audio_path, lang=language)
            
            if transcript:
                if not transcript.text:
                    logger.log("AssemblyAI transcript is empty.", LogLevel.WARNING)
                
                srt_content = assembly_ai_api.get_srt(transcript, chars_per_caption=settings.get('max_words', 10) * 5)
                if srt_content:
                    segments = self._parse_srt_content(srt_content)
                else:
                    logger.log("AssemblyAI get_srt returned empty content.", LogLevel.WARNING)
            else:
                logger.log("AssemblyAI transcription returned a None object.", LogLevel.WARNING)

        elif engine_type == 'standard':
            # --- Standard Python Whisper ---
            logger.log(f"Running Standard Whisper (Python): Model={self.model_path}, Lang={language}", LogLevel.INFO)
            try:
                import whisper
            except ImportError:
                raise ImportError("Library 'openai-whisper' not installed. Run: pip install openai-whisper")

            # Check if model exists and log if it needs to be downloaded
            cache_path = os.path.join(os.path.expanduser("~"), ".cache", "whisper")
            model_file = os.path.join(cache_path, f"{self.model_path}.pt")
            if not os.path.exists(model_file):
                from utils.translator import translator
                logger.log(translator.translate("whisper_model_download_info", "Whisper model '{model_name}' not found. Starting one-time download. This may take some time...").format(model_name=self.model_path), LogLevel.INFO)

            model = whisper.load_model(self.model_path)
            
            # Pass language=None for auto-detection in standard whisper
            whisper_lang = language if language != 'auto' else None
            result = model.transcribe(audio_path, language=whisper_lang)
            
            for s in result['segments']:
                segments.append({
                    'start': s['start'],
                    'end': s['end'],
                    'text': s['text'].strip()
                })

        else: # amd
            # --- AMD / Fork Whisper (EXE) ---
            if not self.exe_path or not os.path.exists(self.exe_path):
                raise FileNotFoundError(f"Whisper EXE not found: {self.exe_path}")
            if not self.model_path or not os.path.exists(self.model_path):
                raise FileNotFoundError(f"Model file not found: {self.model_path}")

            srt_path = os.path.splitext(audio_path)[0] + ".srt"
            if os.path.exists(srt_path):
                os.remove(srt_path)

            cmd = [
                self.exe_path,
                "-m", self.model_path,
                "-f", audio_path,
                "-osrt"       
            ]

            # Add language flag, use the detected language
            cmd.extend(["-l", language])

            startupinfo = None
            if platform.system() == "Windows":
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

            logger.log(f"Running Whisper CLI (AMD): {' '.join(cmd)}", LogLevel.INFO)
            process = subprocess.run(cmd, startupinfo=startupinfo, capture_output=True, text=True)

            possible_files = [
                audio_path + ".srt",
                os.path.splitext(audio_path)[0] + ".srt"
            ]
            
            found_srt = None
            time.sleep(0.5)
            for p in possible_files:
                if os.path.exists(p):
                    found_srt = p
                    break
            
            if not found_srt:
                 # Check stderr, might be helpful
                 if process.stderr:
                     logger.log(f"Whisper Error Output: {process.stderr}", LogLevel.ERROR)
                 raise Exception(f"SRT file not generated by Whisper CLI.")

            try:
                segments = self._parse_srt(found_srt)
            finally:
                if os.path.exists(found_srt):
                    os.remove(found_srt)

        return segments

    def _parse_srt(self, filename):
        encodings = ['utf-8', 'utf-8-sig', 'cp1252', 'cp1251', 'latin-1']
        content = None
        
        for encoding in encodings:
            try:
                with open(filename, "r", encoding=encoding) as f:
                    content = f.read()
                logger.log(f"Successfully read SRT with encoding: {encoding}", LogLevel.DEBUG)
                break
            except UnicodeDecodeError:
                continue
        
        if content is None:
            logger.log("Failed to decode SRT with standard encodings. Fallback to 'utf-8' with replacement.", LogLevel.WARNING)
            with open(filename, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
                
        return self._parse_srt_content(content)

    def _parse_srt_content(self, content):
        pattern = re.compile(r'(\d+)\n(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})\n(.*?)(?=\n\n|\Z)', re.DOTALL)
        segments = []
        for match in pattern.finditer(content):
            start_str = match.group(2).replace(',', '.')
            end_str = match.group(3).replace(',', '.')
            text = match.group(4).replace('\n', ' ')
            segments.append({
                'start': self._time_to_seconds(start_str),
                'end': self._time_to_seconds(end_str),
                'text': text.strip()
            })
        return segments

    def _time_to_seconds(self, time_str):
        parts = time_str.split(':')
        h = int(parts[0])
        m = int(parts[1])
        s = float(parts[2])
        return h * 3600 + m * 60 + s


    def _split_long_lines(self, segments, max_words):
        """
        Iteratively split segments until no segment has more than max_words.
        Distributes time proportionally to the number of words.
        """
        def split_recursive(seg):
            words = seg['text'].split()
            if len(words) <= max_words:
                return [seg]
            
            # Split off the first max_words
            part1_words = words[:max_words]
            part2_words = words[max_words:]
            
            total_words = len(words)
            duration = seg['end'] - seg['start']
            
            # Duration proportional to words
            part1_dur = (len(part1_words) / total_words) * duration
            split_time = seg['start'] + part1_dur
            
            part1 = {'start': seg['start'], 'end': split_time, 'text': " ".join(part1_words)}
            part2 = {'start': split_time, 'end': seg['end'], 'text': " ".join(part2_words)}
            
            # Recursively split the second part if it's still too long
            return [part1] + split_recursive(part2)

        new_segments = []
        for seg in segments:
            new_segments.extend(split_recursive(seg))
            
        return new_segments

    def _write_ass_file(self, segments, filename, s):
        r, g, b = s.get('color', (255, 255, 255))
        ass_color = f"&H00{b:02X}{g:02X}{r:02X}"
        fade_in = s.get('fade_in', 0)
        fade_out = s.get('fade_out', 0)
        margin_v = s.get('margin_v', 50)
        font = s.get('font', 'Arial')
        size = s.get('fontsize', 60)

        # Reverted to original fixed resolution and settings use
        res_x, res_y = 1920, 1080

        header = f"""[Script Info]
ScriptType: v4.00+
PlayResX: {res_x}
PlayResY: {res_y}

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, OutlineColour, BackColour, Bold, Italic, Alignment, BorderStyle, Outline, Shadow, MarginL, MarginR, MarginV, Encoding
Style: Default,{font},{size},{ass_color},&H00000000,&H80000000,-1,0,2,1,2,0,10,10,{margin_v},1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""
        def format_time(seconds):
            td = datetime.timedelta(seconds=seconds)
            hours, remainder = divmod(td.seconds, 3600)
            minutes, seconds = divmod(remainder, 60)
            centiseconds = td.microseconds // 10000
            return f"{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}"

        with open(filename, "w", encoding="utf-8") as f:
            f.write(header)
            for seg in segments:
                start = format_time(seg['start'])
                end = format_time(seg['end'])
                text = seg['text'].strip()
                anim_tag = f"{{\\fad({fade_in},{fade_out})}}" if (fade_in > 0 or fade_out > 0) else ""
                f.write(f"Dialogue: 0,{start},{end},Default,,0,0,0,,{anim_tag}{text}\n")