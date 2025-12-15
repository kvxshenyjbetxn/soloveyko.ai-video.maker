import os
import sys
import subprocess
import shutil

def compile_project():
    # Назва вихідного файлу
    app_name = "new-project-combain"
    
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
    if os.path.exists("dist"):
        shutil.rmtree("dist")
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
    
    cmd = [
        "pyinstaller",
        "--noconfirm",
        "--onefile",
        "--windowed",
        "--icon=assets/icon.ico",
        "--add-data", "assets;assets",
        "--add-data", "gui/qt_material;gui/qt_material",
        "--add-data", f"{whisper_assets};whisper/assets",
        "--hidden-import", "whisper",
        "--exclude-module", "PyQt6",
        "--exclude-module", "PyQt5",
        "--exclude-module", "tkinter",
        "--name", app_name,
        "main.py"
    ]

    print(f"Running compilation: {' '.join(cmd)}")
    subprocess.check_call(cmd)

    # Move to a clean distribution folder
    dist_final = "dist_final"
    if os.path.exists(dist_final):
        shutil.rmtree(dist_final)
    os.makedirs(dist_final)

    # Copy EXE
    exe_src = os.path.join("dist", f"{app_name}.exe")
    exe_dst = os.path.join(dist_final, f"{app_name}.exe")
    if os.path.exists(exe_src):
        shutil.copy(exe_src, exe_dst)
        print(f"Compilation success! Executable is in: {dist_final}")
    else:
        print("Compilation failed: EXE not found.")
        return

    # Create README regarding whisper-cli-amd
    readme_path = os.path.join(dist_final, "README.txt")
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write("Сборка завершена.\n\n")
        f.write("ВАЖЛИВО:\n")
        f.write("1. Папка 'whisper-cli-amd' повинна знаходитися поруч з .exe файлом для роботи AMD Whisper.\n")
        f.write("2. Конфігурація та шаблони будуть створені/зчитані з папки 'config' поруч з .exe.\n")

if __name__ == "__main__":
    compile_project()
