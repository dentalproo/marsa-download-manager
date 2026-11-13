<script lang="ts" module>
  /** What a toolbar button does (M5.5, the design of #83). */
  export type ToolAction =
    | "add"
    | "resume"
    | "pause"
    | "pause-all"
    | "resume-all"
    | "delete"
    | "delete-done"
    | "settings"
    | "schedule"
    | "run-queue"
    | "halt-queue"
    | "ai";
</script>

<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";
  import { tick } from "svelte";
  import { shortcut } from "../keys";
  import { DEFAULT, load, save, shift, shown, toggle, type ToolbarPrefs } from "../toolbar";

  let {
    search = $bindable(),
    can,
    hint,
    onaction,
  }: {
    search: string;
    /** A button's longer name, if it has one (what it acts on). */
    hint: (action: ToolAction) => string | undefined;
    /** Whether a button has something to act on now. */
    can: (action: ToolAction) => boolean;
    onaction: (action: ToolAction) => void;
  } = $props();

  // One calm line-icon per action (no gloss, no gradient): the modern
  // command bar. The labels stay, because an icon alone is a guess.
  const LOOK: Record<ToolAction, { key: string; icon: string }> = {
    add: { key: "tb-add", icon: "circle-plus" },
    resume: { key: "tb-resume", icon: "play" },
    pause: { key: "tb-pause", icon: "pause" },
    "pause-all": { key: "tb-pause-all", icon: "octagon-pause" },
    "resume-all": { key: "tb-resume-all", icon: "play" },
    delete: { key: "tb-delete", icon: "trash" },
    "delete-done": { key: "tb-delete-done", icon: "trash-2" },
    settings: { key: "tb-settings", icon: "settings" },
    schedule: { key: "tb-schedule", icon: "calendar-clock" },
    "run-queue": { key: "tb-run-queue", icon: "list-start" },
    "halt-queue": { key: "tb-halt-queue", icon: "list-end" },
    ai: { key: "tb-ai", icon: "zap" },
  };

  // As the user leaves it (an IDM touch): big or small buttons, which,
  // and in what order; a right-click on the toolbar changes it.
  let prefs = $state<ToolbarPrefs>(load());
  const buttons = $derived(shown(prefs));
  function keep(next: ToolbarPrefs) {
    prefs = next;
    save(next);
  }
  let menu = $state<{ x: number; y: number } | null>(null);
  let customizing = $state(false);
  let panel: HTMLDivElement | undefined = $state();
  function context(event: MouseEvent) {
    if ((event.target as Element).closest(".search")) return;
    event.preventDefault();
    menu = { x: event.clientX, y: event.clientY };
    void tick().then(() => document.querySelector<HTMLButtonElement>(".tbmenu [role^='menuitem']")?.focus());
  }
  function onMenuKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      menu = null;
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const items = [...document.querySelectorAll<HTMLButtonElement>(".tbmenu [role^='menuitem']")];
      const at = items.indexOf(document.activeElement as HTMLButtonElement);
      items[(at + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length]?.focus();
    }
  }
  async function customize() {
    menu = null;
    customizing = true;
    await tick();
    panel?.querySelector<HTMLInputElement>("input")?.focus();
  }
</script>

<svelte:document
  onpointerdown={(e) => {
    const at = e.target as Element;
    if (menu && !at.closest?.(".tbmenu")) menu = null;
    if (customizing && !at.closest?.(".tbpanel")) customizing = false;
  }}
/>

<!-- A right-click on the toolbar: big or small buttons, and customizing. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="toolbar" class:small={prefs.small} role="toolbar" tabindex="-1" aria-label={locale.t("tb-toolbar")} oncontextmenu={context}>
  {#each buttons as b (b.action)}
    {@const look = LOOK[b.action]}
    {#if b.line}<span class="sep" aria-hidden="true"></span>{/if}
    <button
      id="tb-{b.action}"
      class="tb"
      class:primary={b.action === "add"}
      title={hint(b.action) ?? locale.t(look.key)}
      aria-label={locale.t(look.key)}
      disabled={!can(b.action)}
      onclick={() => onaction(b.action)}
    >
      <Icon mono name={look.icon} size={18} stroke={2} />
      {#if !prefs.small}<span class="label">{locale.t(look.key)}</span>{/if}
    </button>
  {/each}
  <label class="search">
    <Icon name="search" size={16} />
    <input id="search" type="search" bind:value={search} placeholder={locale.t("search-placeholder")} aria-label={locale.t("search-placeholder")} />
    <kbd aria-hidden="true">{shortcut("mod+F")}</kbd>
  </label>
</div>

{#if menu}
  <div class="tbmenu" role="menu" tabindex="-1" aria-label={locale.t("tb-toolbar")} style:left="{menu.x}px" style:top="{menu.y}px" onkeydown={onMenuKey}>
    <button class="it" role="menuitemradio" aria-checked={!prefs.small} onclick={() => ((menu = null), keep({ ...prefs, small: false }))}>
      <span class="mark">{#if !prefs.small}<Icon name="check" size={14} />{/if}</span>{locale.t("tb-big")}
    </button>
    <button class="it" role="menuitemradio" aria-checked={prefs.small} onclick={() => ((menu = null), keep({ ...prefs, small: true }))}>
      <span class="mark">{#if prefs.small}<Icon name="check" size={14} />{/if}</span>{locale.t("tb-small")}
    </button>
    <hr />
    <button class="it" role="menuitem" onclick={() => void customize()}><span class="mark"></span>{locale.t("tb-customize")}</button>
  </div>
{/if}

{#if customizing}
  <div class="tbpanel" role="dialog" aria-label={locale.t("tb-customize-title")} bind:this={panel} tabindex="-1" onkeydown={(e) => e.key === "Escape" && (customizing = false)}>
    <h2>{locale.t("tb-customize-title")}</h2>
    <ol>
      {#each prefs.order as action, i (action)}
        {@const look = LOOK[action]}
        <li>
          <label><input type="checkbox" checked={!prefs.hidden.includes(action)} onchange={() => keep(toggle(prefs, action))} />
            <Icon mono name={look.icon} size={17} />{locale.t(look.key)}</label>
          <span class="moves">
            <button class="btn quiet" aria-label={locale.t("tb-earlier", { name: locale.t(look.key) })} disabled={i === 0} onclick={() => keep(shift(prefs, action, -1))}><Icon name="arrow-up" size={14} /></button>
            <button class="btn quiet" aria-label={locale.t("tb-later", { name: locale.t(look.key) })} disabled={i === prefs.order.length - 1} onclick={() => keep(shift(prefs, action, 1))}><Icon name="arrow-down" size={14} /></button>
          </span>
        </li>
      {/each}
    </ol>
    <div class="actions">
      <button class="btn" onclick={() => keep({ ...DEFAULT, order: [...DEFAULT.order] })}>{locale.t("tb-reset")}</button>
      <button class="btn primary" onclick={() => (customizing = false)}>{locale.t("tb-done")}</button>
    </div>
  </div>
{/if}

<style>
  .tbmenu {
    position: fixed;
    z-index: 30;
    min-width: 200px;
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--radius-m);
    box-shadow: var(--shadow-lg);
    padding: 5px;
    display: flex;
    flex-direction: column;
    font-size: 13px;
  }
  .tbmenu .it {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border: 0;
    background: none;
    border-radius: 8px;
    text-align: start;
    font: inherit;
    color: var(--ink);
  }
  .tbmenu .it:hover,
  .tbmenu .it:focus-visible {
    background: var(--sel);
    outline: none;
  }
  .tbmenu .mark {
    width: 14px;
    display: inline-grid;
  }
  .tbmenu hr {
    border: 0;
    border-top: 1px solid var(--line);
    margin: 4px 6px;
  }
  .tbpanel {
    position: fixed;
    z-index: 30;
    top: 110px;
    inset-inline-start: 24px;
    width: 340px;
    max-height: 70vh;
    overflow: auto;
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--radius-l);
    box-shadow: var(--shadow-lg);
    padding: 16px;
    font-size: 13px;
  }
  .tbpanel h2 {
    font-size: 14px;
    margin: 0 0 10px;
  }
  .tbpanel ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tbpanel li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
  }
  .tbpanel label {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .tbpanel .moves .btn {
    padding: 2px 6px;
  }
  .tbpanel .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 14px;
  }

  /* The command bar: one clear primary action, then quiet ones. Labels
     never disappear (the owner's report: sections were cut off). */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 3px;
    min-height: 60px;
    padding: 10px 18px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    flex-wrap: wrap;
    row-gap: 6px;
  }
  .tb {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 12px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    transition: background 140ms ease, color 140ms ease, box-shadow 140ms ease;
  }
  .tb:hover:not(:disabled) {
    background: var(--sel);
    color: var(--ink);
  }
  .tb:active:not(:disabled) {
    transform: translateY(1px);
  }
  .tb:disabled {
    cursor: default;
    opacity: 0.42;
  }
  /* The one action that starts everything. */
  .tb.primary {
    background: var(--accent);
    color: var(--accent-ink);
    border-color: transparent;
    font-weight: 600;
    padding: 0 16px;
    box-shadow: 0 8px 20px color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .tb.primary:hover:not(:disabled) {
    background: var(--accent-hover);
    color: var(--accent-ink);
  }
  .tb.primary:disabled {
    opacity: 0.6;
  }
  .small .tb {
    padding: 0 9px;
  }
  .sep {
    width: 1px;
    height: 22px;
    flex: none;
    background: var(--line);
    margin: 0 7px;
  }
  .search {
    margin-inline-start: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 0 14px;
    flex: 0 1 260px;
    min-width: 150px;
    color: var(--faint);
    background: var(--sunken);
    font-size: 13px;
    transition: border-color 140ms ease, background 140ms ease, box-shadow 140ms ease;
  }
  .search:focus-within {
    border-color: var(--accent);
    background: var(--raised);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 22%, transparent);
  }
  .search input {
    border: 0;
    background: none;
    outline: none;
    min-width: 0;
    flex: 1;
    color: var(--ink);
    font: inherit;
  }
  kbd {
    font: 500 11px/1.4 inherit;
    direction: ltr;
    unicode-bidi: isolate;
    border: 1px solid var(--line);
    border-bottom-width: 2px;
    border-radius: 5px;
    padding: 0 5px;
    color: var(--muted);
    background: var(--raised);
  }
  /* Narrow windows: the labels go before any button does. */
  @media (max-width: 1080px) {
    .tb:not(.primary) .label {
      display: none;
    }
  }
</style>
