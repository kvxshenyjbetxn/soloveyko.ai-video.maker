---
name: soloveyko
description: Use this skill when the user asks to develop, modify, or add features to the 'soloveyko.ai-video.maker.go' project. This skill contains the architectural overview, rules, and best practices for working with this specific Wails + Go + React codebase.
---

# Soloveyko AI Video Maker Development

This skill provides the necessary context and guidelines for developing the **soloveyko.ai-video.maker.go** application.

## 1. Project Overview
**Soloveyko** is a desktop application built using [Wails v2](https://wails.io/), combining a Go backend with a React (TypeScript + Vite) frontend. 
The application acts as a pipeline to generate AI videos. The pipeline consists of stages: Text Generation/Translation/Rewrite, Voiceover (TTS), Image Generation, Subtitle Generation, and finally Montage (using FFmpeg to compile into a final video).

### Core Stack
- **Backend:** Go (Golang)
- **Frontend:** React, TypeScript, Vite, CSS
- **Bridge:** Wails v2 (Go struct methods exposed to JS, `wailsjs` auto-generated bindings; Event-based communication).
- **Media Processing:** FFmpeg (via local binary or system installation).

## 2. Directory Structure & Architecture

### Backend (`backend/`)
- **`pipeline/`**: The core engine of the app. `service.go` orchestrates the stages (Text -> Voice -> Image/Subtitles -> Montage). Each stage has its own file (e.g., `voice.go`, `image.go`, `montage.go`). It uses goroutines, semaphores, and synchronization blocks for concurrency.
- **`api/`**: Integrations with external AI services. Includes OpenRouter, ElevenLabs (TTS, Image, UA, Unlim), VoiceMaker, Pollinations.ai, AssemblyAI, EdgeTTS, and Google integrations.
- **`utils/`**: Helper services for application state. Includes `settings.go`, `history.go`, `production_stats.go`, `templates.go`, and file handlers.
- **`app.go`**: The Wails entry point. Initializes all services from `backend/api`, `backend/utils`, and `backend/pipeline`, passing them dependencies and registering Wails event callbacks (`wruntime.EventsEmit`) to communicate with the frontend.
- **`main.go`**: Bootstraps the Wails application and configures desktop window options (macOS transparent window, colors, asset server).

### Frontend (`frontend/src/`)
- **`App.tsx`**: Main component that handles routing between "tabs" (Text, Queue, Gallery, Settings, Logs). Listens to backend events (e.g., `galleryUpdate`, `applyHistoryEntry`).
- **`tabs/`**: UI pages grouped by feature area (e.g., `text/`, `settings/api/`).
- **`components/`**: Reusable UI parts like `ConfirmModal`, `SystemMonitor`, `HistorySidebar`.
- **`contexts/`**: Context providers for state management (`I18nContext`, `QueueContext`, `LoggerContext`).
- **`locales/`**: JSON localization files (en, uk, ru).

## 3. Strict Development Rules

When developing for this project, you MUST adhere to these rules defined by the user:
1. **Language & Environment:** You are a professional developer in Go and Wails GUI. The OS is Windows, but the app MUST compile and work on both **Windows and MacOS**.
2. **Modularity & Cleanliness:** Maintain strict modularity for ease of development. Separate concerns (API wrappers belong in `api/`, pipeline logic in `pipeline/`, UI components in `frontend/src/components/`). Write clean, well-documented Code.
3. **Go Rules:** Follow standard Go idioms, error handling, and formatting (`gofmt`).
4. **Translations:** The program MUST be translated into Ukrainian (`uk`), English (`en`), and Russian (`ru`). Any new UI text must be added to the respective locale JSON files in `frontend/src/locales/` instead of hardcoded strings. Use the `t('key')` function from `useI18n()`.

## 4. Common Workflows

### Adding a New API Integration
1. **Backend:** Create a new file in `backend/api/new_service.go` defining the struct and methods.
2. **Settings:** If it requires an API key, update `backend/utils/settings.go` to store and load the keys.
3. **App Initialization:** In `app.go`, initialize the new service in `NewApp()` and pass it to the pipeline if needed. Expose getter/setter methods for the frontend.
4. **Frontend API Tab:** Add a new tab component in `frontend/src/tabs/settings/api/` to manage keys and settings. Update `App.tsx` sidebar to list the new tab. Add localizations.

### Updating the Pipeline
1. The pipeline logic is in `backend/pipeline/service.go` (`runPipeline`).
2. If adding a new stage, create a new file like `backend/pipeline/new_stage.go`, implement it, and call it from `service.go`.
3. Use `s.log(...)` and `s.emitStageStatus(...)` to send progress to the UI.
4. Ensure concurrency limits (like `settings.GetMontageMaxConnections()`) are respected if doing heavy work. Use the semaphores pattern already present.

### Adding Frontend Features
- Use the Wails JS bindings found in `frontend/wailsjs/go/main/App.js` to call Go functions. 
- If you add new exposed Go functions to `App` in `app.go`, you must run `wails generate module` or just `wails dev` to regenerate the bindings. (Usually, Vite watcher handles running `wailsjs` updates).
- Make sure to style elements in `App.css` or `style.css` matching the dark aqua aesthetic.
