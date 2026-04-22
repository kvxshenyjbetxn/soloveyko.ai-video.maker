# Development Log

Хронологічний список усіх значущих змін у проєкті.

<!-- NEW_LOG_ENTRY -->

## [2026-04-22] | docs: фіналізація сесії (end wiki) — архітектура та коміт після Googler fallback
- **Задача**: Завершити сесію за `skill-end-wiki`: узгодити wiki з кодом, перевірити ADR та посилання, зафіксувати зміни в репозиторії.
- **Зміни**:
  - [[architecture]]: у розділі «Керування конфігурацією» додано опис `GooglerImageFallbackOrder` / `GooglerVideoFallbackOrder` та `buildFallbackList`; у таблиці API Layer оновлено рядок Googler
  - [[decisions]]: виправлено формат рядка таблиці (подвійний `|`); уточнено текст ADR про Googler fallback
  - Виконано `wails generate module` — у згенерованих біндінгах з’явилися `Get/SaveGooglerImageFallbackOrder` та `Get/SaveGooglerVideoFallbackOrder` (`frontend/wailsjs/go/` у `.gitignore`, оновлюється локально при збірці/dev)
  - Підготовлено git commit (Conventional Commits, українською) коду та wiki
- **Файли**: [[architecture]], [[decisions]], [[log]]
- **Статус**: ✅ Завершено
- **Результат**: Документація відображає потік primary (пайплайн) + налаштовані фалбеки; репозиторій готовий до коміту одним атомарним знімком

## [2026-04-22] | feat(googler): налаштування пріоритету фалбек-провайдерів у GOOGLER
- **Задача**: Дати можливість вручну налаштовувати порядок запасних (fallback) провайдерів для генерації зображень та відео в GOOGLER. Первинний провайдер завжди береться з налаштувань Пайплайну; решта — конфігурований список у API→Зображення→GOOGLER.
- **Зміни**:
  - `backend/utils/settings.go`: у `AppSettings` додано поля `GooglerImageFallbackOrder []string` та `GooglerVideoFallbackOrder []string`; додано `GetGooglerImageFallbackOrder` / `SetGooglerImageFallbackOrder` / `GetGooglerVideoFallbackOrder` / `SetGooglerVideoFallbackOrder` (дефолт: `["flow", "gemini"]`)
  - `app.go`: додано 4 публічні методи `GetGooglerImageFallbackOrder`, `SaveGooglerImageFallbackOrder`, `GetGooglerVideoFallbackOrder`, `SaveGooglerVideoFallbackOrder`
  - `backend/api/googler.go`: хардкодений `allModels := []string{"whisk","flow","gemini"}` замінено на `buildFallbackList(primary, fallbackOrder)` — primary (з пайплайну) завжди перший, далі фалбеки з settings без дублікатів
  - `frontend/src/tabs/settings/api/image/googler.tsx`: нова секція "Пріоритет фалбек-провайдерів" — окремо для зображень та відео; кожен провайдер (whisk/flow/gemini/grok) можна увімкнути/вимкнути та переставити ↑↓; inactive-провайдери показуються сірими з кнопкою "+"
  - `frontend/src/locales/*.json`: додано ключі `fallbackTitle`, `fallbackDesc`, `fallbackImage`, `fallbackVideo`, `fallbackLabel` у блок `api.googlerSettings`
- **Файли**: `backend/utils/settings.go`, `app.go`, `backend/api/googler.go`, `frontend/src/tabs/settings/api/image/googler.tsx`, `frontend/src/locales/uk.json`, `frontend/src/locales/en.json`, `frontend/src/locales/ru.json`, [[log]], [[decisions]]
- **Статус**: ✅ Завершено
- **Результат**: Користувач може в UI вибирати порядок запасних провайдерів (1-й фалбек, 2-й фалбек...) окремо для зображень та відео; первинний провайдер як і раніше задається в Пайплайні

## [2026-04-22] | feat(ui): перемикач стилю інтерфейсу — округлений / строгий
- **Задача**: Додати в налаштуваннях перемикач стилю UI між "Округленим" (поточний) та "Строгим" (Windows 10, без заокруглень).
- **Зміни**:
  - У `Settings` struct додано поле `UIStyle string` з json-ключем `uiStyle`
  - Дефолт `"rounded"` для нових та існуючих конфігів (без поля)
  - У `SettingsService` додано `GetUIStyle()`/`SetUIStyle()` методи
  - У `App` додано wrapper-методи `GetUIStyle()`/`SetUIStyle()`
  - У `ThemeContext.tsx` додано стан `uiStyle` ('rounded'|'sharp'), завантаження з бекенду, функція `setUIStyle`, клас `style-sharp`/`style-rounded` застосовується до `document.body` разом з `theme-*`
  - У `style.css` додано `body.style-sharp * { border-radius: 0 !important }` + виняток для scrollbar thumb (2px)
  - У `general.tsx` після секції акцент-кольору додано секцію "Стиль інтерфейсу" з двома кнопками-опціями
  - Додано переклади `uiStyle`/`uiStyleRounded`/`uiStyleSharp` до `uk.json`, `en.json`, `ru.json`
  - **Фіналізація сесії (wiki)**: оновлено [[architecture]] (ThemeContext, `Settings.uiStyle`, прибрано застаріле про вкладку MCP), дата оновлення в [[index]]; додано ADR у [[decisions]] про глобальний override заокруглень
  - Видалено `startVPS.bat` з репозиторію (локальний SSH tunnel-скрипт; не зберігати в проєкті)
- **Файли**: `backend/utils/settings.go`, `app.go`, `frontend/src/style.css`, `frontend/src/contexts/ThemeContext.tsx`, `frontend/src/tabs/settings/general.tsx`, `frontend/src/locales/uk.json`, `frontend/src/locales/en.json`, `frontend/src/locales/ru.json`, `startVPS.bat` (видалено), `[[architecture]]`, `[[decisions]]`, `[[index]]`
- **Статус**: ✅ Завершено
- **Результат**: Go компілюється чисто; при виборі "Строгий" — всі `border-radius` зникають глобально через CSS клас на `body`, при "Округлений" — повертається звичайний вигляд; налаштування зберігається між запусками. Wiki узгоджена з кодом, рішення зафіксовано в ADR

## [2026-04-15] | fix(googler): виправлено блокування паралельної генерації image/video при rate limit
- **Задача**: Користувач помітив, що коли відео-ліміт Googler API вичерпується і відео-горутини переходять в 5-хвилинне очікування, картинки також перестають генеруватись — хоча image-ліміт ще вільний. Потрібно перевірити та виправити.
- **Зміни**:
  - **Коренева причина**: у `GenerateImage`, `RemixImage`, `GenerateVideo` семафор (`imgSem`/`vidSem`) захоплювався на початку функції через `sem <- struct{}{}` + `defer <-sem` — і утримувався протягом усього циклу retry включно з 5-хвилинним `time.Sleep` після 429. Всі слоти семафора заповнювались сплячими горутинами, нові запити не могли стартувати.
  - Прибрано `sem <- struct{}{}` та `defer <-sem` з початку функцій.
  - Семафор тепер захоплюється безпосередньо перед `generateImageOnce`/`generateVideoOnce`/`remixImageOnce` і відпускається одразу після — тобто обгортає лише активний HTTP-запит.
  - `time.Sleep` (як 5s між спробами, так і 5min rate-limit пауза) виконується **поза семафором**.
- **Файли**: `backend/api/googler.go`
- **Статус**: ✅ Завершено
- **Результат**: image та video генерації тепер повністю незалежні — вичерпання video-ліміту не блокує image-горутини. Семафор як і раніше обмежує максимальну кількість паралельних активних запитів до API.

## [2026-04-15] | fix(montage): виправлено періодичну помилку FFmpeg MP3 decoder + linter cleanup
- **Задача**: Під час монтажу періодично виникала помилка `[aist#83:0/mp3] Error submitting packet to decoder: Invalid data found when processing input`. Потрібно діагностувати та виправити.
- **Зміни**:
  - **Коренева причина**: функція `mergeAudioFiles` у `voice.go` робила просту байтову конкатенацію MP3-файлів через `io.Copy`. Кожен chunk мав свій MP3-заголовок (ID3/Xing/VBR), і при злитті вони ставали "сміттям" всередині фінального `voice.mp3`. FFmpeg `mp3float` декодер періодично натикався на них.
  - Замінено байтову конкатенацію на **FFmpeg concat demuxer** (`-f concat -safe 0 -i list.txt -c copy`) — коректне злиття без перекодування.
  - Для одного файлу — `os.Rename` (без FFmpeg).
  - Виправлено 2 linter-підказки: `if/else if` на `switch` у `voice.go` та `service.go`.
  -.Formatter auto-formatted `service.go` (alignment, trailing whitespace, one-line if→multi-line).
- **Файли**: `backend/pipeline/voice.go` (mergeAudioFiles переписана, імпорти оновлені), `backend/pipeline/service.go` (linter fix + formatter)
- **Статус**: ✅ Завершено
- **Результат**: `voice.mp3` тепер завжди коректний; помилка `Invalid data found` більше не виникатиме. Білд чистий.

## [2026-04-15] | refactor(mcp): видалено SSH tunnel автозапуск і MCP вкладку; bump v0.40.7
- **Задача**: Прибрати весь код, що відповідав за автоматичний запуск `startVPS.bat` та UI-вкладку MCP у налаштуваннях — залишити лише самі MCP-інструменти (server.go).
- **Зміни**:
  - Видалено 3 Go-файли: `app_mcp_forward.go`, `app_mcp_forward_windows.go`, `app_mcp_forward_nonwindows.go` (~290 рядків)
  - Видалено frontend-компонент `frontend/src/tabs/settings/mcp.tsx` (~180 рядків)
  - З `app.go` видалено поля `mcpForwardMu`/`mcpForwardCmd` та виклики `startMCPForwardIfEnabled()` / `stopMCPForwardProcess()` в `startup`/`shutdown`
  - З `backend/utils/settings.go` видалено поле `MCPAutoForwardEnabled` зі struct та обидві функції `GetMCPAutoForwardEnabled` / `SetMCPAutoForwardEnabled`
  - З `App.tsx` видалено import `MCPSettings`, case `settings.mcp` та пункт sidebar
  - З усіх 3 локалей (`uk.json`, `en.json`, `ru.json`) видалено блок `mcpTab` та ключ `settings.mcp`
  - TypeScript оновлено `4.9.5` → `5.9.3`, додано `"ignoreDeprecations": "5.0"` у `tsconfig.json`
  - Версію програми підвищено з `0.40.6` до `0.40.7`
- **Файли**: `app.go`, `app_mcp_forward*.go` (видалено), `backend/utils/settings.go`, `frontend/src/App.tsx`, `frontend/src/tabs/settings/mcp.tsx` (видалено), `frontend/src/locales/*.json`, `frontend/tsconfig.json`, `frontend/package.json`, `backend/utils/version.go`, [[architecture]], [[decisions]], [[index]], [[log]]
- **Статус**: ✅ Завершено
- **Результат**: Кодова база спрощена — 574 рядки видалено, Go та TypeScript компілюються чисто. MCP-інструменти у `backend/mcpserver/server.go` залишились без змін.

## [2026-04-13] | fix(mcp): відновлено видимий запуск bat і зафіксовано reconnect flow
- **Задача**: Повернути стабільний сценарій, де програма просто відкриває `startVPS.bat` у видимому `cmd`, і задокументувати, чому після рестарту desktop app OpenClaw треба перепідключати.
- **Зміни**:
  - У `app_mcp_forward_windows.go` автозапуск tunnel повернуто до прямого `cmd /c start ... startVPS.bat`, щоб користувач бачив окреме вікно батніка
  - У wiki зафіксовано, що `healthz=true` не гарантує валідність старої MCP session, бо `streamable_http` session state живе лише в пам'яті поточного desktop app процесу
  - Додано явний reconnect flow: після закриття/рестарту програми треба дочекатися нового tunnel і створити нову OpenClaw/chat session
- **Файли**: `app_mcp_forward_windows.go`, [[architecture]], [[decisions]], [[index]], [[log]]
- **Результат**: Автозапуск знову працює через звичне видиме вікно `cmd`, а процедура повторного підключення MCP після рестарту програми зафіксована в wiki без двозначностей

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
- **Результат**: Windows-додаток сам піднімає й показує стан SSH reverse tunnel для MCP, а VPS-агенти можуть підключатися до локального MCP через автоматично відкритий `startVPS.bat`

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
*Нові записи — після маркера <!-- NEW_LOG_ENTRY --> (зверху!)*

---
*Додавайте нові записи зверху.*
