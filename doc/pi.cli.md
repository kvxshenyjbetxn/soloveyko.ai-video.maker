# Pi CLI — команди та флаги

> `pi [options] [@files...] [messages...]`  
> За замовчуванням запускає інтерактивну сесію. Для неінтерактивного режиму використовуй `-p/--print`.

---

## Основні флаги

| Флаг | Опис |
|------|------|
| `-p, --print` | Неінтерактивний режим: обробити промпт і завершити |
| `-c, --continue` | Продовжити попередню сесію |
| `-r, --resume` | Вибрати сесію для відновлення |
| `-n, --name <name>` | Задати відображуване ім'я сесії |
| `-v, --version` | Вивести версію |
| `-h, --help` | Показати довідку |
| `--verbose` | Примусово детальний вивід при старті |

---

## Модель та провайдер

| Флаг | Опис |
|------|------|
| `--provider <name>` | Провайдер (за замовчуванням: `google`) |
| `--model <pattern>` | Модель або шаблон ID. Підтримує `provider/id` та `:<thinking>` |
| `--api-key <key>` | API-ключ (за замовчуванням з env vars) |
| `--models <patterns>` | Comma-separated шаблони моделей для циклічного перемикання Ctrl+P. Підтримує glob (`anthropic/*`, `*sonnet*`) та fuzzy |
| `--thinking <level>` | Рівень thinking: `off`, `minimal`, `low`, `medium`, `high`, `xhigh` |

---

## Сесія

| Флаг | Опис |
|------|------|
| `--session <path\|id>` | Використати конкретний файл сесії або частковий UUID |
| `--session-id <id>` | Використати точний ID сесії проекту (створити якщо відсутній) |
| `--fork <path\|id>` | Форкнути конкретну сесію у нову |
| `--session-dir <dir>` | Директорія для зберігання та пошуку сесій |
| `--no-session` | Не зберігати сесію (ефемерний режим) |

---

## Системний промпт

| Флаг | Опис |
|------|------|
| `--system-prompt <text>` | Задати системний промпт |
| `--append-system-prompt <text>` | Додати текст або вміст файлу до системного промпту (можна використовувати кілька разів) |

---

## Інструменти (tools)

| Флаг | Опис |
|------|------|
| `--no-tools, -nt` | Вимкнути всі інструменти (вбудовані та розширення) |
| `--no-builtin-tools, -nbt` | Вимкнути лише вбудовані інструменти, залишивши розширення |
| `--tools, -t <tools>` | Comma-separated allowlist інструментів |
| `--exclude-tools, -xt <tools>` | Comma-separated denylist інструментів |

### Вбудовані інструменти

| Інструмент | Опис | За замовчуванням |
|-----------|------|-----------------|
| `read` | Читати вміст файлів | увімкнено |
| `bash` | Виконувати bash-команди | увімкнено |
| `edit` | Редагувати файли (find/replace) | увімкнено |
| `write` | Записувати файли (створити/перезаписати) | увімкнено |
| `grep` | Пошук у вмісті файлів (read-only) | вимкнено |
| `find` | Пошук файлів за glob (read-only) | вимкнено |
| `ls` | Список вмісту директорій (read-only) | вимкнено |

---

## Розширення та скіли

| Флаг | Опис |
|------|------|
| `--extension, -e <path>` | Завантажити файл розширення (можна кілька разів) |
| `--no-extensions, -ne` | Вимкнути автовідкриття розширень (явні `-e` шляхи залишаються) |
| `--skill <path>` | Завантажити файл або директорію скіла (можна кілька разів) |
| `--no-skills, -ns` | Вимкнути автовідкриття та завантаження скілів |
| `--prompt-template <path>` | Завантажити файл або директорію prompt template (можна кілька разів) |
| `--no-prompt-templates, -np` | Вимкнути автовідкриття prompt templates |
| `--theme <path>` | Завантажити файл або директорію теми (можна кілька разів) |
| `--no-themes` | Вимкнути автовідкриття тем |
| `--no-context-files, -nc` | Вимкнути автовідкриття `AGENTS.md` та `CLAUDE.md` |

---

## Дозволи та контекст

| Флаг | Опис |
|------|------|
| `--approve, -a` | Довіряти локальним файлам проекту для цього запуску |
| `--no-approve, -na` | Ігнорувати локальні файли проекту для цього запуску |
| `--offline` | Вимкнути мережеві операції при старті (аналог `PI_OFFLINE=1`) |

---

## Формат виводу

| Флаг | Опис |
|------|------|
| `--mode <mode>` | Режим виводу: `text` (за замовчуванням), `json`, `rpc` |
| `--export <file>` | Експортувати файл сесії у HTML та завершити |

---

## Підкоманди

| Команда | Опис |
|---------|------|
| `install <source> [-l]` | Встановити джерело розширення та додати до налаштувань |
| `remove <source> [-l]` | Видалити джерело розширення з налаштувань |
| `uninstall <source> [-l]` | Псевдонім для `remove` |
| `update [source\|self\|pi]` | Оновити pi (`--all` — оновити pi та розширення) |
| `list` | Показати встановлені розширення з налаштувань |
| `config` | Відкрити TUI для увімкнення/вимкнення ресурсів пакетів |

---

## Змінні середовища (API-ключі)

| Змінна | Провайдер |
|--------|-----------|
| `ANTHROPIC_API_KEY` | Anthropic Claude |
| `ANTHROPIC_OAUTH_TOKEN` | Anthropic OAuth (альтернатива API key) |
| `OPENAI_API_KEY` | OpenAI GPT |
| `GEMINI_API_KEY` | Google Gemini |
| `OPENROUTER_API_KEY` | OpenRouter |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `GROQ_API_KEY` | Groq |
| `XAI_API_KEY` | xAI Grok |
| `MISTRAL_API_KEY` | Mistral |
| `AZURE_OPENAI_API_KEY` | Azure OpenAI |
| `FIREWORKS_API_KEY` | Fireworks |
| `TOGETHER_API_KEY` | Together AI |
| `CEREBRAS_API_KEY` | Cerebras |
| `NVIDIA_API_KEY` | NVIDIA NIM |

### Системні змінні pi

| Змінна | Опис |
|--------|------|
| `PI_CODING_AGENT_DIR` | Директорія конфігурації (за замовчуванням: `~/.pi/agent`) |
| `PI_CODING_AGENT_SESSION_DIR` | Директорія зберігання сесій (перевизначає `--session-dir`) |
| `PI_PACKAGE_DIR` | Перевизначити директорію пакетів (для Nix/Guix store) |
| `PI_OFFLINE` | Вимкнути мережеві операції при старті (`1/true/yes`) |
| `PI_TELEMETRY` | Перевизначити телеметрію (`1/true/yes` або `0/false/no`) |
| `PI_SHARE_VIEWER_URL` | Base URL для `/share` (за замовчуванням: `https://pi.dev/session/`) |

---

## Приклади

```bash
# Інтерактивний режим
pi

# Інтерактивний режим з початковим промптом
pi "List all .ts files in src/"

# Прикріпити файли до початкового повідомлення
pi @prompt.md @image.png "What color is the sky?"

# Неінтерактивний режим (обробити і завершити)
pi -p "List all .ts files in src/"

# Продовжити попередню сесію
pi --continue "What did we discuss?"

# Назвати сесію
pi --name "Refactor auth module"

# Обрати провайдер та модель
pi --provider openai --model gpt-4o-mini "Help me refactor this code"

# Модель з префіксом провайдера (без --provider)
pi --model openai/gpt-4o "Help me refactor this code"

# Модель з рівнем thinking
pi --model sonnet:high "Solve this complex problem"

# Обмежити cycling конкретними моделями
pi --models claude-sonnet,claude-haiku,gpt-4o

# Cycling з фіксованими рівнями thinking
pi --models sonnet:high,haiku:low

# Read-only режим
pi --tools read,grep,find,ls -p "Review the code in src/"

# Вимкнути один інструмент
pi --exclude-tools ask_question

# Експортувати сесію у HTML
pi --export ~/.pi/agent/sessions/--path--/session.jsonl
```

---

## Моделі

```
# Anthropic
pi --model anthropic/claude-sonnet-4-6
pi --model anthropic/claude-opus-4-8

# Google Gemini (провайдер за замовчуванням)
pi --model gemini-2.5-pro
pi --provider google --model gemini-2.5-flash

# OpenAI
pi --model openai/gpt-4o
pi --model openai/gpt-4o-mini
```
