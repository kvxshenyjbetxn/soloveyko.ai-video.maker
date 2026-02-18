# Soloveyko.AI Video Maker

## Структура проекту


### Система перекладів (i18n)

Файли перекладів знаходяться в `frontend/src/locales/`:
- `uk.json` - Українська мова (за замовчуванням)
- `en.json` - Англійська мова
- `ru.json` - Російська мова

Використання в компонентах:
```tsx
import { useI18n } from './contexts/I18nContext';

const MyComponent = () => {
    const { t } = useI18n();
    return <div>{t('tabs.text')}</div>;
};
```

### Структура вкладок

```
frontend/src/tabs/
├── text/
│   ├── translate.tsx   - Переклад тексту
│   └── rewrite.tsx     - Переписування тексту
│
├── settings/
│   ├── general.tsx     - Загальні налаштування
│   ├── api/
│   │   ├── openrouter.tsx
│   │   ├── voice/
│   │   │   ├── elevenlabsbot.tsx
│   │   │   ├── elevenlabsunlim.tsx
│   │   │   └── voicemaker.tsx
│   │   ├── image/
│   │   │   ├── pollinationsai.tsx
│   │   │   ├── googler.tsx
│   │   │   └── elevenlabsimage.tsx
│   │   └── assemblyai.tsx
│   ├── montage.tsx     - Налаштування монтажу
│   ├── subtitle.tsx    - Налаштування субтитрів
│   └── templates.tsx   - Шаблони
│
└── other/
    ├── statistic.tsx   - Статистика
    └── history.tsx     - Історія
```

## Додавання нової вкладки

1. Створіть компонент у відповідній папці `tabs/`
2. Додайте переклади в `locales/*.json`
3. Імпортуйте та додайте рендеринг в `App.tsx`

## Технології

- **Frontend**: React + TypeScript
- **Framework**: Wails v2 (Go + Web)
- **Стилі**: Vanilla CSS
- **i18n**: Custom context-based solution
