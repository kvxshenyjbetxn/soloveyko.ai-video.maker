# Development Log

Хронологічний список усіх значущих змін у проєкті.

## [2026-04-13] | feat(mcp): додано автофорвард SSH tunnel і вкладку MCP у налаштуваннях
- **Задача**: Дати VPS-агентам доступ до локального MCP програми без ручного Tabby та показати живий статус tunnel у UI.
- **Зміни**:
  - У `SettingsService` додано прапорець `MCPAutoForwardEnabled` із персистенсом у JSON-конфіг
  - У `App` додано Windows-only lifecycle для `startVPS.bat`: автозапуск після `startup()` і завершення tunnel при `shutdown()`
  - `startVPS.bat` тепер використовується як SSH reverse tunnel script для `127.0.0.1:39245 -> 127.0.0.1:39245`, щоб VPS бачив локальний MCP як `http://127.0.0.1:39245/mcp`
  - Додано вкладку `Налаштування -> MCP` з toggle автозапуску, інформацією про знайдений bat-файл та індикаторами `running/scriptFound/PID`
  - Статус tunnel більше не спирається лише на локальний `exec.Cmd`: застосунок шукає реальний `ssh.exe` за command-line signature і тому коректно бачить уже піднятий tunnel
  - Автостарт не плодить дублікати `ssh.exe`, а при закритті програми завершує знайдений процес tunnel
- **Файли**: `app.go`, `app_mcp_forward.go`, `app_mcp_forward_windows.go`, `app_mcp_forward_nonwindows.go`, `backend/utils/settings.go`, `frontend/src/App.tsx`, `frontend/src/tabs/settings/mcp.tsx`, `frontend/src/locales/uk.json`, `frontend/src/locales/en.json`, `frontend/src/locales/ru.json`, `startVPS.bat`, [[architecture]], [[decisions]], [[index]], [[log]]
- **Результат**: Windows-додаток сам піднімає й показує стан SSH reverse tunnel для MCP, а VPS-агенти можуть стабільно підключатися до локального MCP без ручного відкриття терміналу

## [2026-04-13] | fix(mcp): zero-arg tools стали сумісні з OpenClaw
- **Задача**: Прибрати падіння OpenClaw на етапі читання MCP schema з помилкою `object schema missing properties`.
- **Зміни**:
  - У `backend/mcpserver/server.go` додано `NoArgs` wrapper для tool-ів без аргументів
  - Zero-arg schema оновлено для `continue_image_control`, `get_pending_text_controls`, `google_monitor_scan`, `google_monitor_get_tabs`, `get_queue_state`, `clear_queue`
  - Логіку tool-ів не змінено: сервер, як і раніше, ігнорує placeholder і викликає ті самі backend-дії
- **Файли**: `backend/mcpserver/server.go`, [[architecture]], [[decisions]], [[log]]
- **Результат**: OpenClaw більше не валиться на списку MCP tools, і агент може використовувати Soloveyko MCP без schema-конфлікту

## [2026-04-12] | fix(worker): remote tasks тепер зберігають шаблон і short history на воркері
- **Задача**: Виправити worker mode так, щоб remote-задачі з шаблонами коректно відображали `taskName + template`, а також створювали short history для відновлення на самому воркері.
- **Зміни**:
  - У remote payload додано службові метадані `taskType`, `__remoteFolderName`, `__remoteSubName`, `__remoteTemplates`
  - Воркер при `claim` тепер відновлює canonical `folderName/subName`, додає задачу в UI-чергу в правильному вигляді та запускає `ProcessTask` з окремими `taskName` і `subName`
  - Short history на воркері тепер записується одразу при `claim` і містить `subName` та повний `settingsSnapshot` для exact restore
  - Відновлення з history у `PipelineSidebar` тепер пріоритезує `settingsSnapshot` і не залежить від наявності локального шаблону з таким самим ім'ям
- **Файли**: `app.go`, `backend/utils/history.go`, `frontend/src/contexts/QueueContext.tsx`, `frontend/src/components/PipelineSidebar.tsx`, `frontend/src/components/HistorySidebar.tsx`, [[architecture]], [[log]]
- **Результат**: Remote template-задачі на воркері відображаються й виконуються з правильним шаблоном, а після збою або відміни їх можна відновити з лівої short history навіть без локального шаблону

## [2026-04-12] | fix(montage): ім'я фінального відео тепер лише з назви задачі
- **Задача**: Прибрати назву шаблону з імені фінального `.mp4`, щоб файл називався тільки за `taskName`.
- **Зміни**:
  - Спрощено побудову `outputFile` у `backend/pipeline/montage.go`
  - Для звичайних задач фінальна назва тепер формується лише з sanitized `taskName`
  - `preview_task` як і раніше зберігає результат у `final.mp4`
  - Оновлено [[index]] та [[architecture]] під фінальний стан цієї логіки
- **Файли**: `backend/pipeline/montage.go`, [[index]], [[architecture]], [[log]]
- **Результат**: Фінальне відео більше не включає назву шаблону в імені файлу, а wiki синхронізована з поточним станом проєкту

## [2026-04-10] | fix(montage): виправлено падіння FFmpeg на етапі субтитрів (v0.40.6)
- **Задача**: FFmpeg падував при монтажі з помилкою `Unable to parse "original_size" option value` через абсолютний шлях до `.ass` файлу з кирилицею/пробілами/спецсимволами.
- **Зміни**:
  - Замінено абсолютний шлях до субтитрів на відносний (`assName` замість `assPath`) у `montage.go:1178`
  - Видалено проблемне ручне екранування шляху (\\, \\:, \\ ', тощо)
  - Оновлено версію проєкту з 0.40.5 на 0.40.6
- **Файли**: `backend/pipeline/montage.go`, `backend/utils/version.go`, [[index]], [[architecture]], [[decisions]], [[log]]
- **Результат**: FFmpeg коректно резолвить відносний шлях через `cmd.Dir = finalDir` — працює на Windows та macOS

## [2026-04-10] | Link Wiki — Obsidian інтеграція
- **Задача**: Зв'язати `.wiki` проєкту з Obsidian Vault через Junction.
- **Зміни**:
  - Створено Junction: `D:\library\documents\obsidianData\development\soloveyko.ai-video.maker.go` → `E:\vs-code\soloveykoai\soloveyko.ai-video.maker.go\.wiki`
  - `.wiki` залишається в проєкті як єдине джерело даних
  - Obsidian читає/пише ті ж файли через Junction
- **Файли**: [[decisions]]
- **Результат**: Wiki доступна з обох боків — з редактора коду та з Obsidian

## [2026-04-10] | Повна індексація проєкту (LLM Wiki)
- **Задача**: Первинний аналіз та створення бази знань проєкту.
- **Зміни**:
  - Проаналізовано ~55 Go-файлів, ~80+ TypeScript/TSX-файлів
  - Оновлено [[index]] — загальний огляд, стек технологій, структура каталогів
  - Повністю переписано [[architecture]] — детальний опис усіх компонентів, потоки даних, зовнішні інтеграції
- **Файли**: [[index]], [[architecture]]
- **Результат**: База знань повністю заповнена актуальною інформацією про проєкт v0.40.5

---
*Додавайте нові записи зверху.*
