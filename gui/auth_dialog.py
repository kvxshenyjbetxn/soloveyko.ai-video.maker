import requests
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLineEdit, QPushButton, QLabel, QMessageBox, QCheckBox
)
from PySide6.QtCore import Qt

from utils.settings import settings_manager
from utils.hardware_id import get_hardware_id

class AuthDialog(QDialog):
    def __init__(self, server_url, parent=None):
        super().__init__(parent)
        self.server_url = server_url
        self.expires_at = None

        self.setWindowTitle("Автентифікація")
        self.setModal(True)
        self.setMinimumWidth(300)

        self.layout = QVBoxLayout(self)

        self.info_label = QLabel("Будь ласка, введіть ваш API ключ для доступу.")
        self.api_key_input = QLineEdit()
        self.api_key_input.setPlaceholderText("Ваш API ключ")

        self.remember_me_checkbox = QCheckBox("Запам'ятати API ключ")
        
        self.login_button = QPushButton("Увійти")
        self.error_label = QLabel()
        self.error_label.setStyleSheet("color: red;")

        self.layout.addWidget(self.info_label)
        self.layout.addWidget(self.api_key_input)
        self.layout.addWidget(self.remember_me_checkbox)
        self.layout.addWidget(self.login_button)
        self.layout.addWidget(self.error_label)

        self.login_button.clicked.connect(self.handle_login)
        self.api_key_input.returnPressed.connect(self.handle_login)

        # Pre-fill from settings
        saved_key = settings_manager.get('api_key')
        if saved_key:
            self.api_key_input.setText(saved_key)
            self.remember_me_checkbox.setChecked(True)

    def handle_login(self):
        self.error_label.setText("")
        api_key = self.api_key_input.text().strip()
        if not api_key:
            self.error_label.setText("API ключ не може бути порожнім.")
            return

        self.login_button.setEnabled(False)
        self.login_button.setText("Перевірка...")

        try:
            # Отримуємо hardware ID пристрою
            hardware_id = get_hardware_id()
            
            response = requests.post(
                f"{self.server_url}/validate_key/",
                json={"key": api_key, "hardware_id": hardware_id}
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get("valid"):
                    self.expires_at = data.get("expires_at")
                    
                    if self.remember_me_checkbox.isChecked():
                        settings_manager.set('api_key', api_key)
                    else:
                        settings_manager.set('api_key', None)
                    settings_manager.save_settings()

                    self.accept()
                else:
                    # Clear saved key if it's invalid
                    settings_manager.set('api_key', None)
                    settings_manager.save_settings()
                    self.error_label.setText(f"Помилка: {data.get('reason', 'Невідома помилка')}")
            elif response.status_code == 403:
                # Handle hardware mismatch or invalid key
                data = response.json()
                detail = data.get("detail", "")
                
                if "hardware" in detail.lower() or "пристрій" in detail.lower():
                    self.error_label.setText(
                        "Цей ключ прив'язаний до іншого пристрою.\n"
                        "Зверніться до техпідтримки для скидання прив'язки."
                    )
                else:
                    self.error_label.setText(f"Помилка: {detail}")
                    
                # Clear saved key
                settings_manager.set('api_key', None)
                settings_manager.save_settings()
            else:
                self.error_label.setText(f"Помилка сервера: {response.status_code}")

        except requests.RequestException as e:
            self.error_label.setText("Не вдалося підключитися до сервера.")
        finally:
            self.login_button.setEnabled(True)
            self.login_button.setText("Увійти")

    def get_api_key(self):
        return self.api_key_input.text().strip()

    def get_subscription_info(self):
        return self.expires_at

    def reject(self):
        # Override reject to handle the user closing the dialog
        QMessageBox.critical(self, "Доступ заборонено", "Автентифікація необхідна для використання програми.")
        super().reject()
