# Architecture & System Design

> Докладний опис архітектури Soloveyko.AI Video Maker v0.40.5

## Загальна архітектура

Програма побудована за архітектурою **Wails v2** — десктопний фреймворк, що поєднує Go-бекенд з веб-фронтендом (React). Go-частина містить усю бізнес-логіку, фронтенд відповідає лише за UI.

```
┌─────────────────────────────────────────────────────┐
│                    main.go (Entry)                   │
│              Wails Run → App struct                  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────┐      ┌──────────────────────┐  │
│  │   Frontend (React)│◄────►│   App (app.go)       │  │
│  │   via Wails Bind  │      │   Оркестратор        │  │
│  └──────────────────┘      └──────────┬───────────┘  │
│                                        │              │
│                            ┌───────────┼──────────┐   │
│                            ▼           ▼          ▼   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  │
│  │  Pipeline    │ │   API Layer  │ │   Utils      │  │
│  │  Service     │ │  (клієнти)   │ │  (утиліти)   │  │
│  └──────┬───────┘ └──────────────┘ └──────────────┘  │
│         │                                             │
│         ▼                                             │
│  ┌──────────────┐                                     │
│  │  MCP Server  │  ◄── Зовнішні агенти               │
│  └──────────────┘                                     │
└─────────────────────────────────────────────────────┘
```

## Компоненти

### 1. App Struct (`app.go`)

Головний оркестратор. Містить посилання на всі сервіси та виставляє методи у фронтенд через Wails Bind.

**Відповідальність**:
- Ініціалізація всіх сервісів (API, pipeline, utils)
- Маршрутизація викликів від UI до відповідних сервісів
- Обробка подій (логування, статуси задач, галерея)
- Worker Mode (віддалене виконання задач)
- Автооновлення

**Ключові поля**:
- `settings` — `*utils.SettingsService` — всі налаштування
- `pipeline` — `*pipeline.PipelineService` — пайплайн обробки
- `openRouter`, `elevenLabs`, `pollinations`, ... — API-клієнти
- `mcpController` — `*mcpserver.Server` — MCP сервер для агентів

### 2. Pipeline Service (`backend/pipeline/service.go`)

Центральний оркестратор багатоступеневого пайплайну відеовиробництва.

**Потік обробки** (метод `runPipeline`):

```
Текст (вхід)
  │
  ▼
[1] Text Stage (переклад/переписування через OpenRouter LLM)
  │
  ▼
[Control Point] — користувач підтверджує текст (опціонально)
  │
  ├──────────────────────────────┐
  ▼                              ▼
[2] Voice Stage              [3] Image Stage
    (TTS: 5 сервісів)          (генерація: 3 сервіси)
    паралельно                 паралельно
  │                              │
  └──────────────┬───────────────┘
                 ▼
[Subtitle Barrier] — чекає завершення voice + image
                 │
                 ▼
[4] Subtitle Stage (whisper.cpp / AMD / WhisperX / AssemblyAI)
                 │
                 ▼
[5] Montage Stage (FFmpeg — фінальний рендер)
                 │
                 ▼
            final.mp4
```

**Керування паралелізмом**:
- Семафори для subtitle (`standard`, `amd`, `whisperx`) та montage
- Пакетний монтаж (`PrepareMontageBatch` / `WaitForMontageBatch`)
- Barrier-синхронізація між voice/image та subtitle

**Контрольні точки** (Control Points):
- Текст: користувач переглядає результат LLM перед продовженням
- Зображення: підтвердження згенерованих зображень
- Монтаж: перегляд плану монтажу перед рендером

### 3. API Layer (`backend/api/`)

Зовнішні API-клієнти, згруповані за сервісами:

| Сервіс | Файл | Призначення |
|--------|------|-------------|
| OpenRouter | `openrouter.go` | LLM (GPT, Claude, Gemini через єдиний API) |
| Pollinations.ai | `pollinations.go` | Безкоштовна генерація зображень |
| Googler | `googler.go` | Генерація зображень/відео, ремікс |
| ElevenLabs Bot | `elevenlabsbot.go` | TTS через проксі `voiceapi.csv666.ru` |
| ElevenLabs Unlim | `elevenlabsunlim.go` | TTS безліміт через `voicer.mat3u.com` |
| ElevenLabs UA | `elevenlabsua.go` | TTS українська через `11tts.net` |
| ElevenLabs Image | `elevenlabsimage.go` | Генерація зображень ElevenLabs v2 |
| VoiceMaker | `voicemaker.go` | TTS `developer.voicemaker.in` |
| Edge TTS | `edgetts.go` | Безкоштовний Microsoft Edge TTS |
| AssemblyAI | `assemblyai.go` | Хмарна транскрипція |
| Google Parser | `google_parser.go` | Читання Google Sheets/Docs |
| Auth | `auth.go` | Ліцензування (hardware-bound) |
| Telegram | `telegram.go` | Сповіщення через Telegram Bot |

**Особливість**: Усі API-клієнти підтримують:
- Збереження/завантаження API-ключів через `SettingsService`
- Баланс-перевірки (`GetBalance`)
- Named API keys (мультиключова підтримка)
- Rate limiting через семафори
- Колбеки логування (`OnLog`)

### 4. Pipeline Stages

#### 4.1 Text Stage (`backend/pipeline/text.go`)
- Відправка тексту до OpenRouter LLM
- Режими: single-shot або з історією чату (memory mode)
- Підтримка шаблонів `{{content}}`

#### 4.2 Voice Stage (`backend/pipeline/voice.go`)
- Диспетчеризація до одного з 5 TTS-сервісів
- Розбиття довгого тексту на чанки
- Злиття аудіо-файлів через FFmpeg
- Режими: `elevenlabsbot`, `elevenlabsunlim`, `elevenlabsua`, `voicemaker`, `edgetts`

#### 4.3 Image Stage (`backend/pipeline/image.go`)
- Розбиття тексту на сегменти (рядки/речення)
- Генерація промптів через LLM
- Генерація зображень/відео через вибраний сервіс
- Підтримка memory-режиму (збереження контексту персонажів)
- Регенерація окремих зображень

#### 4.4 Subtitle Stage (`backend/pipeline/subtitle.go`)
- 4 движки транскрипції: `standard` (whisper.cpp), `amd`, `whisperx`, `assemblyai`
- Конвертація результату в SRT → ASS
- Karaoke-ефект (word-level alignment)
- Автовизначення орієнтації (PlayRes)

#### 4.5 Montage Stage (`backend/pipeline/montage.go`)
- Побудова складного FFmpeg filter graph
- Ефекти: zoom/pan, sway, xfade/fade переходи
- Субтитри (burn-in), водяні знаки, overlays
- Intro-відео, кастомні водяні знаки
- Автовизначення GPU-кодувальника (NVIDIA/AMD/Apple → libx264 fallback)
- Режим монтажу: single-pass або контрольний (user review)

### 5. Utils Layer (`backend/utils/`)

| Утиліта | Файл | Призначення |
|---------|------|-------------|
| Settings | `settings.go` | 200+ полів конфігурації, JSON-персистенс |
| Engines | `engines.go` | Встановлення бінарників (ffmpeg, whisperx, exiftool) |
| Subtitles | `subtitles.go` | SRT→ASS, JSON→ASS конвертація, karaoke |
| Media | `media.go` | Base64 зображення, тривалість аудіо, мініатюри |
| Sync | `sync.go` | Синхронізація зображень з субтитрами (Levenshtein) |
| Templates | `templates.go` | Шаблони пайплайнів (CRUD) |
| Stats | `stats.go` | CPU/RAM/Disk/GPU моніторинг |
| Gallery | `gallery.go` | In-memory галерея згенерованих зображень |
| History | `history.go` | Легка історія (auto-cleanup 2 дні) |
| Full History | `full_history.go` | Детальна історія з оригінальним/обробленим текстом |
| Production Stats | `production_stats.go` | Щоденна статистика виробництва |
| Updater | `updater.go` | Автооновлення (download + apply) |
| File Loader | `fileloader.go` | HTTP handler для локальних файлів |
| Hardware ID | `hardware_id.go` | SHA-256 хеш MachineGuid/IOPlatformUUID |
| File Logger | `file_logger.go` | Сесійне логування з auto-cleanup 7 днів |
| Text Utils | `text.go` | Чанкінг, Levenshtein, Sanitize filename |
| Zip | `zip.go` | ZIP-розпакування з ZipSlip-захистом |

### 6. MCP Server (`backend/mcpserver/server.go`)

Model Context Protocol сервер для зовнішніх агентів/LLM.

**Інструменти**:
- `set_main_text` / `get_main_text` — текстове поле
- `select_templates` — вибір шаблонів
- `enqueue_task` / `start_queue` — черга задач
- `update_text_control` / `confirm_text_control` — контрольні точки
- `get_gallery_preview` — перегляд галереї
- `navigate` — навігація в UI
- `google_monitor_*` — Google Sheets моніторинг

**Потік**: MCP-клієнт → HTTP endpoint → `Invoker` callback → `app.invokeAgentAction()` → Wails Event → Frontend → Response channel

### 7. Frontend (`frontend/src/`)

#### React Contexts (керування станом):
| Context | Призначення |
|---------|-------------|
| `I18nContext` | Локалізація (uk/en/ru) |
| `ThemeContext` | Темна/світла тема, accent color |
| `QueueContext` | Черга задач пайплайну |
| `TemplateContext` | Шаблони пайплайнів |
| `LoggerContext` | Вивід логів |
| `ServiceContext` | API-сервіси (ключі, баланси) |
| `EditorDraftContext` | Чернетки текстового редактора |
| `GoogleMonitorContext` | Google Sheets моніторинг |
| `ToastContext` | Сповіщення (toast notifications) |

#### Основні UI-компоненти:
- `PipelineSidebar` — налаштування пайплайну (модульний, підкомпоненти)
- `PipelineDashboard` — дашборд виконання
- `MontageEditor` — таймлайн монтажу
- `SystemMonitor` — CPU/RAM/GPU
- `GoogleMonitor` — Google Sheets моніторинг
- `AgentController` — MCP/Agent панель
- `AuthWindow` — ліцензування
- `InitialSetup` / `WelcomeWindow` — перший запуск

## Потоки даних

### Основний потік (Pipeline)
```
Користувач вводить текст
  → Frontend викликає App.ProcessTask() через Wails Bind
    → PipelineService.ProcessTask()
      → ProcessText() → OpenRouter API → перекладений текст
      → [Контрольна точка тексту]
      → Паралельно:
          ProcessVoiceover() → TTS API → voice.mp3
          ProcessImage() → LLM промпти → Image API → images[]
      → Barrier: чекає voice + image
      → ProcessSubtitle() → Whisper/AssemblyAI → subtitles.ass
      → ProcessMontage() → FFmpeg → final.mp4
  → Frontend отримує подію "taskStatus" / "stageStatus"
```

### Worker Mode (віддалене виконання)
```
Віддалений сервер (Railway)
  → App.pollTask() кожні 15с
    → claim task → executeRemoteTask()
      → PipelineService.ProcessTask()
      → sendTaskResult() → сервер
```

### Agent Mode (MCP)
```
Зовнішній MCP-клієнт
  → HTTP POST до MCP Server
    → mcpserver.Server → Invoker callback
      → app.invokeAgentAction()
        → Wails Event "agent:request" → Frontend
        → Frontend обробляє → app.ResolveAgentRequest()
          → Response channel → MCP Server → HTTP Response
```

## Зовнішні інтеграції

| Сервіс | API Endpoint | Призначення |
|--------|-------------|-------------|
| OpenRouter | `openrouter.ai` | LLM (GPT, Claude, Gemini) |
| Pollinations.ai | `pollinations.ai` | Безкоштовні зображення |
| Googler | `googler.fast-gen.ai` | Зображення/відео генерація |
| ElevenLabs Bot | `voiceapi.csv666.ru` | TTS (проксі) |
| ElevenLabs Unlim | `voicer.mat3u.com` | TTS (безліміт) |
| ElevenLabs UA | `11tts.net` | TTS (український) |
| ElevenLabs Image | `api.elevenlabs.io` | Генерація зображень |
| VoiceMaker | `developer.voicemaker.in` | TTS |
| AssemblyAI | `api.assemblyai.com` | Транскрипція |
| Google Sheets/Docs | `googleapis.com` | Парсинг таблиць/документів |
| Auth Server | `*.up.railway.app` | Ліцензування |
| Task Server | `*.up.railway.app` | Worker Mode задачи |
| Telegram Bot | `api.telegram.org` | Сповіщення |
| HuggingFace | `huggingface.co` | Завантаження whisper моделей |

## Паралелізм та семафори

- **Subtitle connections**: окремі ліміти для `standard`, `amd`, `whisperx`
- **Montage connections**: глобальний ліміт одночасних рендерів
- **Image generation**: семафори на рівні API-клієнтів (Pollinations: 3, ElevenLabs Image: 3, Googler: окремі для image/video)
- **Worker polling**: 15с інтервал, паралельне виконання отриманих задач

## Керування конфігурацією

- **Файл налаштувань**: JSON у config directory (~/.soloveyko.ai/ або OS-specific)
- **200+ полів** у `PipelineSettings`: переклад, переписування, озвучка, зображення, субтитри, монтаж, overlays, watermarks, custom stages
- **Named API Keys**: підтримка кількох ключів для кожного сервісу
- **Шаблони**: збереження/завантаження конфігурацій пайплайнів

---
*Оновлено: 2026-04-10*
