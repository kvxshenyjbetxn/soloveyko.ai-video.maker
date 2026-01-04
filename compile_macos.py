#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import subprocess
import shutil

def compile_project():
    # --- Set CWD to script's directory ---
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    # ------------------------------------

    # Назва вихідного файлу (синхронізовано з Windows)
    app_name = "Soloveyko.AI-Video.Maker"
    
    # Перевірка наявності PyInstaller та залежностей
    try:
        subprocess.run(["pyinstaller", "--version"], check=True, stdout=subprocess.DEVNULL)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("PyInstaller not found. Installing...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller"])

    # Встановлення залежностей з requirements.txt
    if os.path.exists("requirements.txt"):
        print("Installing dependencies from requirements.txt...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"])
    
    # Приховуємо попередження (SyntaxWarning) від сторонніх бібліотек
    os.environ["PYTHONWARNINGS"] = "ignore"

    # Очистка попередніх збірок
    if os.path.exists("build"):
        shutil.rmtree("build")
    
    # На Mac створюється .app bundle (папка)
    target_app = os.path.join("dist", f"{app_name}.app")
    if os.path.exists(target_app):
        try:
            shutil.rmtree(target_app)
        except OSError:
            print(f"Warning: Could not remove {target_app}. It might be in use.")

    if os.path.exists(f"{app_name}.spec"):
        os.remove(f"{app_name}.spec")

    # Знаходимо шлях до асетів Whisper
    import whisper
    whisper_path = os.path.dirname(whisper.__file__)
    whisper_assets = os.path.join(whisper_path, 'assets')
    
    # Команда для PyInstaller
    # --windowed: створює .app bundle для macOS
    # --icon: .icns для Mac
    # --add-data: розділювач ':' для Mac
    
    cmd = [
        "pyinstaller",
        "--noconfirm",
        "--clean",
        "--windowed",
        "--icon=assets/icon.icns",
        
        # Assets (формат source:dest)
        "--add-data", "assets/ffmpeg:assets",
        "--add-data", "assets/ffprobe:assets",
        "--add-data", "assets/icon.png:assets",
        "--add-data", "assets/icon.icns:assets",
        "--add-data", "assets/gemini_tts_voices.json:assets",
        "--add-data", "assets/voicemaker_voices.json:assets",
        "--add-data", "assets/translations:assets/translations",
        "--add-data", "assets/styles:assets/styles",
        
        # UI та Whisper
        "--add-data", "gui/qt_material:gui/qt_material",
        "--add-data", f"{whisper_assets}:whisper/assets",
        
        # Приховані імпорти та виключення
        "--hidden-import", "whisper",
        "--exclude-module", "PyQt6",
        "--exclude-module", "PyQt5",
        "--exclude-module", "tkinter",
        
        "--distpath", "dist",
        "--workpath", "build",
        "--name", app_name,
        "main.py"
    ]

    print(f"Running compilation: {' '.join(cmd)}")
    try:
        subprocess.check_call(cmd)
    except subprocess.CalledProcessError:
        print("Compilation failed.")
        return

    if os.path.exists(target_app):
        print(f"Compilation success! App bundle is in: dist/{app_name}.app")
    else:
        print("Compilation failed: .app bundle not found.")
        return

    # Create README regarding native tools
    readme_path = os.path.join("dist", "README_MAC.txt")
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write("Збірка для macOS завершена.\n\n")
        f.write("ПРИМІТКИ:\n")
        f.write("1. ffmpeg та ffprobe включені у збірку (знаходяться всередині .app/Contents/Resources/assets).\n")
        f.write("2. Windows-версія Whisper (whisper-cli-amd) не сумісна з Mac.\n")
        f.write("   Додаток використовуватиме стандартну бібліотеку whisper через Python.\n")
        f.write("3. Конфігурація та шаблони будуть створені/зчитані з папки 'config' поруч з .app.\n")

if __name__ == "__main__":
    compile_project()
