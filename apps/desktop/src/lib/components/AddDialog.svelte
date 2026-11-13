<script lang="ts">
  import { tick } from "svelte";
  import QueuePick from "./QueuePick.svelte";
  import Icon from "./Icon.svelte";
  import FileName from "./FileName.svelte";
  import { nameDir } from "../fname";
  import ColorIcon, { typeIcon } from "./ColorIcon.svelte";
  import { locale } from "../i18n.svelte";
  import { categoryName as nameOf } from "../categoryName";
  import { sizeText } from "../probeText";
  import { chosenFolder, savePath, typedPath } from "../addFolder";
  import { fitWindow } from "../fitWindow";
  import type { ProbeView } from "../bindings/ProbeView";
  import type { AddRequest } from "../bindings/AddRequest";
  import type { QueueView } from "../bindings/QueueView";
  import type { CategoryView } from "../bindings/CategoryView";

  let {
    url,
    probe,
    guess = null,
    error,
    busy,
    queues,
    categories,
    onadd,
    oncancel,
    onchoosefolder,
    onnewcategory,
    onvideo,
    ongrab,
    started = false,
    connections = 8,
    maxConnections = 32,
  }: {
    url: string;
    probe: ProbeView | null;
    guess?: ProbeView | null;
    error: string | null;
    busy: boolean;
    queues: QueueView[];
    categories: CategoryView[];
    onadd: (request: AddRequest, remember: string | null) => void;
    oncancel: () => void;
    onchoosefolder: () => Promise<string | null>;
    onnewcategory: () => void;
    onvideo?: () => void;
    ongrab?: () => void;
    started?: boolean;
    connections?: number;
    maxConnections?: number;
  } = $props();

  // Protocol Manager (v1): which engine handles this link?
  function isFtpLink() {
    return /^ftp:\/\//i.test(url);
  }
  function isTorrrentLink() {
    return /^magnet:/i.test(url.trim());
  }
  let queue = $state("main");
  let ownConnections = $state<number | null>(null);
  function suspiciousDomain(host: string): boolean {
    if (!host) return false;
    return host.startsWith("xn--") || host.includes("xn--") || /^\d+\.\d+\.\d+\.\d+$/.test(host);
  }
  function connectionsTyped(box: HTMLInputElement) {
    const n = Math.round(Number(box.value));
    if (Number.isFinite(n) && n >= 1 && n <= maxConnections) ownConnections = n === connections ? null : n;
    else box.value = String(ownConnections ?? connections);
  }

  let folder = $state<string | null>(null);
  let picked = $state<string | null | undefined>(undefined);
  const shown = $derived(probe ?? guess);
  const category = $derived(picked === undefined ? (shown?.category ?? null) : picked);
  const where = $derived(
    shown
      ? chosenFolder({
          categories,
          detected: shown.category,
          picked: category,
          byHand: folder,
          detectedFolder: shown.folder,
          downloadFolder: shown.download_folder,
        })
      : { shown: "", request: null },
  );
  let remember = $state(false);
  let later = $state(false);
  let advanced = $state(false);
  let main: HTMLElement | undefined = $state();

  let focused = false;
  const page = $derived(probe?.page === true);
  const blocked = $derived(busy || (error !== null && !probe));
  $effect(() => {
    if (!shown || busy) return;
    const typing = document.activeElement instanceof HTMLInputElement || document.activeElement instanceof HTMLSelectElement;
    if (focused && (!page || typing)) return;
    focused = true;
    main?.querySelector<HTMLButtonElement>("[data-autofocus]")?.focus({ preventScroll: true });
  });

  const categoryName = (id: string) => nameOf(id, categories.find((c) => c.id === id)?.name, (k) => locale.t(k));

  let name = $state("");
  let description = $state("");
  let ownName = false;
  $effect(() => {
    if (shown && !ownName) name = shown.name;
  });
  const fullPath = $derived(savePath(where.shown, name));
  let editing = $state(false);
  let typed = $state("");
  function startEditing() {
    typed = fullPath;
    editing = true;
  }
  function takeTyped() {
    editing = false;
    const said = typedPath(typed, where.shown);
    if (said.name && said.name !== name) ownName = true;
    if (!said.name) ownName = false;
    name = said.name || (shown?.name ?? "");
    if (said.folder !== null) folder = said.folder;
  }
  function add(start: boolean, into = queue) {
    if (editing) takeTyped();
    later = false;
    const own = name.trim();
    const request = {
      url,
      name: own && own !== shown?.name ? own : null,
      category: picked === undefined ? null : category,
      description: description.trim() || null,
      queue: into,
      folder: where.request,
      start,
      limit: null,
      connections: ownConnections,
    };
    if (!busy) onadd(request, remember && folder !== null ? category : null);
  }
  async function change() {
    const chosen = await onchoosefolder();
    if (chosen !== null) folder = chosen;
  }
  async function openLater() {
    later = !later;
    if (later) {
      await tick();
      main?.querySelector<HTMLButtonElement>(".queues [role='menuitem']")?.focus();
    }
  }
  function onQueuesKey(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const items = [...(main?.querySelectorAll<HTMLButtonElement>(".queues [role='menuitem']") ?? [])];
    const at = items.indexOf(document.activeElement as HTMLButtonElement);
    items[(at + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length]?.focus();
  }
  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      if (event.target instanceof HTMLButtonElement || later || blocked || page || event.isComposing) return;
      event.preventDefault();
      add(true);
      return;
    }
    if (event.key !== "Escape") return;
    event.preventDefault();
    if (later) {
      later = false;
      document.getElementById("add-later-more")?.focus();
    } else oncancel();
  }
</script>

<svelte:window {onkeydown} />
<svelte:document onpointerdown={(e) => later && !(e.target as Element).closest?.(".split") && (later = false)} />

<main class="info" bind:this={main} use:fitWindow aria-labelledby="add-title">
  <h1 id="add-title" class="visually-hidden">{locale.t("info-title")}</h1>

  <div class="hero">
    <span class="hero-badge" aria-hidden="true"><ColorIcon name={typeIcon(shown?.category ?? null)} size={28} /></span>
    <div class="hero-text">
      <p class="hero-kicker">{locale.t("sli-title")}{" "}·{isFtpLink() ? " FTP" : isTorrrentLink() ? " Torrent" : ""}</p>
      <p class="hero-name" dir={nameDir(name)}><FileName {name} /></p>
      <p class="hero-meta">
        <span class="size">{probe ? sizeText(locale.t, locale.lang, locale.digits, probe) : locale.t("sli-unknown-size")}</span>
        {#if shown?.category}<span>{categoryName(shown.category)}</span>{/if}
        {#if probe?.host}<span dir="ltr">{probe.host}</span>{/if}
        {#if probe}
          <span class="chip" class:ok={probe.resumable}><Icon mono name={probe.resumable ? "check" : "circle-alert"} size={12} />{locale.t(probe.resumable ? "add-resumable" : "add-not-resumable")}</span>
        {:else}
          <span class="muted">{locale.t("add-checking")}</span>
        {/if}
      </p>
    </div>
  </div>

  {#if probe && !page}
    <div class="sli" role="status" aria-label={locale.t("sli-title")}>
      <p class="sli-row"><Icon mono name="check" size={14} /><span>{locale.t("sli-connection")}</span></p>
      {#if "direct" in probe && probe.direct}
        <p class="sli-row"><Icon mono name="check" size={14} /><span>{locale.t("sli-direct")}</span></p>
      {/if}
      {#if probe.size !== null}
        <p class="sli-row"><Icon mono name="check" size={14} /><span>{locale.t("sli-size-known")}</span></p>
      {/if}
      {#if "redirected" in probe && probe.redirected}
        <p class="sli-row warn"><Icon mono name="circle-alert" size={14} /><span>{locale.t("sli-redirected")}</span></p>
      {/if}
      {#if "duplicate" in probe && probe.duplicate}
        <p class="sli-row warn"><Icon mono name="circle-alert" size={14} /><span>{locale.t("sli-duplicate")}</span></p>
      {/if}
    </div>
    <div class="sli sec" role="status" aria-label={locale.t("sec-title")}>
      <p class="kicker">{locale.t("sec-title")}</p>
      <p class="sli-row"><Icon mono name="check" size={14} /><span>{locale.t("sec-tls")}</span></p>
      {#if suspiciousDomain((probe.host ?? "").toLowerCase())}
        <p class="sli-row warn"><Icon mono name="circle-alert" size={14} /><span>{locale.t("sec-domain-suspicious")} — {probe.host}</span></p>
      {:else}
        <p class="sli-row muted"><Icon mono name="circle-alert" size={14} /><span>{locale.t("sec-domain")}: {probe.host ?? ""}</span></p>
      {/if}
    </div>
  {/if}

  {#if error}
    <p class="alert error" role="alert"><Icon mono name="circle-alert" size={16} /><span>{error}</span></p>
  {:else if page}
    <p class="alert warn" role="alert"><Icon mono name="circle-alert" size={16} /><span>{locale.t("add-web-page")}</span></p>
  {/if}
  {#if started}
    <p class="note">{locale.t("add-started-note")}</p>
  {/if}

  <div class="essential">
    <div class="field">
      <label class="label" for="add-name">{locale.t("add-save-as")}</label>
      <div class="with">
        <div class="namebox">
          <input
            id="add-name"
            class="input"
            class:quiet={!editing}
            dir="ltr"
            autocomplete="off"
            spellcheck="false"
            value={editing ? typed : fullPath}
            oninput={(e) => (typed = e.currentTarget.value)}
            onfocus={startEditing}
            onblur={takeTyped}
          />
          {#if !editing}
            <span class="shown" dir="ltr" aria-hidden="true">
              <bdi class="dir">{fullPath.slice(0, fullPath.length - name.length)}</bdi><span class="file" dir={nameDir(name)}><FileName {name} /></span>
            </span>
          {/if}
        </div>
        <button class="btn square" title={locale.t("add-choose-folder")} aria-label={locale.t("add-choose-folder")} onclick={() => void change()}>
          <Icon mono name="folder" size={17} />
        </button>
      </div>
    </div>
  </div>

  <button class="advanced-toggle" aria-expanded={advanced} onclick={() => (advanced = !advanced)}>
    <span>{locale.t("add-folder")} · {locale.t("add-category")} · {locale.t("add-queue")}</span>
    <Icon mono name={advanced ? "chevron-down" : "chevron-down"} size={16} />
  </button>

  {#if advanced}
    <div class="advanced">
      <div class="grid-2">
        <div class="field">
          <label class="label" for="add-category">{locale.t("add-category")}</label>
          <div class="value-wrapper">
            <select
              id="add-category"
              class="select"
              value={category ?? ""}
              onchange={(e) => (picked = e.currentTarget.value || null)}
            >
              {#each categories as c (c.id)}
                <option value={c.id}>
                  {c.id === shown?.category && shown.category_site ? locale.t("add-category-site", { category: categoryName(c.id), site: shown.category_site }) : categoryName(c.id)}
                </option>
              {/each}
              <option value="">{locale.t("category-none")}</option>
            </select>
            <button class="btn icon quiet" title={locale.t("add-new-category")} aria-label={locale.t("add-new-category")} onclick={onnewcategory}>
              <Icon mono name="circle-plus" size={15} />
            </button>
          </div>
        </div>
        <div class="field">
          <label class="label" for="add-queue">{locale.t("add-queue")}</label>
          <div class="value-wrapper">
            <Icon mono name="list-ordered" size={15} />
            <QueuePick id="add-queue" names={queues.map((q) => q.name)} bind:value={queue} disabled={page} />
          </div>
        </div>
      </div>
      <div class="grid-2">
        <div class="field">
          <label class="label" for="add-conns">{locale.t("add-connections")}</label>
          <input
            id="add-conns"
            class="input num"
            type="number"
            min="1"
            max={maxConnections}
            value={ownConnections ?? connections}
            disabled={page}
            onchange={(e) => connectionsTyped(e.currentTarget)}
          />
        </div>
        <div class="field">
          <label class="label" for="add-description">{locale.t("add-description")}</label>
          <input
            id="add-description"
            class="input"
            dir="auto"
            maxlength="1000"
            placeholder={locale.t("add-description-hint")}
            disabled={page}
            bind:value={description}
          />
        </div>
      </div>
      {#if shown && category && folder !== null}
        <label class="check">
          <input type="checkbox" bind:checked={remember} />
          {locale.t("add-remember", { category: categoryName(category) })}
        </label>
      {/if}
    </div>
  {/if}

  <div class="actions">
    <button class="btn ghost-text" onclick={oncancel}>{locale.t("add-cancel")}</button>
    <div class="buttons">
      <div class="split">
        <button class="btn" disabled={blocked} onclick={() => add(false)}>{locale.t("add-later")}</button>
        <button
          id="add-later-more"
          class="btn more"
          disabled={blocked}
          aria-label={locale.t("add-later-queue")}
          aria-haspopup="menu"
          aria-expanded={later}
          onclick={() => void openLater()}
        >
          <Icon mono name="chevron-down" size={14} />
        </button>
        {#if later}
          <div class="queues" role="menu" tabindex="-1" aria-label={locale.t("add-later-queue")} onkeydown={onQueuesKey}>
            {#each queues as q (q.name)}
              <button class="q" role="menuitem" onclick={() => add(false, q.name)}>
                <bdi>{q.name === "main" ? locale.t("queue-main") : q.name}</bdi>
              </button>
            {/each}
          </div>
        {/if}
      </div>
      {#if page && onvideo}
        <button class="btn primary big" onclick={onvideo} data-autofocus>
          <Icon mono name="download" size={17} />{locale.t("add-video-from-page")}
        </button>
      {:else if page && ongrab}
        <button class="btn primary big" onclick={ongrab} data-autofocus>
          <Icon mono name="download" size={17} />{locale.t("disc-scan")}
        </button>
      {:else}
        <button class="btn primary big" disabled={blocked} onclick={() => add(true)} data-autofocus>
          <Icon mono name="download" size={17} />{locale.t("add-start")}
        </button>
      {/if}
    </div>
  </div>
</main>

<style>
  .info {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    background: var(--surface);
    box-sizing: border-box;
  }
  .hero {
    display: flex;
    gap: 14px;
    align-items: center;
    padding: 14px 16px;
    border-radius: var(--radius-l);
    background: var(--sunken);
    border: 1px solid var(--line);
  }
  .hero-badge {
    width: 56px;
    height: 56px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 16px;
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .hero-text {
    min-width: 0;
    flex: 1;
  }
  .hero-kicker {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .sli {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 16px;
    border-radius: var(--radius-l);
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .sli-row {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 0;
    font-size: 12.5px;
    color: var(--green);
  }
  .sli-row.warn { color: var(--amber); }
  .sli-row.muted { color: var(--muted); }
  .sli .kicker {
    margin: 0 0 2px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .hero-name {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hero-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 10px;
    align-items: center;
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--muted);
  }
  .size {
    font-weight: 700;
    color: var(--ink);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 99px;
    font-size: 11px;
    font-weight: 600;
    background: var(--amber-bg);
    color: var(--amber);
  }
  .chip.ok {
    background: var(--green-bg);
    color: var(--green);
  }
  .muted { color: var(--muted); }
  .alert {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    font-size: 12.5px;
    line-height: 1.55;
    border-radius: 10px;
    padding: 10px 12px;
    margin: 0;
  }
  .alert.error { background: var(--red-bg); color: var(--red); }
  .alert.warn { background: var(--amber-bg); color: var(--amber); }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }
  .essential .field { display: flex; flex-direction: column; gap: 6px; }
  .label { font-size: 12px; font-weight: 600; color: var(--muted); }
  .with { display: flex; gap: 8px; }
  .input {
    width: 100%;
    min-height: 40px;
    padding: 8px 12px;
    background: var(--sunken);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-m);
    color: var(--ink);
    font: inherit;
    font-size: 13.5px;
  }
  .input:focus {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 22%, transparent);
    background: var(--surface);
  }
  .namebox { display: grid; flex: 1; min-width: 0; }
  .namebox > * { grid-area: 1 / 1; }
  .quiet { color: transparent; }
  .shown {
    display: flex;
    gap: 0;
    align-items: center;
    min-height: 40px;
    padding: 8px 12px;
    font-size: 13.5px;
    white-space: nowrap;
    overflow: hidden;
  }
  .shown .dir { color: var(--muted); overflow: hidden; text-overflow: ellipsis; flex: 0 1 auto; min-width: 0; }
  .shown .file { flex: 0 0 auto; max-width: 80%; overflow: hidden; text-overflow: ellipsis; color: var(--ink); font-weight: 600; }
  .advanced-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 38px;
    padding: 0 12px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--radius-m);
    background: none;
    color: var(--muted);
    font: inherit;
    font-size: 12.5px;
    cursor: pointer;
  }
  .advanced-toggle:hover { background: var(--sel); color: var(--ink); }
  .advanced {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border-radius: var(--radius-l);
    background: var(--side);
    border: 1px solid var(--line);
  }
  .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .field { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .value-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--sunken);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-m);
    padding: 0 10px;
    min-height: 40px;
    color: var(--muted);
  }
  .select {
    flex: 1;
    min-width: 0;
    border: 0;
    background: none;
    font: inherit;
    color: var(--ink);
    outline: none;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12.5px;
    color: var(--muted);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding-top: 2px;
  }
  .buttons { display: flex; gap: 8px; align-items: center; margin-inline-start: auto; }
  .ghost-text { border-color: transparent; background: none; }
  .big { min-height: 44px; padding: 0 22px; font-size: 14px; }
  .split { position: relative; display: flex; }
  .split > .btn:first-child { border-start-end-radius: 0; border-end-end-radius: 0; }
  .more { border-start-start-radius: 0; border-end-start-radius: 0; border-inline-start: 0; padding-inline: 7px; }
  .queues {
    position: absolute;
    bottom: calc(100% + 4px);
    inset-inline-start: 0;
    min-width: 100%;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 8px;
    box-shadow: var(--shadow);
    padding: 4px;
    display: flex;
    flex-direction: column;
    z-index: 5;
  }
  .q {
    border: 0;
    background: none;
    text-align: start;
    padding: 6px 10px;
    border-radius: 6px;
    font: inherit;
    color: var(--ink);
  }
  .q:hover, .q:focus-visible { background: var(--sel); outline: none; }
</style>
