import sys
import os
import requests

# Suppress FFmpeg logs from Qt Multimedia by default
# For debugging, you can comment this line out or set the variable to "qt.multimedia.*=true"
os.environ['QT_LOGGING_RULES'] = 'qt.multimedia.ffmpeg.debug=false;qt.multimedia.ffmpeg.*=false;qt.text.font.db.*=false'

from PySide6.QtWidgets import QApplication, QDialog
from gui.main_window import MainWindow
from gui.auth_dialog import AuthDialog
from gui.qt_material import apply_stylesheet
from utils.settings import settings_manager

# URL for the authentication server
AUTH_SERVER_URL = "https://new-project-combain-server-production.up.railway.app" 

def main():
    app = QApplication(sys.argv)
    
    # Define extra arguments for the stylesheet
    extra = {'font_family': 'RobotoCondensed'}

    # Get theme from settings or default to light
    theme_name = settings_manager.get('theme', 'light')
    if theme_name == 'light':
        apply_stylesheet(app, theme='light_blue.xml', extra=extra)
    elif theme_name == 'dark':
        apply_stylesheet(app, theme='dark_teal.xml', extra=extra)
    elif theme_name == 'black':
        apply_stylesheet(app, theme='amoled_black.xml', extra=extra) # Closest to Amoled
        # Add custom style for QTextEdit to make it slightly lighter than the background
        custom_style = "QTextEdit { background-color: #121212; }"
        app.setStyleSheet(app.styleSheet() + custom_style)

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
        main_window.show()
        sys.exit(app.exec())

if __name__ == '__main__':
    main()

