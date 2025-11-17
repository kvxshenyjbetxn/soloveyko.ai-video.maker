import sys
from PySide6.QtWidgets import QApplication
from gui.main_window import MainWindow
from gui.qt_material import apply_stylesheet
from utils.settings import settings_manager

def main():
    app = QApplication(sys.argv)
    
    # Get theme from settings or default to light
    theme_name = settings_manager.get('theme', 'light')
    if theme_name == 'light':
        apply_stylesheet(app, theme='light_blue.xml')
    elif theme_name == 'dark':
        apply_stylesheet(app, theme='dark_teal.xml')
    elif theme_name == 'black':
        apply_stylesheet(app, theme='amoled_black.xml') # Closest to Amoled

    main_window = MainWindow(app) # Pass app instance
    main_window.show()
    sys.exit(app.exec())

if __name__ == '__main__':
    main()
