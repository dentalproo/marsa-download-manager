<script lang="ts">
  import Icon from "./Icon.svelte";
  import FileName from "./FileName.svelte";
  import FileMap from "./FileMap.svelte";
  import { locale } from "../i18n.svelte";
  import { left, num, percent, size, speed } from "../format";
  import { isArchiveName } from "../archive";
  import { api } from "../api";
  import type { DetailsView } from "../bindings/DetailsView";

  let { details, duplicates = 0, onback, onpause, onresume, onrefresh, onwithoutcookies }: {
    details: DetailsView;
    /** Organizer: finished files with the same name in the list. */
    duplicates?: number;
    onback: () => void;
    onpause: () => void;
    onresume: () => void;
    /** "Refresh the download link" (M5), for a paused or failed download. */
    onrefresh: () => void;
    /** A download that waits for its browser goes on without its cookies. */
    onwithoutcookies: () => void;
  } = $props();

  const d = $derived(details.download);
  const total = $derived(d.size ?? 0);
  const fraction = $derived(total ? d.done / total : d.status === "done" ? 1 : 0);
  const fmt = (n: number) => num(locale.lang, locale.digits, n, 0);
  // Security Center: Defender scan of the finished file.
  let scanning = $state(false);
  let scanResult = $state<{ status: string; message: string } | null>(null);
  // Precise bandwidth control: this download's own limit (bytes/s).
  let ownLimitText = $state("");
  let ownLimitSaid = $state<string | null>(null);
  async function scanFile() {
    if (scanning) return;
    scanning = true;
    try {
      scanResult = await api.scanFile(d.id);
    } catch {
      scanResult = { status: "unavailable", message: "scan failed" };
    } finally {
      scanning = false;
    }
  }
  function scanText() {
    if (!scanResult) return locale.t("sec-scan");
    return scanResult.status === "scanned"
      ? locale.t("sec-scanned")
      : scanResult.status === "blocked"
        ? locale.t("sec-blocked")
        : locale.t("sec-unavailable");
  }
  async function applyOwnLimit() {
    ownLimitSaid = null;
    const text = ownLimitText.trim().toLowerCase();
    if (!text) {
      try {
        await api.setLimit(d.id, null);
        ownLimitText = "";
      } catch (e) {
        ownLimitSaid = String(e);
      }
      return;
    }
    // Friendly units: "512k"/"512kb" = KiB, "2m"/"2mb" = MiB, plain = bytes/s.
    const unit = text.endsWith("mb") || text.endsWith("m") ? 1048576 : text.endsWith("kb") || text.endsWith("k") ? 1024 : 1;
    const n = Number(text.replace(/[a-z]/g, ""));
    if (!Number.isFinite(n) || n < 0) {
      ownLimitSaid = locale.t("details-unknown");
      return;
    }
    try {
      await api.setLimit(d.id, Math.round(n * unit));
      ownLimitText = "";
    } catch (e) {
      ownLimitSaid = String(e);
    }
  }
  // Smart Archive Manager: finished archives list + extract. Only what
  // Marsa reads: ZIP, TAR and .tar.gz (RAR and 7z are not read).
  const isArchive = $derived(isArchiveName(d.name));
  let archiveBusy = $state(false);
  let archiveResult = $state<string | null>(null);
  let archiveFailed = $state(false);
  async function extractArchive() {
    if (archiveBusy) return;
    archiveBusy = true;
    archiveResult = null;
    archiveFailed = false;
    try {
      const folder = await api.archiveExtract(d.id);
      archiveResult = folder;
    } catch (e) {
      archiveResult = String(e);
      archiveFailed = true;
    } finally {
      archiveBusy = false;
    }
  }

</script>

<section class="details" aria-labelledby="details-title">
  <div class="tool">
    <button class="btn quiet" id="details-back" onclick={onback} aria-label={locale.t("details-back")} title={locale.t("details-back")}><span class="flip"><Icon name="back" /></span></button>
    <span class="title-bar">{locale.t("details-title")}</span>
    <span class="end">
      {#if d.status === "active" || d.status === "waiting"}
        <button class="btn" id="details-pause" onclick={onpause}><Icon name="pause" />{locale.t("action-pause")}</button>
      {:else if d.status === "paused" || d.status === "failed"}
        {#if !d.video}<button class="btn quiet" id="details-refresh" onclick={onrefresh}><Icon name="link" />{locale.t("refresh-action")}</button>{/if}
        <button class="btn" id="details-resume" onclick={onresume}><Icon name="play" />{locale.t("action-resume")}</button>
      {:else if d.status === "done"}
        <button class="btn" id="details-scan" disabled={scanning} onclick={() => void scanFile()}>
          <Icon name="gauge" />{scanning ? "…" : scanText()}
        </button>
        {#if isArchive}
          <button class="btn" id="details-extract" disabled={archiveBusy} onclick={() => void extractArchive()}>
            <Icon name="archive" />{archiveBusy ? "…" : locale.t("arc-extract")}
          </button>
        {/if}
      {/if}
    </span>
  </div>
  <div class="body">
    <div class="main">
      <h2 id="details-title"><FileName name={d.name} /></h2>
      <p class="from">
        {locale.t("details-from-to", { site: d.host ?? "", folder: d.folder })}
      </p>
      {#if d.needs_ok}
        <div class="held" role="status">
          <Icon name="alert" />
          <span>{locale.t("details-needs-ok")}</span>
          <button class="btn" onclick={onresume}><Icon name="down" />{locale.t("add-start")}</button>
        </div>
      {/if}
      {#if d.waits_for_browser}
        <div class="browser" role="status">
          <Icon name="link" />
          <span>{locale.t("details-open-browser")}</span>
          <button class="btn quiet" onclick={onwithoutcookies}>{locale.t("details-without-cookies")}</button>
        </div>
      {/if}
      {#if d.status === "failed" && d.error}
        <p class="failed" role="status"><Icon name="alert" /><span>{locale.t("details-failed", { error: d.error })}</span></p>
      {/if}
      {#if archiveResult}
        <p class={archiveFailed ? "failed" : "from done-note"} role="status">
          <Icon name={archiveFailed ? "alert" : "check"} /><span dir="ltr">{archiveResult}</span>
        </p>
      {/if}
      <div class="total">
        <span class="big num">{percent(locale.lang, locale.digits, fraction)}</span>
        <span class="of num">
          {locale.t("size-of-total", { done: size(locale.lang, locale.digits, d.done), total: size(locale.lang, locale.digits, total) })}
        </span>
      </div>
      {#if total > 0 && details.segments.length > 0}
        <div class="map-head">
          <span>{locale.t("details-file-map", { size: size(locale.lang, locale.digits, total / 512) })}</span>
          <span class="legend">
            <span><i class="on"></i>{locale.t("details-map-done")}</span>
            <span><i class="head"></i>{locale.t("details-map-writing")}</span>
          </span>
        </div>
        <FileMap segments={details.segments} {total} label={locale.t("details-file-map", { size: size(locale.lang, locale.digits, total / 512) })} />
        <div class="lanes">
          {#each details.segments as s, i (s.start)}
            {@const len = (s.end ?? total - 1) - s.start + 1}
            <div class="lane">
              <span>{locale.t("details-part", { number: fmt(i + 1) })}</span>
              <span class="bar"><b style:inline-size="{Math.round(Math.min(s.done / len, 1) * 100)}%"></b></span>
              <span class="num">{s.done >= len ? locale.t("details-part-done") : ""}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
    <dl class="stats">
      <div><dt>{locale.t("details-speed")}</dt><dd class="num">{speed(locale.lang, locale.digits, d.speed) || "—"}</dd></div>
      <div><dt>{locale.t("details-left")}</dt><dd class="num">{left(locale.lang, locale.digits, total - d.done, d.speed) || "—"}</dd></div>
      <div><dt>{locale.t("details-parts")}</dt><dd class="num">{details.segments.length ? fmt(details.segments.length) : locale.t("details-unknown")}</dd></div>
      <div><dt>{locale.t("details-resumable")}</dt><dd>{locale.t(details.resumable === null ? "details-unknown" : details.resumable ? "details-yes" : "details-no")}</dd></div>
      <div><dt>{locale.t("details-limit")}</dt><dd class="num">{details.limit ? speed(locale.lang, locale.digits, details.limit) : locale.t("add-limit-none")}</dd></div>
      <div class="limit-row">
        <dt>{locale.t("bw-limit")}</dt>
        <dd class="limit">
          <input class="input small" bind:value={ownLimitText} dir="ltr" placeholder="e.g. 512KB, 2M" aria-label={locale.t("bw-limit")} />
          <button class="btn small" onclick={() => void applyOwnLimit()}>{locale.t("details-resume")}</button>
        </dd>
      </div>
      {#if ownLimitSaid}<p class="said" role="alert">{ownLimitSaid}</p>{/if}
      <div><dt>{locale.t("org-duplicates")}</dt><dd class="num">{duplicates > 0 ? fmt(duplicates) : locale.t("org-none")}</dd></div>
    </dl>
  </div>
</section>

<style>
  .details {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }
  .tool {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }
  .title-bar {
    font-weight: 600;
  }
  .end {
    margin-inline-start: auto;
    display: flex;
    gap: 8px;
  }
  :global([dir="rtl"]) .flip {
    display: inline-flex;
    transform: scaleX(-1);
  }
  .body {
    padding: 24px 28px;
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 28px;
    overflow: auto;
  }
  h2 {
    margin: 0;
    font-size: 21px;
    font-weight: 600;
    line-height: 1.4;
    display: flex;
    min-width: 0;
  }
  .from {
    color: var(--muted);
    margin: 2px 0 0;
    font-size: 13px;
  }
  .held {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin: 12px 0 0;
    padding: 10px 14px;
    border-radius: var(--radius-m);
    background: var(--amber-bg);
    color: var(--amber);
    font-size: 13px;
  }
  .browser {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin: 12px 0 0;
    padding: 10px 14px;
    border-radius: var(--radius-m);
    background: var(--amber-bg);
    color: var(--amber);
    font-size: 13px;
  }
  .limit { display: flex; gap: 8px; }
  .said { color: var(--red); font-size: 12px; }
  .failed {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin: 12px 0 0;
    padding: 10px 14px;
    border-radius: var(--radius-m);
    background: var(--red-bg);
    color: var(--red);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
  .total {
    display: flex;
    align-items: baseline;
    gap: 14px;
    margin-top: 18px;
  }
  .big {
    font-size: 40px;
    font-weight: 600;
    line-height: 1;
    letter-spacing: -0.01em;
  }
  .of {
    color: var(--muted);
  }
  .map-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin: 22px 0 8px;
    font-size: 12px;
    color: var(--muted);
    flex-wrap: wrap;
  }
  .legend {
    display: flex;
    gap: 12px;
  }
  .legend i {
    display: inline-block;
    width: 9px;
    height: 9px;
    border-radius: 2px;
    margin-inline-end: 4px;
  }
  .legend .on {
    background: var(--accent);
  }
  .legend .head {
    background: color-mix(in oklab, var(--accent) 45%, var(--surface));
  }
  .lanes {
    display: grid;
    gap: 7px;
    margin-top: 18px;
  }
  .lane {
    display: grid;
    grid-template-columns: 90px 1fr 70px;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: var(--muted);
  }
  .bar {
    height: 10px;
    border-radius: 99px;
    background: var(--accent-soft);
    position: relative;
    overflow: hidden;
  }
  .bar b {
    position: absolute;
    inset-block: 0;
    inset-inline-start: 0;
    background: var(--accent);
    border-radius: inherit;
  }
  .stats {
    border: 1px solid var(--line);
    border-radius: var(--radius-l);
    background: var(--surface);
    box-shadow: var(--shadow);
    overflow: hidden;
    align-self: start;
    margin: 0;
  }
  .stats div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 11px 14px;
    border-bottom: 1px solid var(--line);
    font-size: 13.5px;
  }
  .stats div:last-child {
    border-bottom: 0;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
    font-weight: 600;
  }
</style>
