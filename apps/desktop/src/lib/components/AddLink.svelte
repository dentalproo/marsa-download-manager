<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";
  import { fitWindow } from "../fitWindow";

  let {
    url: given,
    fromClipboard,
    problem = null,
    onok,
    oncancel,
  }: {
    url: string;
    fromClipboard: boolean;
    problem?: string | null;
    onok: (url: string) => void;
    oncancel: () => void;
  } = $props();

  let url = $state("");
  let box: HTMLInputElement | undefined = $state();
  $effect(() => {
    url = given;
    void given;
    queueMicrotask(() => {
      box?.focus({ preventScroll: true });
      box?.select();
    });
  });
  function submit(event: SubmitEvent) {
    event.preventDefault();
    const link = url.trim();
    if (link) onok(link);
  }
  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      oncancel();
    }
  }
  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        url = text.trim();
        box?.focus({ preventScroll: true });
      }
    } catch {
      box?.focus({ preventScroll: true });
    }
  }
  function clear() {
    url = "";
    box?.focus({ preventScroll: true });
  }
</script>

<svelte:window {onkeydown} />

<main class="link" use:fitWindow aria-labelledby="addlink-heading">
  <form class="card" onsubmit={submit}>
    <div class="head">
      <span class="badge" aria-hidden="true"><Icon mono name="link" size={22} /></span>
      <div class="titles">
        <h1 id="addlink-heading">{locale.t("addlink-title")}</h1>
        <p>{locale.t("addlink-url")}</p>
      </div>
      <button type="button" class="btn icon quiet close-x" aria-label={locale.t("addlink-cancel")} onclick={oncancel}>
        <Icon mono name="x" size={18} />
      </button>
    </div>

    <label class="url-field">
      <Icon mono name="link" size={18} />
      <input
        id="addlink-url"
        type="url"
        dir="ltr"
        autocomplete="off"
        spellcheck="false"
        placeholder="https://…"
        bind:this={box}
        bind:value={url}
        aria-describedby={fromClipboard ? "addlink-clip" : undefined}
      />
      {#if url}
        <button type="button" class="ghost icon-sm" aria-label={locale.t("addlink-cancel")} onclick={clear}>
          <Icon mono name="x" size={16} />
        </button>
      {:else}
        <button type="button" class="ghost paste-btn" onclick={() => void pasteFromClipboard()}>
          <Icon mono name="folder" size={16} />
          <span>{locale.t("addlink-from-clipboard")}</span>
        </button>
      {/if}
    </label>

    {#if problem}
      <p class="problem" role="alert"><Icon mono name="circle-alert" size={16} />{problem}</p>
    {/if}
    {#if fromClipboard}
      <p id="addlink-clip" class="note"><Icon mono name="check" size={14} />{locale.t("addlink-from-clipboard")}</p>
    {/if}

    <div class="actions">
      <button type="button" class="btn ghost-text" onclick={oncancel}>{locale.t("addlink-cancel")}</button>
      <button type="submit" class="btn primary big" disabled={!url.trim()}>
        <Icon mono name="download" size={18} />
        {locale.t("addlink-ok")}
      </button>
    </div>
  </form>
</main>

<style>
  .link {
    min-height: 100%;
    padding: 28px 24px;
    background: var(--surface);
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .card {
    width: min(560px, 100%);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .badge {
    width: 48px;
    height: 48px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 14px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  .titles h1 {
    margin: 0;
    font-size: 19px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--ink);
  }
  .titles p {
    margin: 2px 0 0;
    font-size: 13px;
    color: var(--muted);
  }
  .close-x {
    align-self: flex-start;
  }
  .url-field {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 52px;
    border: 1px solid var(--line-strong);
    background: var(--sunken);
    border-radius: 12px;
    padding: 6px 6px 6px 14px;
    color: var(--muted);
    transition: border-color 140ms ease, background 140ms ease, box-shadow 140ms ease;
  }
  .url-field:focus-within {
    border-color: var(--accent);
    background: var(--surface);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 22%, transparent);
    color: var(--ink);
  }
  input {
    border: 0;
    background: none;
    outline: none;
    flex: 1;
    min-width: 0;
    font: inherit;
    font-size: 14px;
    color: var(--ink);
    text-align: left;
  }
  input::placeholder {
    color: var(--faint);
  }
  .ghost {
    border: 1px solid transparent;
    background: none;
    border-radius: 9px;
    color: var(--muted);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font: inherit;
    font-size: 12px;
  }
  .ghost:hover {
    background: var(--sel);
    color: var(--ink);
  }
  .icon-sm {
    padding: 6px;
  }
  .problem {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--red);
    background: var(--red-bg);
    border-radius: 10px;
    padding: 10px 12px;
    margin: 0;
  }
  .note {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12.5px;
    color: var(--green);
    background: var(--green-bg);
    border-radius: 10px;
    padding: 8px 12px;
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: flex-end;
    padding-top: 4px;
  }
  .ghost-text {
    border-color: transparent;
    background: none;
  }
  .big {
    min-height: 44px;
    padding: 0 22px;
    font-size: 14px;
  }
</style>
