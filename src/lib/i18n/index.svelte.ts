import { load, type Store } from "@tauri-apps/plugin-store";
import en from "./locales/en.json";
import tr from "./locales/tr.json";

export type Locale = "en" | "tr";

export const locales: Record<Locale, string> = {
  en: "English",
  tr: "Türkçe",
};

const dictionaries: Record<Locale, Record<string, string>> = { en, tr };

const STORE_FILE = "sermobile-settings.json";
const LOCALE_KEY = "sermobile-locale";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load(STORE_FILE, { autoSave: true });
  }
  return storeInstance;
}

class I18nState {
  locale = $state<Locale>("en");

  async init() {
    const store = await getStore();
    const saved = await store.get<Locale>(LOCALE_KEY);
    if (saved && saved in dictionaries) {
      this.locale = saved;
    }
  }

  async setLocale(locale: Locale) {
    this.locale = locale;
    const store = await getStore();
    await store.set(LOCALE_KEY, locale);
  }
}

export const i18n = new I18nState();

export function t(key: string, params?: Record<string, string | number>): string {
  let str = dictionaries[i18n.locale][key] ?? dictionaries.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      str = str.replaceAll(`{${k}}`, String(v));
    }
  }
  return str;
}
