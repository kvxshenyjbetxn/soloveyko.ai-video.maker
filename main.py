import sys
import os

# Suppress FFmpeg logs from Qt Multimedia by default
# For debugging, you can comment this line out or set the variable to "qt.multimedia.*=true"
os.environ['QT_LOGGING_RULES'] = 'qt.multimedia.ffmpeg.debug=false;qt.multimedia.ffmpeg.*=false;qt.text.font.db.*=false'

import sys
import os

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
    auth_dialog = AuthDialog(server_url=AUTH_SERVER_URL)
    if auth_dialog.exec() == QDialog.Accepted:
        # If authentication is successful, proceed to the main window
        subscription_info = auth_dialog.get_subscription_info()
        main_window = MainWindow(app, subscription_info=subscription_info) # Pass app instance and sub info
        main_window.show()
        sys.exit(app.exec())
    else:
        # If authentication fails or is cancelled, exit the application
        sys.exit(0)

if __name__ == '__main__':
    main()

