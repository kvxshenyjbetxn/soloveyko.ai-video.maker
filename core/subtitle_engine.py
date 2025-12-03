import os
import subprocess
import re
import datetime
import time

class SubtitleEngine:
    def __init__(self, exe_path, model_path):
        self.exe_path = exe_path
        self.model_path = model_path

    def generate_ass(self, audio_path, output_path, settings, language='en'):
        # Перевірка вхідних файлів
        if not os.path.exists(self.exe_path):
            raise FileNotFoundError(f"Не знайдено Whisper EXE: {self.exe_path}")
        if not os.path.exists(self.model_path):
            raise FileNotFoundError(f"Не знайдено файл моделі: {self.model_path}")

        # Визначаємо можливі імена файлів
        path_1 = audio_path + ".srt"
        path_2 = os.path.splitext(audio_path)[0] + ".srt"
        filename = os.path.basename(audio_path)
        path_3 = os.path.abspath(filename + ".srt")
        path_4 = os.path.abspath(os.path.splitext(filename)[0] + ".srt")

        possible_files = [path_1, path_2, path_3, path_4]

        # Видаляємо старі файли
        for p in possible_files:
            if os.path.exists(p):
                try:
                    os.remove(p)
                except:
                    pass

        # --- КРОК 1: Запуск Whisper ---
        cmd = [
            self.exe_path,
            "-m", self.model_path,
            "-f", audio_path,
            "-l", language,   # Використовуємо переданий код мови (напр. 'uk', 'en')
            "-osrt"       
        ]

        startupinfo = subprocess.STARTUPINFO()
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

        print(f"🚀 Running Whisper CLI: {' '.join(cmd)}")
        
        process = subprocess.run(cmd, startupinfo=startupinfo, capture_output=True, text=True)

        if process.stdout:
            print(f"📄 Whisper Stdout: {process.stdout[:200]}...") 
        if process.stderr:
            print(f"⚠️ Whisper Stderr: {process.stderr[:200]}...")

        # --- КРОК 2: Пошук SRT ---
        time.sleep(1.0)
        
        found_srt = None
        for p in possible_files:
            if os.path.exists(p):
                found_srt = p
                print(f"✅ Знайдено субтитри: {found_srt}")
                break
        
        if not found_srt:
            err_msg = f"Whisper завершив роботу, але SRT файл не знайдено.\nПеревірені шляхи:\n" + "\n".join(possible_files)
            if process.stderr:
                err_msg += f"\n\nПомилка Whisper CLI:\n{process.stderr}"
            raise Exception(err_msg)

        # --- КРОК 3: Парсинг і конвертація ---
        try:
            segments = self._parse_srt(found_srt)
        finally:
            if os.path.exists(found_srt):
                os.remove(found_srt)

        # --- КРОК 4: Генерація ASS ---
        processed_segments = self._split_long_lines(segments, settings.get('max_words', 10))
        self._write_ass_file(processed_segments, output_path, settings)

    def _parse_srt(self, filename):
        with open(filename, "r", encoding="utf-8") as f:
            content = f.read()
        
        if not content.strip():
            raise Exception("Файл субтитрів порожній! Можливо, Whisper не розпізнав голос.")

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