# ⚖️ Decision Log (ADRs)

Журнал важливих архітектурних та технічних рішень.

| Дата | Рішення | Контекст | Наслідки |
| :--- | :--- | :--- | :--- |
<!-- NEW_DECISION_ENTRY -->
| 2026-04-22 | Googler fallback order зберігається окремо від primary; `buildFallbackList` завжди ставить primary першим | Первинний провайдер з Пайплайну; fallback-и з API→Зображення→GOOGLER | `GooglerImageFallbackOrder` / `GooglerVideoFallbackOrder` — лише запасні; `buildFallbackList` дедуплікує повтор primary; дефолт `["flow","gemini"]` |
| 2026-04-22 | Режим «строгий» UI через глобальний `border-radius: 0` на `body.style-sharp` | Потрібен швидкий перемикач «округлений / Win10-подібний» без переписування десятків CSS-файлів | Один клас на `body` + селектор `body.style-sharp *` з `!important`; для scrollbar thumb залишено мінімальне заокруглення (2px); режим «округлений» не додає правил — візуально як до змін |
| 2026-04-15 | Googler семафор звільняється під час rate-limit sleep | `GenerateImage`/`RemixImage`/`GenerateVideo` утримували `imgSem`/`vidSem` протягом 5-хвилинної паузи після 429 — всі слоти заповнювались сплячими горутинами, і картинки переставали генеруватись поки відео чекали на відновлення ліміту | Семафор тепер обгортає лише безпосередній виклик `generateImageOnce`/`generateVideoOnce`/`remixImageOnce`; sleep відбувається поза семафором; паралелізм image та video генерацій незалежний |
| 2026-04-15 | FFmpeg concat demuxer замість байтової конкатенації MP3 | `mergeAudioFiles` зливала MP3 chunk-и через `io.Copy`, що лишало ID3/Xing заголовки всередині файлу; FFmpeg `mp3float` декодер періодично видавав `Invalid data found` | FFmpeg concat demuxer (`-f concat -safe 0 -c copy`) генерує коректний MP3 без перекодування; впливає на VoiceMaker та Edge TTS (сервіси з чанкуванням тексту) |
| 2026-04-15 | Видалено MCP SSH tunnel автозапуск та вкладку у налаштуваннях | Функція запуску `startVPS.bat` ускладнювала кодову базу і не входить до основного UX; MCP tools залишаються, tunnel запускається вручну | Видалено `app_mcp_forward*.go`, вкладку `settings/mcp.tsx`, поле `MCPAutoForwardEnabled` зі struct та локалей; MCP сервер і інструменти не зачеплені |
| 2026-04-15 | TypeScript оновлено з 4.9 до 5.x, додано `ignoreDeprecations` | TS 4.9 не підтримує сучасний `moduleResolution` та deprecated `esModuleInterop=false`; попередження заважали IDE | Додано `"ignoreDeprecations": "5.0"` у `tsconfig.json`; збірка і type check чисті |
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

*Нові записи — після <!-- NEW_DECISION_ENTRY -->
|| 2026-04-22 | Googler fallback order - buildFallbackList primary завжди перший | Primary провайдер з Пайплайну, fallback-и з окремих налаштувань | GooglerImageFallbackOrder / GooglerVideoFallbackOrder містять лише запасних; buildFallbackList дедуплікує; дефолт [flow,gemini] |
 (зверху!)*
