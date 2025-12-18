import os
import sys
import subprocess
import shutil
import whisper  # Імпортуємо, щоб знайти шлях динамічно

def compile_project():
    # --- Встановлюємо робочу директорію ---
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    # ------------------------------------

    app_name = "CombainAI"
    
    # Очистка попередніх збірок
    if os.path.exists("build"):
        shutil.rmtree("build")
    if os.path.exists("dist"):
        shutil.rmtree("dist")

    # Знаходимо шлях до асетів Whisper
    whisper_path = os.path.dirname(whisper.__file__)
    whisper_assets = os.path.join(whisper_path, 'assets')
    
    # Перевірка іконки (потрібен .icns для Mac)
    icon_path = "assets/icon.icns"
    if not os.path.exists(icon_path):
        print(f"WARNING: {icon_path} not found! Using default icon.")
        icon_option = []
    else:
        icon_option = [f"--icon={icon_path}"]

    # Формування команди PyInstaller
    # УВАГА: На macOS розділювач у --add-data це двокрапка ':', а не ';'
    cmd = [
        "pyinstaller",
        "--noconfirm",
        "--clean",
        "--windowed",  # Створює .app bundle
        # "--onefile", # На Mac краще використовувати .app (це папка), але якщо дуже треба один файл - розкоментуйте, але це уповільнить старт
        "--name", app_name,
        
        # Додавання даних (формат source:dest)
        "--add-data", "assets:assets",
        "--add-data", "assets/gemini_tts_voices.json:assets",
        "--add-data", "assets/voicemaker_voices.json:assets",
        "--add-data", "gui/qt_material:gui/qt_material",
        "--add-data", f"{whisper_assets}:whisper/assets",
        
        # Приховані імпорти та виключення
        "--hidden-import", "whisper",
        "--exclude-module", "PyQt6",
        "--exclude-module", "PyQt5",
        "--exclude-module", "tkinter",
        
        # Головний файл
        "main.py"
    ] + icon_option

    print(f"Running compilation: {' '.join(cmd)}")
    
    try:
        subprocess.check_call(cmd)
        print("\n-----------------------------------")
        print(f"Compilation success!")
        print(f"App is located at: dist/{app_name}.app")
        print("-----------------------------------")
        
        # Опціонально: створення DMG образу (для зручного встановлення)
        # Для цього потрібен інструмент create-dmg (brew install create-dmg)
        # create_dmg_cmd = ["create-dmg", f"dist/{app_name}.app", "dist"]
        # subprocess.call(create_dmg_cmd)
        
    except subprocess.CalledProcessError as e:
        print("Compilation failed.")
        sys.exit(1)

if __name__ == "__main__":
    compile_project()