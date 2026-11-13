<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";

  type File = { name: string; length: number; included?: boolean; components?: string[] };
  type Stat = { downloaded_bytes?: number; uploaded_bytes?: number; download: { download_speed?: number }; upload: { upload_speed?: number }; live: { snapshot: { peer_stats: { live_seeders?: number; live?: number } } } };
  type Item = { id?: number; name?: string; total_pieces?: number; files?: File[] | null; stats?: Stat | null };

  let text = $state("");
  let result = $state<string | null>(null);
  let filter = $state("all");
  let items = $state<Item[]>([]);
  let busy = $state(false);
  let checked = $state<Set<string>>(new Set());

  async function addMagnet() {
    if (!text.trim()) return;
    busy = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      result = await invoke<string>("torrent_add", { magnet: text.trim() });
      text = "";
      await refresh();
    } catch (e) {
      result = String(e);
    } finally {
      busy = false;
    }
  }
  async function refresh() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const raw = await invoke<string>("torrent_list");
      const data = JSON.parse(raw) as Item[] | { torrents?: Item[] };
      items = Array.isArray(data) ? data : data.torrents ?? [];
    } catch (e) {
      result = String(e);
    }
  }
  async function act(id: number, action: "pause" | "start" | "forget") {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke(action === "pause" ? "torrent_pause" : action === "start" ? "torrent_start" : "torrent_forget", { id });
      await refresh();
    } catch (e) {
      result = String(e);
    }
  }
  async function applyFiles(id: number) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const ids = [...checked].map((s) => Number(s)).filter(Number.isFinite);
      await invoke("torrent_select_files", { id, files: ids });
      result = `files ${ids.length}`;
    } catch (e) {
      result = String(e);
    }
  }
  const shown = $derived(items.filter((item) => {
    if (filter === "all") return true;
    const st = item.stats;
    if (!st) return filter === "paused";
    const dl = st.download?.download_speed ?? 0;
    const up = st.upload?.upload_speed ?? 0;
    if (filter === "downloading") return dl > 0;
    if (filter === "seeding") return up > 0;
    if (filter === "completed") return (st.downloaded_bytes ?? 0) > 0 && dl === 0;
    if (filter === "paused") return dl === 0 && up === 0;
    return true;
  }));
  // Torrent Health Analyzer: describe, never promise.
  const availability = (seeders: number, limited: string, good: string, unknown: string) =>
    seeders >= 5 ? good : seeders >= 1 ? limited : unknown;
</script>

<section class="torrents-page" aria-labelledby="torrents-title">
  <header>
    <h1 id="torrents-title">{locale.t("torrents-title")}</h1>
    <p class="sub">{locale.t("torrents-sub")}</p>
  </header>
  <div class="card">
    <div class="row">
      <input class="input grow" dir="ltr" bind:value={text} placeholder="magnet:?xt=urn:btih:…" aria-label={locale.t("torrents-title")} />
      <button class="btn primary" disabled={busy || !text.trim()} onclick={() => void addMagnet()}>{locale.t("tb-add")}</button>
    </div>
    {#if result}<p class="result" role="status">{result}</p>{/if}
  </div>
  <div class="tabs" role="tablist" aria-label={locale.t("torrents-title")}>
    {#each ["all", "downloading", "seeding", "completed", "paused"] as t (t)}
      <button role="tab" aria-selected={filter === t} class:on={filter === t} onclick={() => (filter = t)}>{t}</button>
    {/each}
    <button class="btn" onclick={() => void refresh()}>{locale.t("torrents-refresh")}</button>
  </div>
  {#each shown as item (item.id ?? item.name)}
    <div class="card item">
      <div class="head">
        <strong>{item.name ?? "torrent"}</strong>
        <span class="muted num">{item.total_pieces ?? 0}</span>
      </div>
      {#if item.stats}
        <div class="meta">
          <span class="chip">↓ {item.stats.download?.download_speed ?? 0} B/s</span>
          <span class="chip">↑ {item.stats.upload?.upload_speed ?? 0} B/s</span>
          <span class="chip">S {item.stats.live?.snapshot.peer_stats.live_seeders ?? 0} · P {item.stats.live?.snapshot.peer_stats.live ?? 0}</span>
          <span class="chip">{availability(item.stats.live?.snapshot.peer_stats.live_seeders ?? 0, locale.t("torrents-limited"), locale.t("torrents-good"), locale.t("torrents-unknown"))}</span>
          <span class="chip">{locale.t("torrents-ratio")} {item.stats.uploaded_bytes ?? 0}/{item.stats.downloaded_bytes ?? 0}</span>
        </div>
      {/if}
      {#if item.files?.length}
        <div class="files">
          {#each item.files as f, i (f.name + i)}
            <label class="check">
              <input
                type="checkbox"
                checked={f.included ?? checked.has(String(i))}
                onchange={(e) => {
                  const next = new Set(checked);
                  if (e.currentTarget.checked) next.add(String(i));
                  else next.delete(String(i));
                  checked = next;
                }}
              />
              <span dir="ltr">{f.name}</span>
              <span class="muted num">{f.length}</span>
            </label>
          {/each}
          {#if item.id !== undefined}
            <button class="btn quiet" onclick={() => void applyFiles(item.id!)}>{locale.t("ai-apply")}</button>
          {/if}
        </div>
      {/if}
      {#if item.files?.some((f) => /\.(exe|bat|cmd|msi|com|scr)$/i.test(f.name))}
        <p class="warn" role="status">{locale.t("sec-domain-suspicious")}</p>
      {/if}
      <div class="actions">
        {#if item.id !== undefined}
          <button class="btn quiet" onclick={() => void act(item.id!, "pause")}><Icon mono name="pause" size={15} /></button>
          <button class="btn quiet" onclick={() => void act(item.id!, "start")}><Icon mono name="play" size={15} /></button>
          <button class="btn quiet danger" onclick={() => void act(item.id!, "forget")}><Icon mono name="trash" size={15} /></button>
        {/if}
      </div>
    </div>
  {/each}
  {#if shown.length === 0}<p class="muted">—</p>{/if}
</section>

<style>
  .torrents-page { padding: 28px 32px; overflow: auto; display: flex; flex-direction: column; gap: 12px; }
  header h1 { font-size: 24px; font-weight: 700; margin: 0; }
  .sub { color: var(--muted); font-size: 13px; margin: 6px 0 0; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-l); box-shadow: var(--shadow); padding: 16px; display: flex; flex-direction: column; gap: 10px; }
  .row { display: flex; gap: 8px; align-items: center; }
  .grow { flex: 1; min-width: 0; }
  .result { background: var(--accent-soft); border-radius: 10px; padding: 10px 12px; font-size: 12.5px; margin: 0; }
  .tabs { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; }
  .tabs button { border: 1px solid var(--line-strong); background: var(--raised); color: var(--muted); border-radius: 99px; padding: 6px 14px; font-size: 12.5px; }
  .tabs button.on { background: var(--accent-soft); color: var(--accent); border-color: var(--accent); font-weight: 600; }
  .item .head { display: flex; justify-content: space-between; gap: 10px; }
  .meta { display: flex; gap: 8px; flex-wrap: wrap; }
  .chip { background: var(--sunken); border: 1px solid var(--line); border-radius: 99px; padding: 3px 10px; font-size: 12px; }
  .actions { display: flex; gap: 6px; }
  .files { display: flex; flex-direction: column; gap: 6px; border-top: 1px solid var(--line); padding-top: 10px; }
  .check { display: flex; gap: 10px; align-items: center; font-size: 13px; }
  .muted { color: var(--muted); }
  .num { font-variant-numeric: tabular-nums; }
  .warn { background: var(--amber-bg); color: var(--amber); border-radius: 10px; padding: 8px 12px; font-size: 12.5px; margin: 0; }
  .danger { color: var(--red); }
</style>
