# Codex CLI — команди та флаги

> `codex [options] [prompt]`  
> За замовчуванням запускає інтерактивну сесію. Аналог `-p` з інших CLI — підкоманда `exec`.

**Аналог `-p` (неінтерактивний режим):**
```bash
codex exec "зроби щось"            # запуск з промптом
codex e "зроби щось"               # короткий alias
echo "prompt" | codex exec         # промпт через stdin
codex exec -o out.txt "prompt"     # зберегти відповідь у файл
codex exec --json "prompt"         # вивід як JSONL
```

---

## Основні флаги

| Флаг | Опис |
|------|------|
| `-m, --model <model>` | Обрати модель для агента |
| `-i, --image <file>...` | Прикріпити зображення до початкового промпту |
| `-p, --profile <name>` | Накласти `$CODEX_HOME/<name>.config.toml` поверх базового конфігу |
| `-s, --sandbox <mode>` | Режим пісочниці: `read-only`, `workspace-write`, `danger-full-access` |
| `-C, --cd <dir>` | Вказати робочу директорію агента |
| `--add-dir <dir>` | Додаткова директорія для запису поряд з основним workspace |
| `--search` | Увімкнути live-пошук в інтернеті (інструмент `web_search` без підтвердження) |
| `--no-alt-screen` | Вимкнути alternate screen — TUI в inline-режимі зі збереженням scrollback |
| `-V, --version` | Вивести версію |
| `-h, --help` | Показати довідку |

---

## Конфігурація

| Флаг | Опис |
|------|------|
| `-c, --config <key=value>` | Перевизначити значення з `~/.codex/config.toml`. Приклади: `-c model="o3"`, `-c 'sandbox_permissions=["disk-full-read-access"]'` |
| `--enable <feature>` | Увімкнути feature-flag (повторюваний). Еквівалент `-c features.<name>=true` |
| `--disable <feature>` | Вимкнути feature-flag (повторюваний). Еквівалент `-c features.<name>=false` |
| `--strict-config` | Завершити з помилкою, якщо `config.toml` містить невідомі поля |
| `--oss` | Використовувати open-source провайдер |
| `--local-provider <provider>` | Вказати локальний провайдер: `lmstudio` або `ollama` (разом з `--oss`) |

---

## Дозволи та безпека

| Флаг | Опис |
|------|------|
| `-a, --ask-for-approval <policy>` | Коли запитувати підтвердження перед виконанням команди |
| `--dangerously-bypass-approvals-and-sandbox` | Пропустити всі підтвердження та sandbox. **Вкрай небезпечно** — лише для зовнішньо ізольованих середовищ |
| `--dangerously-bypass-hook-trust` | Запустити хуки без перевірки trust. **Небезпечно** — лише для автоматизації з перевіреними джерелами |

### Значення `--ask-for-approval`

| Значення | Опис |
|----------|------|
| `untrusted` | Запускати лише "довірені" команди (ls, cat, sed) без запиту. Решта — ескалація до користувача |
| `on-failure` | *(Застаріло)* Запускати всі команди без запиту, ескалація лише при помилці. Натомість використовуй `on-request` або `never` |
| `on-request` | Модель сама вирішує, коли запитувати підтвердження |
| `never` | Ніколи не запитувати підтвердження. Помилки виконання одразу повертаються моделі |

---

## Підключення до віддаленого сервера

| Флаг | Опис |
|------|------|
| `--remote <addr>` | Підключити TUI до віддаленого app-сервера. Формат: `ws://host:port`, `wss://host:port`, `unix://`, `unix://PATH` |
| `--remote-auth-token-env <var>` | Ім'я змінної середовища з bearer-токеном для websocket-з'єднання |

---

## Підкоманди

| Команда | Опис |
|---------|------|
| `exec` | Запустити Codex неінтерактивно [alias: `e`] |
| `review` | Запустити code review неінтерактивно |
| `login` | Керування логіном |
| `logout` | Видалити збережені облікові дані |
| `mcp` | Керування зовнішніми MCP-серверами |
| `plugin` | Керування плагінами Codex |
| `mcp-server` | Запустити Codex як MCP-сервер (stdio) |
| `app-server` | *(Experimental)* Запустити app-сервер або пов'язані інструменти |
| `remote-control` | *(Experimental)* Керування daemon app-сервера з увімкненим remote control |
| `app` | Запустити desktop-додаток Codex (встановлює якщо відсутній) |
| `completion` | Згенерувати скрипти автодоповнення для shell |
| `update` | Оновити Codex до останньої версії |
| `doctor` | Діагностика установки, конфігурації, авторизації та runtime |
| `sandbox` | Запустити команди всередині Codex-пісочниці |
| `debug` | Інструменти для відлагодження |
| `apply` | Застосувати останній diff від агента через `git apply` [alias: `a`] |
| `resume` | Відновити попередню інтерактивну сесію (picker або `--last` для останньої) |
| `archive` | Архівувати сесію за id або іменем |
| `unarchive` | Розархівувати сесію за id або іменем |
| `fork` | Форкнути попередню інтерактивну сесію (picker або `--last`) |
| `cloud` | *(EXPERIMENTAL)* Переглянути задачі з Codex Cloud та застосувати зміни локально |
| `exec-server` | *(EXPERIMENTAL)* Запустити standalone exec-server сервіс |
| `features` | Переглянути feature-флаги |

---

## `codex exec` — неінтерактивний режим (аналог `-p`)

> `codex exec [OPTIONS] [PROMPT]`  
> Промпт можна передати аргументом, через stdin (`-`) або обома способами — stdin додається як `<stdin>` блок.

| Флаг | Опис |
|------|------|
| `--skip-git-repo-check` | Дозволити запуск Codex поза Git-репозиторієм |
| `--ephemeral` | Запустити без збереження файлів сесії на диск |
| `--ignore-user-config` | Не завантажувати `$CODEX_HOME/config.toml` (auth все одно використовує CODEX_HOME) |
| `--ignore-rules` | Не завантажувати `.rules`-файли користувача та проекту |
| `--output-schema <file>` | JSON Schema для опису форми фінальної відповіді моделі |
| `--json` | Виводити події в stdout як JSONL |
| `-o, --output-last-message <file>` | Файл для запису останнього повідомлення агента |
| `--color <mode>` | Налаштування кольору виводу: `always`, `never`, `auto` (за замовчуванням) |

### Підкоманди `exec`

| Команда | Опис |
|---------|------|
| `exec resume` | Відновити попередню сесію за id або `--last` для останньої |
| `exec review` | Запустити code review проти поточного репозиторію |

---

## `codex app-server` — сервер без TUI (experimental)

> `codex app-server [OPTIONS] [COMMAND]`  
> Запускає Codex як headless-сервер, до якого підключаються IDE-розширення (VS Code тощо) або інші клієнти.

| Флаг | Опис |
|------|------|
| `--listen <url>` | Transport endpoint. Варіанти: `stdio://` (за замовчуванням), `unix://`, `unix://PATH`, `ws://IP:PORT`, `off` |
| `--stdio` | Використовувати stdio (еквівалент `--listen stdio://`) |
| `--ws-auth <mode>` | Режим автентифікації websocket: `capability-token`, `signed-bearer-token` |
| `--ws-token-file <path>` | Шлях до файлу capability-token |
| `--ws-token-sha256 <hex>` | SHA-256 дайджест capability-token |
| `--ws-shared-secret-file <path>` | Шлях до файлу shared secret для JWT bearer tokens |
| `--ws-issuer <issuer>` | Очікуваний issuer для JWT bearer tokens |
| `--ws-audience <audience>` | Очікувана audience для JWT bearer tokens |
| `--ws-max-clock-skew-seconds <sec>` | Максимальний clock skew при валідації JWT |
| `--analytics-default-enabled` | Увімкнути analytics за замовчуванням (для першочергових клієнтів, напр. VS Code extension) |

### Підкоманди `app-server`

| Команда | Опис |
|---------|------|
| `app-server daemon` | Керування локальним daemon app-сервера |
| `app-server proxy` | Проксі stdio-байтів до запущеного control socket |
| `app-server generate-ts` | *(Experimental)* Згенерувати TypeScript bindings для протоколу |
| `app-server generate-json-schema` | *(Experimental)* Згенерувати JSON Schema для протоколу |

---

## `codex mcp` — керування MCP-серверами

| Команда | Опис |
|---------|------|
| `mcp list` | Показати всі налаштовані MCP-сервери |
| `mcp get` | Отримати інформацію про конкретний MCP-сервер |
| `mcp add` | Додати MCP-сервер |
| `mcp remove` | Видалити MCP-сервер |
| `mcp login` | Авторизуватися в MCP-сервері |
| `mcp logout` | Вийти з MCP-сервера |

---

## Моделі

```
codex --model gpt-5.5
codex --model gpt-5.4-mini
```

---

## Ручне тестування (створення файлу)

Для перевірки роботи неінтерактивного режиму та прав на запис файлів у поточній папці:

```bash
# Через оновлений PATH
codex exec --model gpt-5.4-mini --dangerously-bypass-approvals-and-sandbox "створи файл test.txt у поточній папці з текстом 'hello world'"

# Через прямий шлях у Windows (якщо PATH не оновився)
%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex exec --model gpt-5.4-mini --dangerously-bypass-approvals-and-sandbox "створи файл test.txt у поточній папці з текстом 'hello world'"
```
