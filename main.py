import sys
import os
import requests
import traceback
from datetime import datetime

# Suppress FFmpeg logs from Qt Multimedia by default
# For debugging, you can comment this line out or set the variable to "qt.multimedia.*=true"
os.environ['QT_LOGGING_RULES'] = 'qt.multimedia.ffmpeg.debug=false;qt.multimedia.ffmpeg.*=false;qt.text.font.db.*=false'

from PySide6.QtWidgets import QApplication, QDialog, QMessageBox
from gui.main_window import MainWindow
from gui.auth_dialog import AuthDialog
from gui.qt_material import apply_stylesheet
from utils.settings import settings_manager

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
        with open("crash.log", "a") as f:
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
    # --- Set up global exception handler ---
    sys.excepthook = handle_exception

    app = QApplication(sys.argv)
    app.setStyle("fusion")

    # --- Authentication Flow ---
    authenticated = False
    subscription_info = None
    api_key = None

    saved_key = settings_manager.get('api_key')
    if saved_key:
        try:
            response = requests.post(
                f"{AUTH_SERVER_URL}/validate_key/",
                json={"key": saved_key},
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
        main_window = MainWindow(app, subscription_info=subscription_info, api_key=api_key, server_url=AUTH_SERVER_URL)
        main_window.apply_current_theme()
        main_window.show()
        sys.exit(app.exec())

if __name__ == '__main__':
    main()

