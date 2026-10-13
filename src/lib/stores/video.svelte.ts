import { load } from "@tauri-apps/plugin-store";

const STORE_FILE = "sermobile-settings.json";
const KEY = "sermobile-video-player";

/** YouTube video ID from a watch / youtu.be / embed / shorts / live link. */
export function youtubeId(url: string): string | undefined {
  return url.match(/(?:youtu\.be\/|[?&]v=|\/(?:embed|shorts|live)\/)([\w-]{11})/)?.[1];
}

/** The corner video player shown during builds and deploys. */
class VideoState {
  enabled = $state(true);
  url = $state("");
  /** Starting volume, 0–100. */
  volume = $state(35);
  videoId = $derived(youtubeId(this.url));

  async init() {
    const saved = await (await load(STORE_FILE, { autoSave: true })).get<{ enabled: boolean; url: string; volume?: number }>(KEY);
    if (saved) {
      this.enabled = saved.enabled;
      this.url = saved.url;
      this.volume = saved.volume ?? this.volume;
    }
  }

  async save(changes: Partial<{ enabled: boolean; url: string; volume: number }>) {
    Object.assign(this, changes);
    const store = await load(STORE_FILE, { autoSave: true });
    await store.set(KEY, { enabled: this.enabled, url: this.url, volume: this.volume });
  }
}

export const videoState = new VideoState();
