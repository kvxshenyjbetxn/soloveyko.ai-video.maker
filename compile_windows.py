import os
import sys
import subprocess
import shutil

def compile_project():
    # --- Set CWD to script's directory ---
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    # ------------------------------------

    # Назва вихідного файлу
    app_name = "Soloveyko.AI-Video.Maker v0.2.25-beta"

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
    # Очистка попереднього EXE (але не всієї папки dist, щоб зберегти моделі/конфіги)
    target_exe = os.path.join("dist", f"{app_name}.exe")
    if os.path.exists(target_exe):
        try:
            os.remove(target_exe)
        except OSError:
            print(f"Warning: Could not remove {target_exe}. It might be in use.")

    if os.path.exists(f"{app_name}.spec"):
        os.remove(f"{app_name}.spec")

    # Команда для PyInstaller
    # --onefile: один EXE файл
    # --windowed: без консолі (для GUI)
    # --icon: іконка
    # --add-data: додати папку assets всередину EXE
    # --hidden-import: додати whisper, який може не детектуватися автоматично
    
    import whisper
    whisper_path = os.path.dirname(whisper.__file__)
    whisper_assets = os.path.join(whisper_path, 'assets')
    
    import whisper
    whisper_path = os.path.dirname(whisper.__file__)
    whisper_assets = os.path.join(whisper_path, 'assets')
    
    cmd = [
        "pyinstaller",
        "--noconfirm",
        "--onefile",
        "--windowed",
        "--icon=assets/icon.ico",
        # Assets files manually listed (без efects папки)
        "--add-data", "assets/ffmpeg.exe;assets",
        "--add-data", "assets/ffprobe.exe;assets",
        "--add-data", "assets/icon.ico;assets",
        "--add-data", "assets/icon.png;assets",
        "--add-data", "assets/icon.icns;assets",
        "--add-data", "assets/gemini_tts_voices.json;assets",
        "--add-data", "assets/voicemaker_voices.json;assets",
        "--add-data", "assets/translations;assets/translations",
        "--add-data", "assets/styles;assets/styles",
        # efects папка НЕ додається!
        "--add-data", "gui/qt_material;gui/qt_material",
        "--add-data", f"{whisper_assets};whisper/assets",
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
    subprocess.check_call(cmd)

    if os.path.exists(target_exe):
        print(f"Compilation success! Executable is in: dist/{app_name}.exe")
    else:
        print("Compilation failed: EXE not found.")
        return

    # Create README regarding whisper-cli-amd
    readme_path = os.path.join("dist", "README.txt")
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write("Сборка завершена.\n\n")
        f.write("ВАЖЛИВО:\n")
        f.write("1. Папка 'whisper-cli-amd' повинна знаходитися поруч з .exe файлом для роботи AMD Whisper.\n")
        f.write("2. Конфігурація та шаблони будуть створені/зчитані з папки 'config' поруч з .exe.\n")

if __name__ == "__main__":
    compile_project()
