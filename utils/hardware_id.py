"""
Модуль для отримання унікального ідентифікатора заліза (hardware ID).
Використовується для прив'язки API ключа до конкретного пристрою.
"""
import uuid
import platform


def get_hardware_id() -> str:
    """
    Отримує унікальний ідентифікатор заліза пристрою.
    
    Використовує MAC адресу мережевого адаптера як hardware ID.
    Працює на Windows та macOS.
    
    Returns:
        str: Hardware ID у форматі "XX:XX:XX:XX:XX:XX"
    """
    try:
        # Отримуємо MAC адресу через uuid.getnode()
        # Це працює на Windows і macOS
        mac = uuid.getnode()
        
        # Конвертуємо в читабельний формат
        mac_hex = hex(mac)[2:].upper().zfill(12)
        hardware_id = ':'.join([mac_hex[i:i+2] for i in range(0, 12, 2)])
        
        return hardware_id
    except Exception as e:
        # У випадку помилки використовуємо UUID як fallback
        # Це рідкісний випадок, але краще мати fallback
        fallback_id = str(uuid.uuid4())
        return fallback_id


def get_platform_info() -> dict:
    """
    Отримує додаткову інформацію про платформу (для логування/debug).
    
    Returns:
        dict: Словник з інформацією про систему
    """
    return {
        'system': platform.system(),
        'release': platform.release(),
        'machine': platform.machine(),
        'processor': platform.processor(),
    }


if __name__ == "__main__":
    # Тестування модуля
    hw_id = get_hardware_id()
    print(f"Hardware ID: {hw_id}")
    
    platform_info = get_platform_info()
    print(f"\nPlatform Info:")
    for key, value in platform_info.items():
        print(f"  {key}: {value}")
