<div dir="rtl">

# مَرْسَى — Marsa

**ملفك يرسو سالمًا.**

مَرْسَى مدير تحميل مجاني ومفتوح المصدر، عربي أولًا، يعمل على ويندوز وماك ولينكس. يقسّم الملفات على عدة اتصالات، ويستكملها بعد انقطاع الشبكة أو إغلاق البرنامج، ويحفظ القوائم والفئات وحدود السرعة في نافذة واحدة هادئة.

[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-0e7490.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-0f172a.svg)](#الإصدار)

> **English summary.** Marsa is a free, open-source, Arabic-first download manager for Windows (first release), macOS and Linux. It downloads in parallel, resumes safely, keeps queues, categories and speed limits, captures links from the browser by Native Messaging, and can download video with separately downloaded free tools. This release is a Windows-first `0.1.0`.

> **الحالة:** الكود موجود في هذا المستودع، لكنه لم يُبنَ بعد على جهاز نظيف، ولم يُصدَّر إصدار. راجع [`ROADMAP.md`](ROADMAP.md) و[`CHANGELOG.md`](CHANGELOG.md) لمعرفة ما جُرِّب وما لم يُجرَّب.

## ما يفعله

- **تحميل مجزّأ**: حتى 32 اتصالًا للملف الواحد، وتقسيم ديناميكي للأجزاء البطيئة.
- **استكمال آمن**: يتحقق من `ETag` أو `Last-Modified` قبل أن يكمل، ولا يخلط نسختين من ملف أبدًا. إذا تغيّر الملف على السيرفر، يبدأ من جديد ويقول ذلك.
- **لا يكتب فوق ملف**: كل اسم مأخوذ يصبح `الاسم (1).امتداد`، والملف الجزئي يُقفل طوال التحميل.
- **قوائم وجدولة**: قوائم متعددة، وحد للتحميلات المتزامنة، ونوافذ زمنية لكل قائمة، وحد سرعة عام ولكل تحميل.
- **فئات**: تُصنَّف الملفات حسب الموقع أو الامتداد، مع مجلد لكل فئة.
- **من المتصفح**: إضافة Chrome/Edge/Firefox تلتقط الروابط وترسلها عبر Native Messaging فقط، بلا أي سيرفر محلي.
- **فيديو**: تنزيل الفيديو من صفحاته بأدوات حرة منفصلة (yt-dlp وFFmpeg وQuickJS-NG) تُنزَّل عند أول استعمال بموافقة المستخدم.
- **تحميل موقع ودُفعات**: أنماط `img[001-100].jpg`، وقراءة روابط صفحة بعد احترام `robots.txt`.
- **عربي RTL**: واجهة كاملة من اليمين إلى اليسار، وصيغ الجمع العربية الست، وأسماء ملفات لاتينية تبقى باتجاهها داخل الجملة العربية.

## المتطلبات

- **ويندوز 10/11 (x64 أو arm64)**: لا يحتاج صلاحيات مدير. المثبّت يحمل WebView2 معه، فلا يحتاج اتصالًا للتثبيت.
- **للبناء من المصدر**: Rust (الإصدار مثبّت في [`rust-toolchain.toml`](rust-toolchain.toml))، وNode.js 22 أو أحدث، وأدوات Tauri 2.

## البناء والتشغيل

```bash
# من مجلد المشروع
npm --prefix apps/desktop install
npm --prefix apps/desktop run tauri dev
```

لبناء مثبّت ويندوز:

```powershell
.\scripts\build-windows.ps1
```

الناتج في `apps/desktop/src-tauri/target/release/bundle/nsis/`. التفاصيل في [`docs/BUILD-WINDOWS.md`](docs/BUILD-WINDOWS.md).

## الأداء

قياس المرحلة 2 (8 و32 اتصالًا، وحد السرعة، والذاكرة) وطريقته في [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md):

```bash
cargo test -p marsa-core --release --test performance -- --ignored --nocapture --test-threads=1
```

الاختبارات `#[ignore]`ة: قياس على الجهاز، لا فحص في الـ CI.

## سطر الأوامر

```bash
marsa get https://example.com/file.zip
marsa resume path/to/file.zip
marsa queue add https://example.com/file.zip --queue default
marsa queue run
marsa category list
```

## الخصوصية

- **لا تتبع، ولا إحصاءات، ولا اتصال بأي سيرفر** غير سيرفرات التحميل نفسها وفحص التحديث (ويمكن إيقافه).
- **الكوكيز في الذاكرة فقط**، ولا تُكتب في ملف الحالة أبدًا. والتوكن الذي هو جزء من الرابط يبقى في ملف الحالة لأن الاستكمال يحتاجه، وملف الحالة لا يقرؤه إلا صاحبه.
- **الروابط تُنظَّف** قبل حفظها أو طباعتها: بيانات الدخول والـ query والـ fragment تُحذف من كل رسالة.
- **لا أسرار في المستودع**: لا مفاتيح ولا شهادات توقيع.

## الرخصة

GPL-3.0-or-later. النص الكامل في [`LICENSE`](LICENSE).

## أدوات الفيديو

ينزّل مَرْسَى أدوات الفيديو من مصادرها الرسمية، ويتحقق من بصمة SHA-256 قبل كل تشغيل. ولا يملك مَرْسَى هذه الأدوات ولا يصدرها.

- **FFmpeg / ffprobe**: نسخة LGPL مبنية من المصدر. مصدر البناء الكامل، وطريقته، في [إصدار tools-ffmpeg-8.1.3-1](https://github.com/dentalproo/marsa-download-manager/releases/tag/tools-ffmpeg-8.1.3-1). هذا البرنامج يستخدم كودًا من مشروع FFmpeg برخصة LGPL 2.1 أو أحدث.
- **yt-dlp**: برنامج حر بترخيص GPLv3+، ونسخه الرسمية من مشروعه.
- **QuickJS-NG**: لتشغيل جافاسكربت يوتيوب محليًا، من مشروعه الرسمي.

قائمة التراخيص الكاملة في [`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md)، وإشعارات الطرف الثالث في [`scripts/release/licenses/THIRD-PARTY-NOTICES.md`](scripts/release/licenses/THIRD-PARTY-NOTICES.md).

## الإصدار

**0.1.0** — أول نسخة ويندوز. أرقام الإصدارات تُبنى من الـ CI فقط، وكل إصدار معه بصمات SHA-256 لكل ملف.

## المساهمة

اقرأ [`AGENTS.md`](AGENTS.md) قبل أي تعديل، و[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) لفهم البنية. كل تغيير في فرع، ومع كل ميزة اختباراتها، والقواعد في [`AGENTS.md`](AGENTS.md).

</div>
