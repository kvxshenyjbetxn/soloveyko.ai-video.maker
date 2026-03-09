---
name: soloveyko-templates
description: Specialized skill for managing pipeline templates in Soloveyko.AI. Use this skill whenever adding new settings to the pipeline sidebar, fixing template saving/loading issues, or modifying task creation logic involving templates. Ensure that every new setting is correctly grouped, saved, and applied with template dominance.
---

# Soloveyko Template Management Skill

This skill ensures the integrity and consistency of the template system in Soloveyko.AI Video Maker.

## 🏗️ Template Structure

Templates are stored as JSON files with a structured (grouped) format.
Groups: `api`, `stages`, `control`, `text`, `voiceover`, `image`, `subtitle`, `montage`, `customStages`.

## 🛠️ Adding New Pipeline Settings

When you add a new setting to the Pipeline Sidebar, you **MUST** update these three areas:

### 1. Update `PipelineSettings` Interface
Update the interface in `frontend/src/contexts/TemplateContext.tsx` to include the new field.

### 2. Update `handleSaveTemplate` in `PipelineSidebar.tsx`
The "Save Template" (diskette icon) logic is manual. You MUST add your new field to the correct group:
```typescript
const montageFields = [
    'montageResolution', ..., 'NEW_FIELD' // Add here
];
```

### 3. Update `applyTemplate` in `PipelineSidebar.tsx`
When loading a template, we MUST provide a fallback/default value to prevent `undefined` state issues:
```typescript
return {
    ...prev,
    ...cleanApplied,
    NEW_FIELD: cleanApplied.NEW_FIELD ?? DEFAULT_VALUE, // Add here
    ...
}
```

## 🚨 Template Dominance Rule

- When a user creates a task **without** a template: Use current sidebar settings.
- When a user creates a task **with one or more templates**: The template settings are **PRIMARY**.
- Sidebar settings (except global ones like UI state) should **NOT** bleed into template-based tasks.

## 🔄 Flattening Logic

Templates are saved **grouped** but applied **flat** to the sidebar state.
- Use `flattenSettings` in `PipelineSidebar.tsx` to prepare template data for the sidebar.
- `flattenSettings` MUST handle specialized mappings for `stages` and `control` sections to match the sidebar's expected field names (e.g., `image.enabled` -> `imageEnabled`).

## 📁 Key Files
- `backend/utils/templates.go`: Backend storage logic.
- `frontend/src/contexts/TemplateContext.tsx`: Types and context.
- `frontend/src/components/PipelineSidebar.tsx`: UI logic for Save/Apply.
- `frontend/src/contexts/QueueContext.tsx`: Task creation dominance logic.
