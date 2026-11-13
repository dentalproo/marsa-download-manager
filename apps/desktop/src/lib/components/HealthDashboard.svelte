<script lang="ts">
  import { locale } from "../i18n.svelte";
  import { size } from "../format";
  import type { Snapshot } from "../bindings/Snapshot";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const downloads = $derived(snapshot.downloads);
  const active = $derived(downloads.filter((d) => d.status === "active"));
  const totalSpeed = $derived(active.reduce((s, d) => s + d.speed, 0));
  const avgSpeed = $derived(active.length ? totalSpeed / active.length : 0);
  const totalData = $derived(downloads.reduce((s, d) => s + d.done, 0));
  const success = $derived(downloads.filter((d) => d.status === "done").length);
  const failed = $derived(downloads.filter((d) => d.status === "failed").length);
  const total = $derived(downloads.length);
  const successRate = $derived(total ? (success / total) * 100 : 0);
  const totalSize = $derived(downloads.reduce((s, d) => s + (d.size ?? 0), 0));
  const domains = $derived.by(() => {
    const map = new Map<string, number>();
    for (const d of downloads) {
      if (d.host) map.set(d.host, (map.get(d.host) ?? 0) + 1);
    }
    return [...map.entries()].sort((a, b) => b[1] - a[1]).slice(0, 5);
  });
  const maxDomain = $derived(domains[0]?.[1] ?? 1);

  // Daily buckets for the last 7 days from `added` (RFC 3339).
  const daily = $derived.by(() => {
    const buckets = Array.from({ length: 7 }, (_, i) => {
      const day = new Date();
      day.setDate(day.getDate() - (6 - i));
      return { label: `${day.getMonth() + 1}/${day.getDate()}`, count: 0, bytes: 0 };
    });
    for (const d of downloads) {
      if (!d.added) continue;
      const at = new Date(d.added);
      for (let i = 0; i < 7; i++) {
        const day = new Date();
        day.setDate(day.getDate() - (6 - i));
        if (at.toDateString() === day.toDateString()) {
          buckets[i]!.count++;
          buckets[i]!.bytes += d.done;
          break;
        }
      }
    }
    return buckets;
  });
  const maxCount = $derived(Math.max(1, ...daily.map((b) => b.count)));

  // The dashboard's charts are Canvas (design round): the total downloaded
  // per day, over the last seven days, from the same `daily` buckets.
  let bytesCanvas = $state<HTMLCanvasElement | null>(null);
  const BYTES_W = 560;
  const BYTES_H = 150;
  const BYTES_BASE = 18;

  $effect(() => {
    const canvas = bytesCanvas;
    const buckets = daily;
    if (canvas === null) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = BYTES_W * dpr;
    canvas.height = BYTES_H * dpr;
    const ctx = canvas.getContext("2d");
    if (ctx === null) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, BYTES_W, BYTES_H);
    const top = buckets.reduce((m, b) => (b.bytes > m ? b.bytes : m), 0) || 1;
    const slot = BYTES_W / buckets.length;
    const bw = Math.min(44, slot * 0.6);
    const accent = getComputedStyle(canvas).getPropertyValue("--accent").trim() || "#3b82f6";
    ctx.fillStyle = accent;
    buckets.forEach((b, i) => {
      const h = (b.bytes / top) * (BYTES_H - BYTES_BASE);
      const x = i * slot + (slot - bw) / 2;
      const y = BYTES_H - BYTES_BASE - h;
      ctx.fillRect(x, y, bw, Math.max(0, h));
    });
  });
</script>

<section class="dash" aria-label={locale.t("health-title")}>
  <header>
    <h1>{locale.t("health-title")}</h1>
    <p class="sub">{locale.t("health-sub")}</p>
  </header>

  <div class="cards">
    <div class="card">
      <span class="label">{locale.t("health-avg-speed")}</span>
      <strong class="num">{size(locale.lang, locale.digits, Math.round(avgSpeed))}/ث</strong>
    </div>
    <div class="card">
      <span class="label">{locale.t("health-total")}</span>
      <strong class="num">{size(locale.lang, locale.digits, totalData)}</strong>
    </div>
    <div class="card">
      <span class="label">{locale.t("health-rate")}</span>
      <strong class="num">{Math.round(successRate)}%</strong>
      <small>{success} ✓ · {failed} ✗</small>
    </div>
    <div class="card">
      <span class="label">{locale.t("health-storage")}</span>
      <strong class="num">{size(locale.lang, locale.digits, totalSize)}</strong>
    </div>
  </div>

  <div class="grid-2">
    <div class="panel">
      <h2>{locale.t("health-domains")}</h2>
      {#if domains.length === 0}
        <p class="muted">—</p>
      {:else}
        {#each domains as [host, count] (host)}
          <div class="row">
            <bdi class="host" dir="ltr">{host}</bdi>
            <span class="bar"><b style:inline-size="{Math.round((count / maxDomain) * 100)}%"></b></span>
            <span class="num muted">{count}</span>
          </div>
        {/each}
      {/if}
    </div>
    <div class="panel">
      <h2>{locale.t("health-week")}</h2>
      <div class="bars" role="img" aria-label={locale.t("health-week")}>
        {#each daily as b (b.label)}
          <div class="col">
            <span class="bar-v"><b style:height="{Math.round((b.count / maxCount) * 100)}%" class="fill" title={String(b.count)}></b></span>
            <small class="num">{b.label}</small>
            <small class="muted">{b.count}</small>
          </div>
        {/each}
      </div>
    </div>
  </div>

<div class="panel">
  <h2>{locale.t("health-bytes-week")}</h2>
  <canvas bind:this={bytesCanvas} class="bytes" aria-label={locale.t("health-bytes-week")}></canvas>
</div>
</section>

<style>
  .dash { padding: 28px 32px; overflow: auto; display: flex; flex-direction: column; gap: 18px; }
  header h1 { font-size: 24px; font-weight: 700; margin: 0; }
  .sub { color: var(--muted); font-size: 13px; margin: 6px 0 0; }
  .cards { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-l); box-shadow: var(--shadow); padding: 18px; display: flex; flex-direction: column; gap: 6px; }
  .card .label { font-size: 12px; font-weight: 600; color: var(--muted); }
  .card strong { font-size: 22px; font-weight: 700; }
  .card small { color: var(--muted); font-size: 12px; }
  .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .panel { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-l); box-shadow: var(--shadow); padding: 18px; display: flex; flex-direction: column; gap: 12px; }
  .panel h2 { font-size: 15px; font-weight: 700; margin: 0; }
  .row { display: flex; align-items: center; gap: 10px; font-size: 13px; }
  .host { flex: 0 0 38%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bar { flex: 1; height: 8px; border-radius: 99px; background: var(--line); overflow: hidden; }
  .bar b { display: block; height: 100%; background: var(--accent); border-radius: inherit; }
  .bars { display: flex; gap: 8px; align-items: flex-end; min-height: 120px; }
  .col { flex: 1; display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .bar-v { display: flex; align-items: flex-end; width: 100%; height: 72px; background: var(--sunken); border-radius: 8px; overflow: hidden; }
  .fill { display: block; height: 100%; background: var(--accent); border-radius: inherit; }
  .bytes { display: block; width: 100%; height: auto; }
  .muted { color: var(--muted); }
  .num { font-variant-numeric: tabular-nums; }
</style>
