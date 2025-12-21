import os
import subprocess
import re
import datetime
import time
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
            result = model.transcribe(audio_path, language=language)
            
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
                "-l", language,
                "-osrt"       
            ]

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
        with open(filename, "r", encoding="utf-8") as f:
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
        new_segments = []
        for seg in segments:
            words = seg['text'].split()
            if len(words) <= max_words:
                new_segments.append(seg)
                continue
            
            mid = len(words) // 2
            part1_words = words[:mid]
            part2_words = words[mid:]
            
            duration = seg['end'] - seg['start']
            split_time = seg['start'] + (duration / 2)
            
            new_segments.append({'start': seg['start'], 'end': split_time, 'text': " ".join(part1_words)})
            new_segments.append({'start': split_time, 'end': seg['end'], 'text': " ".join(part2_words)})
            
        return new_segments

    def _write_ass_file(self, segments, filename, s):
        r, g, b = s.get('color', (255, 255, 255))
        ass_color = f"&H00{b:02X}{g:02X}{r:02X}"
        fade_in = s.get('fade_in', 0)
        fade_out = s.get('fade_out', 0)
        margin_v = s.get('margin_v', 50)
        font = s.get('font', 'Arial')
        size = s.get('fontsize', 60)

        header = f"""[Script Info]
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

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