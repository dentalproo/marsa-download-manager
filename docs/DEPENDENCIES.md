<div dir="rtl">

# سجل المكتبات وتراخيصها

**أي مكتبة جديدة تُضاف إلى هذا الجدول في نفس الـ Pull Request الذي يضيفها**، مع:

- ترخيصها.
- سبب الحاجة إليها.
- تأكيد أن الترخيص متوافق مع GPL-3.0 (انظر [ADR-0002](adr/0002-license.md)).

يفحص الـ CI التراخيص تلقائيًا بأداة `cargo-deny` حسب الإعدادات في [`deny.toml`](../deny.toml). ويبقى هذا السجل ضروريًا لأنه يوثّق **سبب** اختيار كل مكتبة، ويشمل أيضًا ما لا يفحصه `cargo-deny`، مثل مكتبات الواجهة (npm)، والأدوات الخارجية، والخطوط والأيقونات.

**نطاق الإصدار الأول: ويندوز.** المحرّك مبني ليعمل على ويندوز وماك ولينكس، لكن ما يُبنى ويُختبر ويُنشر في `0.1.0` هو نسخة ويندوز فقط.

## مكتبات Rust

| المكتبة | الإصدار | الترخيص | تُستخدم في | السبب |
|---|---|---|---|---|
| `clap` | 4.6.7 | MIT أو Apache-2.0 | `marsa-cli` | تحليل أوامر سطر الأوامر ورسائل المساعدة |
| `arboard` | 3.7.0 | MIT أو Apache-2.0 | `marsa-platform` (ماك ولينكس فقط) | الحافظة: NSPasteboard في ماك، وX11 أو Wayland (data-control، عبر `wl-clipboard-rs`) في لينكس، لمراقبة الحافظة. بلا دعم الصور. وفي ويندوز نستعمل Win32 مباشرة، فلا تدخل واجهتها في ويندوز (`clipboard-win` و`error-code`، بترخيص BSL-1.0) أي بناء، وهما مستثنيتان من رسم cargo-deny بسببهما في `deny.toml` |
| `objc2-app-kit` | 0.3.2 | Zlib أو Apache-2.0 أو MIT | `marsa-platform` (ماك فقط) | `NSPasteboard.changeCount`، فلا تُقرأ الحافظة إلا بعد نسخة (مراقبة الحافظة). وكانت في الشجرة تحت `arboard` |
| `getrandom` | 0.4.3 | MIT أو Apache-2.0 | `marsa-platform` و`marsa-core` | أرقام عشوائية من النظام: لاسم الـ named pipe في ويندوز (قناة الإضافة)، ولأسماء مجلدات تنزيل أدوات الفيديو وتشغيلها |
| `dirs` | 7.0.0 | MIT أو Apache-2.0 | `marsa-platform` | مسارات النظام القياسية: Known Folders في ويندوز، وXDG في لينكس (يعتمد على `option-ext` بترخيص MPL-2.0) |
| `serde` | 1.0.229 | MIT أو Apache-2.0 | `marsa-core` | قراءة ملف الحالة وكتابته |
| `serde_json` | 1.0.154 | MIT أو Apache-2.0 | `marsa-core` | صيغة JSON لملف الحالة (ADR-0004) |
| `sha2` | 0.11.1 | MIT أو Apache-2.0 | `marsa-core` | بصمة ملف الحالة، وبصمات الملفات |
| `tokio` | 1.54.1 | MIT | `marsa-core` و`marsa-testserver` | التشغيل غير المتزامن (async) |
| `hyper` | 1.12.0 | MIT | `marsa-testserver` (اختبار فقط) | سيرفر HTTP/1.1 للحالات الصعبة |
| `hyper-util` | 0.1.21 | MIT | `marsa-testserver` (اختبار فقط) | ربط hyper بـ tokio |
| `http-body-util` | 0.1.5 | MIT | `marsa-testserver` (اختبار فقط) | إرسال الجسم على دفعات، وقطعه في المنتصف |
| `bytes` | 1.12.1 | MIT | `marsa-testserver` (اختبار فقط) | مخازن البيانات |
| `reqwest` | 0.13.5 | MIT أو Apache-2.0 | `marsa-core` | عميل HTTP: التحويلات، والمهلات، والبث. **دون الميزات الافتراضية**: rustls دون مزوّد مدمج (نضع `ring`)، ودون بروكسي النظام، ودون فك الضغط |
| `rustls-platform-verifier` | 0.7.1 | MIT أو Apache-2.0 | عبر `reqwest` | التحقق من الشهادات بمخزن النظام وقواعده (ADR-0003). `reqwest` يستخدمه تلقائيًا، ونحن نضع مزوّد التشفير `ring`. يعتمد في لينكس على `rustls-native-certs` و`openssl-probe` (MIT أو Apache-2.0)، ولا يستخدم OpenSSL نفسه |
| `percent-encoding` | 2.3.2 | MIT أو Apache-2.0 | `marsa-core` | فك ترميز اسم الملف من الرابط ومن `filename*` (RFC 8187) |
| `rustls` | 0.23.46 | Apache-2.0 أو ISC أو MIT | `marsa-core` و`marsa-testserver` | TLS. **بمزوّد التشفير `ring` فقط** |
| `ring` | 0.17.14 | Apache-2.0 و ISC | عبر `rustls` و`rcgen` | مزوّد التشفير. اخترناه بدل `aws-lc-rs` لأن ترخيص `aws-lc-sys` يتضمن رخصة OpenSSL غير المتوافقة مع GPL-3.0 |
| `tokio-rustls` | 0.26.6 | MIT أو Apache-2.0 | `marsa-testserver` (اختبار فقط) | سيرفر HTTPS للاختبار |
| `rcgen` | 0.14.10 | MIT أو Apache-2.0 | `marsa-testserver` (اختبار فقط) | ينشئ في كل تشغيل للاختبارات جهة إصدار (CA) مؤقتة في الذاكرة، ويوقّع بها شهادات السيرفر التجريبي (صالحة، ومنتهية)، وشهادة موقّعة ذاتيًا. **لا توجد أي شهادة أو مفتاح في المستودع**. بمزوّد `ring` فقط |
| `landlock` | 0.4.7 | MIT أو Apache-2.0 | `marsa-platform` (لينكس فقط) | صندوق QuickJS-NG ليوتيوب (`marsa_platform::sandbox`، ADR-0009): يمنع الكتابة خارج مجلده والشبكة. من مشروع Landlock نفسه، بدل كتابة واجهة النواة بـ `unsafe` |
| `html5ever` | 0.40.1 | MIT أو Apache-2.0 | `marsa-grab` | قراءة روابط صفحة HTML لتحميل الموقع (ADR-0010): المُجزّئ (tokenizer) وحده، بلا شجرة. من مشروع Servo، ويقرأ HTML كما تقرؤه المتصفحات. ومعه `markup5ever` و`tendril` و`string_cache` و`web_atoms` و`phf` و`siphasher` وغيرها، كلها MIT أو Apache-2.0 |
| `libc` | 0.2.191 | MIT أو Apache-2.0 | `marsa-platform` (ماك ولينكس فقط) | `O_NOFOLLOW`، وإعادة التسمية دون استبدال (`renameat2` و`renamex_np`)، والمساحة الحرة (`statvfs` و`statfs`)، وشجرة عمليات أداة الفيديو (`umask` و`kill` لمجموعة العمليات، و`PR_SET_PDEATHSIG` في لينكس، و`sysctl` و`proc_listallpids` في ماك) |
| `windows-sys` | 0.61.2 | MIT أو Apache-2.0 | `marsa-platform` (ويندوز فقط) | `MoveFileExW` مع `MOVEFILE_WRITE_THROUGH`، و`FILE_FLAG_OPEN_REPARSE_POINT`، وعدد الروابط الصلبة، والملف المتناثر (`FSCTL_SET_SPARSE`)، والمساحة الحرة (`GetDiskFreeSpaceExW`)، وحجم الملف على القرص (`GetCompressedFileSizeW`)، وشجرة عمليات أداة الفيديو في Job Object تنتهي بانتهاء مَرْسَى |
| `dunce` | 1.0.5 | CC0-1.0 أو MIT-0 أو Apache-2.0 | `marsa-core` | مسار مجلد التحميل الحقيقي دون بادئة `\\?\` في ويندوز حين لا تلزم، حتى تبقى المسارات المعروضة للمستخدم مقروءة |
| `icu_properties` | 2.4.0 | Unicode-3.0 | `marsa-core` | بيانات يونيكود (Joining_Type وCanonical_Combining_Class) لقواعد ContextJ في RFC 5892: متى يبقى ZWNJ وZWJ في اسم الملف. موجودة أصلًا في البرنامج عبر `reqwest` ← `url` ← `idna`، فلا تضيف كودًا جديدًا إلى النسخة النهائية |
| `thiserror` | 2.0.21 | MIT أو Apache-2.0 | كل المكتبات | تعريف أنواع الأخطاء |
| `jiff` | 0.2.38 | Unlicense أو MIT | `marsa-core` و`marsa-cli` (اختبارات) | الوقت المحلي والمنطقة الزمنية للنظام وقواعد التوقيت الصيفي، لجدولة القوائم. بميزاتها الافتراضية: منطقة النظام على الأنظمة الثلاثة، وقاعدة بيانات المناطق مضمّنة في ويندوز فقط (`jiff-tzdb-platform` و`jiff-tzdb`، بترخيص Unlicense أو MIT، والبيانات نفسها من IANA في الملكية العامة). ومعها `jiff-static` و`jiff-core` (Unlicense أو MIT)، و`portable-atomic` و`portable-atomic-util` (Apache-2.0 أو MIT) |
| `fluent-bundle` | 0.16.0 | Apache-2.0 أو MIT | `marsa-i18n` | تنسيق رسائل Fluent (ADR-0003) في جهة Rust: قائمة شريط النظام والإشعارات. ومعه `fluent-langneg` و`intl-memoizer` و`type-map` و`rustc-hash` (Apache-2.0 أو MIT)، و`self_cell` (Apache-2.0 أو GPL-2.0-only: نستعمله بـ Apache-2.0)، و`intl_pluralrules` 7.0.2 (Apache-2.0 أو MIT). **وهذه الأخيرة تخطئ في الجمع العربي فوق 100** (103 و111 تعطي «other»)، فلا نختار صيغ الجمع بها: انظر `icu_plurals` |
| `fluent-syntax` | 0.12.0 | Apache-2.0 أو MIT | `marsa-i18n` (عبر `fluent-bundle`، واختبارات) | قراءة ملفات `.ftl`. والاختبارات تمر بها على كل مفتاح ومتغير |
| `unic-langid` | 0.9.6 | MIT أو Apache-2.0 | `marsa-i18n` | معرّف اللغة الذي يطلبه `fluent-bundle`. ومعه `unic-langid-impl` (MIT أو Apache-2.0) |
| `icu_plurals` | 2.4.0 | Unicode-3.0 | `marsa-i18n` | فئات الجمع من CLDR للدالة `PLURAL()` في ملفات الترجمة. من عائلة ICU4X نفسها التي نستعملها (`icu_properties`). ومعها `icu_plurals_data` و`fixed_decimal` و`icu_locale_fallback` و`icu_locale_fallback_data` (Unicode-3.0) |
| `icu_locale_core` | 2.4.0 | Unicode-3.0 | `marsa-i18n` | معرّف اللغة لـ `icu_plurals`. موجودة أصلًا في البرنامج عبر `url` ← `idna` |
| `minisign-verify` | 0.3.0 | MIT | `marsa-core` | التحقق من توقيع minisign لقائمة أدوات الفيديو الموقّعة (ADR-0011)، بنفس صيغة توقيع Tauri |
| `base64` | 0.23.1 | MIT أو Apache-2.0 | `marsa-core` | فك ترميز توقيع minisign وقائمة الأدوات |
| `zip` | 9.1.2 | MIT | `marsa-core` | قراءة حزم yt-dlp الرسمية (onedir) المخزّنة أو المضغوطة بـ deflate فقط، وقراءة أرشيف ZIP في مدير الأرشيف (`archive`)، بلا AES أو bzip2 أو zstd. بميزة `deflate-flate2` وحدها |
| `flate2` | 1.1.10 | MIT أو Apache-2.0 | `marsa-core` | فك ضغط deflate لواجهة `zip`، وفك `.tar.gz` في مدير الأرشيف، بالخلفية Rust (`miniz_oxide`) لا zlib |
| `tauri` | 2.12.5 | Apache-2.0 أو MIT | `marsa-desktop` | نافذة الواجهة فوق محرّك الويب الموجود في النظام: WebView2 في ويندوز، وWKWebView في ماك، وWebKitGTK 4.1 في لينكس (LGPL، يُربط ديناميكيًا). ومعه `tauri-build` 2.7.4 ومكتباتهما (wry وtao وmuda، وgtk-rs 0.18 في لينكس). كلها في مساحة عمل `apps/desktop/src-tauri` المستقلة. **استثناء واحد في `deny.toml`**: RUSTSEC-2024-0370، أي `proc-macro-error` «غير مصونة» (وليست ثغرة)، وهي ماكرو يعمل وقت البناء فقط عبر `glib-macros` 0.18 في لينكس |
| `tauri-plugin-autostart` | 2.7.0 | Apache-2.0 أو MIT | `marsa-desktop` | التشغيل مع بدء النظام: مفتاح الريجستري في ويندوز، وLaunchAgent في ماك، وملف autostart في لينكس |
| `tauri-plugin-notification` | 2.7.0 | Apache-2.0 أو MIT | `marsa-desktop` | إشعارات النظام عند اكتمال تحميل أو فشله. **ثلاثة استثناءات في `deny.toml`** (RUSTSEC-2026-0370 و0371 و0372)، وهي «غير مصونة» لا ثغرات: `zbus_names` و`zvariant` و`zvariant_derive` دُمجت في `zbus` نفسها فلن تصدر لها نسخ، و`zbus` 5 (D-Bus في لينكس) ما زال يستخدمها عبر هذه الإضافة |
| `tauri-plugin-dialog` | 2.8.3 | Apache-2.0 أو MIT | `marsa-desktop` | نوافذ الفتح والحفظ التي يعرضها النظام نفسه: اختيار مجلد التحميل، وملف الاستيراد والتصدير. الصفحة لا تقرأ ولا تكتب ملفات بنفسها، بل تمرر المسار المختار إلى أوامر مَرْسَى |
| `tauri-plugin-opener` | 2.7.0 | Apache-2.0 أو MIT | `marsa-desktop` | «فتح» التحميل المكتمل بالبرنامج الافتراضي و«فتح المجلد» مع تحديد الملف. يستدعيه Rust وحده على مسار التحميل نفسه؛ الصفحة لا تملك صلاحيات هذه الإضافة. ومعه `open` و`is-wsl` و`is-docker` (MIT) |
| `tauri-plugin-single-instance` | 2.5.4 | Apache-2.0 أو MIT | `marsa-desktop` | نسخة واحدة من مَرْسَى للمستخدم: حين تكون النافذة مخفية في شريط النظام، فتح مَرْسَى مرة أخرى يُظهرها بدل أن يبدأ برنامجًا ثانيًا على الطوابير نفسها. لا يُستعمل مع `--data-dir`. لا يجلب حزمًا أخرى |
| `ts-rs` | 12.0.1 | MIT | `marsa-desktop` | توليد أنواع TypeScript من أنواع Rust التي تتبادلها الواجهة والمحرّك (ADR-0003)، والـ CI يتأكد أن الملفات المولَّدة محدّثة |
| `tempfile` | 3.27.0 | MIT أو Apache-2.0 | الاختبارات فقط | مجلدات مؤقتة للاختبارات |
| `winreg` | 0.55.0 | MIT | `marsa-installer` | كتابة مفتاح إلغاء التثبيت `HKCU\...\Uninstall\Marsa` (روابط ويندوز تُنشأ عبر PowerShell COM دون مكتبة جديدة) |
| `tiny_http` | 0.12.0 | MIT أو Apache-2.0 | `marsa-desktop` (الفيئة Remote) | خادم HTTP متزامن صغير للتحكم عن بعد في LAN (اختياري، مع مصادقة) |
| `getrandom` | 0.4.3 | MIT أو Apache-2.0 | `marsa-desktop` (الفيئة Remote) | عشوائية النظام لتوليد توكن التحكم عن بُعد |
| `rustls-native-certs` | 0.8 | MIT أو Apache-2.0 | `marsa-core` (محرك FTP) | حقائب شهادات النظام لفحص FTP عبر TLS |

## مكتبات الواجهة والإضافة (npm)

**الإضافة (`apps/extension`) بلا أي مكتبة وقت تشغيل:** كل ما فيها من المستودع. وهذه أدوات بنائها وفحصها فقط، لا تدخل الإضافة، ويفحص الـ CI تراخيصها بقائمة أدوات `apps/desktop/scripts/check-licenses.mjs` ويشغّل `npm audit`:

| الأداة | الإصدار | الترخيص | السبب |
|---|---|---|---|
| `typescript` | 6.0.3 | Apache-2.0 | فحص الأنواع في ملفات JavaScript (`checkJs`) |
| `vitest` | 5.0.5 | MIT | اختبارات الإضافة |
| `addons-linter` | 10.14.0 | MPL-2.0 | فاحص AMO، وهو ما يشغّله `web-ext lint`. لم نستعمل `web-ext` نفسه لأن جزءه الخاص بأندرويد يجلب `node-forge` بتحذير أمني عالٍ لا إصلاح له بعد |
| `playwright-core` | 1.63.0 | Apache-2.0 | يشغّل متصفحًا حقيقيًا في اختبار زر «نزّل الفيديو» (Chrome الموجود على جهاز الـ CI، أو Chromium محليًا). لا ينزّل متصفحًا بنفسه |

## أدوات خارجية تُشغَّل كبرامج منفصلة

| الأداة | الترخيص | ملاحظات | الحالة |
|---|---|---|---|
| ffmpeg وffprobe | LGPL 2.1 أو أحدث (بناؤنا من المصدر الرسمي، بلا `--enable-gpl` ولا `--enable-nonfree` ولا أي مكتبة خارجية) | يُبنيان في الـ CI من المصدر الرسمي الموقّع (`tools/ffmpeg`)، ويُنشران مع مصدرهما في إصدار `tools-ffmpeg-*` من هذا المستودع ([ADR-0009](adr/0009-video-tools.md)). وينزّلهما مَرْسَى عند أول تحميل فيديو بموافقة المستخدم | معتمدة |
| yt-dlp | GPLv3+ للملف المستقل كما يُبنى (المصدر Unlicense) | تُنزَّل عند أول تحميل فيديو بموافقة المستخدم، ببصمة مثبّتة من `SHA2-256SUMS` الموقّع (`tools/yt-dlp/pins.txt`) | معتمدة |
| QuickJS-NG (`qjs`) | MIT | ليوتيوب فقط، إن لم يوجد Deno أو Node. تُنزَّل مع الأدوات بموافقة المستخدم، ببصمة مثبّتة (`tools/quickjs/pins.txt`؛ لا ينشر QuickJS-NG بصمات ولا توقيعات). ولا يشغّلها yt-dlp إلا عبر `marsa-jsbox`، في صندوق وبحدود (ADR-0009، القسم 3) | معتمدة |
| Deno 2.3+ أو Node 22+ | MIT | ليوتيوب، إن ثبّتها المستخدم بنفسه: لا نوزّعها ولا ننزّلها. وتُقبل فقط إن لم يستطع غير المستخدم أو النظام تغييرها | استثناء بقرار المالك (ADR-0009) |

## الخطوط والأيقونات والصور

| المورد | الترخيص | المصدر | ملاحظات |
|---|---|---|---|
| خط IBM Plex Sans Arabic | OFL-1.1 | `@fontsource/ibm-plex-sans-arabic` | خط الواجهة داخل البرنامج، لا يُحمَّل من الإنترنت |
| أيقونات Lucide | ISC، وما جاء منها من Feather بترخيص MIT | `lucide-static` 1.61.0 | منسوخة في `apps/desktop/src/lib/lucide.ts`، ونص ترخيصها في رأس الملف وفي `public/lucide-LICENSE.txt` |
| شعار مَرْسَى | ملكية المشروع (GPL-3.0-or-later) | `assets/icon/` | مرسوم لهذا المشروع، ولا يأتي من أي برنامج آخر |

**ممنوع أي مورد من برنامج مغلق المصدر أو غير متوافق مع GPL-3.0.**

## أدوات البناء والـ CI (لا تُوزَّع مع البرنامج)

| الأداة | الترخيص | الاستخدام |
|---|---|---|
| cargo-deny | MIT أو Apache-2.0 | فحص التراخيص والثغرات في الـ CI |
| actions/checkout | MIT | سحب الكود في الـ CI |
| Swatinem/rust-cache | LGPL-3.0 | تخزين مؤقت لبناء Rust في الـ CI |
| EmbarkStudios/cargo-deny-action | Apache-2.0 | تشغيل cargo-deny في الـ CI |
| actions/upload-artifact وactions/download-artifact | MIT | نقل الملفات بين مهام الـ CI |
| actions/cache | MIT | تخزين `ffmpeg-testgen` المؤقت في الـ CI |
| msys2/setup-msys2 | MIT | بيئة بناء FFmpeg في ويندوز |
| GnuPG | GPL-3.0 أو أحدث | التحقق من توقيع مصدر FFmpeg في الـ CI |
| مُترجما FFmpeg في ويندوز (MSYS2: gcc لـ x64، وclang لـ arm64) | GPL-3.0 مع استثناء مكتبات التشغيل (libgcc)، وApache-2.0 مع استثناء LLVM (compiler-rt)، والملكية العامة/ZPL لكود بدء التشغيل في MinGW-w64 | الربط الثابت يضع في `ffmpeg.exe` كود بدء التشغيل وlibgcc (x64) أو compiler-rt (arm64) |
| Rust toolchain 1.99.1 (`rust-toolchain.toml`) | MIT أو Apache-2.0 | المترجم والأدوات |

</div>

<div dir="rtl">

## مكتبات الواجهة (npm)، في `apps/desktop`

الإصدارات مثبتة في `package-lock.json`، و`npm ci` يقرأ منه فقط. ويفحص الـ CI التراخيص بقائمة مسموحة (`scripts/check-licenses.mjs`): للمكتبات التي تدخل البرنامج قائمة، ولأدوات البناء والاختبار قائمة أوسع قليلًا. ويشغّل `npm audit` أيضًا.

| المكتبة | الإصدار | الترخيص | تدخل البرنامج؟ | السبب |
|---|---|---|---|---|
| `svelte` | 5.57.2 | MIT | نعم (وقت التشغيل) | مكتبة الواجهة (ADR-0008) |
| `@fluent/bundle` | 0.19.1 | Apache-2.0 | نعم | تنسيق رسائل Fluent في الواجهة، مع الدالة `PLURAL` |
| `@tauri-apps/api` | 2.12.4 | Apache-2.0 أو MIT | نعم | أوامر المحرّك وأحداثه، وعنوان النافذة |
| `@fontsource/ibm-plex-sans-arabic` | 5.3.0 | OFL-1.1 (الخط)، MIT (الحزمة) | نعم (ملفات الخط) | خط الواجهة داخل البرنامج، لا يُحمَّل من الإنترنت |
| `vite` | 8.3.5 | MIT | لا (بناء) | بناء الواجهة |
| `@sveltejs/vite-plugin-svelte` | 7.3.1 | MIT | لا | ربط Svelte بـ Vite |
| `typescript` | 6.0.3 | Apache-2.0 | لا | الأنواع |
| `svelte-check` | 4.7.7 | MIT | لا | فحص الأنواع وتحذيرات الوصول (أخطاء في الـ CI) |
| `@tsconfig/svelte` | 5.0.8 | MIT | لا | إعدادات TypeScript |
| `vitest` | 5.0.5 | MIT | لا (اختبار) | اختبارات منطق الواجهة |
| `@tauri-apps/cli` | 2.12.4 | Apache-2.0 أو MIT | لا (بناء) | بناء التطبيق وتوليد الأيقونات |
| `pngjs` | 7.0.0 | MIT | لا (اختبار) | قراءة صور الواجهة في اختبار الأسماء المختلطة |
| `@tauri-apps/plugin-dialog` | 2.8.3 | Apache-2.0 أو MIT | نعم | واجهة الصفحة لنوافذ الفتح والحفظ (`tauri-plugin-dialog`) |
| `@tauri-apps/plugin-notification` | 2.7.0 | Apache-2.0 أو MIT | نعم | إرسال الإشعار من الصفحة |
| `@tauri-apps/plugin-autostart` | 2.7.0 | Apache-2.0 أو MIT | نعم | تفعيل التشغيل مع بدء النظام وإيقافه من الإعدادات |
| `@resvg/resvg-js` | 2.6.2 | MPL-2.0 | لا (أداة) | رسم أيقونة شريط النظام في ماك من `marsa-mark.svg` (`scripts/tray-icon.mjs`). والصورة الناتجة في المستودع |
| `axe-core` | 4.14.0 | MPL-2.0 | لا (اختبار) | فحص إمكانية الوصول الآلي في بناء `ui-test` فقط |

</div>
