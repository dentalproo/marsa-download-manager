<div dir="rtl">

# بناء مَرْسَى على ويندوز

> **English summary.** Steps to build the Windows installer and a portable ZIP from source, the tools needed, and what the output contains. No administrator rights are needed to build or to install.

## 1. ما تحتاجه

| الأداة | الإصدار | لماذا |
|---|---|---|
| Rust | المثبّت في `rust-toolchain.toml` | المحرّك والواجهة |
| Node.js | 22 أو أحدث | بناء واجهة Svelte |
| Visual Studio Build Tools | 2022، مع «Desktop development with C++» | رابط MSVC وWindows SDK |
| WebView2 SDK | يأتي مع Tauri | واجهة النظام |
| PowerShell | 5.1 أو 7 | سكربت البناء |

ثبّت Rust من <https://rustup.rs>، وNode من <https://nodejs.org>. تأكد:

```powershell
rustc --version
cargo --version
node --version
npm --version
```

## 2. بناء كامل بأمر واحد

```powershell
.\scripts\build-windows.ps1
```

السكربت:

1. يعيد توليد الأيقونات من `assets/icon/marsa-icon.svg` عبر `scripts/make-icons.ps1`.
2. يثبّت اعتماديات الواجهة (`npm ci` إن وُجد `package-lock.json`).
3. يبني واجهة Svelte ثم مثبّت التطبيق عبر [`apps/installer`](../../apps/installer) — تطبيق Tauri مستقل يضمّ `marsa-desktop.exe` و`marsa-host.exe` (مضيف المتصفح، ADR-0005) — وليس مثبّت NSIS الذي يبنيه Tauri.
4. ينسخ المثبّت إلى `dist/windows/`.
5. ينشئ نسخة محمولة `Marsa-portable` ويضغطها ZIP.
6. يكتب `SHA256SUMS`.
7. إن وُجد `MARSA_SIGNING_KEY` (واختياري `MARSA_SIGNING_KEY_PASSWORD`)، وقّع المثبّت واكتب `latest.json` له بلا BOM.

**الناتج:**

```text
dist/windows/
├── Marsa_0.1.0_x64-setup.exe
├── Marsa_0.1.0_x64-setup.exe.sig
├── Marsa_0.1.0_portable_x64.zip
├── SHA256SUMS
└── latest.json        # موجود فقط عند وُجد MARSA_SIGNING_KEY
```

بلا `MARSA_SIGNING_KEY` يبقى البناء بلا توقيع وبلا `latest.json`، ويطبع تحذيرًا صريحًا.

## 3. بناء يدوي

```powershell
# من جذر المشروع
npm --prefix apps/desktop install
npm --prefix apps/desktop run tauri build
```

المثبّت في:

```text
apps/desktop/src-tauri/target/release/bundle/nsis/
```

والملف التنفيذي في:

```text
apps/desktop/src-tauri/target/release/marsa-desktop.exe
```

## 4. ما يحتويه المثبّت

- **تثبيت للمستخدم الحالي**، بلا صلاحيات مدير.
- **WebView2 مضمّن** (offline installer)، فلا يحتاج اتصالًا أثناء التثبيت.
- **مضيف المتصفح**: `marsa-host.exe` إلى جانب `marsa-desktop.exe` في مجلد التثبيت؛ المتصفح (Chrome/Edge/Brave/…) يبدأ المضيف، والمضيف يوصّل التطبيق عبر القناة الخاصة. بلا المضيف لا يصل أي متصفح إلى مَرْسَى (الإضافة تُعلم بذلك «لا يصل المتصفح إلى مَرْسَى»).
- **لغتا المثبّت:** العربية والإنجليزية.
- **مجلد قائمة ابدأ:** Marsa.
- **إلغاء تثبيت نظيف**: لا خدمات ولا مهام مجدولة ولا مفاتيح سجل تُترك.

## 5. النسخة المحمولة

- `Marsa.exe` مع `LICENSE` و`THIRD-PARTY-NOTICES.md` و`README.md`.
- `marsa-host.exe` إلى جانب `Marsa.exe` (اسم المضيف ثابت على ويندوز).
- لا تكتب في مجلد البرنامج؛ بيانات المستخدم في مجلد بيانات النظام المعتاد.
- تحتاج WebView2 runtime على الجهاز (يأتي مع ويندوز 11، وغالبًا مع ويندوز 10 المحدّث).

## 6. استكشاف الأخطاء

| المشكلة | السبب والحل |
|---|---|
| `link.exe not found` | ثبّت Visual Studio Build Tools مع C++ |
| `WebView2Loader.dll` مفقود | أعد بناء المشروع كاملًا، ولا تنسخ exe وحده |
| فشل `npm ci` | احذف `node_modules` و`package-lock.json` ثم أعد المحاولة |
| المثبّت لا يفتح | تحقق من بصمة SHA-256، ومن أن الملف لم يُحجب (خصائص → إلغاء الحجب) |
| «Windows protected your PC» | متوقع قبل التوقيع؛ راجع [DISTRIBUTION](DISTRIBUTION.md) |

## 7. التحقق قبل النشر

```powershell
# البصمات
Get-FileHash .\dist\windows\* -Algorithm SHA256

# لا ميزة ui-test في نسخة الإصدار
cargo test -p marsa-desktop --release
```

ولا تُنشر نسخة لم يبنها الـ CI. التفاصيل في [DISTRIBUTION](DISTRIBUTION.md).

</div>
