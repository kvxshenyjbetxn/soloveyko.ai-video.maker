#AGENTS.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

- [Step] → verify: [check]

- [Step] → verify: [check]

- [Step] → verify: [check]

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. Language Protocol

**Strict adherence to the Ukrainian language for all communication.**

**Always respond in Ukrainian**, regardless of the language of the user's prompt (unless explicitly asked to translate or provide code-only output).
**Write all plans**, success criteria, and step-by-step instructions **exclusively in Ukrainian**.

---

## Про проект

**Soloveyko AI Video Maker** — десктопний AI-інструмент для автоматичного виробництва відео-контенту. Приймає текстовий сценарій і проводить його через повний pipeline: переклад/переписування тексту → синтез голосу → генерація зображень → монтаж відео → субтитри → готовий ролик.

**Стек**: Go + [Wails v2](https://wails.io/) (десктоп), React 18 + TypeScript + Vite (UI), Firebase (розподілена синхронізація), MCP (AI-агентне управління).

**Платформи**: Windows (основна), macOS (підтримується). GPU-прискорення: NVIDIA / AMD / Apple Silicon / CPU fallback.

**Firebase-проект**: `soloveyko-video-maker`

---

## Структура проекту

```
main.go              — точка входу, ініціалізація Wails (1366×768)
app.go               — центральна структура App, агрегує всі сервіси, Wails-міст Go↔React
app_agent.go         — MCP-агентний міст, запускає MCP-сервер, транслює команди в події UI

backend/
  api/               — клієнти зовнішніх API (TTS, зображення, LLM, Telegram, Google)
  pipeline/          — оркестратор pipeline (text→voice→image→montage→subtitle)
  mcpserver/         — MCP-сервер (SSE/HTTP Streamable) для AI-агентного управління
  bin/               — вбудовані бінарники: ffmpeg, ffprobe, whisper (embed по платформі)
  utils/             — утиліти: налаштування, галерея, шаблони, версія, статистика, оновлення

frontend/
  src/components/    — UI-компоненти (PipelineDashboard, MontageEditor, AgentController, …)
  src/contexts/      — React-контексти (налаштування, i18n)
  src/tabs/          — вкладки головного інтерфейсу
  wailsjs/           — автогенеровані Go-байндінги для React
```

---

## Архітектура та ключові рішення

### Розподілена архітектура Воркер + Майстер
- **Контекст**: один ПК (Воркер) рендерить відео, інший (Майстер) контролює результати дистанційно.
- **Рішення**: Firebase Firestore + Realtime DB + Storage як шина повідомлень. Воркер пише `status: WAITING_REMOTE_CONTROL` + прев'ю → Майстер підписується → підтверджує/відхиляє через UI.
- **Файли**: `backend/utils/sync.go`, `frontend/src/components/GoogleMonitor.tsx`

### MCP-сервер для AI-агентного управління
- **Контекст**: дозволяє AI-агенту (Copilot, Claude) повністю керувати pipeline через стандартний протокол.
- **Рішення**: локальний MCP-сервер (Echo + go-sdk) на автовибраному порту. `invokeAgentAction` відправляє Wails-подію у React `AgentController`, реєструє UUID-канал і чекає відповіді через `ResolveAgentRequest` — двостороннє RPC між агентом і UI.
- **Зареєстровані інструменти**: `set_main_text`, `get_main_text`, `select_templates`, `enqueue_task`, `start_queue`, `update_text_control`, `confirm_text_control`, `get_gallery_preview`.
- **Файли**: `backend/mcpserver/server.go`, `app_agent.go`, `frontend/src/components/AgentController.tsx`

### Pipeline з контрольними точками
- **Рішення**: pipeline зупиняється після генерації зображень і чекає підтвердження від UI або MCP-агента перед монтажем. Аналогічно для тексту (`WAITING_TEXT_CONTROL`) і монтажу (`WAITING_MONTAGE_CONTROL`).
- **Callbacks**: `OnRequestControl`, `OnRequestImageControl`, `OnRequestMontageControl` в `service.go`.
- **Файли**: `backend/pipeline/service.go`, `backend/pipeline/image.go`, `backend/pipeline/montage.go`

### GPU-прискорення монтажу (FFmpeg)
- **Рішення**: автодетекція GPU через `utils/engines.go`. Порядок пріоритету: NVIDIA (h264_nvenc) → AMD (h264_amf) → Apple (h264_videotoolbox) → CPU (libx264).
- **Файли**: `backend/utils/engines.go`, `backend/pipeline/montage.go`

### Вбудовані бінарники (embed)
- **Рішення**: ffmpeg, ffprobe, whisper вбудовані в бінарник через `embed.FS` з платформо-специфічними файлами `embed_windows.go` / `embed_darwin.go`. Розпаковуються у тимчасову директорію при старті.
- **Файли**: `backend/bin/embed_windows.go`, `backend/bin/embed_darwin.go`

---

## Нюанси та застереження

### API-ключі та проксі
- ElevenLabs Bot використовує кастомний проксі `voiceapi.csv666.ru` (не офіційний endpoint).
- OpenRouter має семафор паралелізму — не надсилати запити без урахування ліміту.

### Firebase CLI
- Завжди перевіряти активний проект: `firebase use soloveyko-video-maker`.
- Rules деплоїти через `scripts/firebase-deploy-rules.mjs`, не вручну.

### Платформо-специфічні процеси
- Запуск зовнішніх процесів реалізований окремо: `proc_windows.go` та `proc_others.go` — не змішувати логіку.
- Cmd-утиліти аналогічно: `utils/cmd_windows.go` / `utils/cmd_unix.go`.

### Wails-байндінги
- При зміні сигнатур Go-методів в `App` — регенерувати байндінги командою `wails generate module`.
- Байндінги знаходяться у `frontend/wailsjs/go/` — не редагувати вручну.

---

## Вирішені проблеми (помилки та нюанси)

_Тут хронологічно фіксуються нетривіальні проблеми після їх вирішення._