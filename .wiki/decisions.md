# ⚖️ Decision Log (ADRs)

Журнал важливих архітектурних та технічних рішень.

| Дата | Рішення | Контекст | Наслідки |
| :--- | :--- | :--- | :--- |
<!-- NEW_DECISION_ENTRY -->
| 2026-04-10 | Використання Markdown для Вікі | Потрібна проста, текстова база знань | Легко читати ШІ та людям |
| 2026-04-10 | `.wiki` залишається в проєкті, Obsidian через Junction | Зручніше редагувати wiki поруч з кодом; Obsidian як переглядач | Obsidian має Junction-посилання; єдине джерело даних — `.wiki/` в репозиторії |
| 2026-04-10 | Відносні шляхи для `subtitles` filter у FFmpeg | Абсолютні шляхи з кирилицею/пробілами ламають парсер опцій FFmpeg (`original_size` error) | Використовуємо `assName` замість `assPath` + `cmd.Dir = finalDir`; працює кросплатформенно |
| 2026-04-12 | Worker short history зберігає exact snapshot remote-задачі | Відновлення на воркері має працювати навіть без локального шаблону з таким самим ім'ям і після рестарту програми | `history.json` тепер може містити `subName` та повний `settingsSnapshot`, включно з уже інжектованими API-ключами |
| 2026-04-12 | Remote metadata передається службовими top-level ключами в `settings` | Вкладені custom-об'єкти в remote settings розплющуються бекендовим `flattenSettings`, тому окремий namespace не зберігався б стабільно | Використовуються `taskType`, `__remoteFolderName`, `__remoteSubName`, `__remoteTemplates`; воркер відновлює canonical `folderName/subName` до створення queue item і запуску пайплайну |
| 2026-04-13 | MCP reverse tunnel запускається й завершується разом із desktop app | Потрібен стабільний доступ до локального MCP із VPS без ручного Tabby/терміналу та без накопичення фонових `ssh.exe` | Додано `MCPAutoForwardEnabled`, запуск `startVPS.bat` у звичайному `cmd.exe`, пошук живого tunnel по `ssh.exe` command line та kill на shutdown |
| 2026-04-13 | Zero-arg MCP tools описуються через явний `NoArgs` schema wrapper | OpenClaw відхиляв tools із порожнім object schema (`object schema missing properties`) і через це агент ламався навіть на простих запитах | Zero-arg tools зберегли поведінку без аргументів, але стали сумісні з клієнтами, що вимагають `properties` у JSON schema |
| 2026-04-13 | MCP session прив'язана до lifetime desktop app процесу | MCP server працює всередині Wails app і тримає `streamable_http` sessions лише в пам'яті поточного процесу | Після закриття/рестарту програми клієнт має створити нову session; `healthz=true` саме по собі не гарантує, що стара OpenClaw session ще валідна |

---
*Записуйте сюди рішення, про які ви можете пошкодувати (або які доведеться пояснювати) через місяць.*

*Нові записи — після <!-- NEW_DECISION_ENTRY --> (зверху!)*
