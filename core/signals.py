from PySide6.QtCore import QObject, Signal

class GlobalSignals(QObject):
    """
    Глобальні сигнали для комунікації між різними частинами програми,
    зокрема між логікою (core) та інтерфейсом (gui).
    """
    # Сигнал запиту на оновлення інтерфейсу (наприклад, після зміни налаштувань агентом)
    request_ui_refresh = Signal()

# Створюємо глобальний екземпляр
global_signals = GlobalSignals()
