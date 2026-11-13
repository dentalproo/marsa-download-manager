<div dir="rtl">

# توقيع التحديثات (ADR-0011)

> **English summary.** Updates are signed with Tauri's minisign (Ed25519) keys. The two public keys live in `apps/desktop/src-tauri/update-keys/` (`signing.pub` and `emergency.pub`), and the signing one is also in `plugins.updater.pubkey` in `tauri.conf.json`. The private halves never enter the repository. The release workflow signs the installer and writes `latest.json`; the app checks the manifest and the file itself before Tauri's updater is allowed to install anything. Nothing is downloaded or installed without the user's consent.

## 1. المفتاحان

| الاسم | العام في المستودع | البصمة (key id) | الخاص |
|---|---|---|---|
| `signing` | `apps/desktop/src-tauri/update-keys/signing.pub` | `9D2B8D0EB24278BC` | خارج المستودع، وفي `TAURI_SIGNING_PRIVATE_KEY` في بيئة `release` |
| `emergency` | `apps/desktop/src-tauri/update-keys/emergency.pub` | `76141C6BF60104D7` | خارج المستودع، **بلا إنترنت**، عند المالك |

- **الملفان العامان هما ما يُبنى في البرنامج**: `crates/marsa-core/src/update.rs` يضمّهما بـ `include_str!`، فيقبل مَرْسَى توقيعًا بأي منهما. و`plugins.updater.pubkey` في `tauri.conf.json` هو نفسه `signing.pub` حرفًا بحرف، واختبار `tauri_uses_the_signing_key_the_engine_carries` يثبت ذلك.
- **الخاص لا يدخل المستودع أبدًا**، ولا مخرجات البناء، ولا السجل. ملف المفتاح الخاص في `~/.config/marsa/signing.key` و`emergency.key` على جهاز المالك، ونسخة في مدير كلمات المرور.
- **المفتاحان وُلّدا بلا كلمة مرور** (طلب المالك، 2026-11-12). يمكن إعادة توليدهما بكلمة مرور قبل أول إصدار منشور (القسم 5)، ولا يغيّر ذلك شيئًا في البرنامج: المفتاح العام وحده هو ما يُبنى فيه.
- **مفتاح الطوارئ لا يُستعمل إلا عند تسرّب مفتاح التوقيع** (القسم 4 من ADR-0011). واستعماله يُكتب في السجل.

## 2. ما يفحصه مَرْسَى قبل أن يعرض تحديثًا

الفحص في `crates/marsa-core/src/update.rs`، والطلب بمحرّك مَرْسَى نفسه (فسياسة TLS نفسها، و`User-Agent` نفسها):

- **العنوان** (`MANIFEST`): `https://github.com/dentalproo/marsa-download-manager/releases/download/updater/latest.json`. وسم `updater` لا `releases/latest`، لأن `latest` يترك الإصدارات التجريبية وكل `0.x` تجريبي. وworkflow الإصدار يستبدل هذا الملف الواحد عند كل إصدار.
- **الحجم**: 64 كيلوبايت على الأكثر (`MAX_MANIFEST`).
- **الصيغة**: `latest.json` بحقول معروفة فقط (`deny_unknown_fields`)، وفيه `version` و`platforms` (أو الشكل الديناميكي `url` و`signature` الذي يعرفه Tauri أيضًا).
- **النسخة**: `X.Y.Z` فقط، و**أحدث** من النسخة العاملة، وإلا لا يُعرض شيء (فلا إرجاع إلى نسخة أقدم).
- **النظام**: لا يُعرض شيء إن لم يكن للنظام الحالي ملف (`windows-x86_64` مثلًا)، مثل `.deb` في لينكس.
- **رابط الملف**: تحت `https://github.com/dentalproo/marsa-download-manager/releases/download/v<النسخة>/` وحدها، وحروف ASCII فقط، وبلا `..` ولا `%` ولا `?` ولا `#` ولا `\` ولا مسافة ولا رمز تحكم.
- **المفتاح**: بصمة المفتاح في التوقيع هي بصمة أحد المفتاحين المبنيين.
- **التعليق الموثوق**: يحمل `version:<النسخة>`، فلا يُقرن رقم جديد بملف قديم.
- **الملف نفسه**: بعد التنزيل، `Offer::verify` يتحقق من التوقيع على البايتات المنزّلة بالنسخة المعلنة.

ثم يُسلَّم **نص `latest.json` نفسه** إلى Tauri (`Updater::restore_update`) بالمفتاح الذي وقّعه، ويقارن مَرْسَى ما فهمه Tauri بما فهمه هو (النسخة والرابط والتوقيع) قبل أن يعرض «حدّث الآن». و`requireSignedVersion` مفعّل، فـ Tauri يرفض أيضًا توقيعًا لنسخة أخرى.

**لا شيء يُنزَّل ولا يُثبَّت قبل موافقة المستخدم**، وعند فشل أي فحص لا يُثبَّت شيء ويُكتب سطر في السجل.

## 3. عند بناء إصدار (الـ CI وحده)

`.github/workflows/release.yml`، على وسم `vX.Y.Z` من `main`:

1. يتحقق أن الوسم هو `v` + `tauri.conf.json`'s `version`، وأنه على `main`.
2. يكتب المفتاح الخاص من `TAURI_SIGNING_PRIVATE_KEY` في ملف مؤقت (`RUNNER_TEMP`)، ويحذفه في `finally` مهما حدث. ولا يطبعه.
3. يشغّل `scripts/build-windows.ps1` بـ `MARSA_SIGNING_KEY` (وكلمة المرور إن وُجدت).
4. يتحقق من وجود `dist/windows/latest.json` (فلو لم يُوقَّع الملف، يفشل البناء).
5. ينشر الإصدار (و`--prerelease` لكل `0.x`) بملفاته: المثبّت، وتوقيعه `.sig`، والنسخة المحمولة، و`SHA256SUMS`، و`latest.json`.
6. ينشئ إصدار `updater` (تجريبيًا) إن لم يكن موجودًا، ويرفع إليه `latest.json` وحده بـ `--clobber`. **هذا هو العنوان الذي يسأل عنه مَرْسَى.**

المفتاح في **GitHub Actions secrets** داخل environment اسمه `release`، بموافقة المالك على كل تشغيل (ADR-0011، القسم 3).

## 4. عند بناء محلي

```powershell
# بلا توقيع: مثبّت ومحمول وSHA256SUMS فقط، ولا latest.json
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-windows.ps1

# بتوقيع: نفس ما يفعله الـ CI
$env:MARSA_SIGNING_KEY = 'C:\Users\<أنت>\.config\marsa\signing.key'
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-windows.ps1
```

- `MARSA_SIGNING_KEY_PASSWORD` إن كان للمفتاح كلمة مرور.
- `MARSA_DOWNLOADS` لتغيير مكان الملفات في `latest.json` (يجب أن ينتهي بـ `/`). الافتراضي هو إصدارات هذا المستودع.
- البناء بلا مفتاح **ليس خطأ**: يطبع تحذيرًا صريحًا، ويكتب `SHA256SUMS` بلا `latest.json`. ولا شيء يستطيع التحديث إليه، وهذا مقصود.
- السكربت يوقّع بـ `node_modules/.bin/tauri.cmd signer sign -f <key> --app-version <v>`، أي نفس نسخة Tauri المثبّتة التي تبني البرنامج.

**لماذا لا `createUpdaterArtifacts`؟** لأن المثبّت عندنا هو `apps/installer` (تطبيق Tauri مستقل يضمّ `marsa-desktop.exe` و`marsa-host.exe`)، لا مثبّت NSIS الذي يبنيه Tauri. فالسكربت يوقّع المثبّت الناتج ويكتب `latest.json` بيده. وفائدة إضافية: البناء المحلي بلا مفتاح لا يحتاج أي متغير توقيع.

## 5. توليد المفتاحين أو تبديلهما

```powershell
# بلا كلمة مرور (ما فعلناه)، أو -p لطلبها
npx @tauri-apps/cli signer generate -w ~/.config/marsa/signing.key
npx @tauri-apps/cli signer generate -w ~/.config/marsa/emergency.key
```

ثم:

1. محتوى `signing.key.pub` في `update-keys/signing.pub`، و`emergency.key.pub` في `update-keys/emergency.pub`.
2. نفس محتوى `signing.pub` في `plugins.updater.pubkey` في `tauri.conf.json`.
3. المفتاح الخاص في بيئة `release` (`TAURI_SIGNING_PRIVATE_KEY` و`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`)، ونسخة في مدير كلمات المرور.
4. **أعد توقيع الـ fixtures**: `crates/marsa-core/tests/fixtures/update/` موقّعة بالمفتاحين الحاليين، واختبارات `update::` تفشل إن تغيّرا. و`tests/fixtures/tools-list/` كذلك.
5. سطر في `CHANGELOG` بالبصمة القديمة والجديدة.

**تبديل مفتاح مسرّب** له خطة مستقلة (ADR-0011، القسم 4): إصدار يوقَّع بمفتاح الطوارئ يزيل المسرّب ويضيف مفتاحين جديدين، وتنبيه في `SECURITY.md`.

## 6. التحقق اليدوي من توقيع

ملف `.pub` هو نص minisign مغلّف بـ base64 (هكذا يكتبه Tauri). لفكّه:

```powershell
$pub = (Get-Content apps\desktop\src-tauri\update-keys\signing.pub -Raw).Trim()
[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($pub))
# untrusted comment: minisign public key: 9D2B8D0EB24278BC
# RWS8eEKyDo0rncWY/3f90COTZKpPBFiyjYaVlo/DBaD2TuSxRxfZANEl
```

والسطر الثاني هو المفتاح العام الذي تقبله أداة `minisign` في `-P`. وأي أداة minisign تتحقق من التوقيع على الملف نفسه؛ **لم يُشغَّل هذا الفحص بأداة minisign خارجية بعد** (لم يُختبر).

## 7. ما لم يُختبر

- **تحديث حقيقي من إصدار منشور إلى التالي**: لا يوجد بعد إصدار منشور ولا إصدار `updater`، فالعنوان يرد 404 اليوم، وفحص البدء يكتب سطرًا في السجل. الطريق كله من التنزيل إلى إعادة التشغيل **لم يُختبر على جهاز حقيقي**.
- **سلوك SmartScreen بعد التحديث**، و**قفل `marsa-desktop.exe`** أثناء الاستبدال (فيه إعادة محاولة عشر ثوانٍ في المثبّت، بلا اختبار على جهاز بطيء أو برنامج حماية).
- **توقيع `SHA256SUMS`** (minisign بجانبه، ADR-0011 القسم 3/القرار 9): **لم يُبنَ**. البصمات تُنشر بلا توقيع.
- **قائمة الأدوات `tools.json`**: التحقق منها في المحرّك مبني ومختبر بـ fixtures، لكن **لا workflow ينشرها ولا إصدار `tools-list`**، فلا قائمة حقيقية تُجرَّب من الطرفين.

</div>


