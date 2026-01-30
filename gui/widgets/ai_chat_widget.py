from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QTextEdit, 
                               QLineEdit, QPushButton, QLabel, QFrame, QScrollArea, QSizePolicy)
from PySide6.QtCore import Qt, QThread, Signal, QSize
from PySide6.QtGui import QIcon, QFont, QTextCursor
from core.ai.agent import AIAgent
from utils.settings import settings_manager
try:
    import markdown
except ImportError:
    markdown = None

def text_to_html(text):
    if markdown:
        return markdown.markdown(text)
    else:
        # Simple fallback
        import html
        escaped = html.escape(text)
        return escaped.replace("\n", "<br>")

class AIWorker(QThread):
    chunk_received = Signal(str)
    finished = Signal()
    error = Signal(str)

    def __init__(self, agent, user_input):
        super().__init__()
        self.agent = agent
        self.user_input = user_input

    def run(self):
        try:
            for chunk in self.agent.chat(self.user_input):
                self.chunk_received.emit(chunk)
            self.finished.emit()
        except Exception as e:
            self.error.emit(str(e))

class AIChatWidget(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.agent = None
        self.init_ui()
        self.init_agent()

    def init_agent(self):
        # Отримуємо ключ з налаштувань
        api_key = settings_manager.get("openrouter_api_key")
        model = settings_manager.get("ai_assistant_model", "openai/gpt-4o-mini")
        
        # Якщо ключа немає, агент не ініціалізується, покажемо повідомлення
        if api_key:
            try:
                self.agent = AIAgent(api_key=api_key, model=model)
                self.append_system_message(f"Агент готовий до роботи! 🤖 (Модель: {model})")
            except Exception as e:
                self.append_system_message(f"Помилка ініціалізації агента: {e}")
        else:
            self.append_system_message("⚠️ API Key OpenRouter не знайдено. Будь ласка, додайте його в налаштуваннях.")

    def init_ui(self):
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(10, 10, 10, 10)
        self.layout.setSpacing(10)

        # Заголовок
        header_layout = QHBoxLayout()
        title = QLabel("AI Assistant")
        title.setFont(QFont("Segoe UI", 12, QFont.Bold))
        header_layout.addWidget(title)
        header_layout.addStretch()
        self.layout.addLayout(header_layout)

        # Історія чату
        self.chat_history = QTextEdit()
        self.chat_history.setReadOnly(True)
        self.chat_history.setStyleSheet("""
            QTextEdit {
                background-color: #2b2b2b;
                color: #ffffff;
                border: 1px solid #3d3d3d;
                border-radius: 8px;
                padding: 10px;
                font-family: 'Segoe UI', sans-serif;
                font-size: 14px;
            }
        """)
        self.layout.addWidget(self.chat_history)

        # Поле вводу
        input_layout = QHBoxLayout()
        self.input_field = QLineEdit()
        self.input_field.setPlaceholderText("Напишіть ваше питання...")
        self.input_field.setStyleSheet("""
            QLineEdit {
                border: 1px solid #3d3d3d;
                border-radius: 20px;
                padding: 10px;
                background-color: #1e1e1e;
                color: white;
            }
            QLineEdit:focus {
                border: 1px solid #0078d4;
            }
        """)
        self.input_field.returnPressed.connect(self.send_message)
        input_layout.addWidget(self.input_field)

        self.send_btn = QPushButton("➤")
        self.send_btn.setFixedSize(40, 40)
        self.send_btn.setStyleSheet("""
            QPushButton {
                background-color: #0078d4;
                color: white;
                border-radius: 20px;
                border: none;
                font-size: 16px;
            }
            QPushButton:hover {
                background-color: #006cc1;
            }
            QPushButton:pressed {
                background-color: #005a9e;
            }
        """)
        self.send_btn.clicked.connect(self.send_message)
        input_layout.addWidget(self.send_btn)
        
        self.layout.addLayout(input_layout)

    def send_message(self):
        text = self.input_field.text().strip()
        if not text:
            return
            
        if not self.agent:
            self.init_agent() # Try to re-init if key was added
            if not self.agent:
                self.append_system_message("⚠️ Будь ласка, спочатку налаштуйте API ключ.")
                return

        self.input_field.clear()
        self.input_field.setEnabled(False)
        self.send_btn.setEnabled(False)
        self.append_user_message(text)

        self.worker = AIWorker(self.agent, text)
        self.worker.chunk_received.connect(self.handle_chunk)
        self.worker.finished.connect(self.worker_finished)
        self.worker.error.connect(self.worker_error)
        self.worker.start()

    def append_user_message(self, text):
        self.chat_history.append(f"<br><b>👤 Ви:</b> {text}")
        self.scroll_to_bottom()

    def append_system_message(self, text):
        self.chat_history.append(f"<br><i>{text}</i>")
        self.scroll_to_bottom()

    def handle_chunk(self, chunk):
        # Тут ми можемо отримувати або повідомлення про виклик тулзи, або частину відповіді
        # Для простоти поки що просто додаємо текст.
        # Markdown to HTML conversion can be added here if needed
        # For MVP we just append plain text or simple formatting
        
        # Перевірка на спеціальні системні повідомлення від агента (наприклад tool call)
        if chunk.startswith("🤖"):
             self.chat_history.append(f"<code style='color: #4caf50'>{chunk}</code>")
        else:
             # Якщо це відповідь асистента - ми хочемо її форматувати як markdown
             # Але оскільки ми отримуємо просто текст (не стрім в даному випадку, бо генератор повертає блоки в agent.py)
             # то ми можемо просто форматувати блок.
             
             # Оскільки agent.py повертає повну відповідь якщо це final_response
             # то ми просто додаємо її.
             # html = markdown.markdown(chunk) 
             html = text_to_html(chunk)
             self.chat_history.append(f"<br><b>🤖 AI:</b><br>{html}")
        
        self.scroll_to_bottom()

    def worker_finished(self):
        self.input_field.setEnabled(True)
        self.send_btn.setEnabled(True)
        self.input_field.setFocus()

    def worker_error(self, error):
        self.chat_history.append(f"<br><span style='color: red'>Error: {error}</span>")
        self.worker_finished()

    def scroll_to_bottom(self):
        scrollbar = self.chat_history.verticalScrollBar()
        scrollbar.setValue(scrollbar.maximum())
