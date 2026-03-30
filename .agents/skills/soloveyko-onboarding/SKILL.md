---
name: soloveyko-onboarding
description: Essential project context, architecture guidelines, and rules for agents working on the Soloveyko.AI Video Maker project. TRIGGER THIS SKILL immediately when starting any new task, bugfix, or feature in this repository.
---

# Soloveyko.AI Video Maker - Agent Onboarding

Welcome to the Soloveyko.AI Video Maker project. This skill provides you with the fundamental knowledge required to navigate and modify this codebase safely and effectively.

## Agent Instructions: Keep Documentation Alive (YOUR RESPONSIBILITY)
This project evolves rapidly. Because you rely on this documentation to understand the project, future agents will too. 
**MANDATORY RULE**: Whenever you implement a new feature, integrate a new API service, or modify the core pipeline behavior, you MUST proactively **update this very skill file** (`.agents/skills/soloveyko-onboarding/SKILL.md`). Keep the knowledge base accurate and comprehensive for yourself and future agents!

---

# Soloveyko.AI Video Maker - Context & Guidelines

## 1. Project Overview
Soloveyko.AI Video Maker is a high-performance, locally-hosted desktop application built with **Wails v2** (Go backend + React/TypeScript frontend). It is designed for automated, AI-driven video production, orchestrating a complex pipeline of AI services to transform simple text prompts into fully produced videos complete with voiceovers, subtitles, background images/videos, and cinematic montage effects.

## 2. Core Architecture & Pipeline Flow
The application operates as a linear pipeline, managed primarily within the `backend/pipeline/` package. The user initiates a task, and it flows through the following stages:

1.  **Text Generation/Processing (`text.go`)**: 
    - Uses **OpenRouter** LLMs (or direct input) to translate, rewrite, or structure the video script.
2.  **Voiceover Generation (`voice.go`)**: 
    - Converts text to speech using services like **ElevenLabs** (Bot, UA, Unlim), **VoiceMaker**, or **EdgeTTS**.
    - Outputs a main audio file (e.g., `voice.mp3`).
3.  **Subtitle Generation (`subtitle.go`)**: 
    - Extracts timings from the generated audio using speech-to-text engines: **Local Whisper**, **AMD Whisper**, **WhisperX**, or cloud-based **AssemblyAI**.
    - Generates highly customizable `.srt` or `.ass` subtitle files with advanced styling (karaoke effects, outlines, shadows) handled internally or via FFmpeg.
4.  **Visual Asset Generation (`image.go`)**: 
    - Generates images contextually based on the text segments using **Pollinations.ai** or **ElevenLabs Image**.
    - Fetches relevant images via Google search integration (`googler.go`).
5.  **Montage & Assembly (`montage.go`)**: 
    - The most complex part of the backend. It constructs massive **FFmpeg filter graphs** to assemble the audio, images, transitions, effects (sway, zoom), and subtitles into a cohesive 9:16 or 16:9 video format.
    - Supports watermarks and intro video overlays.

## 3. Backend Structure (`backend/`)
The Go backend is firmly decoupled into distinct responsibilities:

- **`app.go`**: The root bridge file. Initializes all services, sets up Wails bindings, handles OS-level events (like folder openings, config management), and wires events between the Go pipeline and the React UI.
- **`backend/api/`**: Integrations with external AI and web services.
  - LLMs: `openrouter.go`
  - Audio/Voice: `elevenlabsbot.go`, `elevenlabsua.go`, `elevenlabsunlim.go`, `edgetts.go`, `voicemaker.go`, `assemblyai.go`
  - Images/Assets: `pollinations.go`, `elevenlabsimage.go`, `googler.go`, `google_parser.go`
    - **Google Parser Integration**: Uses official Google Sheets V4 and Google Docs V1 APIs. Requires `credentials.json` (Service Account) in the root directory. Documents must be shared with the service account email.
  - Other: `telegram.go`, `auth.go`
- **`backend/pipeline/`**: The orchestration heart.
  - `service.go`: Controls the flow state, semaphore concurrency (prevents CPU/GPU overload on heavy tasks), and broadcasts progress/completion events.
  - Specific stage handlers: `text.go`, `voice.go`, `subtitle.go`, `image.go`, `montage.go`.
  - Whisper implementations: `local_whisper.go`, `amd_whisper.go`, `whisperx.go`.
- **`backend/utils/`**: Shared core utilities.
  - **Settings & Context**: `settings.go`, `templates.go`.
  - **Telemetry & Logs**: `stats.go`, `production_stats.go`, `file_logger.go`, `history.go`, `full_history.go`, `hardware_id.go`.
  - **File Operations & Media**: `fs.go`, `fileloader.go`, `zip.go`, `media.go`, `subtitles.go`.
  - **Updates**: `updater.go` checks and applies OTA app updates.
- **`backend/modules/`**: Additional business-logic modules (e.g., `crm` integrations).

## 4. Frontend Structure (`frontend/src/`)
Built with React and Vite. It heavily relies on global Context API for state management, avoiding massive prop-drilling or external stores like Redux.

- **`contexts/`**: Global application states.
  - `ServiceContext.tsx`: API keys, service balances, model selections.
  - `SettingsContext.tsx`: Global user preferences.
  - `LoggerContext.tsx` & `ToastContext.tsx`: App-wide notifications and logging interceptors.
  - `QueueContext.tsx`: Manages the background task queue and parallel processing limits.
  - `TemplateContext.tsx`: Pipeline preset management.
  - `I18nContext.tsx`: Custom localization logic using `locales/*.json`.
  - `ThemeContext.tsx`: UI styling variables.
- **`tabs/`**: The primary views of the application.
  - `text/`, `settings/`, `gallery.tsx`, `logs.tsx`, `queue.tsx`.
- **`components/`**: Reusable micro-UI and floating windows.
  - Major UI constructs: `PipelineDashboard`, `PipelineSidebar`, `MontageEditor`.
  - Modals & Monitors: `SystemMonitor`, `GoogleMonitor`, `ServiceBalanceMonitor`, `AuthWindow`, `UpdateModal`.
- **Styling**: Managed via **Vanilla CSS** modules imported per component (`App.css`, `MontageEditor.css`). There is no Tailwind; custom utility classes map directly to specific UI nodes for rapid styling iterations.

## 5. Building and Running

### Prerequisites
- **Go**: 1.24.0 or higher.
- **Node.js**: Modern LTS version.
- **Wails CLI**: `go install github.com/wailsapp/wails/v2/cmd/wails@latest`
- **FFmpeg, FFprobe & ExifTool**: Crucial for video processing. In production, these are extracted automatically on first run. For dev on Windows, place them in `backend/bin/` or ensure they are present in the system `PATH`.
- **Whisper CLI**: Placed in `backend/bin/` for subtitle extraction.

### Essential Commands (Windows PowerShell)
- **Run Developer Mode**: `wails dev` (Supports Hot Reloading for frontend and auto-recompilation for Go).
- **Production Build**: `wails build -platform windows/amd64`
- **Go Mod Sync**: `go mod tidy`
- **Frontend Sync**: `cd frontend; npm install`

## 6. Development Conventions

### Backend (Go)
1.  **Pipeline Consistency**: When adding a new field to `PipelineSettings` (in `backend/utils/settings.go`), it *must* mirror into the UI state in the frontend contexts, and the logic should be handled explicitly within the corresponding stage handler (`pipeline/montage.go`, etc.).
2.  **Concurrency Safety**: Rely firmly on the semaphores inside `PipelineService` (e.g., `subtitleSem`, `montageSem`). Heavy processes like FFmpeg encoding or Whisper inference must not run unbounded.
3.  **Actionable Logging**: Feed logs out using `app.LogToUI` or the `OnLog` callback bound to services. This ensures logs actually appear in the React "Logs" tab and `app.log` files.
4.  **CLI Execution**: FFmpeg command trees get extremely large. Always use `utils.PrepareHiddenCmd` (found in `proc_windows.go`) on Windows to instantiate `exec.Command` so the user is not spammed by pop-up console windows.

### Frontend (React/TypeScript)
1.  **I18n Compliance**: Never hardcode UI text strings. Use the `useI18n` hook and update `uk.json`, `en.json`, and `ru.json` in `frontend/src/locales/`.
2.  **Wails Bindings**: Do not redefine bridge APIs. Call Go logic via the auto-generated Wails tree: `window.go.main.App.<MethodName>`.
3.  **UI Performance**: Avoid expensive global re-renders. Use localized CSS imports (e.g., `import './Gallery.css'`). 
4.  **State Modifications**: If mutating something that persists (like `Settings` or `API Keys`), update via the respective Go API binding first, wait for a successful response, then update the local React Context state.

### 7. File Structure & Output Conventions
- Project outputs are cleanly structured under the user's defined output folder (e.g., `<Videos_Dir>/Soloveyko/Tasks/TaskName/TemplateName/`).
- Intermediate generation assets created per run:
  - `voice.mp3`: Synthesized dictation.
  - `subtitle.srt` / `.ass`: The timed subtitle map.
  - `images/`: The bucket for Pollinations/ElevenLabs downloaded imagery.
  - `segments.json`: Audio interval mappings.
  - `result.txt`: Final processed transcript.
  - `final.mp4`: The assembled master file.

## 8. Agent Control & MCP Integration
- The app now includes a **local MCP control layer** for high-level agent automation. This is intentionally built around actions like queue control, text draft updates, template selection, and review confirmation instead of DOM-level UI clicking.
- **Frontend shared draft state** lives in `frontend/src/contexts/EditorDraftContext.tsx`. Both `tabs/text/translate.tsx` and `tabs/text/rewrite.tsx` read from this shared context so external agent actions can safely set/read the main draft text even when views remount.
- **Frontend agent bridge** lives in `frontend/src/components/AgentController.tsx`. It listens for `agent:request` Wails runtime events and executes the requested action through existing React contexts (`QueueContext`, `TemplateContext`, `EditorDraftContext`) plus existing Wails bindings.
- **Go-side broker** lives in `app_agent.go`. It emits `agent:request` events into the frontend, waits for `ResolveAgentRequest(...)`, and exposes `GetLocalMCPStatus()` for discovery/debugging.
- **MCP server package** lives in `backend/mcpserver/server.go` and uses the official `github.com/modelcontextprotocol/go-sdk` package with **Streamable HTTP** transport.
- Default local MCP endpoint: `http://127.0.0.1:39245/mcp`
  - Override with env var: `SOLOVEYKO_MCP_ADDR`
- Current MCP tools/actions cover:
  - set/get main text
  - select templates
  - enqueue task
  - start queue
  - continue image control
  - list/update/confirm text-control reviews
  - get queue state
  - clear queue
- Read-oriented MCP tools like `get_main_text`, `get_pending_text_controls`, and `get_queue_state` should keep the actual JSON payload visible in `Content`, not only in structured output, because some MCP clients surface only the text content to the agent model.
- `QueueContext` now also exposes `updateControlDraft(id, text)` so agents can edit a pending mini-editor draft before confirming it.
- The mini editor in `frontend/src/tabs/queue.tsx` keeps its own local textarea state, so it must sync from `task.controlContent` via `useEffect`; otherwise MCP updates can report success while the visible text in the open editor stays stale.
- If you expand this system, prefer adding **high-level app actions** to `AgentController` and exposing them via MCP tools, rather than automating visible button clicks.




