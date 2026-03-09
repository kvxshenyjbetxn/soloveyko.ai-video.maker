# Soloveyko.AI Project Rules & Guidelines

This document defines the core development standards and architectural rules for the Soloveyko.AI Video Maker project. All agents and developers MUST follow these rules.

## 🏗️ Architecture & Core Technologies

- **Backend**: Wails v2 (Go). Handles FFmpeg pipeline, API integrations, and system tasks.
- **Frontend**: React with TypeScript.
- **Cross-Platform**: Code must be compatible with both **Windows** and **macOS**.

## 🎨 Frontend & Styling Rules

- **Vanilla CSS Only**: Prefer Vanilla CSS for ALL new components. 
- **TailwindCSS**: Avoid using TailwindCSS unless explicitly requested for a specific task.
- **Glassmorphism & Aesthetics**: UI must look premium, modern, and high-quality. Use smooth gradients, micro-animations, and vibrant (but curated) color palettes.
- **Semantic HTML**: Use proper HTML5 elements and ensure every interactive element has a unique ID.

## 🌐 Internationalization (i18n)

- **Mandatory Localization**: NEVER hardcode strings in the UI.
- **Supported Languages**: Ukrainian (default), English, Russian.
- **Implementation**: Use `useI18n()` from `I18nContext`.
- **Files**: All strings must be added to `frontend/src/locales/` (`uk.json`, `en.json`, `ru.json`).

## 🛠️ Backend & Development Flow

- **Separation of Concerns**: Keep backend logic (API, Pipeline) separate from frontend presentation.
- **API Registration**: New API services must be implemented in `backend/api/` and registered in `backend/utils/engines.go`.
- **FFmpeg**: Use the system's embedded FFmpeg binaries for all video processing tasks.
- **Go Best Practices**: Follow standard Go patterns, ensure proper error handling and thread safety.

## 📁 Critical Project Structure

- `backend/api/`: External service integrations.
- `backend/pipeline/`: Video production workflow (voice -> image -> montage -> subtitle).
- `frontend/src/tabs/`: Major application views.
- `frontend/src/contexts/`: Shared state and logic (Theme, I18n, Service).

## 📜 Compliance

1. **Check every task** against these rules.
2. **Prioritize Visual Excellence** in every frontend change.
3. **Maintain Modularity** to ensure the project remains maintainable.
