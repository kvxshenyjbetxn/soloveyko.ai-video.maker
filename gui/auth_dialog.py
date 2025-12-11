import requests
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLineEdit, QPushButton, QLabel, QMessageBox
)
from PySide6.QtCore import Qt

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
        
        self.login_button = QPushButton("Увійти")
        self.error_label = QLabel()
        self.error_label.setStyleSheet("color: red;")

        self.layout.addWidget(self.info_label)
        self.layout.addWidget(self.api_key_input)
        self.layout.addWidget(self.login_button)
        self.layout.addWidget(self.error_label)

        self.login_button.clicked.connect(self.handle_login)
        self.api_key_input.returnPressed.connect(self.handle_login)

    def handle_login(self):
        self.error_label.setText("")
        api_key = self.api_key_input.text().strip()
        if not api_key:
            self.error_label.setText("API ключ не може бути порожнім.")
            return

        self.login_button.setEnabled(False)
        self.login_button.setText("Перевірка...")

        try:
            response = requests.post(
                f"{self.server_url}/validate_key/",
                json={"key": api_key}
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get("valid"):
                    self.expires_at = data.get("expires_at")
                    self.accept()
                else:
                    self.error_label.setText(f"Помилка: {data.get('reason', 'Невідома помилка')}")
            else:
                self.error_label.setText(f"Помилка сервера: {response.status_code}")

        except requests.RequestException as e:
            self.error_label.setText("Не вдалося підключитися до сервера.")
        finally:
            self.login_button.setEnabled(True)
            self.login_button.setText("Увійти")

    def get_subscription_info(self):
        return self.expires_at

    def reject(self):
        # Override reject to handle the user closing the dialog
        QMessageBox.critical(self, "Доступ заборонено", "Автентифікація необхідна для використання програми.")
        super().reject()
