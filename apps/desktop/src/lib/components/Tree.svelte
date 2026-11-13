<script lang="ts" module>
  import type { Section } from "../sections";
  /** What the table shows (M5.5): every download, a category (or only its
   * unfinished or finished ones), the downloads of no category, a queue's,
   * or the unfinished, finished or waiting ones (of one category, or of
   * none with `null`), as IDM's tree (the owner's notes on 1.0). */
  export type Filter =
    | { kind: "all" }
    | { kind: "category"; id: string; part?: "unfinished" | "finished" }
    | { kind: "uncategorized" }
    | { kind: "queue"; name: string }
    | { kind: "collection"; id: string }
    | { kind: "status"; status: Section; category?: string | null };

  export type View = "list" | "settings" | "categories" | "health" | "automation" | "plugins" | "torrents";

  /** A category's icon (Lucide), by its built-in id. */
  export function categoryIcon(id: string | null): string {
    return ({ programs: "package", video: "film", music: "music", documents: "file-text", archives: "archive" } as Record<string, string>)[id ?? ""] ?? "file";
  }
</script>

<script lang="ts">
  import ColorIcon, { typeIcon } from "./ColorIcon.svelte";
  import { inSection } from "../sections";
  import { locale } from "../i18n.svelte";
  import { categoryName as nameOf } from "../categoryName";
  import { num } from "../format";
  import { scheduleApi } from "../scheduleApi";
  import type { Snapshot } from "../bindings/Snapshot";

  let {
    snapshot,
    collections = [],
    filter = $bindable(),
    view = $bindable(),
    oncreate,
  }: {
    snapshot: Snapshot;
    collections?: { id: string; name: string; folder: string }[];
    filter: Filter;
    view: View;
    oncreate?: (name: string, folder: string) => Promise<void>;
  } = $props();

  type Download = Snapshot["downloads"][number];

  // The categories whose «unfinished» and «finished» show under them.
  let open = $state<Set<string>>(new Set());

  function show(f: Filter) {
    filter = f;
    view = "list";
  }
  function toggle(id: string) {
    const next = new Set(open);
    if (!next.delete(id)) next.add(id);
    open = next;
  }
  const count = (pick: (d: Download) => boolean) => num(locale.lang, locale.digits, snapshot.downloads.filter(pick).length, 0);
  const same = (a: Filter, b: Filter) => JSON.stringify(a) === JSON.stringify(b);
  const current = (f: Filter) => (view === "list" && same(filter, f) ? "page" : undefined);
  const categoryName = (c: Snapshot["categories"][number]) => nameOf(c.id, c.name, (k) => locale.t(k));
  const queueName = (name: string) => (name === "main" ? locale.t("queue-main") : name);
  // «New queue» right under the queues (owner's report, 2026-10-29: it
  // was only in the schedule window), its name typed in place.
  let adding = $state(false);
  let newName = $state("");
  let said = $state<string | null>(null);
  function stopAdding() {
    adding = false;
    newName = "";
    said = null;
  }
  async function create() {
    const name = newName.trim();
    if (!name) return stopAdding();
    try {
      await scheduleApi.newQueue(name);
      stopAdding();
      show({ kind: "queue", name });
    } catch (e) {
      const text = String(e);
      said = text.includes("invalid queue name") ? locale.t("schedule-new-hint") : text.includes("already a queue") ? locale.t("tree-queue-exists") : text;
    }
  }
  // Collections (v1): name + save folder; filter by that folder.
  let addingCollection = $state(false);
  let collName = $state("");
  let collFolder = $state("");
  let collSaid = $state<string | null>(null);
  async function createCollection() {
    const name = collName.trim();
    const folder = collFolder.trim();
    if (!name || !folder) return;
    try {
      await oncreate?.(name, folder);
      addingCollection = false;
      collName = "";
      collFolder = "";
      collSaid = null;
    } catch (e) {
      collSaid = String(e);
    }
  }
  const SECTIONS = [
    { section: "unfinished", icon: "unfinished", key: "tree-unfinished" },
    { section: "finished", icon: "finished", key: "tree-finished" },
    { section: "waiting", icon: "waiting", key: "tree-waiting" },
  ] as const;
</script>

<nav class="tree" aria-label={locale.t("tree-label")}>
  <button class="node top" aria-current={current({ kind: "all" })} onclick={() => show({ kind: "all" })}>
    <ColorIcon name="all" size={18} /><span class="label">{locale.t("nav-all")}</span><span class="n num">{count(() => true)}</span>
  </button>

  <!-- Under «All downloads»: the unfinished, finished and waiting ones,
       each with its categories inside (as IDM's). -->
  {#each SECTIONS as sec (sec.section)}
    {@const of = (d: Download) => inSection(sec.section, d.status)}
    {@const key = `section:${sec.section}`}
    {@const expanded = open.has(key)}
    <div class="row">
      <button
        class="twist"
        aria-expanded={expanded}
        aria-label={`${locale.t("tree-expand")}: ${locale.t(sec.key)}`}
        onclick={() => toggle(key)}
      ><span class="box" aria-hidden="true">{expanded ? "−" : "+"}</span></button>
      <button class="node" aria-current={current({ kind: "status", status: sec.section })} onclick={() => show({ kind: "status", status: sec.section })}>
        <ColorIcon name={sec.icon} size={17} /><span class="label">{locale.t(sec.key)}</span><span class="n num">{count(of)}</span>
      </button>
    </div>
    {#if expanded}
      {#each snapshot.categories as c (c.id)}
        <button class="node l1" aria-current={current({ kind: "status", status: sec.section, category: c.id })} onclick={() => show({ kind: "status", status: sec.section, category: c.id })}>
          <ColorIcon name={typeIcon(c.id)} size={16} /><span class="label">{categoryName(c)}</span><span class="n num">{count((d) => of(d) && d.category === c.id)}</span>
        </button>
      {/each}
      {#if snapshot.downloads.some((d) => d.category === null)}
        <button class="node l1" aria-current={current({ kind: "status", status: sec.section, category: null })} onclick={() => show({ kind: "status", status: sec.section, category: null })}>
          <ColorIcon name="t-file" size={16} /><span class="label">{locale.t("category-none")}</span><span class="n num">{count((d) => of(d) && d.category === null)}</span>
        </button>
      {/if}
    {/if}
  {/each}

  <h2>{locale.t("nav-categories")}</h2>
  {#each snapshot.categories as c (c.id)}
    {@const mine = (d: Download) => d.category === c.id}
    {@const expanded = open.has(c.id)}
    <div class="row">
      <button
        class="twist"
        aria-expanded={expanded}
        aria-label={`${locale.t("tree-expand")}: ${categoryName(c)}`}
        onclick={() => toggle(c.id)}
      ><span class="box" aria-hidden="true">{expanded ? "−" : "+"}</span></button>
      <button class="node" aria-current={current({ kind: "category", id: c.id })} onclick={() => show({ kind: "category", id: c.id })}>
        <ColorIcon name={typeIcon(c.id)} size={17} /><span class="label">{categoryName(c)}</span><span class="n num">{count(mine)}</span>
      </button>
    </div>
    {#if expanded}
      {#each [{ part: "unfinished", icon: "unfinished", key: "tree-unfinished", pick: (d: Download) => mine(d) && d.status !== "done" }, { part: "finished", icon: "finished", key: "tree-finished", pick: (d: Download) => mine(d) && d.status === "done" }] as const as sub (sub.part)}
        <button class="node l1" aria-current={current({ kind: "category", id: c.id, part: sub.part })} onclick={() => show({ kind: "category", id: c.id, part: sub.part })}>
          <ColorIcon name={sub.icon} size={16} /><span class="label">{locale.t(sub.key)}</span><span class="n num">{count(sub.pick)}</span>
        </button>
      {/each}
    {/if}
  {/each}
  {#if snapshot.downloads.some((d) => d.category === null)}
    <button class="node l0" aria-current={current({ kind: "uncategorized" })} onclick={() => show({ kind: "uncategorized" })}>
      <ColorIcon name="t-file" size={17} /><span class="label">{locale.t("category-none")}</span><span class="n num">{count((d) => d.category === null)}</span>
    </button>
  {/if}

  <h2>{locale.t("nav-queues")}</h2>
  {#each snapshot.queues as q (q.name)}
    <button class="node l0" aria-current={current({ kind: "queue", name: q.name })} onclick={() => show({ kind: "queue", name: q.name })}>
      <ColorIcon name={q.window ? "queue-timed" : "queue"} size={18} />
      <span class="label">
        <bdi>{queueName(q.name)}</bdi>
        {#if q.window}<small><bdi>{locale.t("queue-starts", { time: q.window[0] })}</bdi></small>{/if}
      </span>
      <span class="n num">{count((d) => d.queue === q.name)}</span>
    </button>
  {/each}
  {#if adding}
    <form class="newq" onsubmit={(e) => (e.preventDefault(), void create())}>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        bind:value={newName}
        dir="auto"
        placeholder={locale.t("schedule-new-name")}
        aria-label={locale.t("schedule-new-name")}
        autofocus
        onkeydown={(e) => e.key === "Escape" && (e.stopPropagation(), stopAdding())}
        onblur={() => !newName.trim() && stopAdding()}
      />
      {#if said}<small class="said" role="alert">{said}</small>{/if}
    </form>
  {:else}
    <button class="node l0 add" onclick={() => (adding = true)}>
      <ColorIcon name="add" size={15} /><span class="label">{locale.t("schedule-new")}</span>
    </button>
  {/if}

  <h2>{locale.t("health-title")}</h2>
  <button class="node l0" aria-current={view === "health" ? "page" : undefined} onclick={() => (view = "health")}>
    <ColorIcon name="c-gauge" size={18} /><span class="label">{locale.t("health-title")}</span>
  </button>
  <button class="node l0" aria-current={view === "automation" ? "page" : undefined} onclick={() => (view = "automation")}>
    <ColorIcon name="list-ordered" size={18} /><span class="label">{locale.t("auto-title")}</span>
  </button>
  <button class="node l0" aria-current={view === "plugins" ? "page" : undefined} onclick={() => (view = "plugins")}>
    <ColorIcon name="folder" size={18} /><span class="label">{locale.t("plugins-title")}</span>
  </button>
  <button class="node l0" aria-current={view === "torrents" ? "page" : undefined} onclick={() => (view = "torrents")}>
    <ColorIcon name="folder" size={18} /><span class="label">{locale.t("torrents-title")}</span>
  </button>

  <h2>{locale.t("collections-title")}</h2>
  {#each collections as c (c.id)}
    <button class="node l0" aria-current={current({ kind: "collection", id: c.id })} onclick={() => show({ kind: "collection", id: c.id })}>
      <ColorIcon name="folder" size={18} />
      <span class="label"><bdi>{c.name}</bdi></span>
      <span class="n num">{count((d) => d.folder === c.folder)}</span>
    </button>
  {/each}
  {#if addingCollection}
    <form class="newq" onsubmit={(e) => (e.preventDefault(), void createCollection())}>
      <input bind:value={collName} dir="auto" placeholder={locale.t("collections-new")} aria-label={locale.t("collections-new")} />
      <input bind:value={collFolder} dir="ltr" placeholder="C:\Users\…\Downloads" aria-label={locale.t("add-folder")} />
      {#if collSaid}<small class="said" role="alert">{collSaid}</small>{/if}
      <div class="col-actions">
        <button type="submit" class="btn primary">{locale.t("tb-add")}</button>
        <button type="button" class="btn" onclick={() => (addingCollection = false)}>{locale.t("add-cancel")}</button>
      </div>
    </form>
  {:else}
    <button class="node l0 add" onclick={() => { collSaid = null; addingCollection = true; }}>
      <ColorIcon name="add" size={15} /><span class="label">{locale.t("collections-new")}</span>
    </button>
  {/if}
</nav>

<style>
  .tree {
    background: var(--side);
    border-inline-end: 1px solid var(--line);
    padding: 14px 12px 18px;
    overflow: auto;
    font-size: 13.5px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .newq {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 2px 5px;
  }
  .newq input {
    font: inherit;
    padding: 3px 6px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: var(--surface);
    color: var(--ink);
    min-width: 0;
  }
  .said {
    color: var(--red);
    font-size: 11px;
    line-height: 1.35;
  }
  .add {
    color: var(--muted);
  }
  h2 {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
    margin: 18px 10px 6px;
    padding: 0;
    border: 0;
  }
  .row {
    display: flex;
    align-items: center;
  }
  .twist {
    width: 20px;
    height: 26px;
    display: grid;
    place-items: center;
    border: 0;
    background: none;
    cursor: pointer;
    padding: 0;
    flex: none;
  }
  /* A desktop tree's box: + to open, − to close. */
  .box {
    width: 14px;
    height: 14px;
    display: grid;
    place-items: center;
    border: 1px solid var(--line-strong);
    border-radius: 5px;
    background: var(--raised);
    color: var(--muted);
    font: 600 11px/1 system-ui, sans-serif;
  }
  .twist:hover .box {
    border-color: var(--accent);
    color: var(--accent);
  }
  .node {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 34px;
    padding: 5px 10px;
    border-radius: 10px;
    border: 0;
    background: none;
    color: var(--ink);
    text-align: start;
    cursor: pointer;
    min-width: 0;
    font: inherit;
    transition: background 130ms ease, color 130ms ease;
  }
  /* Wide only beside its twist, in a category's row: alone in the column
     it keeps its own height. */
  .row .node {
    flex: 1;
  }
  .node:hover {
    background: var(--sel);
  }
  .node[aria-current="page"] {
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 600;
    box-shadow: 0 6px 18px color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .node[aria-current="page"] .n {
    color: color-mix(in srgb, var(--accent-ink) 80%, transparent);
  }
  .top {
    font-size: 14px;
    font-weight: 500;
  }
  .l0 {
    margin-inline-start: 20px;
  }
  .l1 {
    margin-inline-start: 38px;
  }
  .label {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  small {
    color: var(--faint);
    font-size: 11px;
    line-height: 1.3;
  }
  .n {
    margin-inline-start: auto;
    color: var(--faint);
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    padding: 0 2px;
  }
  .node[aria-current="page"] .n {
    color: color-mix(in srgb, var(--accent-ink) 88%, transparent);
  }
</style>
