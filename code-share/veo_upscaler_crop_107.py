#!/usr/bin/env python3
import json
import os
import queue
import shutil
import subprocess
import threading
import time
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk


APP_NAME = "Veo Batch Upscaler"
VIDEO_EXTS = {".mp4", ".mov", ".m4v", ".mkv", ".webm"}
TARGET_W = 1920
TARGET_H = 1080
CROP_PRESETS = {
    "fit_16_9": {
        "label": "Без кропу 16:9 (1920x1080)",
        "suffix": "1080p",
        "zoom": 1.0,
        "anchor": "center",
    },
    "crop_103": {
        "label": "Кроп 103% (верх + праворуч)",
        "suffix": "crop103",
        "zoom": 1.03,
        "anchor": "top_right",
    },
    "crop_107": {
        "label": "Кроп 107% (верх + праворуч)",
        "suffix": "crop107",
        "zoom": 1.07,
        "anchor": "top_right",
    },
}


def bundled_path(name):
    bundle_resources = Path(__file__).resolve().parent
    return bundle_resources / name


def find_tool(name):
    local = bundled_path(name)
    if local.exists():
        return str(local)
    return shutil.which(name)


def run_capture(args):
    return subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)


def probe_video(path):
    ffprobe = find_tool("ffprobe")
    if not ffprobe:
        raise RuntimeError("ffprobe не знайдено. Встанови FFmpeg через Homebrew: brew install ffmpeg")

    args = [
        ffprobe,
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height,r_frame_rate,avg_frame_rate,duration,nb_frames:format=duration",
        "-of",
        "json",
        str(path),
    ]
    result = run_capture(args)
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "Не вдалося прочитати відео.")
    data = json.loads(result.stdout)
    streams = data.get("streams", [])
    if not streams:
        raise RuntimeError("У файлі не знайдено відеопотік.")
    stream = streams[0]
    format_duration = float(data.get("format", {}).get("duration") or 0)
    return {
        "width": int(stream.get("width", 0)),
        "height": int(stream.get("height", 0)),
        "fps": stream.get("r_frame_rate") or stream.get("avg_frame_rate") or "30/1",
        "avg_fps": stream.get("avg_frame_rate") or "30/1",
        "duration": float(stream.get("duration") or 0) or format_duration,
        "nb_frames": int(stream.get("nb_frames") or 0),
    }


def rate_to_float(rate):
    if not rate or rate == "0/0":
        return 30.0
    if "/" in rate:
        top, bottom = rate.split("/", 1)
        bottom = float(bottom)
        return float(top) / bottom if bottom else 30.0
    return float(rate)


def stable_fps(info):
    duration = info.get("duration") or 0
    frames = info.get("nb_frames") or 0
    if duration > 0 and frames > 0:
        fps = frames / duration
    else:
        fps = rate_to_float(info.get("fps") or info.get("avg_fps"))
    if fps < 1 or fps > 120:
        fps = 30.0
    return round(fps, 3)


class UpscalerApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title(APP_NAME)
        self.geometry("860x620")
        self.minsize(760, 540)

        self.input_dir = tk.StringVar()
        self.output_dir = tk.StringVar()
        self.crop_format = tk.StringVar(value=CROP_PRESETS["fit_16_9"]["label"])
        self.quality = tk.StringVar(value="balanced")
        self.skip_done = tk.BooleanVar(value=True)
        self.keep_audio = tk.BooleanVar(value=True)
        self.status = tk.StringVar(value="Готово")
        self.worker = None
        self.stop_requested = threading.Event()
        self.log_queue = queue.Queue()

        self.ffmpeg = find_tool("ffmpeg")
        self.ffprobe = find_tool("ffprobe")

        self.build_ui()
        self.after(120, self.flush_log)

    def build_ui(self):
        root = ttk.Frame(self, padding=18)
        root.pack(fill="both", expand=True)

        title = ttk.Label(root, text="Veo Batch Upscaler", font=("Helvetica Neue", 24, "bold"))
        title.pack(anchor="w")
        subtitle = ttk.Label(
            root,
            text="Пакетний апскейл Veo 720p відео до 1920×1080 з якісним ресайзом і різкістю.",
        )
        subtitle.pack(anchor="w", pady=(2, 18))

        paths = ttk.Frame(root)
        paths.pack(fill="x")
        self.path_row(paths, "Папка з відео", self.input_dir, self.choose_input).pack(fill="x", pady=4)
        self.path_row(paths, "Куди зберігати", self.output_dir, self.choose_output).pack(fill="x", pady=4)

        opts = ttk.LabelFrame(root, text="Налаштування", padding=12)
        opts.pack(fill="x", pady=(16, 8))

        ttk.Label(opts, text="Формат").grid(row=0, column=0, sticky="w")
        crop_select = ttk.Combobox(
            opts,
            textvariable=self.crop_format,
            values=[preset["label"] for preset in CROP_PRESETS.values()],
            state="readonly",
            width=24,
        )
        crop_select.grid(row=0, column=1, sticky="w", padx=(12, 0))
        crop_select.current(0)

        ttk.Label(opts, text="Якість").grid(row=1, column=0, sticky="w", pady=(10, 0))
        quality = ttk.Frame(opts)
        quality.grid(row=1, column=1, sticky="w", padx=(12, 0), pady=(10, 0))
        ttk.Radiobutton(quality, text="Швидше", variable=self.quality, value="fast").pack(side="left")
        ttk.Radiobutton(quality, text="Баланс", variable=self.quality, value="balanced").pack(side="left", padx=(14, 0))
        ttk.Radiobutton(quality, text="Максимум", variable=self.quality, value="max").pack(side="left")

        checks = ttk.Frame(opts)
        checks.grid(row=2, column=1, sticky="w", padx=(12, 0), pady=(10, 0))
        ttk.Checkbutton(checks, text="Пропускати вже готові файли", variable=self.skip_done).pack(side="left")
        ttk.Checkbutton(checks, text="Зберігати аудіо", variable=self.keep_audio).pack(side="left", padx=(14, 0))
        opts.columnconfigure(1, weight=1)

        controls = ttk.Frame(root)
        controls.pack(fill="x", pady=(10, 8))
        self.start_btn = ttk.Button(controls, text="Старт", command=self.start)
        self.start_btn.pack(side="left")
        self.stop_btn = ttk.Button(controls, text="Стоп", command=self.stop, state="disabled")
        self.stop_btn.pack(side="left", padx=(8, 0))
        ttk.Label(controls, textvariable=self.status).pack(side="right")

        self.progress = ttk.Progressbar(root, mode="determinate")
        self.progress.pack(fill="x", pady=(0, 12))

        log_frame = ttk.LabelFrame(root, text="Журнал", padding=8)
        log_frame.pack(fill="both", expand=True)
        self.log = tk.Text(log_frame, wrap="word", height=14, relief="flat")
        self.log.pack(side="left", fill="both", expand=True)
        scroll = ttk.Scrollbar(log_frame, command=self.log.yview)
        scroll.pack(side="right", fill="y")
        self.log.configure(yscrollcommand=scroll.set)

        tool_status = []
        tool_status.append("FFmpeg: OK" if self.ffmpeg and self.ffprobe else "FFmpeg: не знайдено")
        ttk.Label(root, text=" · ".join(tool_status)).pack(anchor="w", pady=(8, 0))

    def path_row(self, parent, label, var, command):
        row = ttk.Frame(parent)
        ttk.Label(row, text=label, width=15).pack(side="left")
        ttk.Entry(row, textvariable=var).pack(side="left", fill="x", expand=True, padx=(8, 8))
        ttk.Button(row, text="Вибрати", command=command).pack(side="right")
        return row

    def choose_input(self):
        folder = filedialog.askdirectory(title="Вибери папку з відео")
        if folder:
            self.input_dir.set(folder)
            if not self.output_dir.get():
                self.output_dir.set(str(Path(folder) / "upscaled_1080p"))

    def choose_output(self):
        folder = filedialog.askdirectory(title="Вибери папку для результатів")
        if folder:
            self.output_dir.set(folder)

    def selected_preset(self):
        selected = self.crop_format.get()
        if selected in CROP_PRESETS:
            return CROP_PRESETS[selected]
        for preset in CROP_PRESETS.values():
            if preset["label"] == selected:
                return preset
        return CROP_PRESETS["fit_16_9"]

    def start(self):
        if self.worker and self.worker.is_alive():
            return
        if not self.ffmpeg or not self.ffprobe:
            messagebox.showerror(APP_NAME, "FFmpeg не знайдено. Встанови його: brew install ffmpeg")
            return
        src = Path(self.input_dir.get()).expanduser()
        dst = Path(self.output_dir.get()).expanduser()
        if not src.exists():
            messagebox.showerror(APP_NAME, "Вибери існуючу папку з відео.")
            return
        dst.mkdir(parents=True, exist_ok=True)

        files = [p for p in sorted(src.iterdir()) if p.suffix.lower() in VIDEO_EXTS]
        if not files:
            messagebox.showinfo(APP_NAME, "У вибраній папці немає mp4/mov/mkv/webm відео.")
            return

        self.stop_requested.clear()
        self.progress.configure(maximum=len(files), value=0)
        self.start_btn.configure(state="disabled")
        self.stop_btn.configure(state="normal")
        self.status.set(f"У черзі: {len(files)}")
        self.log_insert(f"Знайдено {len(files)} відео.\n")
        self.worker = threading.Thread(target=self.process_files, args=(files, dst), daemon=True)
        self.worker.start()

    def stop(self):
        self.stop_requested.set()
        self.status.set("Зупиняю після поточної дії...")

    def process_files(self, files, output_dir):
        completed = 0
        try:
            preset = self.selected_preset()
            for index, path in enumerate(files, 1):
                if self.stop_requested.is_set():
                    self.log_queue.put("Зупинено користувачем.\n")
                    break
                out = output_dir / f"{path.stem}_{preset['suffix']}.mp4"
                if self.skip_done.get() and out.exists() and out.stat().st_size > 1024:
                    self.log_queue.put(f"[{index}/{len(files)}] Пропущено: {path.name}\n")
                    completed += 1
                    self.log_queue.put(("PROGRESS", completed))
                    continue
                self.log_queue.put(f"[{index}/{len(files)}] Обробляю: {path.name}\n")
                try:
                    info = probe_video(path)
                    self.ffmpeg_upscale(path, out, info, preset)
                    completed += 1
                    self.log_queue.put(f"Готово: {out.name}\n")
                except Exception as exc:
                    self.log_queue.put(f"Помилка: {path.name}: {exc}\n")
                self.log_queue.put(("PROGRESS", completed))
        finally:
            self.log_queue.put(("DONE", completed, len(files)))

    def ffmpeg_upscale(self, src, dst, info=None, preset=None):
        info = info or probe_video(src)
        preset = preset or self.selected_preset()
        fps = stable_fps(info)
        vf = self.ffmpeg_filter(fps, preset)
        preset = {"fast": "veryfast", "balanced": "medium", "max": "slow"}[self.quality.get()]
        crf = {"fast": "20", "balanced": "18", "max": "16"}[self.quality.get()]
        args = [
            self.ffmpeg,
            "-y",
            "-hide_banner",
            "-fflags",
            "+genpts",
            "-i",
            str(src),
            "-vf",
            vf,
            "-r",
            str(fps),
            "-fps_mode",
            "cfr",
            "-c:v",
            "libx264",
            "-preset",
            preset,
            "-crf",
            crf,
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
        ]
        if self.keep_audio.get():
            args += ["-map", "0:v:0", "-map", "0:a?", "-c:a", "aac", "-b:a", "192k"]
        else:
            args += ["-an"]
        args.append(str(dst))
        self.run_process(args)

    def ffmpeg_filter(self, fps=None, preset=None):
        fps = fps or 30
        preset = preset or self.selected_preset()
        sharpen = {
            "fast": "unsharp=5:5:0.55:3:3:0.25",
            "balanced": "hqdn3d=1.2:1.2:4:4,unsharp=5:5:0.75:5:5:0.35",
            "max": "hqdn3d=1.5:1.5:5:5,unsharp=7:7:0.85:5:5:0.4",
        }[self.quality.get()]
        if preset["anchor"] == "top_right":
            scaled_w = round(TARGET_W * preset["zoom"])
            scaled_h = round(TARGET_H * preset["zoom"])
            fit = (
                f"scale={scaled_w}:{scaled_h}:flags=lanczos:force_original_aspect_ratio=increase,"
                f"crop={TARGET_W}:{TARGET_H}:iw-{TARGET_W}:0"
            )
        else:
            fit = (
                f"scale={TARGET_W}:{TARGET_H}:flags=lanczos:force_original_aspect_ratio=decrease,"
                f"pad={TARGET_W}:{TARGET_H}:(ow-iw)/2:(oh-ih)/2"
            )
        return (
            f"setpts=N/({fps}*TB),"
            f"{fit},"
            f"{sharpen}"
        )

    def run_process(self, args):
        process = subprocess.Popen(
            args,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        last_line = ""
        for line in process.stdout:
            last_line = line.strip()
            if "frame=" in line or "time=" in line:
                self.log_queue.put(".")
            if self.stop_requested.is_set():
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                raise RuntimeError("обробку зупинено")
        code = process.wait()
        self.log_queue.put("\n")
        if code != 0:
            raise RuntimeError(last_line or f"процес завершився з кодом {code}")

    def log_insert(self, text):
        self.log.insert("end", text)
        self.log.see("end")

    def flush_log(self):
        try:
            while True:
                item = self.log_queue.get_nowait()
                if isinstance(item, tuple) and item[0] == "PROGRESS":
                    self.progress.configure(value=item[1])
                    self.status.set(f"Готово файлів: {item[1]}")
                elif isinstance(item, tuple) and item[0] == "DONE":
                    done, total = item[1], item[2]
                    self.start_btn.configure(state="normal")
                    self.stop_btn.configure(state="disabled")
                    self.status.set(f"Завершено: {done}/{total}")
                    self.log_insert(f"\nЗавершено: {done}/{total}\n")
                else:
                    self.log_insert(str(item))
        except queue.Empty:
            pass
        self.after(120, self.flush_log)


if __name__ == "__main__":
    app = UpscalerApp()
    app.mainloop()
