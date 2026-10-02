import { getVersion } from "@tauri-apps/api/app";
import { load, type Store } from "@tauri-apps/plugin-store";
import changelog from "../../../CHANGELOG.md?raw";

const STORE_FILE = "sermobile-settings.json";
const LAST_SEEN_KEY = "sermobile-last-seen-version";

export type ChangeKind = "new" | "improved" | "fixed";
export type Change = { kind: ChangeKind; title: string; body: string };
export type Release = { version: string; date: string; changes: Change[] };

function kindOf(text: string): ChangeKind {
  if (/^(Added|Introduced)\b/.test(text)) return "new";
  if (/^Fixed\b/.test(text)) return "fixed";
  return "improved";
}

// "Added a setup wizard: detects …" → title "Added a setup wizard", body "Detects …".
function toChange(text: string): Change {
  const colon = text.indexOf(": ");
  const kind = kindOf(text);
  if (colon > 0 && colon < 90) {
    const body = text.slice(colon + 2);
    return { kind, title: text.slice(0, colon), body: body[0].toUpperCase() + body.slice(1) };
  }
  return { kind, title: "", body: text };
}

// "## [0.2.11] - 2026-10-01" headings followed by "- item" lines.
export const releases: Release[] = changelog
  .split(/^## /m)
  .slice(1)
  .map((block) => {
    const [heading, ...lines] = block.split("\n");
    const [, version = "", date = ""] = heading.match(/^\[(.+?)\]\s*-\s*(.+)$/) ?? [];
    const changes = lines.filter((l) => l.startsWith("- ")).map((l) => toChange(l.slice(2).trim()));
    return { version, date: date.trim(), changes };
  })
  .filter((r) => r.version && r.changes.length);

function newer(a: string, b: string): boolean {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) if ((pa[i] ?? 0) !== (pb[i] ?? 0)) return (pa[i] ?? 0) > (pb[i] ?? 0);
  return false;
}

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE, { autoSave: true });
  }
  return storeInstance;
}

class ChangelogState {
  open = $state(false);
  shown = $state<Release[]>([]);
  currentVersion = $state("");

  /** Shows the releases since the last seen version; fresh installs only record the version. */
  async init(isFreshInstall: boolean) {
    this.currentVersion = await getVersion();
    const store = await getStore();
    const lastSeen = await store.get<string>(LAST_SEEN_KEY);
    if (lastSeen === this.currentVersion) return;
    if (!lastSeen && isFreshInstall) {
      await store.set(LAST_SEEN_KEY, this.currentVersion);
      return;
    }
    const unseen = releases.filter(
      (r) => !newer(r.version, this.currentVersion) && (!lastSeen || newer(r.version, lastSeen))
    );
    this.shown = lastSeen ? unseen : unseen.slice(0, 1);
    if (this.shown.length) this.open = true;
    else await store.set(LAST_SEEN_KEY, this.currentVersion);
  }

  /** Reopens the notes for the running version (Settings > About). */
  async showCurrent() {
    this.currentVersion ||= await getVersion();
    const current = releases.find((r) => !newer(r.version, this.currentVersion));
    if (!current) return;
    this.shown = [current];
    this.open = true;
  }

  async dismiss() {
    this.open = false;
    const store = await getStore();
    await store.set(LAST_SEEN_KEY, this.currentVersion);
  }
}

export const changelogState = new ChangelogState();
