import { load, type Store } from "@tauri-apps/plugin-store";

export type ThemeMode = "light" | "dark" | "system";

const STORE_FILE = "sermobile-settings.json";
const THEME_KEY = "sermobile-theme";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE, { autoSave: true });
  }
  return storeInstance;
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function applyMode(mode: ThemeMode) {
  const isDark = mode === "dark" || (mode === "system" && prefersDark());
  document.documentElement.classList.toggle("dark", isDark);
}

class ThemeState {
  mode = $state<ThemeMode>("system");

  async init() {
    const store = await getStore();
    const saved = await store.get<ThemeMode>(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system") {
      this.mode = saved;
    }
    applyMode(this.mode);

    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (this.mode === "system") applyMode(this.mode);
    });
  }

  async setMode(mode: ThemeMode) {
    this.mode = mode;
    applyMode(mode);
    const store = await getStore();
    await store.set(THEME_KEY, mode);
  }
}

export const themeState = new ThemeState();
