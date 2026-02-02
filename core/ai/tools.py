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



class SearchDocsTool(BaseTool):
    """Інструмент для пошуку в документації."""
    name = "search_docs"
    description = "Шукає інформацію у внутрішній документації програми."

    def run(self, query: str) -> str:
        import os
        
        # Визначаємо шлях до папки з документами
        base_path = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        docs_path = os.path.join(base_path, "assets", "docs")
        
        if not os.path.exists(docs_path):
            return "Помилка: Папка з документацією не знайдена."
            
        results = []
        query_lower = query.lower().strip()
        query_words = query_lower.split()
        
        try:
            for filename in os.listdir(docs_path):
                if filename.endswith(".md") or filename.endswith(".txt"):
                    filepath = os.path.join(docs_path, filename)
                    with open(filepath, 'r', encoding='utf-8') as f:
                        content = f.read()
                        
                    content_lower = content.lower()
                    filename_lower = filename.lower()
                    
                    match_found = False
                    reason = ""
                    
                    # 1. Пошук за назвою файлу
                    if query_lower in filename_lower:
                        match_found = True
                        reason = "співпадіння в назві файлу"
                    
                    # 2. Точний пошук фрази в контенті
                    elif query_lower in content_lower:
                        match_found = True
                        reason = "знайдено фразу в тексті"
                        
                    # 3. Пошук всіх слів (якщо їх > 1)
                    elif len(query_words) > 1 and all(word in content_lower for word in query_words):
                        match_found = True
                        reason = "знайдено всі ключові слова"
                    
                    # 4. Пошук за частиною слова (якщо запит короткий, e.g. "google")
                    elif len(query_lower) > 3 and query_lower in content_lower:
                         match_found = True
                         reason = "часткове співпадіння"

                    if match_found:
                        results.append(f"--- Документ: {filename} ({reason}) ---\n{content}\n{'='*30}")
                        
        except Exception as e:
            return f"Помилка при пошуку: {str(e)}"
            
        if not results:
            return f"На жаль, я не знайшов точної інформації за запитом '{query}' у документації. Спробуйте перефразувати або шукати ширші поняття (наприклад 'Googler', 'Налаштування')."
            
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


class SearchWebTool(BaseTool):
    """Інструмент для пошуку в інтернеті."""
    name = "search_web"
    description = "Шукає інформацію в інтернеті..."

    def run(self, query: str) -> str:
        try:
            from duckduckgo_search import DDGS
        except ImportError:
            return "Помилка, сервіс не працюе"
        
        try:
            results = []
            with DDGS() as ddgs:
                # Отримуємо перші 3 результати для швидкості
                for r in ddgs.text(query, max_results=3):
                    title = r.get('title', '')
                    href = r.get('href', '')
                    body = r.get('body', '')
                    results.append(f"Заголовок: {title}\nПосилання: {href}\nОпис: {body}\n")
            
            if not results:
                return "На жаль, пошук не дав результатів."
                
            return "\n---\n".join(results)
        except Exception as e:
            return f"Помилка при виконанні пошуку: {str(e)}"

    def get_parameters_schema(self) -> Dict[str, Any]:
        return {
            "type": "object", 
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Запит для пошуку в Google/DuckDuckGo"
                }
            }, 
            "required": ["query"]
        }

tool_registry.register(SearchDocsTool())
tool_registry.register(SearchWebTool())
