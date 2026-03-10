# Soloveyko.AI Project Reference

## Directory Map

### Backend (Go)
- `backend/api/`: API clients for external services (ElevenLabs, Google, etc.).
- `backend/pipeline/`: Core logic for video creation.
  - `voice.go`: Voice synthesis stage.
  - `image.go`: Image generation stage.
  - `montage.go`: FFmpeg montage stage.
  - `subtitle.go`: Subtitle generation stage.
- `backend/utils/`: Shared utilities (settings, history, engines).
- `backend/bin/`: Binaries for FFmpeg, FFprobe, Whisper.

### Frontend (React + TS)
- `frontend/src/App.tsx`: Main entry and routing.
- `frontend/src/contexts/`: React Contexts (Theme, I18n, Service, etc.).
- `frontend/src/tabs/`: Major views (Gallery, Settings, Queue, Text).
- `frontend/src/components/`: Reusable components (Modals, Sidebars, Monitors).
- `frontend/src/locales/`: Translation files (`en.json`, `uk.json`, `ru.json`).

## Naming Conventions
- **Go**: Use `PascalCase` for exported functions/structs, `camelCase` for internal ones.
- **Frontend**: Components in `PascalCase`, styles in `camelCase` or matching the component name.
- **CSS**: Use specific class names to avoid global conflicts (e.g., `.task-name-modal`).

## I18n Keys Structure
- `common.*`: Shared strings (Save, Cancel, Error).
- `tabs.*`: Tab names.
- `settings.*`: Settings labels and descriptions.
- `pipeline.*`: Status messages during video processing.

## FFmpeg Usage
The project wraps FFmpeg commands in Go. Key files to check:
- `backend/pipeline/montage.go`: How videos are stitched together.
- `backend/pipeline/subtitle.go`: How SRT files are burnt into the video.
