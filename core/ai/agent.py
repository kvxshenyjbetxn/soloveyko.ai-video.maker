import os
import json
from typing import List, Dict, Any, Generator
try:
    from openai import OpenAI
except ImportError:
    OpenAI = None
from core.ai.tools import tool_registry

from core.ai.knowledge import APP_CONTEXT

class AIAgent:
    """
    Основний клас AI агента.
    Відповідає за спілкування з OpenRouter API та виконання інструментів.
    """
    def __init__(self, api_key: str, model: str = "openai/gpt-4o-mini"):
        if OpenAI is None:
            raise ImportError("Бібліотека 'openai' не встановлена. Будь ласка, встановіть її командою: pip install openai")
            
        self.client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )
        self.model = model
        self.messages: List[Dict[str, str]] = [
            {"role": "system", "content": (
                "Ти - розумний помічник, вбудований у програму Soloveyko.AI Video Maker. "
                "Твоя мета - допомагати користувачу, відповідати на питання та керувати програмою через доступні інструменти. "
                "Відповідай українською мовою. Будь ввічливим і лаконічним. "
                "\n"
                f"{APP_CONTEXT}"
            )}
        ]

    def chat(self, user_input: str) -> Generator[str, None, None]:
        """
        Відправляє повідомлення агенту і повертає відповідь (потоково).
        Обробляє виклики інструментів автоматично.
        """
        self.messages.append({"role": "user", "content": user_input})

        while True:
            # 1. Відправляємо запит до AI
            try:
                response = self.client.chat.completions.create(
                    model=self.model,
                    messages=self.messages,
                    tools=tool_registry.get_openai_tools(),
                    tool_choice="auto"
                )
            except Exception as e:
                error_msg = f"Помилка API: {str(e)}"
                yield error_msg
                return

            message = response.choices[0].message
            
            # Якщо є контент, додаємо його до історії (це може бути проміжна думка перед викликом функції)
            if message.content:
                # self.messages.append({"role": "assistant", "content": message.content})
                # OpenAI API вимагає додавати tool_calls до message, якщо вони є.
                # Якщо ми просто додамо assistant content, це ок, але треба бути обережним з порядком.
                pass 

            # 2. Перевіряємо, чи хоче AI викликати інструмент
            if message.tool_calls:
                # Додаємо повідомлення асистента з викликом функції в історію
                self.messages.append(message)
                
                # Обробляємо кожен виклик
                for tool_call in message.tool_calls:
                    function_name = tool_call.function.name
                    arguments = json.loads(tool_call.function.arguments)
                    
                    tool = tool_registry.get_tool(function_name)
                    if tool:
                        yield f"🤖 Виконую: {tool.description}..."
                        try:
                            result = tool.run(**arguments)
                        except Exception as e:
                            result = f"Error: {str(e)}"
                    else:
                        result = f"Error: Tool {function_name} not found."

                    # Додаємо результат виконання в історію
                    self.messages.append({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": str(result)
                    })
                
                # Після виконання інструментів, ми повинні знову викликати AI, 
                # щоб він згенерував фінальну відповідь на основі результатів.
                continue
            
            else:
                # 3. Якщо немає викликів інструментів, це фінальна відповідь
                final_response = message.content
                self.messages.append({"role": "assistant", "content": final_response})
                yield final_response
                break
