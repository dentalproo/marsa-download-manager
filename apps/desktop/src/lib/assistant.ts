// AI Download Assistant (v1): local intent recognition. No language model.
// Arabic + English keywords over the live snapshot.
import type { Snapshot } from "./bindings/Snapshot";

export type Intent =
  | { kind: "stopped" }
  | { kind: "duplicates" }
  | { kind: "week" }
  | { kind: "add"; url: string }
  | { kind: "sort" }
  | { kind: "unknown" };

const URL_RE = /https?:\/\/\S+/i;

export function parseIntent(text: string): Intent {
  const low = text.toLowerCase().trim();
  const url = text.match(URL_RE)?.[0];
  if (url) return { kind: "add", url };
  if (/توقف|stopped|paused|فشل|failed/.test(low)) return { kind: "stopped" };
  if (/مكرر|duplicates|duplicate/.test(low)) return { kind: "duplicates" };
  if (/أسبوع|week|مساحة|كم/.test(low)) return { kind: "week" };
  if (/رتب|sort|النوع|category/.test(low)) return { kind: "sort" };
  return { kind: "unknown" };
}

/** Run the intent against the snapshot; returns a spoken answer. */
export function answer(intent: Intent, snapshot: Snapshot): string {
  const list = snapshot.downloads;
  switch (intent.kind) {
    case "stopped": {
      const stopped = list.filter((d) => d.status === "paused" || d.status === "failed");
      return stopped.length
        ? `${stopped.length}: ${stopped.slice(0, 3).map((d) => d.name).join("، ")}${stopped.length > 3 ? "…" : ""}`
        : "—";
    }
    case "duplicates": {
      const byName = new Map<string, number>();
      for (const d of list) byName.set(d.name, (byName.get(d.name) ?? 0) + 1);
      const dupes = [...byName.entries()].filter(([, n]) => n > 1);
      return dupes.length ? dupes.map(([n, c]) => `${c}× ${n}`).join("، ") : "—";
    }
    case "week": {
      const cutoff = Date.now() - 7 * 24 * 60 * 60 * 1000;
      const bytes = list.filter((d) => d.added && new Date(d.added).getTime() >= cutoff).reduce((s, d) => s + d.done, 0);
      const mb = (bytes / 1024 / 1024).toFixed(1);
      return `${mb} MB`;
    }
    case "sort":
      return list.length ? list.map((d) => d.name).join("، ") : "—";
    case "add":
      return intent.url;
    default:
      return "";
  }
}
