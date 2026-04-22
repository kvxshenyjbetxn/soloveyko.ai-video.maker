# Soloveyko.AI Video Maker — Project Index

> **Версія**: 0.40.7 | **Дата останньої індексації**: 2026-04-22

## Огляд проєкту

**Soloveyko.AI Video Maker** — десктопна програма для автоматизованого виробництва відео. Перетворює текст на готові відеоролики через багатоступеневий пайплайн: переклад/переписування тексту → синтез голосу → генерація зображень/відео → створення субтитрів → фінальний монтаж (FFmpeg).

## Мета

Автоматизація створення контент-відео: YouTube Shorts, TikTok, Instagram Reels тощо. Користувач вводить текст, обирає шаблон та налаштування — програма генерує готове відео з озвучкою, зображеннями, субтитрами та ефектами.

## Стек технологій

| Шар | Технологія |
|-----|-----------|
| **Фреймворк** | Wails v2 (Go + Web) |
| **Backend** | Go 1.25 |
| **Frontend** | React 18 + TypeScript + Vite 3 |
| **Стилі** | Vanilla CSS |
| **i18n** | Custom Context-based (uk, en, ru) |
| **Монтаж** | FFmpeg (embedded binary) |
| **Транскрипція** | whisper.cpp, WhisperX, AMD Whisper, AssemblyAI |
| **LLM** | OpenRouter API |
| **TTS** | ElevenLabs (3 сервіси), VoiceMaker, Edge TTS |
| **Зображення** | Pollinations.ai, Googler.fast-gen.ai, ElevenLabs Image |
| **Агент** | MCP Server |
| **Платформи** | Windows, macOS |

## Основні розділи Wiki

- [[architecture|Архітектура]] — компоненти, модулі та взаємодія
- [[log|Журнал змін]] — хронологія робіт
- [[decisions|Прийняті рішення]] — архітектурні рішення (ADR)

## 📁 Інструкції для агентів (instruction/)

- [[instruction/skill-start-wiki|skill-start-wiki]] — початок сесії (відновлення контексту)
- [[instruction/skill-checkpoint-wiki|skill-checkpoint-wiki]] — збереження прогресу під час роботи
- [[instruction/skill-end-wiki|skill-end-wiki]] — завершення сесії
- [[instruction/skill-index-wiki|skill-index-wiki]] — оновлення index та architecture
- [[instruction/skill-link-wiki|skill-link-wiki]] — зв'язування wiki з Obsidian
- [[instruction/skill-commit-massage|skill-commit-massage]] — генерація commit-повідомлень
- [[instruction/skill-init-wiki|skill-init-wiki]] — ініціалізація wiki для нового проєкту
- [[instruction/skill-migrate-wiki|skill-migrate-wiki]] — міграція wiki до нової версії

## 📐 Правила

- [[SCHEMA]] — правила, заборони та позиційні маркери wiki

## Структура каталогів

```
soloveyko.ai-video.maker.go/
├── main.go                    # Точка входу Wails-додатку
├── app.go                     # Головний App struct (оркестратор)
├── app_agent.go               # MCP Agent інтеграція
├── go.mod / go.sum            # Go-залежності
├── wails.json                 # Конфігурація Wails
├── backend/
│   ├── api/                   # Зовнішні API-клієнти
│   │   ├── openrouter.go      # LLM (чат-комплішни)
│   │   ├── pollinations.go    # Генерація зображень
│   │   ├── googler.go         # Зображення/відео через Google API
│   │   ├── elevenlabsbot.go   # TTS (проксі)
│   │   ├── elevenlabsunlim.go # TTS (безліміт)
│   │   ├── elevenlabsua.go    # TTS (український)
│   │   ├── elevenlabsimage.go # Генерація зображень
│   │   ├── voicemaker.go      # TTS VoiceMaker
│   │   ├── edgetts.go         # Безкоштовний Edge TTS
│   │   ├── assemblyai.go      # Транскрипція (хмарна)
│   │   ├── google_parser.go   # Google Sheets/Docs інтеграція
│   │   ├── auth.go            # Ліцензування
│   │   └── telegram.go        # Telegram-сповіщення
│   ├── pipeline/              # Пайплайн обробки відео
│   │   ├── service.go         # Головний оркестратор пайплайну
│   │   ├── text.go            # Переклад/переписування
│   │   ├── voice.go           # Синтез голосу
│   │   ├── image.go           # Генерація зображень/відео
│   │   ├── subtitle.go        # Субтитри (диспетчер)
│   │   ├── montage.go         # Монтаж FFmpeg
│   │   ├── local_whisper.go   # whisper.cpp
│   │   ├── amd_whisper.go     # AMD GPU whisper
│   │   ├── whisperx.go        # WhisperX
│   │   ├── assemblyai.go      # AssemblyAI (empty)
│   │   └── fs.go              # Файлова система
│   ├── mcpserver/             # MCP Server для агентів
│   │   └── server.go
│   ├── utils/                 # Утиліти
│   │   ├── settings.go        # Налаштування (200+ полів)
│   │   ├── engines.go         # Бінарні залежності
│   │   ├── subtitles.go       # SRT/ASS конвертація
│   │   ├── media.go           # Медіа-утиліти
│   │   ├── sync.go            # Синхронізація зображень
│   │   ├── templates.go       # Шаблони пайплайнів
│   │   ├── stats.go           # Системний моніторинг
│   │   ├── gallery.go         # Галерея зображень
│   │   ├── history.go         # Історія задач
│   │   ├── full_history.go    # Розширена історія
│   │   ├── production_stats.go # Статистика виробництва
│   │   ├── updater.go         # Автооновлення
│   │   ├── version.go         # Версія (0.40.6)
│   │   └── ...інші утиліти
│   └── bin/                   # Embedded бінарники (platform-specific)
├── frontend/
│   └── src/
│       ├── App.tsx            # Головний React-компонент
│       ├── components/        # UI-компоненти
│       ├── contexts/          # React Context (стан)
│       ├── tabs/              # Вкладки: text, settings, gallery, logs, queue, settings/mcp
│       └── locales/           # Переклади (uk, en, ru)
├── models/                    # (порожня, моделі завантажуються runtime)
├── build/                     # Build конфігурації
├── assets/                    # Статичні ресурси
└── .wiki/                     # База знань LLM Wiki
```

## Кількість файлів

- **Go-файлів**: ~55
- **TypeScript/TSX-файлів**: ~80+
- **Мова**: переважно українська та англійська

---
*Індексовано: 2026-04-13*
