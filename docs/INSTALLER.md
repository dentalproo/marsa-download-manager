<div dir="rtl">

# تجربة تثبيت مَرْسَى

> **English summary.** Marsa uses a modern 4-phase installer (Welcome → Setup → Installing → Complete) built as a dedicated Tauri application (`apps/installer/`) — NOT classic NSIS with Next/Finish buttons. It shares the app brand (navy surface, harbour blue), supports silent install (`/S`, `/P` via Windows args or `--silent`) and updates (`/UPDATE`), and creates a desktop shortcut + launch-after option on the finish screen. Staged binaries ship as resources; the installer copies them, writes the uninstall registry entry, and creates shortcuts. The browser integration's host (`marsa-host.exe`) is staged beside the app and copied with it, so that the browsers have something to start beside the app. Brand assets live in `apps/desktop/src-tauri/installer/` for legacy NSIS (deprecated in favour of the new wizard) and `apps/installer/`.

المثبّت لا يستخدم مظهر "Next / Next / Finish" التقليدي بأزرار عتيقة وشريط تقدّم قديم. التجربة أربع مراحل بهوية مَرْسَى نفسها (أزرق مينائي `#3b82f6` على كحلي `#0e131c`، علامة المرساة). **المثبّت الجديد تطبيق Tauri مستقل (`apps/installer/`) بشريط جانبي بخطوات (٠١–٠٤) ومحتوى حديث، بينما المثبّت القديم NSIS مهجور (ملفاته باقية للمراجعة).**

## الخريطة (شاشات المثبّت الحديث → تجربة المنتج)

| # | مرحلة المنتج | الشاشة (Svelte) | ملاحظات |
|---|---|---|---|
| 1 | **Welcome** (ترحيب بهوية المنتج + عرض مختصر للمميزات) | `App.svelte: step 0` | علامة مرسى كبيرة + قائمة مميزات |
| 2 | **Setup** (مسار التثبيت + اختصارات) | `App.svelte: step 1` | حقل مسار + تصفح + اختصار سطح المكتب وقائمة ابدأ + تشغيل بعد التثبيت |
| 3 | **Installing** (تقدّم هادئ) | `App.svelte: step 2` | حلقة تقدّم وشريط أنيق + سجل النسخ |
| 4 | **Complete** (نجاح + تشغيل البرنامج) | `App.svelte: step 3` | علامة نجاح + زر "تشغيل مرسى" الأساسي + "إنهاء" |

التحقق الفعلي (نسخ الملف، الاختصارات، الريجستري) يتم في Rust (`do_install`)؛ الواجهة تقدّم حالة `invoke("start_install")`.

## التثبيت الصامت والتحديث

```powershell
Marsa_0.1.0_x64-setup.exe /S                  # تثبيت صامت
Marsa_0.1.0_x64-setup.exe /P                  # صامت أيضًا (هو ما يمرّره Tauri)
Marsa_0.1.0_x64-setup.exe /S /UPDATE /R       # تحديث، ثم فتح مَرْسَى من جديد
```

| الوسيط | المعنى |
|---|---|
| `/S` أو `/s` أو `-s` أو `--silent` | بلا واجهة إطلاقًا |
| `/P` أو `/UPDATE` أو `--update` | بلا واجهة أيضًا؛ `/P` هو ما يمرّره محدّث Tauri (وضع `passive`)، ومَرْسَى لا يعرض واجهة أثناء التحديث |
| `/R` أو `--launch` أو `/launch` | يفتح مَرْسَى بعد التثبيت |
| `/ARGS <وسائط>` أو `--args <وسائط>` | الوسائط التي تُمرَّر إلى مَرْسَى عند فتحه (يمرّرها محدّث Tauri) |

- **محدّث Tauri** يشغّل المثبّت بـ `/P /UPDATE /R /ARGS <وسائط مَرْسَى الحالية>`، ثم يُغلق مَرْسَى. والمثبّت ينتظر ملف مَرْسَى القديم حتى يُترك (40 محاولة × 250 مللي ثانية = عشر ثوانٍ)، ثم يكتب الجديد، ثم يفتح مَرْسَى بنفس الوسائط. الاختبارات: `an_update_is_silent_and_opens_marsa_again` و`the_arguments_after_args_are_marsa_s_own` و`a_plain_install_shows_the_window`.
- **`/NS` غير مدعوم**: المثبّت لا يقرأه. التثبيت الصامت ينشئ اختصارَي سطح المكتب وقائمة ابدأ دائمًا.
- **WebView2 لا يثبّته المثبّت**: مَرْسَى يحتاج WebView2 runtime (يأتي مع ويندوز 11، وغالبًا مع ويندوز 10 المحدّث). و`webviewInstallMode` في `tauri.conf.json` لا يخصّ هذا المثبّت، لأنه يُبنى بـ `--no-bundle`.
- وضع التثبيت: المستخدم الحالي (`currentUser`)، والمجلد `%LOCALAPPDATA%\Programs\Marsa`، ومجلد قائمة ابدأ: `Marsa`.
- **محتوى مجلد التثبيت**: `marsa-desktop.exe` و`marsa-host.exe` (مضيف المتصفح)، و`marsa-desktop.exe.bak`، و`README.txt`.
- **لم يُختبر على جهاز حقيقي**: تحديث من إصدار منشور إلى التالي، وإعادة الفتح بالوسائط نفسها، وسلوك SmartScreen بعد الاستبدال.

## الهوية البصرية

- التمهيد: `apps/desktop/src-tauri/installer/welcome-sidebar.bmp`
- الترويسة: `apps/desktop/src-tauri/installer/header.bmp`
- الأيقونة: `apps/desktop/src-tauri/icons/icon.ico`
- ألوان: خلفية كحلية `#0e131c`، للأزرق `#3b82f6`، سطح `#141b27`
- التوليد: `node installer/gen-bitmaps.mjs` (BMP 24-bit من الصفر، بدون اعتماديات)

## البناء

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows.ps1
```

يحصل التحقق من `SHA256SUMS` ضمن خطوة البناء. الإصدارات تُبنى من الـ CI فقط.

</div>
