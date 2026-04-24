# План впровадження дизайну "Industrial Tech" (Factory.ai Style)

Цей план описує кроки для створення сучасного, високотехнологічного інтерфейсу в стилі Industrial Minimalism / Engineering Brutalism. Основний фокус — глибокий чорний колір, тонка типографіка, сітки та ефекти атмосферного світіння.

## Основні принципи стилю
- **Колір:** Pure Black (#000000) для фону, Pure White (#FFFFFF) для тексту, International Orange (#FF5733) для акцентів.
- **Типографіка:** Поєднання сучасного Sans-serif (Inter/Geist) з Monospaced (JetBrains Mono) для технічних деталей.
- **Геометрія:** Чіткі сітки (Bento Grid), тонкі межі (1px), заокруглення 12-16px.
- **Ефекти:** Ambient Glow (периферійне світіння), фонові паттерни (dots/grid).

---

## Технічні деталі реалізації

### 🎨 1. Дизайн-система (CSS)
Для відтворення "атмосфери" Factory.ai необхідно налаштувати глобальні стилі:

```css
:root {
  --bg-color: #000000;
  --fg-color: #ffffff;
  --accent-color: #FF5733; /* International Orange */
  --border-color: #1a1a1a;
  --mono-font: 'JetBrains Mono', monospace;
  --sans-font: 'Inter', sans-serif;
}

body {
  background-color: var(--bg-color);
  color: var(--fg-color);
  /* Фонова сітка */
  background-image: radial-gradient(circle at 1px 1px, #111 1px, transparent 0);
  background-size: 32px 32px;
}
```

### 🧱 2. Базові компоненти

#### Industrial Card
Картка повинна мати ледь помітну рамку та ефект матового скла:
```css
.card {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 24px;
}
```

#### Ambient Glow
Ефект "світла за екраном":
```css
.glow {
  position: absolute;
  width: 50vw;
  height: 50vh;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.08) 0%, transparent 70%);
  filter: blur(100px);
  pointer-events: none;
}
```

### 📐 3. Структура (Bento Grid)
Розташування контенту в модульних блоках різного розміру:
- Використання `display: grid` з `grid-template-columns: repeat(12, 1fr)`.
- Блоки займають різну кількість колонок (наприклад, 8 та 4).

---

## План виконання (кроки)

1. **Налаштування середовища:**
   - [ ] Додати шрифти Inter та JetBrains Mono.
   - [ ] Встановити базову палітру кольорів у CSS.

2. **Створення компонентної бази:**
   - [ ] Розробити універсальну картку `IndustrialCard`.
   - [ ] Реалізувати компонент `GlowBackground`.

3. **Стилізація деталей:**
   - [ ] Оформити кнопки (Solid White для головних, Outline для другорядних).
   - [ ] Додати моноширинні теги для технічних даних.

4. **Анімація:**
   - [ ] Додати плавні переходи (transition: all 0.2s ease) для всіх інтерактивних елементів.
