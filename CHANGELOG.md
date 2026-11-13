<div dir="rtl">

# سجل التغييرات (CHANGELOG)

كل تغيير مهم يُسجَّل هنا. التنسيق مبني على [Keep a Changelog](https://keepachangelog.com/)، والترقيم [SemVer](https://semver.org/).

> **English summary.** `0.1.0` is Marsa's first Windows release. The source is present, and the Windows build script produces the installer and a portable ZIP locally; no release is published yet. Nothing below is a claim of a published artefact.

## [غير مُصدَر]

### أُضيف
- **workflow نشر FFmpeg** ([`.github/workflows/tools-ffmpeg.yml`](.github/workflows/tools-ffmpeg.yml)): يبني `ffmpeg` و`ffprobe` من مصدر FFmpeg الرسمي الموقّع (`tools/ffmpeg/get-source.sh` و`build.sh` و`smoke-test.sh` و`check-linked.sh`) على ويندوز (x64 وarm64)، وماك (ملف universal)، ولينكس (x64 وarm64)، ثم ينشر إصدار `tools-ffmpeg-<النسخة>-<رقم>` في مستودع مَرْسَى. كان هذا workflow غير مضافًا بعد، فكان تنزيل الأدوات بـ 404 عند أول تحميل فيديو.
- **لوحة حالة التحميل**: رسم بياني أسبوعي بـ Canvas (سبعة أيام، إجمالي البايتات المحمّلة) بجانب الإحصاءات، ويتكيّف مع كثافة الشاشة، ويأخذ لون `--accent` من الثيم. المفتاح الجديد `health-bytes-week` في العربية والإنجليزية.
- **عدّاد الذاكرة** `marsa_platform::memory` (`rss_bytes` و`peak_rss_bytes`) على ويندوز (`GetProcessMemoryInfo`) ولينكس (`getrusage` و`/proc/self/statm`) وماك (`getrusage`). لا يقرأه أي مسار تحميل.
- **قياس أداء المرحلة 2** في [`crates/marsa-core/tests/performance.rs`](crates/marsa-core/tests/performance.rs) ونتائجه في [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md): 8 و32 اتصالًا، وحد السرعة على الساعة الحقيقية، والذاكرة. الاختبارات `#[ignore]`ة لأنها قياس لا فحص.
- **مدير الأرشيف**: `marsa_core::archive` يقرأ ZIP وTAR (USTAR واسم GNU الطويل) و`.tar.gz` من بايتات الملف لا من امتداده، ويستخرج إلى `<الاسم>_extracted/` بأمان: كل جزء من الاسم يمر بـ `sanitize_file_name`، و`..` يوقف الاستخراج، ومداخل الروابط لا تُنشأ، والكتابة لا تمر عبر رابط (`open_regular_nofollow`)، واسم المجلد نفسه يبقى ضمن `MAX_NAME_BYTES`. واختبارات في نفس الملف: القائمة والاستخراج للصيغ الثلاث، والمسار الخارج، والرابط الرمزي والصلب المزروعين في الوجهة، وترويسة تالفة، وإعادة الاستخراج، واسم المجلد عند الحد بأسماء عربية (`the_extract_folder_stays_within_the_name_limit`). RAR لا يُقرأ (رخصة unRAR تمنع الهندسة العكسية)، و7z لم يُبنَ.
- **التحديث الموقّع** (`crates/marsa-core/src/update.rs`، ADR-0011): `latest.json` يُقرأ من وسم `updater`، ولا يُعرض تحديث إلا إذا وقّعه أحد مفتاحَي المشروع، وكانت نسخته أحدث، ورابطه تحت `releases/download/v<النسخة>/` وحدها، وتعليقه الموثوق `version:<النسخة>`. ثم يُتحقق من توقيع الملف نفسه على البايتات المنزّلة قبل التثبيت. ولا تنزيل ولا تثبيت بلا موافقة المستخدم. و12 اختبار HTTPS في [`crates/marsa-core/tests/update_manifest.rs`](crates/marsa-core/tests/update_manifest.rs): تحديث صحيح يُقبل، وتوقيع غريب أو لنسخة أخرى أو ملف تغيّر بعد توقيعه يُرفض، وشهادة TLS غير موثوقة أو موقّعة ذاتيًا تُرفض.
- **مفتاحا التحديث**: `apps/desktop/src-tauri/update-keys/signing.pub` (بصمة `9D2B8D0EB24278BC`) و`emergency.pub` (بصمة `76141C6BF60104D7`)، يُبنيان في البرنامج (`KEYS`)، و`signing.pub` هو نفسه `plugins.updater.pubkey`. والخاص خارج المستودع ([SIGNING](docs/SIGNING.md)).
- **workflow الإصدار** ([`.github/workflows/release.yml`](.github/workflows/release.yml)): على وسم `vX.Y.Z` من `main`، في بيئة `release` بموافقة المالك، يبني ويوقّع وينشر، ثم يستبدل `latest.json` على إصدار `updater`، وهو العنوان الذي يسأل عنه مَرْسَى.
- **توقيع البناء المحلي** في [`scripts/build-windows.ps1`](scripts/build-windows.ps1): `MARSA_SIGNING_KEY` (و`MARSA_SIGNING_KEY_PASSWORD`) يوقّع المثبّت ويكتب `latest.json` بلا BOM. وبلا مفتاح يطبع تحذيرًا صريحًا ولا يكتب `latest.json`.
- **المثبّت في التحديث** (`apps/installer`): يقرأ `/R` و`/ARGS` ويفتح مَرْسَى بالوسائط نفسها بعد الاستبدال، ويكتب `marsa-desktop.exe` بـ 40 محاولة على 250 مللي ثانية (النافذة القديمة قد تمسك الملف). اختبارات `an_update_is_silent_and_opens_marsa_again` و`the_arguments_after_args_are_marsa_s_own` و`a_plain_install_shows_the_window`.

### غُيّر
- `ROADMAP.md`: قياس الأداء صار منفَّذًا، وحُذف تكرار قديم لقسم المراحل كان يخالف ما فوقه، وصُحّح سجل الميزات لما هو مبني فعلًا (التحكم عن بعد، والإضافات، والمجموعات، والمساعد، والأتمتة، والتورنت، وFTP/FTPS: نسخ v1 موجودة).
- زر «استخراج» في «التفاصيل» لا يظهر إلا لما يُقرأ فعلًا (ZIP وTAR و`.tar.gz`، لا RAR ولا 7z)، وفشل الاستخراج يظهر كخطأ لا كنجاح. الاختبار: `tests/archive.test.ts`.
- **عنوان التحديث الموقّع**: `releases/download/updater/latest.json` بدل `releases/latest/download/latest.json`، لأن `latest` يترك الإصدارات التجريبية وكل `0.x` تجريبي، فكان يمكن أن يبقى مَرْسَى على إشعار بلا ملف. والعنوان في `MANIFEST` و`tauri.conf.json` معًا.

### أُصلح
- **صفحة التورنت**: أوامر `torrent_list` و`torrent_pause` و`torrent_start` و`torrent_select_files` و`torrent_forget` لم تكن مسجَّلة في `generate_handler!`، فكانت الصفحة تجيب «command not found» عند الفتح وعند كل زر. سُجّلت في قائمتي المعالجات. اختبار `the_torrents_page_s_buttons_are_registered_commands` يثبت الأربعة التي تأخذ `id`، و`torrent_list` (لا وسائط له، فنداؤه يبدأ جلسة تورنت) يحرس تسجيله خطأ الدالة غير المستخدمة تحت `-D warnings`، وCI يشغّل المجموعتين.
- **رابط magnet في قائمة التحميل**: `downloads::add` صار يرفضه برسالة تشرح أنه يُضاف من صفحة التورنت، بدل إضافته إلى القائمة ثم فشله (اختبار `a_magnet_link_is_refused_with_the_page_that_adds_it`). و`ProtocolKind::of` صار يعرف `ftps`، وصار مستخدمًا في المسار بدل أن يكون ملفًا معزولًا.

- **المثبّت و`marsa-host`**: المثبّت (`apps/installer`) لم يكن يضمّ `marsa-host.exe` (مضيف المتصفح الذي يبدأه المتصفح عند كل اتصال)، فكان البناء يوقف، والتثبيت لا يكتب المضيف، فكان «لا يصل المتصفح إلى مَرْسَى» رغم تسجيل الإضافة. أصبح الـ bin داخل حزمة سطح المكتب (`apps/desktop/src-tauri/src/bin/marsa-host.rs` يستدعي مكتبة `marsa-host` المشتركة)، والسكربت يحضّر المضيف في `resources` والمثبّت يكتبه بجانب البرنامج. اختبارات المثبّت والبناء.

### سيُضاف
- إصدار منشور من الـ CI (مثبّت + محمول + توقيع + `latest.json` + `SHA256SUMS`)، في بيئة `release` بموافقة المالك.
- نشر إصدار `updater`، وتجربة تحديث حقيقي من إصدار منشور إلى التالي.
- نشر قائمة الأدوات الموقّعة `tools.json` (التحقق منها مبني، ولا ناشر بعد).
- توقيع `SHA256SUMS` نفسه بمفتاح التحديث (`SHA256SUMS.minisig`).
- مثبّت NSIS موقّع بشهادة تجارية ويندوز.

## [0.1.0] — قيد البناء

### أُضيف
- **هوية مَرْسَى الكاملة**: الاسم، والشعار (مرساة فوق موجة)، والأيقونات، والألوان، والخط، والدليل في [`docs/BRAND.md`](docs/BRAND.md).
- **معرّفات ثابتة** لكل نظام، وإضافة المتصفح ([ADR-0001](docs/adr/0001-naming.md)).
- **بنية ثلاثية الطبقات**: محرّك Rust، وسطر أوامر، وواجهة Tauri 2 + Svelte 5.
- **تحميل مجزّأ** حتى 32 اتصالًا، وتقسيم ديناميكي، واستكمال آمن بـ `ETag`/`Last-Modified`.
- **قوائم وجدولة وحدود سرعة وفئات**.
- **إضافة Chrome/Edge/Firefox** عبر Native Messaging فقط.
- **فيديو** بأدوات حرة منفصلة (yt-dlp وFFmpeg وQuickJS-NG) ببصمات SHA-256.
- **تحميل موقع ودُفعات** مع احترام `robots.txt`.
- **واجهة عربية RTL** بصيغ الجمع الست، وإنجليزية، ووضع فاتح وداكن.
- **وثائق جديدة**: `README`، و`AGENTS`، و`ROADMAP`، و`BRAND`، و`BUILD-WINDOWS`، و`DISTRIBUTION`، و`MANUAL-TESTS`، وADRs للهوية وويندوز أولًا.
- **سكربت بناء ويندوز**: مثبّت + محمول + `SHA256SUMS` في `scripts/build-windows.ps1`.
- **قوالب winget** في `packaging/winget/`.
- **CI**: `.github/workflows/ci.yml` للأنظمة الثلاثة، مع أقراص اختبار FAT32/exFAT/ext4/btrfs/APFS/NTFS، وفحص تراخيص cargo-deny، وبناء سطح المكتب على ويندوز وماك، و`dependabot.yml` وقالب PR.
- **قوالب Issues** في `.github/ISSUE_TEMPLATE/`: bug_report، feature_request، phase، config.

### غُيّر
- الإصدار إلى `0.1.0`.
- هوية المشروع ومعرّفاته بالكامل.
- الأيقونات إلى شعار مَرْسَى الجديد.
- واجهة سطح المكتب إلى تصميم Fluent 2 هادئ.

### أُزيل
- كل ما لا يخص مَرْسَى من الأصول والوثائق الموروثة.

### ملاحظات
- **لم يُبنَ ولم يُختبر بعد في هذا المستودع.** لا توجد سلسلة أدوات مثبّتة.
- ماك ولينكس: الكود موجود، لكن **لم يُصدَّر ولم يُختبر** في هذه النسخة ([ADR-0013](docs/adr/0013-windows-first.md)).
- التحديث التلقائي معطّل حتى يُولَّد مفتاح minisign ([ADR-0011](docs/adr/0011-signed-updates.md)).
- اختبار `tools_list` أُعيد توقيع fixture-ه بـ keypair جديد (minisign) بعد إعادة التسمية.

[غير مُصدَر]: https://github.com/dentalproo/marsa-download-manager/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/dentalproo/marsa-download-manager/releases/tag/v0.1.0

</div>
