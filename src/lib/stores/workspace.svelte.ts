import { load, type Store } from "@tauri-apps/plugin-store";

export type WorkspaceMode = "sermobileboss" | "generic";

const STORE_FILE = "sermobile-settings.json";
const WORKSPACE_PATH_KEY = "sermobile-workspace-path";
const WORKSPACE_MODE_KEY = "sermobile-workspace-mode";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE, { autoSave: true });
  }
  return storeInstance;
}

class WorkspaceState {
  path = $state<string | null>(null);
  mode = $state<WorkspaceMode | null>(null);
  loaded = $state(false);

  async init() {
    const store = await getStore();
    this.path = (await store.get<string>(WORKSPACE_PATH_KEY)) ?? null;
    this.mode = (await store.get<WorkspaceMode>(WORKSPACE_MODE_KEY)) ?? null;
    this.loaded = true;
  }

  async setWorkspace(path: string, mode: WorkspaceMode) {
    this.path = path;
    this.mode = mode;
    const store = await getStore();
    await store.set(WORKSPACE_PATH_KEY, path);
    await store.set(WORKSPACE_MODE_KEY, mode);
  }

  async clear() {
    this.path = null;
    this.mode = null;
    const store = await getStore();
    await store.delete(WORKSPACE_PATH_KEY);
    await store.delete(WORKSPACE_MODE_KEY);
  }
}

export const workspaceState = new WorkspaceState();
