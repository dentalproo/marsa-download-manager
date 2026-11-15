<script lang="ts">
  // The dashboard (the owner's Nova-inspired design, 2026-11-14): one
  // breathing ring for what is moving now, a minute of speed, and the
  // first downloads. It reads the same snapshot as the table and changes
  // nothing: every action still belongs to the toolbar and the windows.
  import { onMount } from "svelte";
  import Icon from "./Icon.svelte";
  import FileName from "./FileName.svelte";
  import Mark from "./Mark.svelte";
  import { locale } from "../i18n.svelte";
  import { left, num, percent, size, speed } from "../format";
  import type { Snapshot } from "../bindings/Snapshot";
  import type { DownloadView } from "../bindings/DownloadView";

  let {
    snapshot,
    shown,
    welcome = false,
    onadd,
    onopen,
    onshowall,
    ondismiss,
  }: {
    snapshot: Snapshot;
    shown: DownloadView[];
    welcome?: boolean;
    onadd: () => void;
    onopen: (id: number) => void;
    onshowall: () => void;
    ondismiss: (start: boolean) => void;
  } = $props();

  const active = $derived(snapshot.downloads.filter((d) => d.status === "active"));
  const waiting = $derived(snapshot.downloads.filter((d) => d.status === "waiting").length);
  const finishedBytes = $derived(
    snapshot.downloads.filter((d) => d.status === "done").reduce((sum, d) => sum + (d.size ?? d.done), 0),
  );
  const allBytes = $derived(snapshot.downloads.reduce((sum, d) => sum + (d.size ?? d.done), 0));
  const connections = $derived(active.reduce((sum, d) => sum + d.segments, 0));
  const speedNow = $derived(active.reduce((sum, d) => sum + d.speed, 0));

  // The busiest running download leads the ring; the ring itself sums
  // every running one, as the taskbar's progress does.
  const primary = $derived([...active].sort((a, b) => b.speed - a.speed)[0] ?? null);
  const totalBytes = $derived(active.reduce((sum, d) => sum + (d.size ?? 0), 0));
  const doneBytes = $derived(active.reduce((sum, d) => sum + d.done, 0));
  const single = $derived(primary && (primary.size ?? 0) > 0 ? primary.done / (primary.size ?? 1) : 0);
  const fraction = $derived(totalBytes > 0 ? Math.min(1, doneBytes / totalBytes) : Math.min(1, Math.max(0, single)));
  const remaining = $derived(
    totalBytes > 0 ? Math.max(0, totalBytes - doneBytes) : primary?.size ? Math.max(0, primary.size - primary.done) : 0,
  );

  const percentText = $derived(
    new Intl.NumberFormat(`${locale.lang}-u-nu-${locale.digits}`, {
      style: "percent",
      maximumFractionDigits: 1,
    }).format(fraction),
  );
  const shortPercent = (d: DownloadView) =>
    percent(locale.lang, locale.digits, d.size ? d.done / d.size : d.status === "done" ? 1 : 0);

  // A minute of speed, sampled once a second. Reading the prop inside the
  // timer is intentional: it is not a dependency of the effect.
  const SAMPLES = 48;
  let samples = $state<number[]>(Array.from({ length: SAMPLES }, () => 0));
  onMount(() => {
    const timer = setInterval(() => {
      samples = [...samples.slice(1), Math.max(0, speedNow)];
    }, 1000);
    return () => clearInterval(timer);
  });

  const W = 320;
  const H = 112;
  const maxSample = $derived(Math.max(1, speedNow, ...samples));
  const points = $derived(
    samples.map((value, i) => [(i / (SAMPLES - 1)) * W, H - (Math.min(value, maxSample) / maxSample) * (H - 10)] as const),
  );
  // A smooth line through the samples (quadratic midpoints), then the
  // same line closed to the floor for the soft fill.
  const linePath = $derived.by(() => {
    const [first, ...rest] = points;
    if (!first) return "";
    let path = `M ${first[0].toFixed(1)} ${first[1].toFixed(1)}`;
    let previous = first;
    for (const point of rest) {
      const midX = (previous[0] + point[0]) / 2;
      const midY = (previous[1] + point[1]) / 2;
      path += ` Q ${previous[0].toFixed(1)} ${previous[1].toFixed(1)} ${midX.toFixed(1)} ${midY.toFixed(1)}`;
      previous = point;
    }
    const last = points[points.length - 1];
    return last ? `${path} L ${last[0].toFixed(1)} ${last[1].toFixed(1)}` : path;
  });
  const areaPath = $derived(`${linePath} L ${W} ${H} L 0 ${H} Z`);
  const peak = $derived(Math.max(...samples));

  function statusOf(d: DownloadView): string {
    if (d.waits_for_browser) return locale.t("status-open-browser");
    if (d.waits_for_site) return locale.t("status-waiting-site");
    if (d.needs_ok) return locale.t("status-needs-ok");
    return locale.t(
      { active: "status-active", waiting: "status-waiting", paused: "status-paused", done: "status-done", failed: "status-failed" }[
        d.status
      ],
    );
  }
  const tone = (d: DownloadView) => (d.status === "failed" ? "bad" : d.status === "done" ? "ok" : d.status === "active" ? "go" : "held");

  const CIRCUMFERENCE = 2 * Math.PI * 104;
</script>

<div class="dash">
  <header class="dash-head">
    <div class="titles">
      <span class="eyebrow"><span class="pulse" aria-hidden="true"></span>{locale.t("dash-live")}</span>
      <h1>{locale.t("nav-dashboard")}</h1>
      <p>{locale.t("dash-sub")}</p>
    </div>
    <div class="chips">
      <span class="chip"><span class="dot go" aria-hidden="true"></span>{locale.t("status-active")}<b class="num">{num(locale.lang, locale.digits, active.length, 0)}</b></span>
      <span class="chip"><span class="dot held" aria-hidden="true"></span>{locale.t("status-waiting")}<b class="num">{num(locale.lang, locale.digits, waiting, 0)}</b></span>
      <span class="chip speed-chip"><Icon name="gauge" size={15} /><b class="num">{speed(locale.lang, locale.digits, speedNow) || size(locale.lang, locale.digits, 0)}</b></span>
      <button class="btn primary" onclick={onadd}><Icon name="plus" size={16} stroke={2.2} />{locale.t("tb-add")}</button>
    </div>
  </header>

  {#if welcome}
    <section class="welcome" aria-labelledby="dash-welcome-title">
      <span class="welcome-art" aria-hidden="true"><Mark size={46} /></span>
      <div>
        <h2 id="dash-welcome-title">{locale.t("welcome-title")}</h2>
        <p>{locale.t("welcome-sub")}</p>
      </div>
      <div class="welcome-actions">
        <button class="btn primary" onclick={() => ondismiss(true)}><Icon name="down" size={17} />{locale.t("welcome-start")}</button>
        <button class="btn quiet" onclick={() => ondismiss(false)}>{locale.t("welcome-skip")}</button>
      </div>
    </section>
  {/if}

  <div class="grid">
    <section class="card hero" aria-labelledby="dash-hero-title">
      <h2 id="dash-hero-title" class="visually-hidden">{locale.t("dash-now-title")}</h2>
      <div class="ring">
        <svg viewBox="0 0 260 260" aria-hidden="true">
          <defs>
            <linearGradient id="hero-ring" x1="24" y1="24" x2="236" y2="236" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="var(--accent-2)" />
              <stop offset="52%" stop-color="var(--accent)" />
              <stop offset="100%" stop-color="var(--accent-3)" />
            </linearGradient>
          </defs>
          <circle class="orbit" cx="130" cy="130" r="124" />
          <circle class="orbit orbit-2" cx="130" cy="130" r="116" />
          <circle class="track" cx="130" cy="130" r="104" />
          <circle
            class="bar"
            cx="130"
            cy="130"
            r="104"
            stroke-dasharray={CIRCUMFERENCE}
            stroke-dashoffset={CIRCUMFERENCE * (1 - fraction)}
          />
        </svg>
        <div class="ring-inner">
          {#if primary}
            <span class="big num" dir="ltr">{percentText}</span>
            <span class="ring-caption">{locale.t("dash-downloading")}</span>
            <span class="ring-name"><FileName name={primary.name} /></span>
            <span class="ring-meta num">
              {size(locale.lang, locale.digits, doneBytes)} / {size(locale.lang, locale.digits, totalBytes || (primary.size ?? 0))}
            </span>
            <span class="ring-speed num">{speed(locale.lang, locale.digits, speedNow)}</span>
            {#if left(locale.lang, locale.digits, remaining, speedNow)}
              <span class="ring-eta">{locale.t("dash-eta", { time: left(locale.lang, locale.digits, remaining, speedNow) })}</span>
            {/if}
          {:else}
            <span class="idle-mark" aria-hidden="true"><Mark size={40} /></span>
            <span class="ring-caption">{locale.t("dash-idle")}</span>
            <span class="ring-hint">{locale.t("dash-idle-hint")}</span>
            <button class="btn primary ring-add" onclick={onadd}><Icon name="plus" size={16} />{locale.t("tb-add")}</button>
          {/if}
        </div>
      </div>
      <div class="hero-foot">
        <span class="stat"><b class="num">{size(locale.lang, locale.digits, finishedBytes)}</b><small>{locale.t("dash-finished-size")}</small></span>
        <span class="stat"><b class="num">{size(locale.lang, locale.digits, allBytes)}</b><small>{locale.t("dash-total-size")}</small></span>
        <span class="stat"><b class="num">{num(locale.lang, locale.digits, connections, 0)}</b><small>{locale.t("dash-connections")}</small></span>
      </div>
    </section>

    <section class="card chart" aria-labelledby="dash-chart-title">
      <header class="card-head">
        <h2 id="dash-chart-title">{locale.t("dash-speed-title")}</h2>
        <span class="tag">{locale.t("dash-history")}</span>
      </header>
      <div class="now">
        <span class="now-speed num">{speed(locale.lang, locale.digits, speedNow) || "—"}</span>
        <span class="now-label">{locale.t("dash-speed-now")}</span>
      </div>
      <div class="chart-body">
        <svg class="speed-chart" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" aria-hidden="true">
          <defs>
            <linearGradient id="speed-fill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="var(--accent-2)" stop-opacity="0.42" />
              <stop offset="100%" stop-color="var(--accent-2)" stop-opacity="0" />
            </linearGradient>
            <linearGradient id="speed-stroke" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%" stop-color="var(--accent-2)" />
              <stop offset="60%" stop-color="var(--accent)" />
              <stop offset="100%" stop-color="var(--accent-3)" />
            </linearGradient>
          </defs>
          <g class="grid-lines">
            <line x1="0" y1={H * 0.25} x2={W} y2={H * 0.25} />
            <line x1="0" y1={H * 0.5} x2={W} y2={H * 0.5} />
            <line x1="0" y1={H * 0.75} x2={W} y2={H * 0.75} />
          </g>
          <path class="area" d={areaPath} fill="url(#speed-fill)" />
          <path class="line" d={linePath} fill="none" stroke="url(#speed-stroke)" stroke-width="2.5" stroke-linecap="round" />
        </svg>
        {#if peak === 0 && speedNow === 0}
          <span class="chart-empty">{locale.t("dash-no-traffic")}</span>
        {/if}
      </div>
      <footer class="chart-foot">
        <span>{locale.t("dash-minute")}</span>
        <span class="num">{locale.t("dash-peak", { speed: speed(locale.lang, locale.digits, peak) || size(locale.lang, locale.digits, 0) })}</span>
      </footer>
    </section>

    <section class="card list" aria-labelledby="dash-list-title">
      <header class="card-head">
        <h2 id="dash-list-title">{locale.t("dash-list-title")}</h2>
        <button class="btn quiet see-all" onclick={onshowall}>{locale.t("dash-see-all")}<Icon name="arrow-left" size={14} /></button>
      </header>
      {#if shown.length === 0}
        <div class="list-empty">
          <span aria-hidden="true"><Mark size={34} /></span>
          <p>{locale.t("dash-empty")}</p>
          <button class="btn" onclick={onadd}><Icon name="plus" size={15} />{locale.t("list-empty-add")}</button>
        </div>
      {:else}
        <div class="rows">
          {#each shown.slice(0, 4) as d (d.id)}
            <button
              class="row"
              class:is-done={d.status === "done"}
              onclick={() => onopen(d.id)}
              aria-label={`${d.name} — ${statusOf(d)}`}
              title={d.name}
            >
              <span class="tone {tone(d)}" aria-hidden="true"></span>
              <span class="row-main">
                <span class="row-name"><FileName name={d.name} /></span>
                <span class="row-sub">
                  {statusOf(d)}
                  {#if d.status === "active" && d.speed > 0}<span class="dot-sep" aria-hidden="true">·</span><span class="num">{speed(locale.lang, locale.digits, d.speed)}</span>{/if}
                  {#if d.status === "waiting" && d.size !== null}<span class="dot-sep" aria-hidden="true">·</span><span class="num">{size(locale.lang, locale.digits, d.size)}</span>{/if}
                </span>
              </span>
              <span class="row-end">
                <span class="row-pct num">{shortPercent(d)}</span>
                <span class="mini" aria-hidden="true"><i class={tone(d)} style:inline-size={shortPercent(d)}></i></span>
              </span>
            </button>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .dash {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 14px 18px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .dash-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 18px;
    flex-wrap: wrap;
    padding: 2px 2px 0;
  }
  .titles h1 {
    margin: 2px 0 2px;
    font-size: 25px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .titles p {
    color: var(--muted);
    font-size: 13px;
  }
  .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent-2);
  }
  .pulse {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-2);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent-2) 18%, transparent), 0 0 12px var(--glow-2);
  }
  .chips {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 13px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--raised) 82%, transparent);
    color: var(--muted);
    font-size: 12.5px;
    white-space: nowrap;
  }
  .chip b {
    color: var(--ink);
    font-weight: 700;
    margin-inline-start: 1px;
  }
  .speed-chip b {
    direction: ltr;
    unicode-bidi: isolate;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }
  .dot.go,
  .tone.go {
    background: var(--accent-2);
    box-shadow: 0 0 8px var(--glow-2);
  }
  .dot.held,
  .tone.held {
    background: var(--amber);
  }
  .tone.ok {
    background: var(--green);
  }
  .tone.bad {
    background: var(--red);
  }
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1.55fr) minmax(290px, 0.9fr);
    grid-template-areas: "hero chart" "hero list";
    gap: 14px;
    align-items: stretch;
  }
  .card {
    border: 1px solid var(--line);
    border-radius: var(--radius-l);
    background: var(--surface);
    box-shadow: var(--shadow);
    min-width: 0;
  }
  /* The one place with the design's glow: the ring and its halo, only
     while something is moving. */
  .hero {
    grid-area: hero;
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 6px;
    padding: 16px 18px;
    overflow: hidden;
    background:
      radial-gradient(540px 380px at 50% 42%, color-mix(in srgb, var(--accent) 26%, transparent), transparent 68%),
      var(--surface);
    border-color: color-mix(in srgb, var(--accent) 30%, var(--line));
  }
  .hero::before {
    content: "";
    position: absolute;
    inset: 0;
    background: var(--hero-soft);
    opacity: 0.35;
    pointer-events: none;
  }
  .ring {
    position: relative;
    width: min(322px, 100%);
    aspect-ratio: 1;
    margin: 0 auto 6px;
    display: grid;
    place-items: center;
  }
  .ring svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .orbit {
    fill: none;
    stroke: color-mix(in srgb, var(--accent) 42%, transparent);
    stroke-width: 1;
    stroke-dasharray: 2 7;
    opacity: 0.55;
  }
  .orbit-2 {
    stroke: color-mix(in srgb, var(--accent-2) 40%, transparent);
    stroke-dasharray: 1 12;
    opacity: 0.4;
  }
  .track {
    fill: none;
    stroke: color-mix(in srgb, var(--ink) 9%, transparent);
    stroke-width: 13;
  }
  .bar {
    fill: none;
    stroke: url(#hero-ring);
    stroke-width: 13;
    stroke-linecap: round;
    transform: rotate(-90deg);
    transform-origin: 130px 130px;
    filter: drop-shadow(0 0 14px color-mix(in srgb, var(--accent-3) 55%, transparent));
    transition: stroke-dashoffset 550ms ease;
  }
  .ring-inner {
    position: relative;
    z-index: 1;
    width: 74%;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 5px;
  }
  .big {
    font-size: clamp(42px, 4.6vw, 60px);
    font-weight: 800;
    line-height: 1.02;
    letter-spacing: -0.03em;
    background: var(--hero);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .ring-caption {
    font-size: 13px;
    font-weight: 600;
    color: var(--ink);
  }
  .ring-name {
    margin-top: 3px;
    padding: 3px 10px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--sunken) 70%, transparent);
    border: 1px solid var(--line);
    font-size: 12px;
    color: var(--muted);
    max-width: 94%;
  }
  .ring-name :global(.fn) {
    max-width: 100%;
    display: inline-block;
    vertical-align: bottom;
  }
  .ring-meta {
    font-size: 12px;
    color: var(--muted);
  }
  .ring-speed {
    font-size: 17px;
    font-weight: 700;
    color: var(--ink);
  }
  .ring-eta,
  .ring-hint {
    font-size: 11.5px;
    color: var(--faint);
  }
  .ring-add {
    margin-top: 6px;
  }
  .idle-mark {
    display: grid;
    place-items: center;
    width: 76px;
    height: 76px;
    margin-bottom: 4px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--line));
    box-shadow: 0 0 34px color-mix(in srgb, var(--accent) 20%, transparent);
  }
  .hero-foot {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-around;
    gap: 10px;
    margin-top: 6px;
    padding-top: 13px;
    border-top: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    min-width: 0;
  }
  .stat b {
    font-size: 14.5px;
    font-weight: 700;
  }
  .stat small {
    font-size: 11px;
    color: var(--faint);
  }
  .chart {
    grid-area: chart;
    padding: 12px 14px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .card-head h2 {
    font-size: 14px;
    font-weight: 650;
  }
  .tag {
    font-size: 10.5px;
    color: var(--faint);
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 1px 8px;
  }
  .now {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .now-speed {
    font-size: 21px;
    font-weight: 750;
    letter-spacing: -0.01em;
  }
  .now-label {
    font-size: 11px;
    color: var(--faint);
  }
  .speed-chart {
    width: 100%;
    height: 100%;
    min-height: 72px;
    overflow: visible;
  }
  .chart-body {
    position: relative;
    flex: 1;
    min-height: 72px;
    display: flex;
  }
  .chart-empty {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    font-size: 11.5px;
    color: var(--faint);
    pointer-events: none;
  }
  .grid-lines line {
    stroke: var(--line);
    stroke-width: 1;
    stroke-dasharray: 2 4;
    opacity: 0.7;
  }
  .line {
    filter: drop-shadow(0 0 5px color-mix(in srgb, var(--accent-2) 45%, transparent));
  }
  .chart-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 10.5px;
    color: var(--faint);
  }
  .list {
    grid-area: list;
    padding: 12px 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
  }
  .see-all {
    font-size: 11.5px;
    padding: 3px 8px;
    gap: 4px;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 5px 7px;
    border: 0;
    border-radius: 9px;
    background: none;
    color: var(--ink);
    text-align: start;
    font: inherit;
    cursor: pointer;
    transition: background 130ms ease;
  }
  .row:hover {
    background: var(--sel);
  }
  .row:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .tone {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex: none;
  }
  .row-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .row-name {
    font-size: 12.5px;
    font-weight: 550;
    overflow: hidden;
  }
  .row-name :global(.fn) {
    display: block;
    max-width: 100%;
  }
  .row-sub {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    color: var(--faint);
    white-space: nowrap;
  }
  .dot-sep {
    opacity: 0.6;
  }
  .row-end {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    width: 56px;
  }
  .row-pct {
    font-size: 10.5px;
    color: var(--muted);
  }
  .mini {
    display: block;
    width: 56px;
    height: 4px;
    border-radius: 99px;
    background: var(--line);
    overflow: hidden;
  }
  .mini i {
    display: block;
    height: 100%;
    border-radius: inherit;
    transition: inline-size 400ms ease;
  }
  .mini i.go {
    background: linear-gradient(90deg, var(--accent-2), var(--accent));
  }
  .mini i.ok {
    background: var(--green);
  }
  .mini i.held {
    background: var(--amber-fill);
  }
  .mini i.bad {
    background: var(--red);
  }
  .list-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 14px 8px 18px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
  }
  .list-empty p {
    max-width: 26ch;
  }
  .welcome {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 18px;
    border: 1px solid color-mix(in srgb, var(--accent) 34%, var(--line));
    border-radius: var(--radius-l);
    background:
      radial-gradient(520px 180px at 12% 50%, color-mix(in srgb, var(--accent) 20%, transparent), transparent 70%),
      var(--surface);
    box-shadow: var(--shadow);
    flex-wrap: wrap;
  }
  .welcome-art {
    display: grid;
    place-items: center;
    width: 62px;
    height: 62px;
    border-radius: 18px;
    background: var(--accent-soft);
    border: 1px solid var(--line);
    flex: none;
  }
  .welcome h2 {
    font-size: 16px;
    margin-bottom: 2px;
  }
  .welcome p {
    color: var(--muted);
    font-size: 12.5px;
    max-width: 70ch;
  }
  .welcome-actions {
    display: flex;
    gap: 8px;
    margin-inline-start: auto;
  }
  @media (max-width: 980px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "hero" "chart" "list";
    }
    .ring {
      width: min(300px, 100%);
    }
  }
  @media (max-height: 700px) {
    .ring {
      width: min(236px, 100%);
    }
    .big {
      font-size: 36px;
    }
  }
</style>
