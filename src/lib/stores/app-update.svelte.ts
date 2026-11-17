import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type UpdateStatus = "idle" | "checking" | "up-to-date" | "available" | "downloading" | "error";

const DAY_MS = 24 * 60 * 60 * 1000;

/** This app's own version and whether a newer one is published; also reported to the MCP server. */
class AppUpdateState {
  version = $state("");
  status = $state<UpdateStatus>("idle");
  latestVersion = $state<string | null>(null);
  notes = $state<string | null>(null);
  lastChecked = $state<string | null>(null);
  error = $state<string | null>(null);

  private update: Update | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;

  async checkNow() {
    if (this.status === "checking" || this.status === "downloading") return;
    this.status = "checking";
    this.error = null;
    try {
      if (!this.version) this.version = await getVersion();
      const result = await check();
      this.update = result;
      this.latestVersion = result?.version ?? null;
      this.notes = result?.body ?? null;
      this.status = result ? "available" : "up-to-date";
    } catch (e) {
      this.error = String(e);
      this.status = "error";
    }
    this.lastChecked = new Date().toISOString();
  }

  async install() {
    if (!this.update) return;
    this.status = "downloading";
    try {
      await this.update.downloadAndInstall();
      await relaunch();
    } catch (e) {
      this.error = String(e);
      this.status = "error";
    }
  }

  /** Checks now, then once a day for as long as the app stays open. */
  startDailyCheck() {
    if (this.timer) return;
    getVersion().then((v) => (this.version = v));
    this.checkNow();
    this.timer = setInterval(() => this.checkNow(), DAY_MS);
  }

  stopDailyCheck() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
}

export const appUpdateState = new AppUpdateState();
