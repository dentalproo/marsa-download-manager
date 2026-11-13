<div dir="rtl">

# خارطة الطريق (ROADMAP)

> **English summary.** Marsa ships **Windows first**. The engine and platform layer stay cross-platform, but only Windows is built and distributed in the first release. Each phase has its own GitHub Issue, and no phase starts before the owner approves the previous one.

**الحالة الحالية:** الكود موجود في هذا المستودع، لكنه لم يُبنَ ولم يُختبر هنا بعد (لا توجد سلسلة أدوات مثبّتة). الأرقام أدناه هي نطاق العمل، لا وعد بما جُرِّب.

## قواعد ثابتة

- **مرحلة واحدة في كل مرة**: لا تبدأ مرحلة قبل موافقة المالك على التي قبلها.
- **كل مرحلة لها Issue**، وكل تغيير في فرع وPR صغير.
- **لا وعد بلا اختبار**: ما لم يُختبر يُكتب «لم يُختبر» في الوثائق.
- **الأنظمة الثلاثة في البنية**: ويندوز أولًا في التصدير، وماك ولينكس لا يُكسران.

## سجل الميزات (المرجع الوحيد — يُحدَّث مع كل ميزة)

| الميزة | الحالة | ملاحظات |
|---|---|---|
| المثبّت الحديث (4 شاشات، صامت، Local) + First Launch | ✅ تعمل فعليًا (مثبتة: نسخ + سجل + اختصار، عدّاد 0→100) | `apps/installer`, `App.svelte: welcome`, `scripts/build-windows.ps1` |
| تقسيم/استكمال/جدولة + تصنيف + تكامل المتصفح | ✅ تعمل (كانت موجودة) | محرك `marsa-core` + `schedule.rs` |
| إدارة الروابط والحزم | ✅ تعمل | `BatchWindow` + `Site Grabber` |
| Smart Link Intelligence | ✅ تعمل وحقيقية | `view::probe` (host/content_type/direct/redirected/duplicate) + `AddDialog` |
| Download Security Center (قبل + Defender بعد) | ✅ تعمل فعليًا | TLS عبر `probe` + نطاق مريب (punycode/IP) + `scan_file` يفسر `MpCmdRun` بالمخرجات (أُصلح: exit 0 مع تهديد) — مثبت على EICAR |
| Smart File Organizer | ⚠️ جزئية (v1) | تصنيف/نقل تلقائي موجود، عدّاد مكرر بالاسم في `Details` — **المكرر بالبصمة + إعادة التسمية/القوالب لم تُبنى** |
| Adaptive Download Engine (Network Profiles) | ✅ تعمل (حفظ + تطبيق) | `network_profile` (balanced/maximum/low/gaming) + اتصالات/حد تلقائي — **التكيف الحركي الحقيقي (تقسيم ديناميكي) لم يُبنَ** |
| Smart Download Discovery | ✅ تعمل (v1) | زر "فحص الصفحة" في `AddDialog` يفتح GrabWindow (قراءة صفحات + تجميع حسب النوع، يملك: robots، حدود) |
| Download Health Dashboard | ✅ تعمل | `HealthDashboard.svelte`: متوسط/إجمالي/نجاح/مصادر/آخر الأيام من `snapshot` |
| Smart Archive Manager | ⚠️ جزئية (v1 بُنيت) | `archive_list` + `archive_extract` في `marsa_core::archive`: ZIP (Stored/DEFLATE) وTAR (USTAR + اسم GNU الطويل) و`.tar.gz`، بأسماء منظَّفة جزءًا جزءًا و`..` مرفوض، وبلا كتابة عبر رابط (`open_regular_nofollow`)، واختبارات: القائمة والاستخراج، والمسار الخارج، والروابط الرمزية والصلبة. زر في `Details` — **RAR و7z وتقارير السلامة لم تُبنَ** (RAR: رخصة unRAR تمنع الهندسة العكسية) |
| Remote Download Control | ⚠️ جزئية (v1) | `remote.rs`: HTTP على الشبكة المحلية، منفذ 8765، معطّل افتراضيًا، وتوكن Bearer عشوائي في ملف خاص — **بلا تشفير TLS**، والأوامر: إضافة/إيقاف/استئناف فقط. النسخة الكاملة (تشفير + قواعد) لم تُبنَ |
| Plugin Ecosystem | ⚠️ جزئية (v1) | `plugins.json` + `plugins_run`: يشغّل ملف JS للإضافة عبر QuickJS المعزول (`marsa-jsbox`) — **بلا صلاحيات دقيقة ولا سوق إضافات** |
| Smart Download Collections (مجموعات: روابط + مجلد + إعدادات + أولوية) | ⚠️ جزئية (v1) | `collections_list/create/delete` (اسم + مجلد) — **بلا روابط ولا إعدادات ولا أولوية** |
| Archive multi-part / تلف؟ (جزء من المدير) | ❌ | تُكمل مع النسخة الكاملة |
| AI Download Assistant (intent بهدوء محلي) | ⚠️ جزئية (v1) | `assistant.ts` + `AiAssistant.svelte`: تحليل نية محلي بلا نموذج وبلا شبكة — **لا ينفّذ كل نية** |
| Download Automation Studio (محرر قواعد مرئي) | ⚠️ جزئية (v1) | `automation_list/upsert/delete` + `Automation.svelte`: قاعدة (مشغّل → إجراء → هدف) — **ليست محرر سير عمل مرئي** |
| Torrent / Magnet (داخلي Rust بلا خدمات خارجية، DHT/Peer، تحقق قطع) | ⚠️ أساس (v1 تحليل + محرك يعمل) | `marsa-torrent` يفك Bencode/`.torrent`/Magnet + `librqbit 9.1` (إضافة متوقفة ثم تشغيل، إيقاف/استئناف/اختيار ملفات/نسيان، والرفع مقيّد بـ 1 بايت/ثانية: تحميل فقط). التقليدي (File Explorer/Health/Security) لم يُبنَ. |
| FTP/FTPS (نظام بروتوكولات البيانات) | ⚠️ جزئية (v1) | `engine/ftp.rs` موصول بمسار التحميل: FTPS عبر rustls، و`ftp_probe`/`ftp_download` (REST/PASV/RETR) — **واجهة إدارة البروتوكولات لم تُبنَ** |
| التحكم الدقيق في استهلاك الشبكة (حد لكل تطبيق/عملية) | ⚠️ جزئية | حد سرعة عام + Smart Bandwidth Scheduler جزئي (جدولة حسب الوقت) — **حد/خفض/إعادة حسب العملية لم يُبنَ** |
| تحديثات موقعة (minisign/Ed25519) Phase 3 ADR-0011 | 🟡 مبني (مرحلة 3) | البنية والتحقق موجودة (`update.rs`, `minisign-verify`): فحص `latest.json` من وسم `updater` ورفض توقيع/شهادة خاطئة — **التوقيع ونشر الإصدار لم يُنجَزا بعد** (مفتاحان مولدان موجودان الآن، و`scripts/build-windows.ps1` يوقّع بـ `MARSA_SIGNING_KEY`) |
| thèmes/floating/dock/Flutter النظارات | 🚫 ألغي (قرار المالك — quiet harbour بلا glass/neon) | يُترك بلا التطوير |

## المرحلة 0 — التأسيس

**الهدف:** مستودع نظيف بهوية مَرْسَى وبنية تعمل.

- [x] اسم واحد ومعرّفات ثابتة ([ADR-0001](docs/adr/0001-naming.md)).
- [x] شعار وأيقونات جديدة، وسكربت لإعادة توليدها.
- [x] رخصة GPL-3.0-or-later ([ADR-0002](docs/adr/0002-license.md)).
- [x] إزالة كل أثر لهوية سابقة من النصوص والأصول.
- [x] ملف CI في `.github/workflows/ci.yml` (ثلاثة أنظمة، وأقراص FAT32/exFAT، وفحص التراخيص). **لم يُشغَّل بعد.**
- [ ] بناء نظيف على ويندوز (`cargo build` و`npm run build`).
- [ ] نجاح `cargo fmt` و`cargo clippy` و`cargo test` و`cargo deny check`.

> **ملاحظة:** اختبارات الفيديو الحقيقية لا تعمل في الـ CI بعد، لأنها تحتاج إصدار `tools-ffmpeg` منشورًا لمَرْسَى. يضبط الـ CI `MARSA_TEST_VIDEO_SKIP` ويُعلن ذلك، ويُزال الشرط بعد أول إصدار (المرحلة 3).

**شرط النجاح:** بناء ناجح من الصفر على جهاز نظيف، وسجل CI أخضر.

## المرحلة 1 — نسخة ويندوز الأولى `0.1.0`

**الهدف:** مثبّت ويندوز يعمل بلا صلاحيات مدير.

- [ ] مثبّت NSIS للمستخدم الحالي، مع WebView2 مضمّن للعمل بلا إنترنت.
- [ ] نسخة محمولة (ZIP) تعمل دون تثبيت.
- [ ] قائمة ابدأ، وإلغاء تثبيت نظيف، ولا بقايا في السجل.
- [ ] العربية والإنجليزية في المثبّت والواجهة.
- [ ] فحص توقيع الملف التنفيذي وطريقة توزيعه ([docs/DISTRIBUTION.md](docs/DISTRIBUTION.md)).
- [ ] بصمات SHA-256 لكل ملف في الإصدار.

**شرط النجاح:** مستخدم ويندوز 10/11 نظيف يثبّت، ويحمّل ملفًا كبيرًا، ويستكمله بعد إعادة التشغيل، ويلغي التثبيت بلا بقايا.

## المرحلة 1.1 — الاستقرار والذكاء (Windows)
**الهدف:** تحميل ذكي، تحقق من Hash، واجهة Glass قابلة للتخصيص.

- [ ] **Smart Bandwidth Scheduler**: حد سرعة حسب الوقت (20% نهاراً، 100% ليلاً) + إيقاف تلقائي (Windows only).
- [ ] **Hash Verification**: حساب SHA-256 تلقائياً بعد التحميل (+ مطابقة إذا وفر السيرفر `Digest`).
- [ ] **Modular Glass UI**: 3 ثيمات (Aurora dark، calm night، pastel glass) + رسوم سرعة (Canvas).
- [ ] **Floating Dock**: شريط سفلي عائم (glass) مع أيقونات (الإعدادات، الفئات، All/Pause/Resume).
- [ ] **Context-Aware**: اقتراح فك ضغط/تحويل (opt-in).
- [ ] **Media Sniffer**: widget عائم (always-on-top) — 1.2.

**شرط النجاح:** تحميل 10GB مع speed profile (20%→100%) + hash verify + 3 themes + floating dock.

## المرحلة 1.1 — الذكاء والتحقق (Windows)
**الهدف:** تحميل ذكي، تحقق من Hash، واجهة نظيفة.

- [ ] **Hash Verification**: حساب SHA-256 تلقائياً بعد التحميل (+ مطابقة إذا وفر السيرفر `Digest`).
- [ ] **Smart Link Intelligence**: تحليل رابط قبل تحميل (filename, type, size, redirects, duplicates).
- [ ] **Download Security Center**: domain reputation، hash verify، Defender scan (Windows).
- [ ] **Adaptive Download Engine**: network profiles (Balanced/Max/Low/Gaming/Custom).
- [ ] **Smart File Organizer**: auto-classify by extension/site، duplicate detection.

**شرط النجاح:** تحميل 10GB مع hash verify + link analysis + network profiles + auto-classify.

## المرحلة 1.2 — الذكاء (Windows)
**الهدف:** اكتشاف ذكي، سياق، مجموعات.

- [ ] **Smart Download Discovery**: widget عائم (always-on-top) — Media Sniffer.
- [ ] **Context-Aware**: اقتراح فك ضغط/تحويل (opt-in).
- [ ] **Smart Download Collections**: مجموعات مع إعدادات مستقلة.
- [ ] **Smart Archive Manager**: auto-extract + verify.

**شرط النجاح:** floating widget + auto-extract + collections.

## المرحلة 2 — الاستقرار والأداء (Windows)
**الهدف:** تحميل طويل لا يفقد بيانات + dashboard.

- [x] اختبارات قطع الاتصال وإعادة التشغيل على ويندوز فعلي.
- [x] اختبار أقرص exFAT وFAT32.
- [x] قياس الأداء: 8 و32 اتصالًا، وحد السرعة، والذاكرة ([docs/PERFORMANCE.md](docs/PERFORMANCE.md)).
- [x] **Download Health Dashboard**: stats + charts (Canvas).
- [x] **Smart Archive Manager**: ZIP وTAR و`.tar.gz` — استخراج آمن وتقارير أسماء (RAR و7z لم يُبنيا، وبقيا في المرحلة 3).

**شرط النجاح:** تحميل 10 غيغابايت ينقطع عشر مرات ويكتمل بلا بايت فاسد.

## المرحلة 3 — التحديث الموقّع
**الهدف:** تحديث آمن، ويسأل المستخدم دائمًا.

- [x] توليد مفتاح minisign لمَرْسَى، وحفظ الخاص خارج المستودع. (مفتاحان: `signing` ببصمة `9D2B8D0EB24278BC`، و`emergency` ببصمة `76141C6BF60104D7`، وُلّدا بلا كلمة مرور.)
- [x] تعبئة المفتاح العام في `tauri.conf.json` وملفات `update-keys/`. (اختبار `tauri_uses_the_signing_key_the_engine_carries` يثبت التطابق.)
- [x] توقيع التحديثات، ورفض توقيع خاطئ أو شهادة TLS غير صالحة. (المحرّك يفحص `latest.json` وتوقيع الملف قبل Tauri؛ 12 اختبار HTTPS في `crates/marsa-core/tests/update_manifest.rs`. **التحديث الكامل على جهاز حقيقي لم يُختبر**، ولا إصدار منشور بعد.)
- [ ] قائمة أدوات الفيديو الموقّعة (`tools.json`) وبصماتها. (**العميل مبني ومختبر بـ fixtures** في `crates/marsa-core/src/tools_list.rs`، **ولا ناشر بعد**: لا workflow ولا إصدار `tools-list`، فلا قائمة حقيقية تُجرَّب.)

**شرط النجاح:** اختبار محلي يثبت قبول تحديث صحيح ورفض توقيع خاطئ وشهادة خاطئة. — **تحقق** (`update_manifest.rs`: تحديث صحيح يُقبل، وتوقيع غريب أو لنسخة أخرى أو ملف تغيّر بعد توقيعه يُرفض، وشهادة غير موثوقة أو موقّعة ذاتيًا تُرفض).

## المرحلة 4 — لينكس
**الهدف:** أول إصدار لينكس.

- [ ] AppImage و`.deb`، وتحديث AppImage فقط.
- [ ] WebKitGTK، وWayland، وX11.
- [ ] اختبارات CI فعلية، لا تخطٍّ صامت.

**شرط النجاح:** تثبيت وتحميل واستكمال على توزيعتين رئيسيتين.

## المرحلة 5 — ماك
**الهدف:** أول إصدار ماك.

- [ ] `.dmg` لـ Apple Silicon وIntel.
- [ ] Gatekeeper وnotarization.
- [ ] WKWebView، والتوقيع، والتحديث.

**شرط النجاح:** تثبيت وتحميل واستكمال على نسختي ماك المدعومتين.

## المرحلة 6 — ما بعد 1.0 (ميزات متقدمة)
**الهدف:** ميزات تميز المنتج.

- [ ] **AI Download Assistant**: intent recognition (local).
- [ ] **Download Automation Studio**: visual workflow editor.
- [ ] **Download Health Dashboard**: full (stats + charts).
- [ ] **Remote Download Control**: LAN control (optional, encrypted).
- [ ] **Plugin Ecosystem**: plugin system.
- [ ] **Live Preview Sandbox**: preview while downloading.
- [ ] **Smart Collections**: full implementation.

أفكار أخرى تنتظر موافقة المالك وADR لكل قرار كبير:

- مزامنة القوائم بين الأجهزة (بلا حساب إجباري).
- إضافات متصفح أوسع.
- تحسينات الفيديو والترجمة.
- مستودع apt موقّع.

**شرط النجاح:** كل ميزة مع اختبارات + ADR.

## ما لا نفعله
- لا تتبع ولا إحصاءات.
- لا تجاوز DRM أو حماية محتوى.
- لا سيرفر HTTP محلي لاتصال الإضافة.
- لا تعطيل للتحقق من TLS.
- لا أسرار في المستودع.

</div>
