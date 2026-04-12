# Development Log

Хронологічний список усіх значущих змін у проєкті.

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
