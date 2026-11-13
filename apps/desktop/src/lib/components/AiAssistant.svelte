<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";
  import { parseIntent, answer } from "../assistant";
  import type { Snapshot } from "../bindings/Snapshot";

  let { snapshot, onclose, onapply }: {
    snapshot: Snapshot;
    onclose: () => void;
    onapply: (intent: ReturnType<typeof parseIntent>) => void;
  } = $props();

  let text = $state("");
  let result = $state("");
  function run() {
    const intent = parseIntent(text);
    if (intent.kind === "unknown") {
      result = locale.t("ai-nothing");
      return;
    }
    result = answer(intent, snapshot);
  }
</script>

<div class="scrim" role="presentation">
  <section class="dialog" aria-labelledby="ai-title">
    <header>
      <span class="badge" aria-hidden="true"><Icon mono name="zap" size={20} /></span>
      <h1 id="ai-title">{locale.t("ai-title")}</h1>
      <button class="btn icon quiet close-x" aria-label={locale.t("ai-cancel")} onclick={onclose}><Icon mono name="x" size={18} /></button>
    </header>
    <p class="hint">{locale.t("ai-hint")}</p>
    <form onsubmit={(e) => (e.preventDefault(), run())}>
      <input bind:value={text} class="input" dir="auto" placeholder={locale.t("ai-hint")} />
      {#if result}<p class="result" role="status">{result}</p>{/if}
      <div class="actions">
        <button type="button" class="btn ghost-text" onclick={onclose}>{locale.t("ai-cancel")}</button>
        <button type="submit" class="btn primary big">{locale.t("tb-ai")}</button>
        <button type="button" class="btn" onclick={() => onapply(parseIntent(text))}>{locale.t("ai-apply")}</button>
      </div>
    </form>
  </section>
</div>

<style>
  .scrim {
    position: fixed; inset: 0; z-index: 40;
    background: var(--scrim);
    display: flex; align-items: center; justify-content: center;
    padding: 24px;
  }
  .dialog {
    width: min(560px, 100%);
    background: var(--surface); border: 1px solid var(--line);
    border-radius: var(--radius-l); box-shadow: var(--shadow-lg);
    padding: 24px; display: flex; flex-direction: column; gap: 14px;
    animation: pop 200ms ease both;
  }
  @keyframes pop { from { opacity: 0; transform: scale(0.96) translateY(8px); } to { opacity: 1; transform: none; } }
  header { display: flex; align-items: center; gap: 12px; }
  .badge { width: 44px; height: 44px; display: grid; place-items: center; border-radius: 12px; background: var(--accent-soft); color: var(--accent); }
  h1 { font-size: 19px; flex: 1; margin: 0; }
  .hint { margin: 0; color: var(--muted); font-size: 13px; line-height: 1.6; }
  .input { width: 100%; }
  .result { background: var(--accent-soft); border-radius: 10px; padding: 10px 12px; font-size: 13px; margin: 0; word-break: break-word; }
  .actions { display: flex; gap: 10px; justify-content: flex-end; align-items: center; }
</style>
