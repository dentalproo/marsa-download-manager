<div dir="rtl">

# ADR-0015: واجهة Glass قابلة للتخصيص (Modular Glass UI)

| البند | القيمة |
|---|---|
| الحالة | **مقترح** |
| التاريخ | 2026-11-08 |
| صاحب القرار | Marsa (@marsa-app) |

## السياق

المستخدم يرغب في:
- 3 ثيمات (Aurora dark، calm night، pastel glass).
- لوحة تحكم (dashboard) مع رسوم سرعة (Canvas).
- تصميم glassmorphism (Nava OS-inspired).

## الخيارات

| الخيار | المزايا | العيوب |
|---|---|---|
| CSS variables swap | بسيط، instant | limited |
| **Theme system (3 themes)** | قابل للتخصيص | تعقيد |
| Canvas speed graph | smooth 60fps | canvas API |

## القرار

- **3 themes**: Aurora dark (default)، calm night، pastel glass.
- **Theme switcher**: في Settings.
- **Speed graph**: Canvas (animated wave) في Dashboard.
- **التنفيذ**: في 1.1 (Windows).

## العواقب

- `app.css`: theme variables per theme.
- `theme.ts`: theme switcher + persistence.
- Canvas component: speed wave animation (60fps).
- Tests: visual (manual).

## المصادر

- ADR-0003: language and UI framework
- ADR-0008: UI library

</div>
