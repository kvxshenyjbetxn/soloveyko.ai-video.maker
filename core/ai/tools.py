from typing import Callable, List, Dict, Any, Optional
import inspect
from config.version import APP_VERSION

class BaseTool:
    """Базовий клас для всіх інструментів агента."""
    name: str
    description: str
    
    def run(self, **kwargs) -> str:
        raise NotImplementedError

    def to_schema(self) -> Dict[str, Any]:
        """Повертає схему інструменту для OpenAI API."""
        # Автоматична генерація схеми з сигнатури методу run. 
        # Для простоти на цьому етапі ми будемо визначати параметри вручну в підкласах,
        # але в майбутньому можна додати авто-генерацію.
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.get_parameters_schema()
            }
        }

    def get_parameters_schema(self) -> Dict[str, Any]:
        """Повертає JSON-схему параметрів функції."""
        return {"type": "object", "properties": {}, "required": []}

class GetVersionTool(BaseTool):
    """Інструмент для отримання поточної версії програми."""
    name = "get_app_version"
    description = "Повертає поточну версію програми. Використовуйте це, коли користувач запитує версію."

    def run(self) -> str:
        return f"Soloveyko.AI Video Maker v{APP_VERSION}"
    
    def get_parameters_schema(self) -> Dict[str, Any]:
        # Ця функція не приймає параметрів
        return {"type": "object", "properties": {}, "required": []}

class ToolRegistry:
    """Реєстр доступних інструментів."""
    def __init__(self):
        self._tools: Dict[str, BaseTool] = {}

    def register(self, tool: BaseTool):
        self._tools[tool.name] = tool

    def get_tool(self, name: str) -> Optional[BaseTool]:
        return self._tools.get(name)

    def get_all_tools(self) -> List[BaseTool]:
        return list(self._tools.values())

    def get_openai_tools(self) -> List[Dict[str, Any]]:
        """Повертає список інструментів у форматі OpenAI API."""
        return [tool.to_schema() for tool in self._tools.values()]

class SetImageProviderTool(BaseTool):
    """Інструмент для зміни провайдера генерації зображень."""
    name = "set_image_provider"
    description = "Змінює сервіс (провайдера) для генерації зображень. Використовуйте це ТІЛЬКИ після явного погодження з користувачем."

    def run(self, provider: str) -> str:
        valid_providers = ['pollinations', 'googler', 'elevenlabs_image']
        if provider not in valid_providers:
             return f"Помилка: Невідомий провайдер '{provider}'. Доступні: {', '.join(valid_providers)}"
        
        from utils.settings import settings_manager
        settings_manager.set('image_generation_provider', provider)
        
        # Сповіщаємо інтерфейс про необхідність оновлення
        from core.signals import global_signals
        global_signals.request_ui_refresh.emit()
        
        return f"Успішно змінено провайдер генерації зображень на '{provider}'. Будь ласка, перезавантажте сторінку або перевірте налаштування, щоб побачити зміни."

    def get_parameters_schema(self) -> Dict[str, Any]:
        return {
            "type": "object", 
            "properties": {
                "provider": {
                    "type": "string",
                    "enum": ['pollinations', 'googler', 'elevenlabs_image'],
                    "description": "Назва провайдера (pollinations, googler, або elevenlabs_image)"
                }
            }, 
            "required": ["provider"]
        }

class SearchDocsTool(BaseTool):
    """Інструмент для пошуку в документації."""
    name = "search_docs"
    description = "Шукає інформацію у внутрішній документації програми. Використовуй це, коли користувач питає деталі про функції (наприклад, Googler, Pollinations), а ти не знаєш відповіді."

    def run(self, query: str) -> str:
        import os
        
        # Визначаємо шлях до папки з документами
        # Припускаємо, що tools.py знаходиться в core/ai/
        base_path = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        docs_path = os.path.join(base_path, "assets", "docs")
        
        if not os.path.exists(docs_path):
            return "Помилка: Папка з документацією не знайдена."
            
        results = []
        query = query.lower()
        
        try:
            for filename in os.listdir(docs_path):
                if filename.endswith(".md") or filename.endswith(".txt"):
                    filepath = os.path.join(docs_path, filename)
                    with open(filepath, 'r', encoding='utf-8') as f:
                        content = f.read()
                        
                    # Дуже простий пошук: якщо запит є в тексті
                    if query in content.lower():
                        # Повертаємо назву файлу і частину контенту (або весь контент, якщо він малий)
                        results.append(f"--- Знайдено у файлі {filename} ---\n{content}\n----------------")
        except Exception as e:
            return f"Помилка при пошуку: {str(e)}"
            
        if not results:
            return f"На жаль, я не знайшов інформації за запитом '{query}' у документації."
            
        return "\n\n".join(results)

    def get_parameters_schema(self) -> Dict[str, Any]:
        return {
            "type": "object", 
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Ключове слово або фраза для пошуку (наприклад, 'Googler', 'API', 'Keys')"
                }
            }, 
            "required": ["query"]
        }

# Глобальний екземпляр реєстру
tool_registry = ToolRegistry()
tool_registry.register(GetVersionTool())
tool_registry.register(SetImageProviderTool())
tool_registry.register(SearchDocsTool())
