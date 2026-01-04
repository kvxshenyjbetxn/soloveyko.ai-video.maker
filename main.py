import sys
import os
import requests
import traceback
import platform
import subprocess
import multiprocessing
from datetime import datetime

# Avoid potential crashes on macOS with faulthandler + GUI
# and use devnull instead of custom NullWriter for early logs
if platform.system() == "Darwin":
    # Suppress FFmpeg logs from Qt Multimedia on macOS to prevent some driver/lib conflicts
    os.environ['QT_LOGGING_RULES'] = 'qt.multimedia.ffmpeg.debug=false;qt.multimedia.ffmpeg.*=false;qt.text.font.db.*=false'

if sys.stdout is None:
    try:
        sys.stdout = open(os.devnull, 'w')
    except:
        pass
if sys.stderr is None:
    try:
        sys.stderr = open(os.devnull, 'w')
    except:
        pass

from PySide6.QtWidgets import QApplication, QDialog, QMessageBox
from gui.main_window import MainWindow
from gui.auth_dialog import AuthDialog
from gui.qt_material import apply_stylesheet
from utils.settings import settings_manager
from utils.yt_dlp_updater import YtDlpUpdater


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
    
    # Визначаємо назву файлу залежно від ОС
    ffmpeg_name = "ffmpeg.exe" if platform.system() == "Windows" else "ffmpeg"
    ffmpeg_path = os.path.join(assets_dir, ffmpeg_name)
    
    if os.path.exists(ffmpeg_path):
        # Додаємо assets на початок PATH, щоб програма спочатку шукала там
        os.environ["PATH"] = assets_dir + os.pathsep + os.environ["PATH"]
        
        # На macOS додаємо права на виконання
        if platform.system() == "Darwin":
            try:
                import stat
                for tool in ["ffmpeg", "ffprobe", "yt-dlp"]:
                    tool_p = os.path.join(assets_dir, tool)
                    if os.path.exists(tool_p):
                        # Remove quarantine and set exec bit
                        try:
                            # 'xattr -d com.apple.quarantine' removes the "downloaded from internet" block
                            subprocess.run(["xattr", "-d", "com.apple.quarantine", tool_p], stderr=subprocess.DEVNULL)
                        except: pass
                        
                        st = os.stat(tool_p)
                        os.chmod(tool_p, st.st_mode | stat.S_IEXEC)
                
                # Також для yt-dlp у папці налаштувань (куди користувач кладе його вручну)
                from utils.settings import settings_manager
                yt_dlp_ext = "yt-dlp.exe" if platform.system() == "Windows" else "yt-dlp"
                yt_dlp_data_p = os.path.join(settings_manager.base_path, yt_dlp_ext)
                if os.path.exists(yt_dlp_data_p):
                    try:
                        subprocess.run(["xattr", "-d", "com.apple.quarantine", yt_dlp_data_p], stderr=subprocess.DEVNULL)
                    except: pass
                    st = os.stat(yt_dlp_data_p)
                    os.chmod(yt_dlp_data_p, st.st_mode | stat.S_IEXEC)
            except Exception as e:
                # print(f"DEBUG: Failed to setup macOS permissions: {e}")
                pass
    else:
        # Fallback: навіть якщо не знайшли файл у assets, додамо шлях про всяк випадок
        os.environ["PATH"] = assets_dir + os.pathsep + os.environ["PATH"]
        print(f"Dependencies: WARNING. {ffmpeg_name} not found in {assets_dir}")

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
    try:
        logger.log("Application starting...", level=LogLevel.INFO)
        # Clean up old logs on startup
        logger.cleanup_old_logs(max_days=7)
    except Exception as e:
        print(f"DEBUG: Early logging error: {e}")

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
        
        try:
            main_window.apply_current_theme()
        except Exception as e:
            print(f"DEBUG: Failed to apply theme: {e}")
            # Fallback to standard theme if possible
            try:
                main_window.app.setStyleSheet("")
            except:
                pass
                
        main_window.show()

        # --- Start yt-dlp Auto-Updater in background ---
        try:
            updater = YtDlpUpdater(main_window)
            # We keep a reference to prevent garbage collection
            main_window._yt_dlp_updater = updater
            updater.start()
        except Exception as e:
            print(f"DEBUG: Failed to start yt-dlp updater: {e}")

        exit_code = app.exec()
        
        # --- Uninitialize Windows COM after Qt event loop ends ---
        if com_initialized:
            try:
                import pythoncom
                pythoncom.CoUninitialize()
            except:
                pass
        
        # Use os._exit to force immediate termination, skipping global object cleanup
        # This prevents 0x8001010d errors caused by threads trying to accept/read
        # from closed sockets/pipes during interpreter shutdown.
        os._exit(exit_code)

if __name__ == '__main__':
    multiprocessing.freeze_support()
    main()

