import sys
from PySide6.QtWidgets import QApplication
from gui.main_window import MainWindow
from utils.translator import translator
from utils.settings import AppSettings

def main():
    """
    The main function of the application.
    Initializes the QApplication and the MainWindow.
    """
    app = QApplication(sys.argv)
    
    # Set language from settings
    settings = AppSettings()
    translator.set_language(settings.get_language())

    window = MainWindow(settings)
    window.show()
    return app.exec()

if __name__ == '__main__':
    sys.exit(main())
