import sys
import os
import requests
import traceback
import platform
from datetime import datetime

# Це виправить помилку 'NoneType object has no attribute write'
class NullWriter:
    def write(self, text):
        pass
    def flush(self):
        pass

if sys.stdout is None:
    sys.stdout = NullWriter()
if sys.stderr is None:
    sys.stderr = NullWriter()

# Suppress FFmpeg logs from Qt Multimedia by default
# For debugging, you can comment this line out or set the variable to "qt.multimedia.*=true"
os.environ['QT_LOGGING_RULES'] = 'qt.multimedia.ffmpeg.debug=false;qt.multimedia.ffmpeg.*=false;qt.text.font.db.*=false'

from PySide6.QtWidgets import QApplication, QDialog, QMessageBox
from gui.main_window import MainWindow
from gui.auth_dialog import AuthDialog
from gui.qt_material import apply_stylesheet
from utils.settings import settings_manager

# --- НОВА ФУНКЦІЯ: ДОДАЄ ASSETS У PATH ---
def setup_dependency_paths():
    """
    Додає папку assets у системний PATH процесу.
    Це дозволяє запускати 'ffmpeg' та 'ffprobe' без вказання повного шляху,
    навіть якщо вони не встановлені в Windows.
    """
    if getattr(sys, 'frozen', False):
        # Якщо запущено як скомпільований EXE, ресурси лежать у тимчасовій папці
        base_dir = sys._MEIPASS
    else:
        # Якщо запущено через Python скрипт
        base_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Шлях до папки assets
    assets_dir = os.path.join(base_dir, "assets")
    
    # Перевіряємо, чи є там ffmpeg
    ffmpeg_exe = os.path.join(assets_dir, "ffmpeg.exe")
    
    if os.path.exists(ffmpeg_exe):
        # Додаємо assets на початок PATH, щоб програма спочатку шукала там
        os.environ["PATH"] = assets_dir + os.pathsep + os.environ["PATH"]
        # print(f"Dependencies: Added {assets_dir} to PATH. FFmpeg found.")
    else:
        print(f"Dependencies: WARNING. FFmpeg not found in {assets_dir}")

# URL for the authentication server
AUTH_SERVER_URL = "https://new-project-combain-server-production.up.railway.app"

def handle_exception(exc_type, exc_value, exc_traceback):
    """
    Handles uncaught exceptions by logging them to a file and showing a dialog.
    """
    # Format the traceback
    tb_lines = traceback.format_exception(exc_type, exc_value, exc_traceback)
    tb_text = "".join(tb_lines)
    
    # Prepare the log message
    log_message = f"--- Uncaught Exception: {datetime.now()} ---\n"
    log_message += tb_text
    log_message += "-------------------------------------------------\n"

    # Log to a file
    try:
        crash_log_path = os.path.join(settings_manager.base_path, "crash.log")
        with open(crash_log_path, "a", encoding="utf-8") as f:
            f.write(log_message)
    except IOError as e:
        print(f"Could not write to crash.log: {e}")

    # Show a friendly error message to the user
    error_title = "Непередбачена помилка"
    error_text = (
        "Сталася критична помилка, і програма не може продовжити роботу.\n\n"
        "Будь ласка, повідомте розробнику про цю проблему, надіславши файл 'crash.log', "
        "який було створено в папці з програмою."
    )
    
    # We need a QApplication instance to show a QMessageBox,
    # but it might not be available if the error is very early.
    if QApplication.instance():
        msg_box = QMessageBox()
        msg_box.setIcon(QMessageBox.Critical)
        msg_box.setWindowTitle(error_title)
        msg_box.setText(error_text)
        msg_box.setDetailedText(tb_text) # Allows user to see details if they want
        msg_box.setStandardButtons(QMessageBox.Ok)
        msg_box.exec()
    else:
        # Fallback for very early crashes before QApplication is running
        print("--- CRITICAL ERROR ---")
        print(log_message)
        # In a GUI app, a graphical fallback is better if possible,
        # but this is a last resort.

    # The default excepthook prints to stderr. We've replaced it,
    # so we'll do it ourselves.
    sys.__excepthook__(exc_type, exc_value, exc_traceback)
    sys.exit(1)


def main():
    # --- Initialize Windows COM for Qt dialogs ---
    com_initialized = False
    if platform.system() == "Windows":
        try:
            import pythoncom
            pythoncom.CoInitialize()
            com_initialized = True
        except ImportError:
            # pythoncom not available, continue anyway
            pass
    
    # --- Set up global exception handler ---
    sys.excepthook = handle_exception
    
    # Try to enable faulthandler if available to catch segfaults
    try:
        import faulthandler
        log_dir = os.path.join(settings_manager.base_path, "logs")
        if not os.path.exists(log_dir):
            os.makedirs(log_dir)
        timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        log_file = os.path.join(log_dir, f"crash_faulthandler_{timestamp}.log")
        faulthandler.enable(file=open(log_file, "w", encoding="utf-8"), all_threads=True)
    except:
        pass

    from utils.logger import logger, LogLevel
    logger.log("Application starting...", level=LogLevel.INFO)
    
    # Clean up old logs on startup
    logger.cleanup_old_logs(max_days=7)

    app = QApplication(sys.argv)
    app.setStyle("fusion")

    # --- Authentication Flow ---
    authenticated = False
    subscription_info = None
    api_key = None

    saved_key = settings_manager.get('api_key')
    if saved_key:
        try:
            from utils.hardware_id import get_hardware_id
            hardware_id = get_hardware_id()
            
            response = requests.post(
                f"{AUTH_SERVER_URL}/validate_key/",
                json={"key": saved_key, "hardware_id": hardware_id},
                timeout=5 # seconds
            )
            if response.status_code == 200:
                data = response.json()
                if data.get("valid"):
                    subscription_info = data.get("expires_at")
                    api_key = saved_key
                    authenticated = True
        except requests.RequestException:
            # Fall through to the dialog if server is unreachable
            pass

    if not authenticated:
        auth_dialog = AuthDialog(server_url=AUTH_SERVER_URL)
        if auth_dialog.exec() == QDialog.Accepted:
            subscription_info = auth_dialog.get_subscription_info()
            api_key = auth_dialog.get_api_key()
            authenticated = True
        else:
            # User cancelled authentication
            sys.exit(0)
    
    if authenticated:
        setup_dependency_paths() # Call the new function here
        main_window = MainWindow(app, subscription_info=subscription_info, api_key=api_key, server_url=AUTH_SERVER_URL)
        main_window.apply_current_theme()
        main_window.show()
        exit_code = app.exec()
        
        # --- Uninitialize Windows COM after Qt event loop ends ---
        if com_initialized:
            try:
                import pythoncom
                pythoncom.CoUninitialize()
            except:
                pass
        
        sys.exit(exit_code)

if __name__ == '__main__':
    main()

