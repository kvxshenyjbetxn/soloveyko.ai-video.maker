# Lumean Public API — справочник для LLM

> Этот файл — самодостаточная спецификация публичного API Lumean для языковых моделей.
> Отдайте его нейросети целиком: в нём есть авторизация, соглашения, все эндпоинты с точными
> полями, enum'ы, схемы ответов, real-time (WebSocket) и сквозные рабочие примеры.
> Все идентификаторы, имена полей, enum-значения и JSON приведены **дословно** — используйте их
> буква-в-букву. Пояснения на русском, технические имена — в оригинале.

Машиночитаемые спецификации (истина о схемах, дополняют этот файл):
`GET /docs/public-openapi.json` (OpenAPI 3.1, REST) и `GET /docs/public-asyncapi.json` (AsyncAPI 2.6, WebSocket).

---

## 0. Что это за API

Lumean — SaaS для генерации медиа ИИ: **озвучка текста (TTS)**, **генерация и редактирование
изображений**, **звуковые эффекты (SFX)**, **музыка**, **клонирование голоса**. Работа строится
вокруг **заказов** (`order`): вы отправляете задачу, сервер разбивает её на чанки, обрабатывает
асинхронно воркерами и отдаёт файлы-результаты.

Публичный API (`/api/public/*`) предназначен для программных интеграций **по API-ключу** (заголовок
`X-API-KEY`). Это отдельный контур от фронтового JWT-API. Ключ создаётся пользователем в личном
кабинете (веб), у него набор **прав** (permissions) и **лимиты**. Здесь описан ТОЛЬКО публичный
контур по ключу.

**Ключевые факты, которые часто понимают неверно (прочитайте до генерации запросов):**

1. **TTS-голос задаётся только через шаблон**, а не полем заказа. Нет `orders.voice_id`.
   Порядок: положить `voice_id` в `config.tts_settings.voice_id` шаблона → создать заказ по этому
   шаблону. См. §11 (Рецепты).
2. **Текст для TTS передаётся в top-level поле `input_text`** запроса `POST /orders` (сырая строка).
   НЕ кладите текст TTS в `task_data`. А вот для template-less SFX/music промпт идёт в `task_data`.
3. **У заказов НЕТ update/delete.** Доступны только `index`, `store`, `show`, плюс действия
   `cancel`/`retry`. `PUT/PATCH/DELETE /orders/{order}` вернут 405/404.
4. **Два независимых лимита:** rate-limit по числу запросов (429, без `Retry-After`) и токен-квота
   по объёму генерации (429, `reason: token_quota_exceeded`, **с** `Retry-After`). См. §6.
5. **Пагинация с разными базами:** внешние каталоги голосов (`voices/elevenlabs/library`, `voices/heygen`)
   считают `page` **с 0**; обычные списки (`orders`, `templates/browse`) — `current_page` **с 1**.
6. **Субтитры и другие сервисные файлы лежат ОТДЕЛЬНО** — в `result.service_files[]`, не в
   `result.files[]`. Оба поля — плоские массивы строк-путей (напр. `.../output/final/service/subtitles.srt`),
   генерируется только для TTS. Скачивается тем же `POST /storage/url`. См. §7.10.
7. **Сбой чанка чинится поштучно.** У чанка бывают статусы `failed` (тех. ошибка) и `policy_flagged`
   (контент отклонён политикой провайдера). Частично готовый заказ (`partially_completed`) доводят
   до `completed` через `POST /orders/{order}/items/{item}/retry` по каждому сбойному чанку — НЕ через
   order-level `retry`. При `policy_flagged` повтор с тем же текстом бессмыслен: передайте
   исправленный `text`. См. §7.11.

---

## 1. TL;DR — путь от нуля до готового TTS-заказа

Предполагается, что у вас уже есть API-ключ с правами `orders.write`, `templates.write`,
`voices.read`, `orders.download`, `billing.read`. **Внимание:** готовый пресет `automation` НЕ
включает `templates.write` (только `templates.read`) — для этого сценария (создание шаблона)
берите пресет `full` либо добавьте `templates.write` к `automation` вручную при создании ключа.

```bash
BASE=https://api.lumean.app/api/public
KEY=<ваш_api_ключ>

# 1. Найти voice_id (например, из публичной библиотеки ElevenLabs)
curl -s "$BASE/voices/elevenlabs/library?page=0&page_size=20" -H "X-API-KEY: $KEY"

# 2. Создать шаблон TTS с этим voice_id
curl -s -X POST "$BASE/templates" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" -d '{
  "service_key": "elevenlabs",
  "name": "My TTS",
  "config": {
    "tts_settings": {
      "mode": "mode_v1",
      "model_id": "eleven_multilingual_v2",
      "voice_id": "<VOICE_ID_ИЗ_ШАГА_1>",
      "voice_settings": { "stability": 0.5, "similarity_boost": 0.75, "use_speaker_boost": true, "speed": 1.0 }
    }
  }
}'
# → data.id = <TEMPLATE_UUID>

# 3. Создать заказ по шаблону (текст в input_text)
curl -s -X POST "$BASE/orders" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" -d '{
  "template_id": "<TEMPLATE_UUID>",
  "input_text": "Привет! Это тестовая озвучка через Lumean API."
}'
# → data.id = <ORDER_UUID>, data.status = "created"

# 4. Дождаться готовности: опрос статуса (или WebSocket, см. §10)
curl -s "$BASE/orders/<ORDER_UUID>" -H "X-API-KEY: $KEY"
# ждём data.status = "completed" (или "partially_completed")

# 5. Забрать файл-результат: путь берём из data.result.files[] (элемент — строка-путь)
curl -s -X POST "$BASE/storage/url" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" \
  -d '{ "path": "<строка_из_result.files[]>" }'
# → data.url = временная ссылка; качаем обычным GET
```

В шаге 2 `tts_settings.mode` — обязателен, а `tts_settings.language_code` — опциональный ISO-639-1
код (не показан выше для краткости минимального примера); подробнее — §8.2.

---

## 2. Базовый URL и окружения

| Что | Значение (production) |
| --- | --- |
| REST base | `https://api.lumean.app/api/public` |
| WebSocket host / port / scheme | `ws.lumean.app` / `443` / `wss` |
| WS app key (публичный) | `7c4dd881d8116ac76fe89aff71d8c2ff` |
| WS auth endpoint | `POST https://api.lumean.app/api/public/broadcasting/auth` |
| OpenAPI (REST) | `https://api.lumean.app/docs/public-openapi.json` |
| AsyncAPI (WS) | `https://api.lumean.app/docs/public-asyncapi.json` |

Все запросы — HTTPS. Тело — JSON (`Content-Type: application/json`), кроме загрузки файлов
(`multipart/form-data`). Ответы — JSON в UTF-8.

---

## 3. Аутентификация — заголовок `X-API-KEY`

Каждый запрос обязан нести заголовок:

```
X-API-KEY: <ваш_ключ>
```

Ключ хешируется (sha256) и резолвится с кэшем ~1 час. При успехе сервер действует от имени
владельца ключа.

**Ошибки авторизации (HTTP 401), тело `{ "success": false, "message": <строка>, "data": null }`:**

| Причина | message-ключ |
| --- | --- |
| Нет заголовка | `auth.api_key.missing` |
| Неизвестный / неактивный ключ / удалённый или заблокированный пользователь | `auth.api_key.invalid` |
| Ключ истёк (`expires_at` в прошлом) | `auth.api_key.expired` |

### Права (permissions)

Каждый эндпоинт требует конкретное право. Ключ без нужного права → **HTTP 403**
`{ "success": false, "message": <auth.api_key.forbidden>, "data": null }`.

| Право | Открывает |
| --- | --- |
| `orders.read` | Чтение заказов, чанков, WebSocket-подписка на заказы |
| `orders.write` | Создание/отмена/повтор заказов, retry/regenerate чанков |
| `orders.download` | `POST /storage/url` — ссылки на файлы-результаты |
| `templates.read` | Чтение шаблонов, `browse`, `config-options` |
| `templates.write` | Создание/изменение/удаление шаблонов |
| `voices.read` | Все каталоги голосов (LumVoice + ElevenLabs + HeyGen) |
| `billing.read` | `usage`, `subscriptions`, (при включённом кошельке) `wallets`/`ledger-transactions` |
| `profile.read` | `GET /user` (профиль) |
| `*` (wildcard) | Проходит любую проверку прав |

**Пресеты прав** (набор, который обычно назначают ключу):
- `read_only` = все read-права: `orders.read`, `templates.read`, `voices.read`, `billing.read`, `profile.read`, `orders.download`.
- `automation` = `orders.read`, `orders.write`, `orders.download`, `templates.read`, `voices.read`, `billing.read`, `profile.read`.
- `full` = все права.

> Можно перечислить несколько прав через запятую в требовании — достаточно любого. `orders.download`
> — это отдельное право для скачивания (не покрывается `orders.read`).

---

## 4. Соглашения

### Конверт ответа

**Успех** (200/201):
```json
{ "success": true, "message": "<человекочитаемо>", "data": <payload> }
```

**Ошибка домена/инфраструктуры** (400, 404, 429 rate-limit, 500, 503 и пр.):
```json
{ "success": false, "message": "<причина>" }
```
Ключ `errors` в публичном API на этих кодах не используется — не путайте с 422 ниже, где он
обязателен. Некоторые доменные 4xx (напр. 402/409 биллинга, 429 квоты) несут дополнительные
машиночитаемые поля поверх `success`/`message` — см. §12.

**Ошибка валидации** (422) — стандартный формат Laravel, **без** ключа `success`:
```json
{ "message": "<итоговое>", "errors": { "field.name": ["текст ошибки", "..."] } }
```

### Пагинация

Пагинированные списки кладут в `data`:
```json
{ "items": [ ... ], "current_page": 1, "last_page": 5, "per_page": 20, "total": 93 }
```
Параметры запроса: `page` (с 1), `per_page` (обычно 1..100). **Исключение:** внешние каталоги
голосов используют свою обёртку `{ voices, total, page, page_size, has_more }` с `page` **от 0**.

### Типы идентификаторов

| Ресурс | Тип `id` |
| --- | --- |
| Order | **UUID** (строка) |
| Template | **UUID** (строка) |
| User, ApiKey, Subscription, Service | **integer** |
| LumVoice | id голоса (в пути `voices/{voice}`) |
| Внешний голос (ElevenLabs/HeyGen) | строковый `voice_id` провайдера |

Даты — ISO-8601 (`2026-07-08T12:34:56+00:00`). Локаль ответа — по `Accept-Language`/настройке
пользователя (влияет на локализованные `message`/labels).

---

## 5. Типы заказов (`task_type`)

Enum `task_type` (дословно): `tts`, `image`, `txt2img`, `remix`, `image_edit`, `voice_clone`,
`sfx`, `music`.

Как создаётся каждый тип на `POST /orders`:

| task_type | Способ | Что передавать |
| --- | --- | --- |
| `tts` | по шаблону | `template_id` + `input_text` (текст озвучки) |
| `image` / `txt2img` / `remix` / `image_edit` | по шаблону | `template_id` + `input_text` (промпт) и/или `input_files[]` (для edit/remix) |
| `voice_clone` | по шаблону/сервису | `input_files[]` (сэмплы голоса), параметры в `task_data`/шаблоне |
| `sfx` | template-less | `task_type: "sfx"` + `task_data` (см. §8.1) |
| `music` | template-less | `task_type: "music"` + `task_data` (см. §8.1) |

Правило: на `POST /orders` обязателен **один из** `template_id` **или** `task_type`. Шаблон
однозначно задаёт сервис и его настройки; `task_type` — для генеративных сервисов без шаблона.

### Статусы заказов (`order.status`)

Enum `OrderStatus` (дословно, 9 значений): `created`, `pending`, `in_progress`, `completed`,
`result_delivered`, `failed`, `compensated`, `cancelled`, `partially_completed`.

**Терминальные** (заказ достиг финального состояния, дальше не меняется): `completed`,
`result_delivered`, `failed`, `compensated`, `cancelled`. **Нетерминальные**: `created`, `pending`,
`in_progress`, `partially_completed` — последний не терминален специально: его можно доретраить в
`completed` через `POST /orders/{order}/items/{item}/retry` (§7.2).

Строгий фильтр `status` в `GET /orders` (§7.1) принимает только эти значения — иное → 422.

---

## 6. Лимиты: rate-limit и токен-квота

Это **два разных** механизма, оба возвращают 429 — различайте по телу.

### 6.1 Rate-limit (число запросов)

Ограничение частоты по окнам ключа (`limits.requests.per_minute|per_hour|per_day`). Превышение →
**HTTP 429**, тело `{ "success": false, "message": "..." }`. Заголовка `Retry-After` (и
`X-RateLimit-*`) здесь **нет** — это не токен-квота (та несёт `Retry-After`, см. §6.2);
ориентируйтесь на настроенное окно ключа и делайте backoff самостоятельно. Ключ без
заданных request-лимитов новым создаётся с серверным дефолтом (по умолчанию 60/мин), не безлимитный.

### 6.2 Токен-квота (объём генерации)

Поверх rate-limit'а, но считает **токены** — учётную единицу объёма работы сервиса
(`токены = стоимость операции в токенах`; это НЕ деньги/LMC). Окна: `per_minute`, `per_hour`,
`per_day`, `per_month` из `limits.tokens.*` ключа. Ключ без токен-лимитов — без ограничения по токенам.

Проверяется **до** создания при `POST /orders` (по оценке), `POST /orders/{order}/retry`
(по сумме чанков оригинала) и `.../regenerate`. Превышение → **HTTP 429** + `Retry-After`:

```json
{
  "success": false,
  "message": "API key token quota exceeded",
  "reason": "token_quota_exceeded",
  "window": "per_minute",
  "limit": 10000,
  "used": 9800,
  "requested": 500,
  "reset_at": "2026-07-08T13:00:00+00:00",
  "retry_after": 42
}
```
`window` — первое нарушенное окно. При 429 по квоте заказ **не создаётся**.

---

## 7. Справочник эндпоинтов

Формат: `МЕТОД путь` — `право` — параметры → ответ. Все пути относительны base `…/api/public`.

### 7.1 Orders

- **`GET /orders`** — `orders.read`
  Query: `page` (int≥1), `per_page` (int 1..100, деф. 20), `template_id` (uuid).
  `status` — строгий enum (невалидное значение → 422): `created`, `pending`, `in_progress`,
  `completed`, `result_delivered`, `failed`, `compensated`, `cancelled`, `partially_completed`
  (терминальные статусы — см. §5).
  `task_type` — строка или массив (`?task_type[]=tts&task_type[]=sfx`); мягкая валидация:
  неизвестный тип → пустой список, не ошибка.
  → `data` = пагинированный список `Order` (только ваши заказы).

- **`POST /orders`** — `orders.write` — см. §8.0 (тело). → 201, `data` = `Order`.

- **`GET /orders/{order}`** — `orders.read`. → `data` = `Order` (с `items`). 404/403.

- **`POST /orders/{order}/cancel`** — `orders.write`.
  Отменяет заказ в статусе `created`/`pending`/`partially_completed`, разблокирует средства,
  возвращает токен-квоту. → `data` = `Order`. Недопустимо → 400.

- **`POST /orders/{order}/retry`** — `orders.write`.
  Создаёт **новый** заказ с теми же параметрами (исходный должен быть `completed`/`result_delivered`).
  Оценка квоты = сумма `price_units` чанков оригинала → 429 при превышении. → 201, `data` = новый `Order`.

### 7.2 Order Items (чанки заказа)

Все — ownership по заказу (право `orders.*`).

- **`GET /orders/{order}/items`** — `orders.read` → `data` = массив `OrderItem` (без пагинации).
- **`GET /orders/{order}/items/{item}`** — `orders.read` → `data` = `OrderItem`.
- **`GET /orders/{order}/items/{item}/text`** — `orders.read` →
  `data` = `{ "text": string|null, "length": int, "original_text_length": int }`.
- **`POST /orders/{order}/items/{item}/retry`** — `orders.write`.
  Тело: `{ "text"?: string(max 50000) }`. Повторяет **сбойный** чанк — `policy_flagged` **или**
  `failed` (заказ при этом `partially_completed`). Токен-квоту НЕ тратит (использует уже залоченный
  остаток `price_units − consumed_units`). Без `text` повторяется **тот же** текст; с `text` —
  заменяет текст чанка (длина ≤ `original_text_length`). Лимит попыток — 5. Недоступно для
  `sfx`/`music`. → 201, `data` = `OrderItem`. **При `policy_flagged` шлите исправленный `text` —
  см. плейбук §7.11.**
- **`POST /orders/{order}/items/{item}/regenerate`** — `orders.write`.
  Тело: `{ "text": string(required, max 50000) }` (пустой → 422; длина ≤ `original_text_length`).
  Платная перегенерация **`completed`**-чанка (не сбойного!), квота-precheck → 429. Недоступно для
  `sfx`/`music`. → 201, `data` = `OrderItem`.

### 7.3 Storage (скачивание результатов)

- **`POST /storage/url`** — `orders.download`.
  Тело: `{ "path": string(required) }` — путь файла как в `order.result.files[]` (строка целиком).
  → `data` = `{ "url": string }` (временная ссылка, качать обычным GET).
  Коды: 200 / 403 (чужой файл) / 404 (файл не найден) / 422 (`path` не передан) / 500 (временная
  ошибка генерации, повторить).

### 7.4 Templates

- **`GET /templates`** — `templates.read`. Корневые шаблоны (`folder_id = null`).
  → `data` = массив `Template` (без пагинации).
- **`GET /templates/browse`** — `templates.read`.
  Query: `page` (≥1), `per_page` (1..100, деф. 15), `sort_by` (`name|created_at`, деф. `name`),
  `sort_order` (`asc|desc`, деф. `asc`), `folder_id` (uuid), `service` (code сервиса).
  → `data` = `{ items:[Template-lite], current_page, last_page, per_page, total, current_folder, breadcrumbs }`.
- **`GET /templates/config-options`** — `templates.read`.
  Query: `service` (required, code активного сервиса). → `data` = карта опций конфига сервиса
  (дескрипторы полей: `type`, `options`, `min`, `max`, `step`, `default`, `nullable`). Используйте
  её, чтобы узнать допустимые ключи/значения `config` для шаблона данного сервиса.
- **`POST /templates`** — `templates.write` — см. §8.2 (тело). → 201, `data` = `Template`.
- **`GET /templates/{template}`** — `templates.read` → `data` = `Template`.
- **`PUT|PATCH /templates/{template}`** — `templates.write`. Частичное обновление
  (`name`, `is_public`, `folder_id`, и `service_key`+`config` вместе). → `data` = `Template`.
- **`DELETE /templates/{template}`** — `templates.write` → `data: null`.

### 7.5 Voices — LumVoice (внутренние голоса)

Право `voices.read`. Элемент — `LumVoice` (см. §9).

- **`GET /voices`** — мои голоса. Query: `voice_status`
  (`pending_clone|cloning|ready|partially_ready|clone_failed|archived`; `partially_ready` — часть
  языков склонирована, часть упала, рабочие языки доступны), `publication_status`
  (`private|pending_review|public|rejected`), `language_code`, `search`, `sort_by`
  (`created_at|updated_at|display_name|last_used_at|usage_count_total`, деф. `created_at`),
  `sort_order` (`asc|desc`, деф. `desc`), `per_page` (1..100, деф. 20). → пагинированный список `LumVoice`.
- **`GET /voices/library`** — моя библиотека. Query: `available` (bool), `search`, `language_code`,
  `sort_by` (`added_at|created_at|nickname`), `sort_order`, `per_page`.
  → пагинированный список `{ id, voice_id, nickname, added_at, available, unavailable_reason, origin("own"|"library"), voice: LumVoice|null }`.
- **`GET /voices/public`** — публичный каталог. Query: `tag_ids` (array<int>), `language_code`,
  `gender` (`male|female|neutral`), `search`, `sort` (`popular|newest|name`, деф. `popular`),
  `per_page`. → пагинированный список `LumVoice` (только публичные, ready, разрешённые в заказах).
- **`GET /voices/tags`** → `data` = массив `{ id, code, name, description, color, sort_order }`.
- **`GET /voices/{voice}`** → `data` = `LumVoice` (свой / публичный / из библиотеки). 404/403.

### 7.6 Voices — внешние провайдеры (ElevenLabs, HeyGen)

Право `voices.read`, read-only. Обёртка: `{ voices: [...], total, page, page_size, has_more }`,
`page` **с 0**. Отсюда берут `voice_id` для TTS-шаблонов.

- **`GET /voices/elevenlabs/library`** — публичная Voice Library ElevenLabs.
  Query: `search`, `sort`, `required_languages`, `accent`, `gender`, `age`, `use_cases`, `page` (≥0,
  деф. 0), `page_size` (≥1, деф. 30). Элементы — объекты ElevenLabs.
  **Зависит от внешнего API ElevenLabs → 503 при недоступности.**
- **`GET /voices/heygen`** — каталог голосов HeyGen.
  Query: `gender`, `language`, `accent`, `voice_engine` (CSV), `search`, `sort` (`name|newest|default`),
  `page` (≥0, деф. 0), `page_size` (1..200, деф. 55).
  Элемент: `{ voice_id, voice_name, display_name, gender, language, locale, accent, flag_url,
  preview_url, voice_engines:[...], labels:{...}, support_realtime, emotion_support, support_locale }`.
- **`GET /voices/heygen/filters`** →
  `data = { voice_engines:[{value,count}], genders:[...], languages:[...], accents:[...], labels:[...] }`.

`preview_url` — подписанная ссылка на аудио-превью, играется обычным `<audio>`.

### 7.7 Billing / Usage

Право `billing.read`.

- **`GET /usage`** → `data` = массив записей потребления:
  `{ limit_id, service_id, service_code, service_name, entitlement_id, entitlement_code, limit_type,
  limit_value, used, remaining, period_start, period_end, resets_in }`.
- **`GET /usage/{service}`** — `{service}` = integer id сервиса. Тот же формат, по одному сервису. 404.

> `GET /wallets` и `GET /ledger-transactions` существуют только при включённом кошельке
> (модель «без аванса» по умолчанию их не отдаёт → 404). В типовой конфигурации баланса нет:
> деньги входят прямой оплатой подписки, публичный surface состояния — `usage` и `subscriptions`.

### 7.8 Subscriptions

- **`GET /subscriptions`** — `billing.read`. Без параметров. → `data`:
```json
{
  "available_models": ["<model_key>", "..."],
  "subscriptions": [
    {
      "id": 1, "status": "active",
      "current_period_start": "…", "current_period_end": "…",
      "items": [
        {
          "plan": { "code": "pro", "name": "Pro" },
          "status": "active", "is_usable": true, "billing_period": "monthly",
          "start_at": "…", "end_at": "…",
          "token_allowance": 1000000, "tokens_used": 12345, "tokens_remaining": 987655
        }
      ]
    }
  ]
}
```
Возвращаются только «живые» подписки (Active и срок не истёк). Денежных полей нет.

### 7.9 Profile

- **`GET /user`** — `profile.read`.
  Query: `include` (CSV, whitelist; неизвестное игнорируется):
  `wallets, telegram, subscriptions, referral, order_settings, entitlements, usage, unread_notifications_count`.
  → `data` = `User` (см. §9) + подмешанные `extras` из include (напр. `usage`, `unread_notifications_count`).

### 7.10 Сервисные файлы (субтитры, alignment)

Помимо основного результата (`result.files[]` — аудио/изображения), заказ может нести **сервисные
файлы** в **отдельном** поле `result.service_files[]`. О них не догадаться из `files` — это разные
контейнеры.

**Форма.** `service_files` — **плоский массив строк-путей**, ровно как и `files[]`:
```json
"result": {
  "user_message": null,
  "files": [ "storage/501/orders/<ORDER_UUID>/output/final/result.mp3" ],
  "service_files": [
    "storage/501/orders/<ORDER_UUID>/output/final/service/subtitles.srt",
    "storage/501/orders/<ORDER_UUID>/output/final/service/subtitles.vtt",
    "storage/501/orders/<ORDER_UUID>/output/final/service/result.json"
  ]
}
```
Пути абсолютные (как у `files`), готовы к обмену на ссылку.

**Типы (видимые потребителю).** Различаются **по расширению / сегменту `/service/`** в самой строке —
отдельного поля `type` у элемента НЕТ:
- **субтитры** — `.srt` / `.vtt` / `.lrc` (basename `subtitles.*`);
- **alignment** — пословное/посимвольное выравнивание текст↔аудио (`alignment.*`);
- **result.json** — богатые тайминги (el-timestamps): караоке, пословная подсветка, точная синхронизация.

(Типы `raw_chunk` — сырые данные чанка — и `metadata` отдаются только админу, потребителю по ключу
не видны: скачивание такого пути вернёт 403.)

**Только TTS.** Субтитры/alignment генерируют TTS-сервисы. У `sfx`/`music` поля `service_files`
**нет** (заказ его не содержит).

**Скачивание** — тем же `POST /storage/url` c `path` = строкой из `service_files[]` (право
`orders.download`):
```bash
curl -s -X POST "$BASE/storage/url" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" \
  -d '{ "path": "storage/501/orders/<ORDER_UUID>/output/final/service/subtitles.srt" }'
```

**Только на уровне заказа.** `service_files` есть в `order.result` (`GET /orders/{order}` и в списке
`GET /orders`). На уровне отдельного чанка (`GET /orders/{order}/items/{item}`) сервисные файлы **не
отдаются** — там только основной `result_file` чанка.

### 7.11 Плейбук: обработка сбоев чанка

Заказ разбивается на чанки (`OrderItem`). Статусы чанка: `pending`, `processing`, `completed`,
`policy_flagged`, `failed`. Если часть чанков не удалась, заказ переходит в **`partially_completed`**
(нетерминальный) — успешные чанки готовы, сбойные ждут вашего действия.

**Два вида сбоя чанка:**
- **`failed`** — техническая ошибка (таймаут воркера, сбой генерации). Обычно достаточно повторить
  как есть.
- **`policy_flagged`** — контент отклонён политикой провайдера (напр. ElevenLabs). **Повтор с тем же
  текстом будет отклонён снова** — нужен исправленный текст.

**Три инструмента (не путать):**

| Действие | Для чанка в статусе | Что делает | Стоимость |
| --- | --- | --- | --- |
| `POST .../items/{item}/retry` | `policy_flagged` **или** `failed` | повторяет сбойный чанк (опц. новый `text`); заказ должен быть `partially_completed` | бесплатно (залоченный остаток) |
| `POST .../items/{item}/regenerate` | `completed` | переделывает уже готовый чанк новым `text` | платно (новая токен-квота, precheck → 429) |
| `POST /orders/{order}/retry` | — (весь заказ) | создаёт **новый** заказ теми же параметрами; исходный должен быть `completed`/`result_delivered` без `failed`-чанков | платно (новый заказ) |

**Как дотянуть `partially_completed` до `completed`:** пройдитесь `POST .../items/{item}/retry` по
каждому чанку со статусом `policy_flagged`/`failed`. Order-level `POST /orders/{order}/retry` для
этого **не подходит** (он не принимает `partially_completed` и падает при наличии `failed`-чанков).

**Правило policy_flagged:**
1. Получите текущий текст чанка: `GET /orders/{order}/items/{item}/text`.
2. Исправьте формулировку (уберите/перефразируйте потенциально проблемный фрагмент).
3. Повторите с исправленным телом: `POST .../items/{item}/retry` `{ "text": "<новый текст>" }`.

> **Причина policy/сбоя наружу НЕ отдаётся.** API не сообщает, *что именно* не понравилось
> провайдеру: поля с reason нет, а у полностью упавшего заказа `result` приходит `null`. Правьте
> текст по своему усмотрению — не ищите несуществующее поле с причиной.

**`can_retry` / `can_regenerate` ≠ гарантия успеха.** Эти булевы флаги в `OrderItem` смотрят
**только на статус чанка** (`can_retry` = `policy_flagged`/`failed`; `can_regenerate` = `completed`).
Фактический вызов может всё равно вернуть **400**, если: заказ не `partially_completed`, исчерпан
лимит попыток (5), не осталось залоченного остатка, либо тип задачи — `sfx`/`music`.

**`sfx`/`music`:** item-retry и regenerate для них **недоступны** (одноразовая генерация). При
неудаче — отмените (`cancel`, пока возможно) и создайте заказ заново.

---

## 8. Тела POST-запросов (точные правила валидации)

### 8.0 `POST /orders`

| Поле | Правило |
| --- | --- |
| `template_id` | nullable, **required_without:task_type**, exists (UUID шаблона) |
| `task_type` | nullable, **required_without:template_id**, one of `tts,image,txt2img,remix,image_edit,voice_clone,sfx,music` |
| `input_text` | nullable, string — текст/промпт (для TTS — озвучиваемый текст) |
| `task_data` | nullable, array (или JSON-строка) — параметры для template-less сервисов (§8.1) |
| `input_files` | nullable, array; `input_files.*` — file, max 20480 КБ (20 МБ/файл) |
| `confirm_payg_topup` | sometimes, boolean — согласие на PAYG-добор после 402 |
| `quote_token` | sometimes, string |

Провенанс (`source=api`, `api_key_id`) проставляет сервер. Возможные коды: 201 / 400 / 402
(`payg_topup_required` — не хватает квоты подписки, требуется подтверждение добора: тело несёт
`shortfall_tokens`, `shortfall_lmc`, `shortfall_lmc_minor`, `quote_token`, `expires_at` — повторите
запрос с `confirm_payg_topup: true` и тем же `quote_token`) / 409 (`quote_mismatch` — добор
подорожал, `quote_token` устарел: запросите 402 заново) / 422 / 429.

### 8.1 `task_data` для template-less сервисов

**SFX** (`task_type: "sfx"`):
| Ключ | Правило |
| --- | --- |
| `text` | **required**, string — описание звука (промпт) |
| `prompt_influence` | nullable, numeric 0..1 |
| `duration_seconds` | nullable, numeric (в пределах сервиса) |
| `loop` | nullable, boolean |
| `output_format` | nullable, string (значение из набора сервиса) |

**Music** (`task_type: "music"`):
| Ключ | Правило |
| --- | --- |
| `prompt` | **required**, string |
| `n_variants` | nullable, integer ≥1 |
| `lyrics_text` | nullable, string |
| `force_instrumental` | nullable, boolean |
| `music_length_ms` | nullable, integer (в пределах сервиса) |
| `model_id` | nullable, string (значение из набора сервиса) |

### 8.2 `POST /templates`

| Поле | Правило |
| --- | --- |
| `service_key` | **required**, code активного сервиса (напр. `elevenlabs`, `lumimg`) |
| `name` | **required**, string, max 255 |
| `config` | **required**, JSON-объект — по схеме сервиса (динамические правила `config.*`) |
| `is_public` | boolean |
| `folder_id` | nullable, exists (UUID вашей папки) |

`config` валидируется по правилам конкретного сервиса. Точную структуру для нужного сервиса берите
из `GET /templates/config-options?service=<code>`. Пример **минимального валидного `config`
для `elevenlabs`** (TTS):
```json
{
  "tts_settings": {
    "mode": "mode_v1",
    "model_id": "eleven_multilingual_v2",
    "voice_id": "<VOICE_ID>",
    "voice_settings": {
      "stability": 0.5,
      "similarity_boost": 0.75,
      "use_speaker_boost": true,
      "speed": 1.0
    }
  }
}
```
Допустимые `model_id` (ElevenLabs): `eleven_v3`, `eleven_multilingual_v2`, `eleven_flash_v2_5`,
`eleven_turbo_v2_5`, `eleven_turbo_v2`, `eleven_flash_v2`. Диапазоны `voice_settings`:
`stability` 0..1, `similarity_boost` 0..1 (обязателен, кроме `eleven_v3`), `style` 0..1 (nullable),
`speed` 0.7..1.2. Опциональные секции: `pause_settings.*`,
`output_settings.*` (`audio_format` из `mp3,wav,ogg`; `audio_quality` из
`low,medium,high,lossless`; `sample_rate` из `22050,44100,48000`).

**`tts_settings.language_code`** (опционально, nullable, ISO-639-1) — язык синтеза. Отсутствие
поля или `null` = ElevenLabs сама определяет язык по тексту (ненадёжно на коротких/смешанных
фразах) — явный код надёжнее. Полный список из 73 допустимых значений — в
`GET /templates/config-options?service=elevenlabs` (`tts_settings.language_code.options`).
29 базовых кодов (`eleven_multilingual_v2`, `eleven_flash_v2_5`, `eleven_turbo_v2_5`):
`en, ja, zh, de, hi, fr, ko, pt, it, es, id, nl, tr, fil, pl, sv, bg, ro, ar, cs, el, fi, hr, ms,
sk, da, ta, uk, ru`. Остальные 44 (например `he`, `kk`, `vi`, `cy`) поддерживает только
`eleven_v3`: на других моделях они теперь **отклоняются валидацией (422)**. Раньше такой
запрос проходил, но ElevenLabs отвечала 400 и язык молча терялся — озвучка выходила на
автоопределённом языке. `GET /templates/config-options` отдаёт полный список в `options`
и суженный до 29 — в `conditional_overrides` для не-`eleven_v3` моделей.

---

## 9. Схемы ресурсов (ключи в user-контексте)

По API-ключу все ресурсы отдаются в **пользовательском** контексте — админ-поля скрыты, даже если
ключ принадлежит администратору.

**`Order`:**
```
id (UUID), user_id, template (Template|null), template_id,
task_type, status (enum `OrderStatus`, 9 значений — см. §5), price_units (int), total_tokens (int|null),
task_data (object|null),
result ({ user_message, files:[string], service_files:[string] } | null),
created_at, updated_at,
eta_seconds (int|null), eta_at (ISO|null),
completed_chunks (int), total_chunks (int), progress_percent (int),
items: [ OrderItem ]
```
`result` (пользовательский контекст): `files` — основные файлы (плоский массив строк-путей; после
склейки TTS содержит ровно один элемент — финальную дорожку); `service_files` —
сервисные файлы (плоский массив строк-путей: субтитры/alignment, только TTS, whitelist — см. §7.10),
присутствует только при непустом наборе видимых файлов; `user_message` — passthrough от воркера,
**обычно `null`** (не гарантированный текст статуса). У упавшего заказа (`failed`/`compensated`)
`result` = **`null`** (детали ошибки скрыты). Скрыто (admin): `source`, `user`, `execution_meta`,
`metrics`, error-детали и не-whitelisted сервисные файлы (`metadata`).

**`OrderItem`:**
```
id, order_id, item_type (chunk|retry|regeneration|pause_edit|stitch), chunk_index (int|null),
parent_item_id (string|null), original_text_length (int), current_text_length (int),
price_units (int), consumed_units (int), remaining_locked (int),
status (pending|processing|completed|policy_flagged|failed),
result_file (object|string|null), attempt_number (int), can_retry (bool), can_regenerate (bool),
created_at, updated_at, parent_item (OrderItem|null), retries ([OrderItem])
```
`status`: `policy_flagged` (контент отклонён политикой) и `failed` (тех. ошибка) — оба сбойные,
оба доступны для item-retry; `completed` — доступен для regenerate (см. плейбук §7.11).
`result_file` — **основной** файл этого чанка (не заказа; per-chunk service_files здесь не отдаются).
`can_retry`/`can_regenerate` отражают только статус чанка — **не гарантируют** успех вызова (§7.11).
Скрыто (admin): `metadata` (в т.ч. причина policy/сбоя — потребителю по ключу не видна).

**`Template`:**
```
id (UUID), user_id, slug, name, service_key, service (Service|null),
config (object), is_public (bool), folder_id, folder (при загрузке), created_at, updated_at
```

**`Service`** (вложенный):
```
id, code, name, display_name, payg_access, billing_mode, unit_asset_id, unit_asset (Asset),
required_entitlement_id, required_entitlement, units_per_call, units_per_char, units_per_image,
units_per_second, min_billable_chars, lumcoin_per_token, lumc_precision,
price_lumc_minor (int|null, money-first цена), unit_size, uses_money_first_pricing (bool),
rate_unit, display_category, is_premium, display_order, metadata, is_active, created_at, updated_at
```
Скрыто (admin): `config_schema, default_config, validation_schema, routing_config, task_types,
default_task_type, features, cost_config`.

**`LumVoice`:**
```
id, display_name, description, gender, accent, default_language_code,
voice_status, publication_status, allow_usage_in_orders (bool),
preview_generated_at, created_at, updated_at,
tags: [{id,code,name,...}], languages: [...],
preview_urls: { "<language_code>": <signed_url|null> }
```
Скрыто (admin): статистика использования, модерация, `owner_user_id`, `metadata` и т.п.

**`User`** (по api-key):
```
id, email, lang_code, display_currency (ISO-4217|null), has_telegram (bool), created_at, updated_at
```
При включённых фичах добавляются `wallets` (если кошелёк включён) и реферальные поля. Динамические
include подмешивают `telegram, subscriptions, order_settings, entitlements, usage,
unread_notifications_count`. Всегда скрыто: `is_admin, email_verified_at, allowed_models,
referrer_id, is_blocked` и прочие приватные/админ-поля.

---

## 10. Real-time (WebSocket) — статус заказов без поллинга

Протокол Pusher-совместимый (`pusher-js` или Laravel Echo).

```js
import Pusher from 'pusher-js';

const pusher = new Pusher('7c4dd881d8116ac76fe89aff71d8c2ff', {
  wsHost: 'ws.lumean.app', wsPort: 443, wssPort: 443,
  forceTLS: true, enabledTransports: ['ws', 'wss'], cluster: '',
  channelAuthorization: {
    endpoint: 'https://api.lumean.app/api/public/broadcasting/auth',
    headers: { 'X-API-KEY': 'ВАШ_API_КЛЮЧ' },   // нужно право orders.read
  },
});

const ch = pusher.subscribe('private-orders.42');   // 42 = ID владельца ключа (user_id)
ch.bind('status.changed', (e) => console.log(e.new_status, e.order.id));
ch.bind('progress',       (e) => console.log(`${e.completed}/${e.total} (${e.percent}%)`));
```

**Каналы:**
- `private-orders.{userId}` — все заказы аккаунта (`userId` = ваш `user_id`, из `GET /user`).
- `private-order.task.{orderId}` — один заказ (`orderId` = UUID заказа).

**События** (для `pusher-js` имя без ведущей точки; для Echo — `.listen('.status.changed')`):

| Событие | Канал | payload |
| --- | --- | --- |
| `created` | orders | `{ order }` |
| `status.changed` | оба | `{ order, previous_status, new_status, items_summary: {total, by_status} }` |
| `item.status.changed` | оба | `{ item, previous_status, new_status, completed_chunks, total_chunks, progress_percent }` |
| `progress` | orders | `{ order_id, completed, total, percent }` |
| `progress` | order.task | `{ task_id, progress, chunks_done, chunks_total, timestamp }` |

> `progress` на двух каналах имеет **разную форму** — различайте по каналу. Подписка возможна
> только на свои каналы (иначе 403). Файлы в payload — только метаданные; скачивание — через
> `POST /storage/url`. Права канала: `orders.*` требуют `orders.read`; квоты/леджер — `billing.read`.

---

## 11. Рецепты (сквозные примеры)

### Рецепт A. TTS на ElevenLabs
См. §1 (TL;DR) — полный путь: голос → шаблон → заказ → ожидание → `storage/url`.
Аналогично для голосов **HeyGen** (`GET /voices/heygen`) и **LumVoice**
(`GET /voices/public` или `GET /voices`): берёте оттуда `voice_id`/`id` и кладёте в
`config.tts_settings.voice_id` шаблона соответствующего сервиса.

### Рецепт B. Звуковой эффект (SFX, без шаблона)
```bash
curl -s -X POST "$BASE/orders" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" -d '{
  "task_type": "sfx",
  "task_data": { "text": "Ocean waves crashing on rocks", "duration_seconds": 8 }
}'
```

### Рецепт C. Музыка (без шаблона)
```bash
curl -s -X POST "$BASE/orders" -H "X-API-KEY: $KEY" -H "Content-Type: application/json" -d '{
  "task_type": "music",
  "task_data": { "prompt": "Lo-fi hip hop, calm, rainy night", "music_length_ms": 30000 }
}'
```

### Рецепт D. Проверить остаток токенов подписки
```bash
curl -s "$BASE/subscriptions" -H "X-API-KEY: $KEY"
# items[].tokens_remaining — сколько токенов ещё доступно
```

### Рецепт E. Скачать все файлы готового заказа
```bash
# 1) GET /orders/{id} → result.files[] (основные) + result.service_files[] (субтитры, TTS)
# 2) для каждого path (из files[] И из service_files[]): POST /storage/url { "path": path } → data.url
# 3) GET data.url → бинарник файла
```
`result.files[]` и `result.service_files[]` — строки-пути; кладите строку целиком в `path`. Если поля нет — субтитров
у этого заказа не сгенерировано (напр. `sfx`/`music`).

### Рецепт F. Дотянуть частично готовый заказ (`partially_completed`)
```bash
# 1) GET /orders/{id}/items → найдите items со status = policy_flagged или failed
# 2a) failed (тех. ошибка): просто повторить
curl -s -X POST "$BASE/orders/<ORDER>/items/<ITEM>/retry" -H "X-API-KEY: $KEY"
# 2b) policy_flagged: сначала взять текст, исправить, послать исправленный
curl -s "$BASE/orders/<ORDER>/items/<ITEM>/text" -H "X-API-KEY: $KEY"          # → data.text
curl -s -X POST "$BASE/orders/<ORDER>/items/<ITEM>/retry" -H "X-API-KEY: $KEY" \
  -H "Content-Type: application/json" -d '{ "text": "<исправленный текст>" }'
# 3) повторяйте по всем сбойным чанкам, пока заказ не станет completed (лимит 5 попыток/чанк)
```

---

## 12. Коды ответов (сводка)

| Код | Когда | Тело |
| --- | --- | --- |
| 200 | Успех чтения/действия | `{ success:true, message, data }` |
| 201 | Ресурс создан (order/template/item) | `{ success:true, message, data }` |
| 400 | Ошибка домена (недопустимое действие) | `{ success:false, message }` |
| 401 | Нет/невалидный/истёкший ключ | `{ success:false, message, data:null }` |
| 402 | `payg_topup_required` — не хватает квоты подписки, нужен добор | `{ success:false, message, reason:"payg_topup_required", shortfall_tokens, shortfall_lmc, shortfall_lmc_minor, quote_token, expires_at }` — повторите `POST /orders` с `confirm_payg_topup:true` и тем же `quote_token` |
| 403 | Нет права ключа / чужой ресурс | `{ success:false, message, data:null }` |
| 404 | Ресурс не найден / файл не найден | `{ success:false, message }` |
| 405 | Метод не поддержан (напр. `DELETE /orders/{id}`) | стандартный |
| 409 | `quote_mismatch` — добор подорожал, `quote_token` устарел (гонка) | `{ success:false, message, reason:"quote_mismatch", actual_lmc_minor, quoted_lmc_minor }` — запросите 402 заново |
| 422 | Ошибка валидации | `{ message, errors:{field:[...]} }` (без `success`) |
| 429 | Rate-limit **или** токен-квота — различайте по телу | rate: `{ success:false, message }`, **без** `Retry-After`; квота: `{ success:false, message, reason:"token_quota_exceeded", window, limit, used, requested, reset_at, retry_after }` + заголовок `Retry-After` |
| 500 | Временная инфраструктурная ошибка | `{ success:false, message }`, безопасный текст, повторить |
| 503 | Внешний ElevenLabs library недоступен | только `voices/elevenlabs/library` |

---

## 13. FAQ / частые ошибки для LLM

- **«Как прикрепить голос к заказу?»** — Никак напрямую. Голос → в шаблон
  (`config.tts_settings.voice_id`) → заказ по шаблону. Поля `orders.voice_id` не существует.
- **«Куда положить текст TTS?»** — В top-level `input_text` запроса `POST /orders`. Не в `task_data`.
- **«Почему пустой список голосов ElevenLabs?»** — `page` начинается **с 0**. `page=1` — это вторая страница.
- **«Заказ создан, где файл?»** — Дождитесь `status = completed`/`partially_completed`, возьмите
  `result.files[]` (строка-путь), обменяйте на ссылку через `POST /storage/url`.
- **«Обновить/удалить заказ?»** — Нельзя. Только `cancel` (до обработки) или `retry` (создаёт новый).
- **«429 — что делать?»** — Если тело несёт `reason: token_quota_exceeded` — исчерпана квота по
  токенам, ждать до `reset_at` (заголовок `Retry-After` тоже есть). Иначе это rate-limit —
  `Retry-After` здесь НЕ присылается, ориентируйтесь на окно ключа (`per_minute`/`per_hour`/`per_day`)
  и делайте backoff самостоятельно.
- **«Какие поля config у шаблона?»** — Спросите `GET /templates/config-options?service=<code>`;
  структура зависит от сервиса.
- **«403 на, казалось бы, доступном эндпоинте?»** — У ключа нет нужного права. Скачивание требует
  отдельного `orders.download`, каталоги голосов — `voices.read`, профиль — `profile.read`.
- **«Где субтитры (.srt/.vtt) заказа?»** — В отдельном поле `result.service_files[]` (не в
  `result.files[]`), только для TTS. Это массив строк-путей; скачивайте их тем же `POST /storage/url`.
  См. §7.10.
- **«Чанк заказа `failed`/`policy_flagged` — что делать?»** — Заказ стал `partially_completed`.
  Доведите его поштучно: `POST /orders/{order}/items/{item}/retry` по каждому сбойному чанку. Для
  `policy_flagged` передайте **исправленный** `text` (тот же текст отклонят снова). Order-level
  `retry` здесь не поможет. См. §7.11.
- **«API не говорит, почему сработала policy — где причина?»** — Её нет в ответе по ключу (это
  admin-only). Правьте текст по своему усмотрению; не ищите поле с reason — его не отдают.
- **«Заказ `failed`, но `result: null` — где детали ошибки?»** — Их нет в пользовательском контексте
  (отфильтрованы). Ориентируйтесь на `status` заказа и статусы чанков (`failed`/`policy_flagged`).
- **«`can_retry: true`, но retry вернул 400?»** — Флаг смотрит только на статус чанка. Реальный вызов
  ещё требует: заказ `partially_completed`, попыток < 5, остался залоченный остаток, тип не `sfx`/`music`.
