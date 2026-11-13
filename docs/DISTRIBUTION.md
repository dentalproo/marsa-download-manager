<div dir="rtl">

# توزيع مَرْسَى على ويندوز

> **English summary.** Marsa is distributed as an NSIS installer for the current user and as a portable ZIP, both with SHA-256 sums. Releases are built by CI only. The executable is not code-signed with a commercial certificate in `0.1.x`, so SmartScreen may warn; the project publishes sums and a signed update channel. A winget manifest lets users install with one command once the release exists.

## 1. القنوات

| القناة | الملف | لمن |
|---|---|---|
| المثبّت | `Marsa_0.1.0_x64-setup.exe` | معظم المستخدمين |
| المحمولة | `Marsa_0.1.0_portable_x64.zip` | من لا يريد تثبيتًا |
| winget | `Marsa.Marsa` | التثبيت بأمر واحد |
| GitHub Releases | كل ما سبق + `SHA256SUMS` | المصدر الرسمي الوحيد |

وفي الإصدار نفسه `Marsa_0.1.0_x64-setup.exe.sig` (توقيع المحدّث)، و`latest.json`. و**العنوان الذي يسأل عنه المحدّث** هو `releases/download/updater/latest.json`، أي نسخة واحدة تُستبدل عند كل إصدار، لا `releases/latest`.

## 2. البصمات

كل إصدار معه `SHA256SUMS`:

```powershell
Get-FileHash .\Marsa_0.1.0_x64-setup.exe -Algorithm SHA256
# قارن الناتج بسطر الملف
```

على ماك ولينكس:

```bash
sha256sum -c SHA256SUMS
```

## 3. التوقيع والثقة

- **`0.1.x` غير موقّع بشهادة تجارية.** شهادة توقيع الكود (EV) مكلفة وتحتاج هوية قانونية، والقرار متروك للمالك.
- **ماذا يعني ذلك عمليًا:**
  - قد يظهر «Windows protected your PC» عند أول تشغيل. اضغط «More info» ثم «Run anyway» فقط إذا طابقت البصمة.
  - لا يغيّر التوقيع سلوك البرنامج، لكنه يقلل التحذير ويثبت الناشر.
- **التحديثات موقّعة بمفتاح المشروع** (minisign/Ed25519) حتى لو لم يكن الملف التنفيذي موقّعًا بشهادة تجارية. التوقيع يثبت أن التحديث جاء من المشروع.
- **عند شراء شهادة لاحقًا:** تُوقَّع الملفات في الـ CI، ويبقى المفتاح الخاص خارج المستودع، ويُوثَّق التغيير في `CHANGELOG`.

## 4. winget

الملفات في [`packaging/winget/`](../packaging/winget/):

```text
packaging/winget/
├── Marsa.Marsa.yaml
├── Marsa.Marsa.locale.ar-SA.yaml
├── Marsa.Marsa.locale.en-US.yaml
└── Marsa.Marsa.installer.yaml
```

قبل كل إصدار:

1. املأ `PackageVersion` و`InstallerUrl` و`InstallerSha256` في `Marsa.Marsa.installer.yaml`.
2. تحقق محليًا:
   ```powershell
   winget validate --manifest packaging\winget
   ```
3. أرسل الملفات إلى مستودع `microsoft/winget-pkgs` في مسار `manifests/m/Marsa/Marsa/<version>/`.

## 5. ما لا نفعله

- **لا ننشر ملفًا لم يبنه الـ CI.**
- **لا نعيد توقيع** ملفات طرف ثالث (yt-dlp وFFmpeg) على أنها منّا؛ تُنزَّل من مصادرها وتُتحقق بصماتها.
- **لا نضع مفاتيح خاصة** في المستودع أو في مخرجات البناء.
- **لا نوزّع** نسخة `ui-test` أو نسخة تطوير.

## 6. التحديث

- **التحديث الموقّع مبني** وفق [ADR-0011](adr/0011-signed-updates.md): مَرْسَى يسأل `latest.json` على وسم `updater`، ويتحقق من توقيعه بأحد مفتاحَي المشروع قبل أن يعرض «حدّث الآن»، ثم يتحقق من الملف نفسه بعد تنزيله. ولا شيء يُنزَّل أو يُثبَّت بلا موافقة المستخدم.
- **التوقيع minisign/Ed25519**، والمفتاحان العامان في `apps/desktop/src-tauri/update-keys/`، والخاص خارج المستودع ([SIGNING](SIGNING.md)).
- **`SHA256SUMS` بلا توقيع بعد**؛ `SHA256SUMS.minisig` في ADR-0011 **لم يُبنَ**.
- **لم يُختبر تحديث حقيقي من إصدار منشور إلى التالي**، ولا إعادة التشغيل بعد الاستبدال: لا إصدار منشور بعد، ولا إصدار `updater`. وما يثبته الاختبار الآن هو قبول تحديث صحيح ورفض توقيع خاطئ وشهادة TLS خاطئة على سيرفر محلي (`crates/marsa-core/tests/update_manifest.rs`).

</div>
