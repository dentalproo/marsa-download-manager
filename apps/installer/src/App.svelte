<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";

  type Lang = "ar" | "en";
  let lang = $state<Lang>("ar");
  const dir = $derived(lang === "ar" ? "rtl" : "ltr");
  $effect(() => {
    document.documentElement.dir = dir;
    document.documentElement.lang = lang;
  });

  const T: Record<Lang, Record<string, string>> = {
    ar: {
      welcomeTitle: "مرحباً بك في مَرْسَى",
      welcomeSub: "مدير تحميل هادئ وسريع، عربي أولًا، يعمل على ويندوز.",
      f1: "تسريع واستكمال بعد الانقطاع",
      f2: "تكامل كامل مع المتصفح",
      f3: "تصنيف تلقائي وإدارة ذكية للملفات",
      setupTitle: "أين تثبّت مرسى؟",
      setupSub: "اختر المسار والخيارات — يمكنك تغييرها لاحقًا من الإعدادات.",
      path: "مسار التثبيت",
      browse: "تصفح…",
      desktop: "إنشاء اختصار على سطح المكتب",
      startmenu: "إضافة إلى قائمة ابدأ",
      launch: "تشغيل مرسى بعد التثبيت",
      cancel: "إلغاء",
      back: "رجوع",
      continue_: "متابعة",
      installNow: "تثبيت مرسى",
      installing: "جارٍ تثبيت مرسى…",
      doneTitle: "اكتمل التثبيت",
      doneSub: "مرسى جاهز للعمل. استمتع بتحميلات هادئة وسريعة.",
      launchApp: "تشغيل مرسى",
      finish: "إنهاء",
      step1: "الترحيب",
      step2: "الإعداد",
      step3: "التثبيت",
      step4: "الانتهاء",
      err: "تعذّر التثبيت: {message}",
    },
    en: {
      welcomeTitle: "Welcome to Marsa",
      welcomeSub: "A calm, fast, Arabic-first download manager for Windows.",
      f1: "Acceleration and resume after disconnects",
      f2: "Full browser integration",
      f3: "Smart auto-organisation and file management",
      setupTitle: "Where to install Marsa?",
      setupSub: "Choose the path and options — you can change them later in Settings.",
      path: "Install path",
      browse: "Browse…",
      desktop: "Create a Desktop shortcut",
      startmenu: "Add to Start menu",
      launch: "Launch Marsa after install",
      cancel: "Cancel",
      back: "Back",
      continue_: "Continue",
      installNow: "Install Marsa",
      installing: "Installing Marsa…",
      doneTitle: "Setup complete",
      doneSub: "Marsa is ready. Enjoy calm, fast downloads.",
      launchApp: "Launch Marsa",
      finish: "Finish",
      step1: "Welcome",
      step2: "Setup",
      step3: "Installing",
      step4: "Done",
      err: "Install failed: {message}",
    },
  };
  const t = (k: string) => T[lang][k] ?? k;
  const win = getCurrentWindow();

  let step = $state(0);
  let path = $state("");
  let desktop = $state(true);
  let startMenu = $state(true);
  let launchAfter = $state(true);
  let busy = $state(false);
  let progress = $state(0);
  let error = $state<string | null>(null);

  // Load default path on start
  invoke<string>("get_default_path").then((p) => { if (!path) path = p; }).catch(() => { path = "C:\\Users\\you\\AppData\\Local\\Programs\\Marsa"; });

  async function browse() {
    try {
      const picked = await openDialog({ directory: true, multiple: false });
      if (typeof picked === "string") path = picked;
    } catch { /* ignore */ }
  }

  async function install() {
    busy = true; error = null; progress = 0;
    step = 2;
    // Natural steady count: 0, 1, 2, … up to 100 — one percent every 26ms,
    // about 2.6 seconds total, like a real copy. Never jumps, never freezes.
    // The hold at 99% only happens if the real install is slower than 2.6s.
    let rustDone = false;
    let failed = false;
    const installPromise = invoke<{ ok: boolean; message: string; installed_path: string }>("start_install", {
      path, desktopShortcut: desktop, startMenu, runAfter: false,
    }).then((res) => {
      if (!res.ok) throw new Error(res.message);
      rustDone = true;
    }).catch((e) => {
      failed = true;
      error = t("err").replace("{message}", String(e));
    });
    for (let p = 0; p <= 100; p++) {
      if (failed) break;
      progress = p;
      if (p === 100) break;
      // Hold at 99% while the real work finishes, counting in place.
      if (p === 99 && !rustDone) { await new Promise((r) => setTimeout(r, 90)); continue; }
      await new Promise((r) => setTimeout(r, 26));
    }
    await installPromise;
    if (!failed) {
      progress = 100;
      await new Promise((r) => setTimeout(r, 280));
      step = 3;
    } else {
      step = 1;
    }
    busy = false;
  }

  async function launchApp() {
    try { await invoke("launch_app"); } catch { /* ignore */ }
    await win.close().catch(() => {});
  }
  async function finish() { await win.close().catch(() => {}); }
  async function cancel() { await win.close().catch(() => {}); }

  const steps = ["step1", "step2", "step3", "step4"];
</script>

<div class="installer" {dir}>
  <aside class="side" data-tauri-drag-region="deep">
    <div class="brand">
      <span class="mark" aria-hidden="true">
        <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="5" r="2.2"/><path d="M12 7v13M6 11h12M8.5 16.5 12 20l3.5-3.5M5 20.5c2 .8 4 .8 7 0s5-.8 7 0"/></svg>
      </span>
      <div>
        <strong>{lang === "ar" ? "مَرْسَى" : "Marsa"}</strong>
        <span>0.1.0</span>
      </div>
    </div>
    <ol class="steps">
      {#each steps as s, i (s)}
        <li class:on={step === i} class:done={step > i}>
          <span class="dot">{step > i ? "✓" : i + 1}</span>
          {t(s)}
        </li>
      {/each}
    </ol>
    <p class="side-hint">{lang === "ar" ? "تثبيت حديث سريع وآمن" : "Modern, safe, fast install"}</p>
    <button class="lang-toggle" onclick={() => (lang = lang === "ar" ? "en" : "ar")}>{lang === "ar" ? "English" : "العربية"}</button>
  </aside>

  <main class="main">
    {#if step === 0}
      <section class="page" aria-labelledby="w-title">
        <div class="hero">
          <span class="hero-badge" aria-hidden="true">
            <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="1.7" stroke-linecap="round"><circle cx="12" cy="5" r="2.2"/><path d="M12 7v13M6 11h12M8.5 16.5 12 20l3.5-3.5M5 20.5c2 .8 4 .8 7 0s5-.8 7 0"/></svg>
          </span>
          <h1 id="w-title">{t("welcomeTitle")}</h1>
          <p>{t("welcomeSub")}</p>
        </div>
        <ul class="features">
          <li>{t("f1")}</li>
          <li>{t("f2")}</li>
          <li>{t("f3")}</li>
        </ul>
        <div class="footer">
          <button class="btn ghost-text" onclick={cancel}>{t("cancel")}</button>
          <button class="btn primary big" onclick={() => (step = 1)}>{t("continue_")}</button>
        </div>
      </section>
    {:else if step === 1}
      <section class="page" aria-labelledby="s-title">
        <h1 id="s-title">{t("setupTitle")}</h1>
        <p class="sub">{t("setupSub")}</p>
        <div class="field">
          <label class="label" for="install-path">{t("path")}</label>
          <div class="with">
            <input id="install-path" class="input" dir="ltr" bind:value={path} />
            <button class="btn" onclick={browse}>{t("browse")}</button>
          </div>
        </div>
        <div class="options">
          <label class="check"><input type="checkbox" bind:checked={desktop} />{t("desktop")}</label>
          <label class="check"><input type="checkbox" bind:checked={startMenu} />{t("startmenu")}</label>
          <label class="check"><input type="checkbox" bind:checked={launchAfter} />{t("launch")}</label>
        </div>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        <div class="footer">
          <button class="btn ghost-text" onclick={cancel}>{t("cancel")}</button>
          <div class="footer-end">
            <button class="btn" onclick={() => (step = 0)}>{t("back")}</button>
            <button class="btn primary big" disabled={busy || !path.trim()} onclick={install}>{t("installNow")}</button>
          </div>
        </div>
      </section>
    {:else if step === 2}
      <section class="page center" aria-labelledby="i-title">
        <h1 id="i-title">{t("installing")}</h1>
        <div class="ring" style="--p: {Math.round(progress)}">
          <span class="ring-inner">{Math.round(progress)}%</span>
        </div>
        <div class="bar" role="progressbar" aria-valuenow={Math.round(progress)} aria-valuemin={0} aria-valuemax={100}>
          <b style:inline-size="{progress}%"></b>
        </div>
        <p class="sub" dir="ltr">{path}</p>
      </section>
    {:else}
      <section class="page center" aria-labelledby="d-title">
        <span class="success" aria-hidden="true">
          <svg width="38" height="38" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 12.5l5 5L19.5 7.5"/></svg>
        </span>
        <h1 id="d-title">{t("doneTitle")}</h1>
        <p class="sub">{t("doneSub")}</p>
        <div class="footer center">
          {#if launchAfter}
            <button class="btn primary big" onclick={launchApp}>{t("launchApp")}</button>
          {/if}
          <button class="btn" onclick={finish}>{t("finish")}</button>
        </div>
      </section>
    {/if}
  </main>
</div>

<style>
  .installer { display: grid; grid-template-columns: 228px 1fr; height: 100vh; background: var(--surface); }
  .side {
    background: linear-gradient(180deg, var(--side), var(--sunken));
    border-inline-end: 1px solid var(--line);
    padding: 22px 16px;
    display: flex; flex-direction: column; gap: 22px;
  }
  .brand { display: flex; gap: 10px; align-items: center; }
  .brand strong { display: block; font-size: 15px; }
  .brand span { font-size: 11px; color: var(--muted); }
  .mark { width: 44px; height: 44px; display: grid; place-items: center; background: var(--accent-soft); border-radius: 12px; }
  .steps { list-style: none; display: flex; flex-direction: column; gap: 6px; }
  .steps li { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; color: var(--muted); font-size: 13px; }
  .steps li.on { background: var(--accent-soft); color: var(--accent); font-weight: 600; }
  .steps li.done { color: var(--muted); }
  .dot { width: 22px; height: 22px; border-radius: 99px; display: grid; place-items: center; font-size: 11px; background: var(--raised); border: 1px solid var(--line-strong); flex: none; }
  .steps li.on .dot { background: var(--accent); border-color: transparent; color: #fff; }
  .side-hint { margin-top: auto; font-size: 12px; color: var(--muted); padding: 0 4px; }
  .lang-toggle { border: 1px solid var(--line); background: var(--raised); color: var(--muted); border-radius: 8px; padding: 6px 10px; font-size: 12px; }
  .main { padding: 28px 32px; overflow: auto; }
  .page { display: flex; flex-direction: column; gap: 18px; min-height: 100%; animation: fade 240ms ease both; }
  @keyframes fade { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }
  .center { align-items: center; text-align: center; justify-content: center; }
  h1 { font-size: 24px; font-weight: 700; letter-spacing: -0.01em; margin: 0; }
  .sub { color: var(--muted); font-size: 13px; max-width: 52ch; line-height: 1.6; }
  .hero { display: flex; flex-direction: column; gap: 10px; align-items: flex-start; }
  .hero-badge { width: 72px; height: 72px; display: grid; place-items: center; background: var(--accent-soft); border-radius: 20px; }
  .features { list-style: none; display: flex; flex-direction: column; gap: 8px; }
  .features li { padding: 12px 16px; border-radius: 12px; background: var(--sunken); border: 1px solid var(--line); font-size: 13.5px; }
  .field { display: flex; flex-direction: column; gap: 8px; }
  .label { font-size: 12px; font-weight: 600; color: var(--muted); }
  .with { display: flex; gap: 8px; }
  .with .input { flex: 1; }
  .options { display: flex; flex-direction: column; gap: 10px; padding: 14px 16px; border-radius: 12px; background: var(--sunken); border: 1px solid var(--line); }
  .check { display: flex; gap: 10px; align-items: center; font-size: 13.5px; }
  .check input { accent-color: var(--accent); width: 16px; height: 16px; }
  .error { background: var(--red-bg); color: var(--red); border-radius: 10px; padding: 10px 12px; font-size: 12.5px; }
  .footer { display: flex; gap: 10px; align-items: center; margin-top: auto; padding-top: 10px; }
  .footer-end { display: flex; gap: 10px; margin-inline-start: auto; }
  .footer.center { justify-content: center; margin-top: 8px; }
  .ring {
    width: 120px; height: 120px; border-radius: 99px; display: grid; place-items: center;
    background: conic-gradient(var(--accent) calc(var(--p) * 1%), var(--line) 0);
    position: relative;
  }
  .ring-inner {
    width: 96px; height: 96px; border-radius: 99px; background: var(--surface);
    display: grid; place-items: center; font-size: 20px; font-weight: 700;
  }
  .bar { width: min(380px, 100%); height: 8px; border-radius: 99px; background: var(--line); overflow: hidden; }
  .bar b { display: block; height: 100%; background: var(--accent); border-radius: inherit; transition: width 300ms ease; }
  .success { width: 80px; height: 80px; border-radius: 99px; display: grid; place-items: center; background: var(--green-bg); animation: pop 320ms ease both; }
  @keyframes pop { from { transform: scale(0.85); opacity: 0; } to { transform: scale(1); opacity: 1; } }
</style>
