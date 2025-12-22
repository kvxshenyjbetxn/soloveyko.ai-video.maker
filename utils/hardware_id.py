"""
Модуль для отримання унікального ідентифікатора заліза (hardware ID).
Використовується для прив'язки API ключа до конкретного пристрою.
"""
import subprocess
import platform
import re
import hashlib


def get_hardware_id() -> str:
    """
    Отримує унікальний ідентифікатор заліза пристрою (Platform UUID).
    
    На Windows: MachineGuid з реєстру.
    На macOS: IOPlatformUUID.
    
    Returns:
        str: Хешований Hardware ID для анонімізації та одностайності формату.
    """
    system = platform.system()
    hw_id = ""

    try:
        if system == "Windows":
            # Отримуємо MachineGuid через reg query
            cmd = 'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Cryptography" /v MachineGuid'
            result = subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode()
            # Шукаємо UUID у форматі 8-4-4-4-12
            match = re.search(r'[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}', result)
            if match:
                hw_id = match.group(0)

        elif system == "Darwin":  # macOS
            # Отримуємо IOPlatformUUID через ioreg
            cmd = 'ioreg -rd1 -c IOPlatformExpertDevice'
            result = subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode()
            # Шукаємо IOPlatformUUID
            match = re.search(r'"IOPlatformUUID" = "([^"]+)"', result)
            if match:
                hw_id = match.group(1)

        if not hw_id:
            # Fallback на назву вузла та процесора, якщо специфічні методи не спрацювали
            hw_id = f"{platform.node()}-{platform.processor()}-{platform.machine()}"

        # Хешуємо ID, щоб він мав однаковий формат (наприклад, SHA-256) і був анонімним
        return hashlib.sha256(hw_id.encode()).hexdigest()

    except Exception:
        # Абсолютний fallback
        return hashlib.sha256(f"{platform.node()}-fallback".encode()).hexdigest()


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
    print(f"Stable Hardware ID: {hw_id}")
    
    platform_info = get_platform_info()
    print(f"\nPlatform Info:")
    for key, value in platform_info.items():
        print(f"  {key}: {value}")
