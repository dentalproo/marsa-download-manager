<script lang="ts">
  import Icon from "./Icon.svelte";
  import { locale } from "../i18n.svelte";
  import { api } from "../api";

  export type Plugin = { id: string; name: string; kind: string; enabled: boolean };

  let { plugins = $bindable() }: { plugins: Plugin[] } = $props();

  let name = $state("");
  let kind = $state("browser_connector");
  let said = $state<string | null>(null);
  let resultByPlugin = $state<Record<string, string>>({});
  async function run(id: string) {
    try {
      const out = await api.pluginsRun(id, "{}");
      resultByPlugin = { ...resultByPlugin, [id]: out };
    } catch (e) {
      resultByPlugin = { ...resultByPlugin, [id]: String(e) };
    }
  }

  async function add() {
    if (!name.trim()) return;
    try {
      const plugin = await api.pluginsUpsert({ id: "", name: name.trim(), kind, enabled: true });
      plugins = [...plugins, plugin];
      name = ""; said = null;
    } catch (e) {
      said = String(e);
    }
  }
  async function setEnabled(plugin: Plugin, enabled: boolean) {
    try {
      const updated = await api.pluginsUpsert({ ...plugin, enabled });
      plugins = plugins.map((p) => (p.id === updated.id ? updated : p));
    } catch { /* reload decides */ }
  }
  async function remove(id: string) {
    await api.pluginsDelete(id).catch(() => {});
    plugins = plugins.filter((p) => p.id !== id);
  }
</script>

<section class="plugins-page" aria-labelledby="plugins-title">
  <header>
    <h1 id="plugins-title">{locale.t("plugins-title")}</h1>
    <p class="sub">{locale.t("plugins-sub")}</p>
  </header>
  <div class="card">
    <div class="row">
      <input class="input grow" bind:value={name} dir="auto" placeholder={locale.t("plugins-new")} aria-label={locale.t("plugins-new")} />
      <select class="input" bind:value={kind} aria-label={locale.t("plugins-kind")}>
        <option value="browser_connector">{locale.t("plugins-kind-browser")}</option>
        <option value="file_analyzer">{locale.t("plugins-kind-analyzer")}</option>
        <option value="metadata">{locale.t("plugins-kind-metadata")}</option>
        <option value="archive_tools">{locale.t("plugins-kind-archive")}</option>
        <option value="automation">{locale.t("plugins-kind-automation")}</option>
      </select>
    </div>
    {#if said}<p class="error" role="alert">{said}</p>{/if}
    <button class="btn primary" onclick={() => void add()}>{locale.t("auto-add")}</button>
  </div>
  {#each plugins as plugin (plugin.id)}
    <div class="card rule">
      <label class="check">
        <input type="checkbox" checked={plugin.enabled} onchange={(e) => void setEnabled(plugin, e.currentTarget.checked)} />
        <span class="rname">{plugin.name}</span>
        <span class="muted">{plugin.kind}</span>
      </label>
      <button class="btn quiet" onclick={() => void run(plugin.id)}>{locale.t("plugins-run")}</button>
      <button class="btn icon quiet" aria-label={locale.t("tb-delete")} onclick={() => void remove(plugin.id)}><Icon mono name="trash" size={16} /></button>
    </div>
    {#if resultByPlugin[plugin.id]}<p class="result" dir="ltr">{resultByPlugin[plugin.id]}</p>{/if}
  {/each}
</section>

<style>
  .plugins-page { padding: 28px 32px; overflow: auto; display: flex; flex-direction: column; gap: 12px; }
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
