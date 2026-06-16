# Інтеграція AGY CLI як агента

## Що таке AGY

`agy` — CLI-інструмент для запуску AI-агентів. Інтерфейс аналогічний до `claude` та `gemini` CLI: приймає промт, виконує інструменти, повертає відповідь.

## Довідка agy --help

```
Usage of agy.exe:
  --add-dir                       Add a directory to the workspace (repeatable)
  -c                              Short alias for --continue
  --continue                      Continue the most recent conversation
  --conversation                  Resume a previous conversation by ID
  --dangerously-skip-permissions  Auto-approve all tool permission requests without prompting
  -i                              Short alias for --prompt-interactive
  --log-file                      Override CLI log file path
  --model                         Model for the current CLI session
  -p                              Short alias for --print
  --print                         Run a single prompt non-interactively and print the response
  --print-timeout                 Timeout for print mode wait (default 5m0s)
  --prompt                        Alias for --print
  --prompt-interactive            Run an initial prompt interactively and continue the session
  --sandbox                       Run in a sandbox with terminal restrictions enabled

Available subcommands:
  changelog       Show changelog and release notes
  help            Show help for subcommands
  install         Configure environment paths and shell settings
  models          List available models
  plugin          Manage plugins (install, uninstall, list, enable, disable)
  plugins         Alias for plugin
  update          Update CLI
```

## Порівняння з Claude та Gemini CLI

| Можливість              | Claude CLI                          | Gemini CLI                          | AGY CLI                              |
|-------------------------|-------------------------------------|-------------------------------------|--------------------------------------|
| Одноразовий промт       | `-p "<prompt>"`                     | `--prompt "<prompt>"`               | `-p "<prompt>"`                      |
| Інтерактивний режим     | `-i "<prompt>"`                     | —                                   | `-i "<prompt>"`                      |
| Пропуск дозволів        | `--dangerously-skip-permissions`    | `--yolo`                            | `--dangerously-skip-permissions`     |
| Вибір моделі            | `--model <model>`                   | `--model <model>`                   | `--model <model>`                    |
| JSON-вивід              | `--output-format stream-json`       | `--output-format json`              | —                                    |
| Відновлення сесії       | `--resume <session_id>`             | `--resume <session_id>`             | `--conversation <id>` або `--continue` |
| Продовжити останню      | `--resume <session_id>`             | `--resume <session_id>`             | `--continue` / `-c`                  |

## Ключові відмінності AGY від Claude/Gemini

### 1. Відновлення сесії
- Claude/Gemini: `--resume <session_id>` — передається конкретний ID
- AGY: `--conversation <id>` — аналог, або `--continue`/`-c` щоб продовжити *останню* розмову

### 2. Виведення
- Claude: потоковий NDJSON (`--output-format stream-json`), кожен рядок — JSON-подія
- Gemini: блоковий JSON в кінці (`--output-format json`), поле `response`
- AGY: вивід у stdout, формат потребує перевірки (`agy models` для діагностики)

### 3. Пропуск дозволів
- Claude: `--dangerously-skip-permissions`
- Gemini: `--yolo` + `--skip-trust`
- AGY: `--dangerously-skip-permissions` (як у Claude)

### 4. Додавання директорій
- AGY має унікальний прапор `--add-dir <path>` — дозволяє додати папки до workspace агента (повторюваний)

## Як реалізовано виклик CLI в проекті

### Правило запуску (критично для Windows)

**НЕ використовуй** `new_cli_command()` для claude/gemini/agy — він використовує `cmd /C` на Windows, що ламає передачу довгих аргументів та промтів. Замість цього:

```rust
// ПРАВИЛЬНО — пряме виконання без cmd /C
let mut cmd = crate::bundle::new_direct_cli_command("agy");

// НЕПРАВИЛЬНО — ламає аргументи на Windows
let mut cmd = crate::bundle::new_cli_command("agy");
```

### Структура виклику у src/api/claude.rs

```rust
// Новий сеанс з потоковим виводом
pub fn call_claude_code_new_session_streaming(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String>

// Відновлення сеансу
pub fn call_claude_code_resume(
    model: &str,
    message: &str,
    session_id: &str,
    ...
) -> Result<String, String>

// Простий одноразовий виклик
pub fn call_claude_code(
    model: &str,
    user_content: &str,
    ...
    allow_tools: bool,
) -> Result<String, String>
```

### Лімітер одночасних запитів

Обидва модулі `claude.rs` та `gemini.rs` мають власний семафор — `ClaudeLimiter` / `GeminiLimiter`. При інтеграції AGY потрібен аналогічний `AgyLimiter`:

```rust
pub struct AgyLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
    max_threads: Mutex<usize>,
}

impl AgyLimiter {
    pub fn get() -> &'static Self {
        static LIMITER: OnceLock<AgyLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| AgyLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
            max_threads: Mutex::new(5),
        })
    }
    // ... acquire() / release() аналогічно до ClaudeLimiter
}
```

## Шаблон нового файлу src/api/agy.rs

```rust
use std::io::{BufReader, Read};
use std::process::Stdio;
use std::sync::{Condvar, Mutex, OnceLock};

pub struct AgyLimiter { /* ... */ }
// impl ... (копіювати структуру з claude.rs або gemini.rs)

/// Новий сеанс AGY (без потокового JSON — чекаємо завершення)
pub fn call_agy_new_session(
    model: &str,
    user_content: &str,
    session_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    on_chunk: impl Fn(&str),
) -> Result<(String, String), String> {
    let _permit = AgyLimiter::get().acquire();

    // НЕ new_cli_command — використовуємо new_direct_cli_command
    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(user_content)
        .arg("--dangerously-skip-permissions")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Додаємо робочу директорію якщо передана
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        format!("Failed to launch agy CLI: {}. Make sure agy is installed and in PATH.", e)
    })?;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        on_chunk(&response);
        Ok((response, session_id.to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("AGY CLI error: {}", stderr))
    }
}

/// Продовжує останню сесію AGY (--continue)
pub fn call_agy_resume_last(
    model: &str,
    message: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(message)
        .arg("--continue")
        .arg("--dangerously-skip-permissions");

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| format!("Failed to launch agy CLI: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Продовжує конкретну сесію AGY (--conversation <id>)
pub fn call_agy_resume(
    model: &str,
    message: &str,
    conversation_id: &str,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let mut cmd = crate::bundle::new_direct_cli_command("agy");
    cmd.arg("--model").arg(model)
        .arg("-p").arg(message)
        .arg("--conversation").arg(conversation_id)
        .arg("--dangerously-skip-permissions");

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| format!("Failed to launch agy CLI: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
```

## Реєстрація в src/api/mod.rs

Після створення `src/api/agy.rs` додай у `src/api/mod.rs`:

```rust
pub mod agy;
```

## Додавання до уніфікованого виклику call_llm

У `src/pipeline/` (або де використовується `call_llm`) додай варіант `Agy`:

```rust
match agent_type {
    AgentType::Claude  => api::claude::call_claude_code(model, prompt, ...),
    AgentType::Gemini  => api::gemini::call_gemini_cli(model, prompt, ...),
    AgentType::Agy     => api::agy::call_agy_new_session(model, prompt, ...),
}
```

## Встановлення AGY

```bash
# Список доступних моделей
agy models

# Встановлення плагінів
agy plugin install <name>
agy plugin list
agy plugin enable <name>
agy plugin disable <name>

# Оновлення CLI
agy update

# Налаштування PATH та shell
agy install
```

## Важливі застереження

1. **Windows PATH**: `new_direct_cli_command` на Windows просто викликає `Command::new("agy")` — бінарник має бути у `%PATH%`. На macOS використовується `find_binary_macos()` для пошуку в нестандартних місцях.

2. **Sandbox режим**: AGY має `--sandbox` для обмеженого середовища. Якщо агент не може виконувати команди — перевір чи не запущено в sandbox-режимі.

3. **Таймаут print**: за замовчуванням `--print-timeout 5m0s`. Для довгих завдань збільши через `--print-timeout 30m0s`.

4. **Директорія workspace**: `--add-dir <path>` можна передавати кілька разів для додавання кількох директорій до контексту агента.
