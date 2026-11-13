<div dir="rtl">

# مفاتيح تحديث مَرْسَى

> **English summary.** The two public keys Marsa accepts an update from. The private halves live outside this repository. `signing.pub` is also `plugins.updater.pubkey` in `tauri.conf.json`. See `docs/SIGNING.md`.

مَرْسَى يقبل تحديثًا فقط إذا وقّعه أحد هذان المفتاحين العامين:

| الملف | البصمة | لمن |
|---|---|---|
| `signing.pub` | `9D2B8D0EB24278BC` | التوقيع العادي، في بيئة `release` في GitHub |
| `emergency.pub` | `76141C6BF60104D7` | الطوارئ، بلا إنترنت، عند المالك |

- **الملفان نص minisign مغلّف بـ base64**، كما يكتبه `tauri signer generate`.
- **`signing.pub` هو نفسه `plugins.updater.pubkey` في `tauri.conf.json`**، حرفًا بحرف. اختبار `tauri_uses_the_signing_key_the_engine_carries` يفشل إن اختلفا.
- **المحرّك يضمّ الملفين** (`crates/marsa-core/src/update.rs`، `KEYS`)، فيقبل توقيعًا بأي منهما.
- **لا مفتاح خاص هنا، ولا نسخة منه.** ملف المفتاح الخاص خارج المستودع، ونسخة في مدير كلمات المرور.
- **المفتاحان وُلّدا بلا كلمة مرور** (2026-11-12). إعادة التوليد بكلمة مرور ممكنة قبل أول إصدار منشور، بلا تغيير في البرنامج.

للتوليد أو التبديل أو التحقق اليدوي: [`docs/SIGNING.md`](../../../docs/SIGNING.md). وللتصميم كله: [`docs/adr/0011-signed-updates.md`](../../../docs/adr/0011-signed-updates.md).

</div>
