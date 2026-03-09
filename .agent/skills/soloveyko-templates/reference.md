# Soloveyko Template System Reference

## Grouping Schema

When saving a template in `PipelineSidebar.tsx` -> `handleSaveTemplate`, settings must be organized into these specific groups:

### `api`
- All fields ending in `KeyID`.
- Purpose: Ensure API keys are tied to the template if needed.

### `stages`
- Flags: `voiceover`, `image`, `subtitle`, `montage`, `translate`, `rewrite`.
- Purpose: Defines which parts of the pipeline are active.

### `control`
- Flags: `translate`, `image`.
- Purpose: Defines if manual review/control is enabled for these stages.

### `text`
- Fields: `translateModel`, `translatePrompt`, `rewriteModel`, `rewritePrompt`, etc.
- Also includes `Enabled` flags and `OutputPath`.

### `voiceover`
- Root fields: `voiceoverService`.
- `services` sub-object:
    - `elevenlabsbot`: `voiceoverTemplate`.
    - `elevenlabsunlim`: `elevenLabsUnlimVoiceID`, `stability`, etc.
    - `elevenlabsua`: `elevenLabsUAVoiceID`, `model`, etc.
    - `voicemaker`: `voiceMakerVoiceID`, etc.
    - `edgetts`: `edgeTTSVoiceID`, etc.

### `image`
- Root fields: `imageService`, `imageMode`, `imageGenerationMethod`, etc.
- `services` sub-object:
    - `pollinations`: `imageModel`, `imageWidth`, `imageHeight`, etc.
    - `googler`: `imageGooglerModel`, `imageGooglerAspectRatio`, etc.
    - `elevenlabsimage`: `elevenLabsImageAspectRatio`.

### `subtitle`
- Fields: `subtitleService`, `subtitleModel`, `subtitleFont`, `subtitleColor`, etc.
- Includes animation and karaoke settings.

### `montage`
- Fields: `montageResolution`, `montageFPS`, `montageTransitionEffect`, etc.
- Includes watermark and overlay settings.

---

## Flattening Mapping (PipelineSidebar.tsx -> flattenSettings)

The UI expects flat fields. `flattenSettings` performs these critical conversions:

| Template Path | Sidebar State Field |
| --- | --- |
| `stages.voiceover` | `voiceoverEnabled` |
| `stages.image` | `imageEnabled` |
| `stages.subtitle` | `subtitleEnabled` |
| `stages.montage` | `montageEnabled` |
| `control.translate` | `translateControlEnabled` |
| `control.image` | `imageControlEnabled` |
