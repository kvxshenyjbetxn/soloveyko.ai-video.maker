# Gemini CLI — команди та флаги

> `gemini [options] [command] [query]`  
> За замовчуванням запускає інтерактивну сесію. Для неінтерактивного виводу використовуй `-p/--prompt`.

---

## Основні флаги

| Флаг | Опис |
|------|------|
| `-p, --prompt <text>` | Запустити у неінтерактивному (headless) режимі з вказаним промптом. Додається до stdin (якщо є) |
| `-i, --prompt-interactive <text>` | Виконати промпт і продовжити в інтерактивному режимі |
| `-d, --debug` | Запустити в режимі відлагодження (відкрити debug-консоль через F12) |
| `-v, --version` | Вивести версію |
| `-h, --help` | Показати довідку |

---

## Модель та сесія

| Флаг | Опис |
|------|------|
| `-m, --model <model>` | Обрати модель |
| `-r, --resume <id\|"latest">` | Відновити попередню сесію. Використовуй `"latest"` для останньої або індекс (напр. `--resume 5`) |
| `--session-file <path>` | Завантажити сесію з JSON-файлу |
| `--session-id <uuid>` | Почати нову сесію з вказаним UUID |
| `--list-sessions` | Показати доступні сесії поточного проекту та завершити |
| `--delete-session <index>` | Видалити сесію за індексом (перевір через `--list-sessions`) |

---

## Дозволи та безпека

| Флаг | Опис |
|------|------|
| `--approval-mode <mode>` | Режим підтвердження: `default` (запитувати), `auto_edit` (авто-підтвердження редагування), `yolo` (авто-підтвердження всіх дій), `plan` (тільки читання) |
| `-y, --yolo` | Автоматично приймати всі дії (YOLO-режим) |
| `--skip-trust` | Довіряти поточному workspace для цієї сесії |
| `--policy <files>` | Додаткові файли або директорії з policy (через кому або кілька `--policy`) |
| `--admin-policy <files>` | Адміністративні файли policy (через кому або кілька `--admin-policy`) |
| `--allowed-tools <tools>` | *(Застаріло: використовуй Policy Engine)* Інструменти, що запускаються без підтвердження |

---

## Формат виводу

| Флаг | Опис |
|------|------|
| `-o, --output-format <format>` | Формат виводу: `text` (за замовчуванням), `json`, `stream-json` |
| `--raw-output` | Відключити санітизацію виводу моделі (дозволяє ANSI-послідовності). **Увага: небезпечно для ненадійного виводу** |
| `--accept-raw-output-risk` | Приховати попередження безпеки при використанні `--raw-output` |

---

## Розширення та плагіни

| Флаг | Опис |
|------|------|
| `-e, --extensions <list>` | Список розширень для використання. Якщо не вказано — використовуються всі |
| `-l, --list-extensions` | Показати всі доступні розширення та завершити |

---

## Воркспейс та середовище

| Флаг | Опис |
|------|------|
| `-w, --worktree [name]` | Запустити Gemini у новому git worktree. Якщо ім'я не вказано — генерується автоматично |
| `-s, --sandbox` | Запустити у sandbox-режимі |
| `--include-directories <dirs>` | Додаткові директорії для включення у workspace (через кому або кілька `--include-directories`) |
| `--screen-reader` | Увімкнути режим screen reader для доступності |

---

## MCP (Model Context Protocol)

| Флаг | Опис |
|------|------|
| `--allowed-mcp-server-names <names>` | Дозволені імена MCP-серверів |
| `--acp` | Запустити агента в ACP-режимі |
| `--experimental-acp` | *(Застаріло)* Запустити в ACP-режимі (використовуй `--acp`) |

---

## Підкоманди

### `gemini mcp` — керування MCP-серверами

| Команда | Опис |
|---------|------|
| `mcp add <name> <commandOrUrl> [args...]` | Додати MCP-сервер |
| `mcp remove <name>` | Видалити MCP-сервер |
| `mcp list` | Показати всі налаштовані MCP-сервери |
| `mcp enable <name>` | Увімкнути MCP-сервер |
| `mcp disable <name>` | Вимкнути MCP-сервер |

---

### `gemini extensions` — керування розширеннями (aliases: `extension`)

| Команда | Опис |
|---------|------|
| `extensions install <source> [--auto-update] [--pre-release]` | Встановити розширення з git-репозиторію або локального шляху |
| `extensions uninstall [names..]` | Видалити одне або кілька розширень |
| `extensions list` | Показати встановлені розширення |
| `extensions update [<name>] [--all]` | Оновити всі або конкретне розширення до останньої версії |
| `extensions disable [--scope] <name>` | Вимкнути розширення |
| `extensions enable [--scope] <name>` | Увімкнути розширення |
| `extensions link <path>` | Прив'язати розширення з локального шляху (зміни відображаються одразу) |
| `extensions new <path> [template]` | Створити нове розширення з шаблону |
| `extensions validate <path>` | Валідувати розширення з локального шляху |
| `extensions config [name] [setting]` | Налаштувати параметри розширення |

---

### `gemini skills` — керування навичками агента (aliases: `skill`)

| Команда | Опис |
|---------|------|
| `skills list [--all]` | Показати знайдені навички агента |
| `skills enable <name>` | Увімкнути навичку |
| `skills disable <name> [--scope]` | Вимкнути навичку |
| `skills install <source> [--scope] [--path]` | Встановити навичку з git-репозиторію або локального шляху |
| `skills link <path>` | Прив'язати навичку з локального шляху (зміни відображаються одразу) |
| `skills uninstall <name> [--scope]` | Видалити навичку за іменем |

---

### `gemini hooks` — керування хуками (aliases: `hook`)

| Команда | Опис |
|---------|------|
| `hooks migrate` | Мігрувати хуки з Claude Code до Gemini CLI |

---

### `gemini gemma` — керування локальною моделлю Gemma

| Команда | Опис |
|---------|------|
| `gemma setup` | Завантажити та налаштувати локальну модель Gemma |
| `gemma start` | Запустити LiteRT-LM сервер |
| `gemma stop` | Зупинити LiteRT-LM сервер |
| `gemma status` | Перевірити статус локального маршрутизації Gemma |
| `gemma logs` | Переглянути логи LiteRT-LM сервера |

---

## Моделі

```
gemini --model gemini-3.1-pro-preview
gemini --model gemini-3-flash-preview
gemini --model gemini-3.1-flash-lite-preview
gemini --model gemini-2.5-pro
gemini --model gemini-2.5-flash
gemini --model gemini-2.5-flash-lite
gemini --model gemma-4-31b-it
gemini --model gemma-4-26b-a4b-it
```
