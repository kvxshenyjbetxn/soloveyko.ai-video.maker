---
name: soloveyko
description: Main developer skill for Soloveyko.AI Video Maker. You MUST use this skill whenever the user asks to modify, extend, or debug any part of the Soloveyko.AI application, including the Go backend (Wails, FFmpeg pipeline, API integrations) and the React frontend (Vanilla CSS, i18n, Context API). Use it even if the user only mentions a specific file or feature in these areas.
---

# Soloveyko.AI Development Skill

This skill provides the core logic and project-specific conventions for developing **Soloveyko.AI Video Maker**, an AI-powered video creation tool built with Wails v2 (Go backend) and React/TypeScript frontend.

## 🏗️ Project Architecture

Always respect the split between backend logic and frontend presentation:
- **Backend (Go)**: Handles video processing, API integrations (TTS, Image, LLM), and system utilities.
- **Frontend (React)**: Handles the GUI, user settings, and task queue management.
- **Communication**: Done via Wails bindings (see `app.go` and `main.go`).

## 🛠️ Backend Development (Go)

### Adding New API Services
1. Create a new file in `backend/api/` (e.g., `new_service.go`).
2. Implement the service logic (follow patterns in `elevenlabsbot.go` or `openrouter.go`).
3. Register the service in `backend/utils/engines.go` so it can be managed by the application.
4. Ensure appropriate error handling and logging using the project's internal logger.

### Modifying the Pipeline
- The video production pipeline is located in `backend/pipeline/`.
- Stages: `voice.go` → `image.go` → `montage.go` → `subtitle.go`.
- Use the `ffmpeg` skill for complex media transformations, but ensure they are integrated into the Go pipeline correctly.

### Build and Test
- Use `wails dev` for hot-reload development.
- Production build: `wails build -ldflags "-H windowsgui -s -w"` (Windows).

## 🎨 Frontend Development (React + TS)

### Styling Conventions
- **STRICT RULE**: Use **Vanilla CSS**. Avoid TailwindCSS unless explicitly requested by the user.
- Every component should have a corresponding `.css` file in the same directory.
- Use the CSS variables defined in the theme context for consistency.

### Adding New Tabs
1. Create the tab component in `frontend/src/tabs/`.
2. Add necessary translations to `frontend/src/locales/`.
3. Register the new tab in `frontend/src/App.tsx`.

### State Management
- Use React Context API (`frontend/src/contexts/`).
- Major contexts: `I18n`, `Logger`, `Queue`, `Service`, `Template`, `Theme`.

## 🌐 Internationalization (i18n)

- Support **Ukrainian (uk)**, **English (en)**, and **Russian (ru)**.
- Locales are in `frontend/src/locales/`.
- **Usage**:
  ```tsx
  import { useI18n } from '../contexts/I18nContext';
  const { t } = useI18n();
  // ...
  <span>{t('settings.api_key')}</span>
  ```
- When adding new UI elements, **always** add the corresponding keys to ALL translation files.

## 📜 FFmpeg Integration

- The project relies heavily on FFmpeg for montage and subtitles.
- Refer to `backend/pipeline/montage.go` and `backend/pipeline/subtitle.go` for implementation details.
- Use the `ffmpeg` skill to debug or generate complex FFmpeg commands before implementing them in Go.

## 🚨 Critical Rules
1. **Always** check `GEMINI.md` for current project state and high-level architecture.
2. **Never** use hardcoded strings in the UI; always use `i18n`.
3. **Ensure** cross-platform compatibility (Windows and macOS).
4. **Follow** Go best practices (proper naming, error wrapping, concurrent safety).
