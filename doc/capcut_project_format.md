# CapCut Project Format — дослідження структури

> CapCut 8.7.0 (Windows), папка чернеток: `E:\capcut\com.lveditor.draft`

---

## Структура папки проекту

```
{draft_name}/                          ← назва = draft_name з meta_info
├── draft_content.json                 ← ГОЛОВНИЙ файл: tracks, materials, canvas, fps
├── draft_content.json.bak             ← автобекап
├── draft_meta_info.json               ← метадані + медіа-пул (draft_materials)
├── draft_virtual_store.json           ← з'являється після додавання медіа в пул
├── draft_agency_config.json           ← конфіг агентства (неважливо)
├── draft_biz_config.json              ← порожній
├── draft_settings                     ← бінарні налаштування
├── timeline_layout.json               ← layout таймлайну в UI
├── template-2.tmp                     ← копія draft_content.json
├── .locked                            ← lock-файл (порожній)
├── adjust_mask/
├── common_attachment/
├── matting/
├── qr_upload/
├── smart_crop/
├── subdraft/
├── Resources/
│   ├── audioAlg/
│   └── videoAlg/
└── Timelines/
    ├── project.json                   ← список таймліній + main_timeline_id
    ├── project.json.bak
    └── {main_timeline_uuid}/
        ├── draft_content.json         ← копія кореневого draft_content.json
        ├── draft_content.json.bak
        ├── template-2.tmp
        ├── template.tmp
        └── common_attachment/
```

---

## Одиниці часу

| Одиниця | Значення |
|---------|----------|
| Мікросекунди | основна одиниця для всіх `duration`, `start` у треках і матеріалах |
| 1 секунда | = 1_000_000 мкс |
| 1 фрейм @ 30fps | = 33_333 мкс |

---

## Що змінюється на якому кроці

| Дія в CapCut | Файл |
|---|---|
| Створення порожнього проекту | `draft_content.json`, `draft_meta_info.json`, `Timelines/project.json` |
| Додавання медіа в **пул** (без таймлайну) | `draft_meta_info.json` (+запис у `draft_materials`), `draft_virtual_store.json` (новий) |
| Перетягування медіа на **таймлайн** | `draft_content.json` (`tracks[]` + відповідні масиви у `materials`) |

---

## Файл: `draft_content.json` — повна схема з таймлайном

### Верхній рівень

```json
{
  "id": "5752C193-F3BA-43ce-ABB4-D1D4BA406AEB",  // main_timeline_id
  "version": 360000,
  "new_version": "171.0.0",
  "fps": 30.0,
  "duration": 57066666,     // мкс — загальна тривалість (найдовший елемент)
  "color_space": 0,         // 0 = SDR; -1 = порожній проект
  "canvas_config": {
    "ratio": "original",
    "width": 1920,
    "height": 1072,         // реальна висота після додавання медіа (може відрізнятись)
    "background": null
  },
  "tracks": [ ... ],        // треки — дивись нижче
  "materials": { ... },     // матеріали — дивись нижче
  "keyframes": { "videos":[], "audios":[], ... }
}
```

---

### `tracks[]` — треки таймлайну

Кожен трек має тип (`"video"` або `"audio"`) і масив сегментів.

#### Відео-трек (тип `"video"`)

```json
{
  "id": "778245E7-D53B-42b4-A146-C9BF8A16F7F2",
  "type": "video",
  "flag": 0,
  "attribute": 0,
  "name": "",
  "is_default_name": true,
  "segments": [ ... ]
}
```

#### Аудіо-трек (тип `"audio"`)

```json
{
  "id": "2C0FF6AB-AB5B-4558-B560-9008AF2CE315",
  "type": "audio",
  "flag": 0,
  "attribute": 0,
  "name": "",
  "is_default_name": true,
  "segments": [ ... ]
}
```

---

### `tracks[].segments[]` — сегмент на треку

Один сегмент = один кліп на таймлайні.

```json
{
  "id": "C228FD0B-0718-4d8f-BD89-A9648986DA3C",

  // Яку частину джерела використовувати (мкс)
  "source_timerange": { "start": 0, "duration": 5000000 },

  // Де розміщений на таймлайні (мкс)
  "target_timerange": { "start": 0, "duration": 5000000 },

  // Посилання на матеріал у materials.videos[] або materials.audios[]
  "material_id": "B01C66A9-09F4-4a44-86BA-88392D3C6D7F",

  // Посилання на допоміжні матеріали (speed, canvas, sound_channel_mapping тощо)
  "extra_material_refs": [
    "27A35353-...",  // → materials.speeds[]
    "099E378F-...",  // → materials.placeholder_infos[]
    "0DAD3EED-...",  // → materials.canvases[]       (тільки відео)
    "2D550F6D-...",  // → materials.sound_channel_mappings[]
    "671532EE-...",  // → materials.material_colors[]  (тільки відео)
    "466498FB-..."   // → materials.vocal_separations[]
  ],

  "speed": 1.0,
  "volume": 1.0,
  "last_nonzero_volume": 1.0,
  "visible": true,
  "reverse": false,
  "is_loop": false,
  "render_index": 0,
  "track_render_index": 0,

  // Трансформації кліпу (тільки відео)
  "clip": {
    "scale": { "x": 1.0, "y": 1.0 },
    "rotation": 0.0,
    "transform": { "x": 0.0, "y": 0.0 },
    "flip": { "vertical": false, "horizontal": false },
    "alpha": 1.0
  },

  "state": 0,
  "source": "segmentsourcenormal",
  "group_id": "",
  "template_id": "",
  "is_placeholder": false,
  "keyframe_refs": [],
  "common_keyframes": [],
  "caption_info": null,
  "hdr_settings": { "mode": 1, "intensity": 1.0, "nits": 1000 },  // тільки відео
  "render_timerange": { "start": 0, "duration": 0 }
}
```

#### Приклад: картинка 0001.jpg (5 сек, з позиції 0)
```json
"source_timerange": { "start": 0, "duration": 5000000 },
"target_timerange": { "start": 0, "duration": 5000000 },
"material_id": "B01C66A9-..."   // → materials.videos[] (тип "photo")
```

#### Приклад: відео 0007.mp4 (8 сек, одразу після картинки)
```json
"source_timerange": { "start": 0, "duration": 8000000 },
"target_timerange": { "start": 5000000, "duration": 8000000 },
"material_id": "D7C37330-..."   // → materials.videos[] (тип "video")
```

#### Приклад: аудіо voice.mp3 (57 сек, з позиції 0)
```json
"source_timerange": { "start": 0, "duration": 57066666 },
"target_timerange": { "start": 0, "duration": 57066666 },
"material_id": "FC37478E-..."   // → materials.audios[]
"clip": null,                   // для аудіо clip = null
"uniform_scale": null           // для аудіо uniform_scale = null
```

---

### `materials.videos[]` — матеріали відео і фото

**Увага: і фото, і відео потрапляють у `materials.videos[]`, не в `materials.images[]`.**

#### Фото (`type: "photo"`)
```json
{
  "id": "B01C66A9-09F4-4a44-86BA-88392D3C6D7F",
  "type": "photo",
  "path": "F:/test/Задача 1/media/0001.jpg",   // абсолютний шлях, прямі слеші
  "material_name": "0001.jpg",
  "width": 1376,
  "height": 768,
  "duration": 10800000000,    // для фото = 3 год (умовна нескінченність)
  "has_audio": false,
  "local_material_id": "",    // для фото порожній (відрізняється від відео!)
  "category_name": "local",
  "source": 0,
  "source_platform": 0,
  "crop": {
    "upper_left_x": 0.0, "upper_left_y": 0.0,
    "upper_right_x": 1.0, "upper_right_y": 0.0,
    "lower_left_x": 0.0, "lower_left_y": 1.0,
    "lower_right_x": 1.0, "lower_right_y": 1.0
  },
  "crop_ratio": "free",
  "crop_scale": 1.0,
  "unique_id": "",
  "media_path": "",
  "reverse_path": "",
  "formula_id": "",
  "check_flag": 62978047,
  "picture_from": "none",
  "live_photo_timestamp": -1,
  // ... багато порожніх/дефолтних полів (matting, stable, video_algorithm тощо)
}
```

#### Відео (`type: "video"`)
```json
{
  "id": "D7C37330-A1B9-4e1d-81DA-7E5356606789",
  "type": "video",
  "path": "F:/test/Задача 1/media/0007.mp4",
  "material_name": "0007.mp4",
  "width": 1280,
  "height": 720,
  "duration": 8000000,        // реальна тривалість відео в мкс
  "has_audio": true,
  "local_material_id": "4ccf6bd4-eec8-48db-96fc-67d1b1df91db",  // = id з draft_meta_info пулу
  // ... решта полів аналогічна фото
}
```

---

### `materials.audios[]` — аудіо матеріал

```json
{
  "id": "FC37478E-23E0-488b-BDEF-E3C4968A1949",
  "type": "extract_music",    // тип для локального аудіо
  "name": "voice.mp3",
  "path": "F:/test/Задача 1/voice.mp3",
  "duration": 57066666,       // реальна тривалість в мкс
  "category_name": "local",
  "local_material_id": "6b52fccb-b994-4e53-8a77-b9c3d937d0b1",  // = id з draft_meta_info пулу
  "source_platform": 0,
  "wave_points": [],
  "music_id": "444352e1-1b26-411a-8b9e-72f53d1ba8d9",  // внутрішній uuid
  // ... решта порожніх полів (tone_type, effect_id тощо)
}
```

---

### Допоміжні матеріали (з `extra_material_refs`)

Кожен сегмент тягне за собою набір допоміжних матеріалів.

#### `materials.speeds[]`
```json
{ "id": "27A35353-...", "type": "speed", "mode": 0, "speed": 1.0, "curve_speed": null }
```

#### `materials.canvases[]` (тільки відео-сегменти)
```json
{ "id": "0DAD3EED-...", "type": "canvas_color", "color": "", "blur": 0.0, "image": "", ... }
```

#### `materials.sound_channel_mappings[]`
```json
{ "id": "2D550F6D-...", "type": "", "audio_channel_mapping": 0, "is_config_open": false }
```

#### `materials.material_colors[]` (тільки відео-сегменти)
```json
{ "id": "671532EE-...", "is_color_clip": false, "is_gradient": false, "solid_color": "", ... }
```

#### `materials.vocal_separations[]`
```json
{ "id": "466498FB-...", "type": "vocal_separation", "choice": 0, "removed_sounds": [], ... }
```

#### `materials.placeholder_infos[]`
```json
{ "id": "099E378F-...", "type": "placeholder_info", "meta_type": "none", "res_path": "", ... }
```

#### `materials.beats[]` (тільки аудіо-сегменти)
```json
{ "id": "3F8E825F-...", "type": "beats", "enable_ai_beats": false, "gear": 404, "mode": 404, ... }
```

---

### Відповідність `extra_material_refs` для кожного типу сегменту

| Позиція | Відео/Фото сегмент | Аудіо сегмент |
|---------|-------------------|---------------|
| [0] | `speeds[]` | `speeds[]` |
| [1] | `placeholder_infos[]` | `placeholder_infos[]` |
| [2] | `canvases[]` | `beats[]` |
| [3] | `sound_channel_mappings[]` | `sound_channel_mappings[]` |
| [4] | `material_colors[]` | `vocal_separations[]` |
| [5] | `vocal_separations[]` | — |

---

## Файл: `draft_meta_info.json` — медіа-пул

Зберігає список всіх завантажених файлів у `draft_materials[type=0].value[]`.

### Поля запису:

```json
{
  "id": "f63bd0ca-...",                           // UUID (прив'язується до local_material_id у videos/audios)
  "file_Path": "F:/test/Задача 1/media/0001.jpg", // абсолютний шлях (прямі слеші)
  "extra_info": "0001.jpg",                        // ім'я файлу
  "metetype": "photo",                             // "photo" | "video" | "music" | "none"
  "width": 1376,
  "height": 768,
  "duration": 5000000,                             // мкс (для music = точна тривалість)
  "create_time": 1780659656,                       // unix сек (mtime файлу)
  "import_time": 1780828044,                       // unix сек
  "import_time_ms": 1780828042994611,              // unix мкс
  "item_source": 1,
  "type": 0,
  "roughcut_time_range": { "duration": -1, "start": -1 },  // -1 для фото; реальна для відео/аудіо
  "sub_time_range": { "duration": -1, "start": -1 }
}
```

### Перший елемент — системна заглушка (завжди присутня):
```json
{ "metetype": "none", "duration": 33333, "width": 0, "height": 0, "file_Path": "" }
```

---

## Файл: `Timelines/project.json`

```json
{
  "id": "{project_uuid}",
  "main_timeline_id": "5752C193-F3BA-43ce-ABB4-D1D4BA406AEB",
  "timelines": [{
    "id": "5752C193-F3BA-43ce-ABB4-D1D4BA406AEB",
    "name": "Временная шкала 01",
    "create_time": 1780827973436594,
    "update_time": 1780827973436594,
    "is_marked_delete": false
  }],
  "create_time": 1780827973436594,
  "update_time": 1780827973436594,
  "version": 0,
  "config": { "color_space": -1, "render_index_track_mode_on": false, "use_float_render": false }
}
```

---

## Кілька треків (рядків таймлайну)

### Правила для `flag` та `track_render_index`

Кожен трек має два поля що визначають порядок відображення:

| Поле | Значення | Сенс |
|------|----------|------|
| `flag` | `0` | Основний відео-трек (нижній шар) АБО будь-який аудіо-трек |
| `flag` | `2` | Додатковий відео-трек (overlay, вище основного) |
| `track_render_index` | `0, 1, 2, 3...` | Глобальний порядок рендеру по всіх треках |
| `render_index` (в сегменті) | `0, 1...` | Порядок всередині одного типу відео-треку |

### Порядок треків у `tracks[]` та `track_render_index`

Порядок у масиві `tracks[]` = порядок відображення в UI (зверху вниз).
`track_render_index` — глобальний лічильник, зростає по всіх треках:

```
tracks[0]: type="video", flag=0  → segments[].track_render_index = 0  (основний відео, низ)
tracks[1]: type="video", flag=2  → segments[].track_render_index = 1  (overlay відео, поверх)
tracks[2]: type="audio", flag=0  → segments[].track_render_index = 2  (перший аудіо)
tracks[3]: type="audio", flag=0  → segments[].track_render_index = 3  (другий аудіо)
```

### `render_index` всередині сегменту

```
Основний відео-трек (flag=0):   render_index = 0
Overlay відео-трек (flag=2):    render_index = 1
Аудіо-треки:                    render_index = 0 (завжди)
```

### Приклад: два відео-ряди + два аудіо

```json
"tracks": [
  { "type": "video", "flag": 0, "segments": [{ "render_index": 0, "track_render_index": 0 }] },
  { "type": "video", "flag": 2, "segments": [{ "render_index": 1, "track_render_index": 1 }] },
  { "type": "audio", "flag": 0, "segments": [{ "render_index": 0, "track_render_index": 2 }] },
  { "type": "audio", "flag": 0, "segments": [{ "render_index": 0, "track_render_index": 3 }] }
]
```

---

## Схема генерації проекту з `timeline.json`

Щоб згенерувати CapCut-проект з нашого `timeline.json`:

1. **Створити папку** `{draft_root}/{project_name}/`
2. **`draft_meta_info.json`** — записати всі медіафайли + аудіо у `draft_materials[type=0].value[]`
3. **`draft_content.json`** — заповнити:
   - `tracks[0]` (video): по одному сегменту на кожен `SegmentTiming`, `target_timerange.start/duration` з `start_secs/duration_secs * 1_000_000`
   - `tracks[1]` (audio): один сегмент на весь аудіофайл
   - `materials.videos[]`: по одному запису на кожен медіафайл (фото → `type:"photo"`, відео → `type:"video"`)
   - `materials.audios[]`: один запис на аудіофайл
   - Для кожного сегменту згенерувати UUID і допоміжні матеріали в `extra_material_refs`
4. **`Timelines/project.json`** — записати `main_timeline_id`
5. **`Timelines/{uuid}/draft_content.json`** — копія кореневого `draft_content.json`
6. **`.locked`** — порожній файл

### Тривалість фото на таймлайні
- `materials.videos[].duration` = `10800000000` (3 год — умовна нескінченність для фото)
- Реальна тривалість задається в `segment.target_timerange.duration`

### Шляхи файлів
- Завжди абсолютні, прямі слеші (`F:/test/media/0001.jpg`)
- Капкат не копіює файли — посилається на оригінальне розташування
