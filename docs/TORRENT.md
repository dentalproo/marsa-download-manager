<div dir="rtl">

# تصميم محرك التورنت (المستقبل)

> **English summary.** Torrent support is planned as an internal Rust engine (librqbit studied as a library, never as an external app), behind the same cards/queues/schedules as direct downloads. Seeding is **off by default** (download-only, to protect the user's bandwidth). Do not start until the current chain finishes (Archive complete, AI, Automation, Remote, Plugins, Collections).

## 1) تجربة المستخدم

قسم جانبي مستقل `Torrents` بنفس التوشيح (بطاقات موحدة، لا برنامجين):

- **All Torrents**
- **Downloading**
- **Seeding** (معطلة افتراضيًا — انظر 4)
- **Completed**
- **Paused**
- **Queued**
- **Error**

## 2) Advanced Torrent Engine

- Magnet + `.torrent` + BitTorrent + DHT + PEX + Local Peer Discovery + WebTorrent/Web Seeds عند دعم librqbit
- استكمال، Seeds/Peers، رفع/تنزيل، اتصالات، جدولة، اختيار ملفات داخل التورنت

## 3) Smart Torrent Management

تحليل حجم المحتوى وعدد الملفات/المجلدات **قبل البدء** + اختيار ملفات معينة فقط + أولوية لكل ملف + مكان الحفظ + تصنيف تلقائي + اكتشاف مكرر + انتظار

## 4) Intelligent Seeding Manager (قرار المالك)

**Seeding معطل افتراضيًا.** المستخدم يريد التحميل فقط للحفاظ على باقته. عند التفعيل الصريح فقط: نسبة رفع مستهدفة، مدة مشاركة، سرعة رفع، إيقاف AA الشروط، أولوية للرفع، تشغيل/إيقاف تلقائي وفق قواعد. يجب التوضيح أن الرفع قد يستهلك الباقة.

## 5) Torrent Health Analyzer

قبل الإضافة/البدء: Seeds المعروفين، Peers المعروفين، حجم المحتوى، عدد الملفات، توفر القطع، إمكانية الوصول، Tracker، حالة التورنت؛ تقييم وصفي: Good / Limited / Low / Unknown (لا ضمان من Seeds وحده)

## 6) Torrent Security Center

فحص `.torrent` قبل الإضافة، أسماء/امتدادات، تحذير من تنفيذية داخل الأرشيفات، Defender للملفات المكتملة، تحقق من سلامة البيانات، تحذير من مصادر غير موثوقة، **بدون تشغيل أي ملف تلقائيًا**. لا تعرض **Safe** لمجرد Seeds كثيرة

## 7) أوامر AI للتورنت

- اعرض التورنتات المكتملة.
- أوقف جميع عمليات الرفع.
- إحصائيات التورنتات.
- أوقف التحميلات التي تتجاوز حد استهلاك معين.
- اجعل هذا التورنت أولوية.
- انقل التورنت المكتمل إلى مجلد معين.

تكون هذه الأوامر محدودة بالصلاحيات والإعدادات التي يحددها المستخدم

## 8) المعمارية المقترحة

| المكون | التقنية |
|---|---|
| Desktop UI | Tauri 2 + Svelte (موحد) |
| Core Engine | Rust |
| Torrent Engine | `librqbit` يُدرس كمكتبة (GitHub) |
| Direct Downloads | المحرك الحالي (HTTP) |
| Hash | SHA-256 |
| Security | Defender Integration |
| Browser Integration | Browser Extension |

محركان داخليان منفصلان (HTTP + Torrent) **بواجهة موحدة**. لا `aria2` ولا تطبيق تورنت خارجي.

## 9) Protocol Manager (مستقبل)

ليس توقفًا عند HTTP/Torrent: التوسعة (الإضافات) تفتح بروتوكولات ومصادر أخرى

</div>
