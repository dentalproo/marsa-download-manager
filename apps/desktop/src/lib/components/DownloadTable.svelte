<script lang="ts" module>
  export function errorCode(error: string | null): string | null {
    return /\banswered (\d{3})\b/.exec(error ?? "")?.[1] ?? null;
  }
</script>

<script lang="ts">
  import { tick } from "svelte";
  import Icon from "./Icon.svelte";
  import FileName from "./FileName.svelte";
  import ColorIcon, { typeIcon } from "./ColorIcon.svelte";
  import { locale } from "../i18n.svelte";
  import { lastTry, left, percent, size, speed } from "../format";
  import type { DownloadView } from "../bindings/DownloadView";

  let {
    downloads,
    selected = $bindable(),
    label,
    chooser = $bindable(false),
    onopen,
    oncontext,
  }: {
    downloads: DownloadView[];
    selected: number[];
    label: string;
    chooser: boolean;
    onopen: (id: number) => void;
    oncontext: (x: number, y: number) => void;
  } = $props();

  let scroller: HTMLElement | undefined = $state();
  
  // Virtualization logic
  const VIRTUAL_FROM = 150;
  const AROUND = 30;
  let scrollTop = $state(0);
  let viewHeight = $state(800);
  const rowHeight = 88; // Card height approximate
  const virtual = $derived(downloads.length > VIRTUAL_FROM);
  const first = $derived(virtual ? Math.max(0, Math.floor(scrollTop / rowHeight) - AROUND) : 0);
  const last = $derived(virtual ? Math.min(downloads.length, Math.ceil((scrollTop + viewHeight) / rowHeight) + AROUND) : downloads.length);
  const rows = $derived(virtual ? downloads.slice(first, last) : downloads);
  const chosen = $derived(new Set(selected));

  function context(event: MouseEvent, id: number) {
    event.preventDefault();
    if (!selected.includes(id)) selected = [id];
    oncontext(event.clientX, event.clientY);
  }

  const fraction = (d: DownloadView) => (d.size ? d.done / d.size : d.status === "done" ? 1 : 0);
  const pct = (d: DownloadView) => percent(locale.lang, locale.digits, fraction(d));

  let focus = $state<number | null>(null);
  let anchor = $state<number | null>(null);

  function pick(event: MouseEvent, id: number) {
    if (event.ctrlKey || event.metaKey) {
      const set = new Set(selected);
      if (!set.delete(id)) set.add(id);
      selected = downloads.filter((d) => set.has(d.id)).map((d) => d.id);
    } else {
      selected = [id];
    }
    anchor = id;
    focus = id;
  }
</script>

<div class="scroll" bind:this={scroller} onscroll={() => virtual && scroller && (scrollTop = scroller.scrollTop)}>
  <div class="cards-grid" aria-label={label}>
    {#if virtual && first > 0}<div style:height={`${first * rowHeight}px`}></div>{/if}
    
    {#each rows as d (d.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        id="row-{d.id}"
        class="card"
        class:selected={chosen.has(d.id)}
        role="button"
        tabindex="0"
        onclick={(e) => pick(e, d.id)}
        ondblclick={() => onopen(d.id)}
        oncontextmenu={(e) => context(e, d.id)}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            onopen(d.id);
          }
          if (e.key === " ") {
            e.preventDefault();
            selected = [d.id];
            anchor = d.id;
            focus = d.id;
          }
        }}
      >
        <span class="ft" aria-hidden="true"><ColorIcon name={typeIcon(d.category)} size={32} /></span>
        
        <div class="info">
          <div class="name-row">
            <FileName name={d.name} />
          </div>
          
          <div class="progress-bar">
            <div class="mini" aria-hidden="true">
              <b class:go={d.status === 'active'} class:held={d.status === 'paused'} class:bad={d.status === 'failed'} style:inline-size="{Math.round(fraction(d) * 100)}%"></b>
            </div>
            <span class="pct num">{pct(d)}</span>
            <span class="meta">{speed(locale.lang, locale.digits, d.speed)}</span>
            <span class="meta">{left(locale.lang, locale.digits, (d.size ?? 0) - d.done, d.speed)}</span>
          </div>
        </div>
      </div>
    {/each}

    {#if virtual && last < downloads.length}<div style:height={`${(downloads.length - last) * rowHeight}px`}></div>{/if}
  </div>
</div>

<style>
  .scroll {
    overflow: auto;
    flex: 1;
    min-height: 0;
    padding: 16px;
  }
  .cards-grid {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--raised);
    border-radius: var(--radius-m);
    border: 1px solid var(--line);
    transition: all 130ms ease;
    cursor: default;
  }
  .card:hover {
    background: var(--sel);
    border-color: var(--line-strong);
  }
  .card.selected {
    background: var(--accent-soft);
    border-color: var(--accent);
  }
  .ft {
    flex: none;
    padding: 8px;
    background: var(--surface);
    border-radius: var(--radius-s);
  }
  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  .name-row {
    font-weight: 500;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .progress-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    color: var(--muted);
  }
  .mini {
    flex: none;
    width: 120px;
    height: 6px;
    border-radius: 99px;
    background: var(--line);
    overflow: hidden;
    position: relative;
  }
  .mini b {
    position: absolute;
    inset-block: 0;
    inset-inline-start: 0;
    border-radius: inherit;
    transition: width 300ms ease;
  }
  .go { background: var(--accent); }
  .held { background: var(--amber-fill); }
  .bad { background: var(--red); }
  .meta {
    font-variant-numeric: tabular-nums;
  }
</style>
