<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { LoaderCircle, RefreshCw, CircleAlert, Sparkles, DownloadCloud, Compass } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let { onRestartTour }: { onRestartTour?: () => void } = $props();

  let appVersion = $state("");
  getVersion().then((v) => (appVersion = v));

  type Status = "idle" | "checking" | "up-to-date" | "available" | "downloading" | "error";
  let status = $state<Status>("idle");
  let update = $state<Update | null>(null);

  async function checkForUpdates() {
    status = "checking";
    update = null;
    try {
      const result = await check();
      if (result) {
        update = result;
        status = "available";
      } else {
        status = "up-to-date";
      }
    } catch {
      status = "error";
    }
  }

  async function installUpdate() {
    if (!update) return;
    status = "downloading";
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch {
      status = "error";
    }
  }
</script>

<div class="space-y-5">
  <div>
    <h3 class="text-sm font-semibold">{t("settings.aboutTab")}</h3>
    <p class="text-xs text-muted-foreground mt-0.5">{t("about.version", { version: appVersion || "…" })}</p>
  </div>

  <div class="space-y-3">
    {#if status === "available" && update}
      <div class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-primary/30 bg-primary/5">
        <Sparkles class="w-4 h-4 shrink-0 text-primary" />
        <p class="flex-1 min-w-0 text-foreground">{t("about.updateAvailable", { version: update.version })}</p>
      </div>
      <Button class="gap-2" onclick={installUpdate}>
        <DownloadCloud class="w-4 h-4" />
        {t("about.installAndRestart")}
      </Button>
    {:else if status === "downloading"}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-muted-foreground">
        <LoaderCircle class="w-4 h-4 animate-spin" />
        {t("about.installing")}
      </div>
    {:else}
      {#if status === "up-to-date"}
        <p class="text-sm text-muted-foreground">{t("about.upToDate")}</p>
      {:else if status === "error"}
        <div class="flex items-center gap-2 text-sm text-red-600">
          <CircleAlert class="w-4 h-4 shrink-0" />
          {t("about.checkFailed")}
        </div>
      {/if}
      <Button variant="outline" class="gap-2" onclick={checkForUpdates} disabled={status === "checking"}>
        {#if status === "checking"}
          <LoaderCircle class="w-4 h-4 animate-spin" />
          {t("about.checking")}
        {:else}
          <RefreshCw class="w-4 h-4" />
          {t("about.checkForUpdates")}
        {/if}
      </Button>
    {/if}
  </div>

  {#if onRestartTour}
    <div class="pt-2 border-t border-border/50">
      <Button variant="outline" class="gap-2" onclick={onRestartTour}>
        <Compass class="w-4 h-4" />
        {t("about.restartTour")}
      </Button>
    </div>
  {/if}
</div>
