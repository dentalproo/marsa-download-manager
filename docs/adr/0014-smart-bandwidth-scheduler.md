<div dir="rtl">

# ADR-0014: جدولة السرعة (Smart Bandwidth Scheduler)

| البند | القيمة |
|---|---|
| الحالة | **مقترح** |
| التاريخ | 2026-11-08 |
| صاحب القرار | Marsa (@marsa-app) |

## السياق

التحميلات الكبيرة تؤثر على سرعة الإنترنت في المهام الأخرى. المستخدم يرغب في:
- تقليل سرعة التحميل during العمل (مثلاً 20% من السرعة).
- رفع السرعة إلى القصوى ليلاً.
- إيقاف جهاز الكمبيوتر تلقائياً عند انتهاء القائمة.

## الخيارات

| الخيار | المزايا | العيوب |
|---|---|---|
| حد سرعة ثابت | بسيط | لا يتكيف مع الوقت |
| **ملفات سرعة حسب الوقت (Speed Profiles)** | يتكيف، قابل للتخصيص | تعقيد في الواجهة |
| إيقاف تلقائي (Auto Shutdown) | توفير طاقة | platform trait needed |

## القرار

- **Speed Profiles**: المستخدم يحدد نوافذ زمنية (مثلاً 08:00-22:00 → 20%، 22:00-08:00 → 100%).
- **Auto Shutdown**: على Windows only (platform trait `PowerControl::shutdown()`).
- **التنفيذ**: في 1.1 (Windows). ADR-0006 (platform abstraction) يُستخدم.
- **الإعدادات**: في `SettingsFile` (ADR-0007).

## العواقب

- `marsa-platform` needs `shutdown()` trait method (Windows: `ExitWindowsEx`).
- UI: profile editor (time ranges + speed %).
- Tests: mock clock (already have `ManualClock`).

## المصادر

- ADR-0006: platform abstraction
- ADR-0007: queues and settings storage

</div>
