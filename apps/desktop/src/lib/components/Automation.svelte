<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";
  import { api } from "../api";

  export type Rule = { id: string; name: string; trigger: string; action: string; target: string; enabled: boolean };

  let { rules = $bindable(), ontoggle }: { rules: Rule[]; ontoggle?: (rule: Rule) => void } = $props();

  let name = $state("");
  let trigger = $state("finished");
  let action = $state("extract");
  let target = $state("");
  let said = $state<string | null>(null);

  async function add() {
    if (!name.trim()) return;
    try {
      const rule = await api.automationUpsert({ id: "", name: name.trim(), trigger, action, target: target.trim(), enabled: true });
      rules = [...rules, rule];
      name = ""; target = ""; said = null;
    } catch (e) {
      said = String(e);
    }
  }
  async function setEnabled(rule: Rule, enabled: boolean) {
    try {
      const updated = await api.automationUpsert({ ...rule, enabled });
      rules = rules.map((r) => (r.id === updated.id ? updated : r));
    } catch { /* reload decides */ }
    ontoggle?.(rule);
  }
  async function remove(id: string) {
    await api.automationDelete(id).catch(() => {});
    rules = rules.filter((r) => r.id !== id);
  }
</script>

<section class="automation-page" aria-labelledby="auto-title">
  <header>
    <h1 id="auto-title">{locale.t("auto-title")}</h1>
    <p class="sub">{locale.t("auto-sub")}</p>
  </header>
  <div class="card">
    <div class="row">
      <input class="input grow" bind:value={name} dir="auto" placeholder={locale.t("auto-name")} aria-label={locale.t("auto-name")} />
      <select class="input" bind:value={trigger} aria-label={locale.t("auto-trigger")}>
        <option value="finished">{locale.t("auto-on-finished")}</option>
        <option value="failed">{locale.t("auto-on-failed")}</option>
        <option value="added">{locale.t("auto-on-added")}</option>
      </select>
      <select class="input" bind:value={action} aria-label={locale.t("auto-action")}>
        <option value="extract">{locale.t("auto-do-extract")}</option>
        <option value="move_category">{locale.t("auto-do-move")}</option>
        <option value="notify">{locale.t("auto-do-notify")}</option>
      </select>
    </div>
    {#if action === "move_category"}
      <input class="input" bind:value={target} dir="auto" placeholder={locale.t("auto-target")} aria-label={locale.t("auto-target")} />
    {/if}
    {#if said}<p class="error" role="alert">{said}</p>{/if}
    <button class="btn primary" onclick={() => void add()}>{locale.t("auto-add")}</button>
  </div>
  {#each rules as rule (rule.id)}
    <div class="card rule">
      <label class="check">
        <input type="checkbox" checked={rule.enabled} onchange={(e) => void setEnabled(rule, e.currentTarget.checked)} />
        <span class="rname">{rule.name}</span>
        <span class="muted">{rule.trigger} → {rule.action}{rule.target ? ` (${rule.target})` : ""}</span>
      </label>
      <button class="btn icon quiet" aria-label={locale.t("tb-delete")} onclick={() => void remove(rule.id)}><Icon mono name="trash" size={16} /></button>
    </div>
  {/each}
</section>

<style>
  .automation-page { padding: 28px 32px; overflow: auto; display: flex; flex-direction: column; gap: 12px; }
  header h1 { font-size: 24px; font-weight: 700; margin: 0; }
  .sub { color: var(--muted); font-size: 13px; margin: 6px 0 0; }
  .card { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius-l); box-shadow: var(--shadow); padding: 16px; display: flex; flex-direction: column; gap: 10px; }
  .row { display: flex; gap: 8px; }
  .grow { flex: 1; min-width: 0; }
  .rule { flex-direction: row; align-items: center; justify-content: space-between; }
  .check { display: flex; gap: 10px; align-items: center; }
  .rname { font-weight: 600; }
  .muted { color: var(--muted); font-size: 12px; }
  .error { background: var(--red-bg); color: var(--red); border-radius: 10px; padding: 10px 12px; font-size: 12.5px; }
</style>
