<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Store, ExternalLink, Package, LoaderCircle } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";
  import type { StorePlatform } from "./types";

  let {
    open = $bindable(false),
    stores,
    workspacePath,
    onRecheck,
  }: { open: boolean; stores: StorePlatform[]; workspacePath: string; onRecheck?: () => void } = $props();

  let buildingAab = $state(false);
  let aabError = $state("");

  async function buildAab() {
    buildingAab = true;
    aabError = "";
    try {
      const path = await invoke<string>("setup_build_aab", { workspacePath });
      await revealItemInDir(path);
    } catch (error) {
      aabError = String(error);
    } finally {
      buildingAab = false;
    }
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-lg">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Store class="w-5 h-5" />
        {t("firstRelease.title")}
      </DialogTitle>
      <DialogDescription class="pt-1">{t("firstRelease.description")}</DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      {#if stores.includes("ios")}
        <section class="space-y-2 rounded-lg border p-4">
          <h3 class="font-medium">App Store Connect</h3>
          <ol class="list-decimal space-y-1 pl-5 text-sm text-muted-foreground">
            <li>{t("firstRelease.ios.step1")}</li>
            <li>{t("firstRelease.ios.step2")}</li>
            <li>{t("firstRelease.ios.step3")}</li>
          </ol>
          <Button variant="outline" size="sm" onclick={() => openUrl("https://appstoreconnect.apple.com/apps")}>
            <ExternalLink />
            {t("firstRelease.openAppStore")}
          </Button>
        </section>
      {/if}

      {#if stores.includes("android")}
        <section class="space-y-2 rounded-lg border p-4">
          <h3 class="font-medium">Google Play Console</h3>
          <ol class="list-decimal space-y-1 pl-5 text-sm text-muted-foreground">
            <li>{t("firstRelease.android.step1")}</li>
            <li>{t("firstRelease.android.step2")}</li>
            <li>{t("firstRelease.android.step3")}</li>
          </ol>
          <div class="flex flex-wrap gap-2">
            <Button variant="outline" size="sm" onclick={() => openUrl("https://play.google.com/console")}>
              <ExternalLink />
              {t("firstRelease.openPlayConsole")}
            </Button>
            <Button variant="outline" size="sm" onclick={buildAab} disabled={buildingAab}>
              {#if buildingAab}
                <LoaderCircle class="animate-spin" />
                {t("firstRelease.buildingAab")}
              {:else}
                <Package />
                {t("firstRelease.buildAab")}
              {/if}
            </Button>
          </div>
          {#if aabError}
            <pre class="max-h-40 overflow-auto rounded bg-zinc-950 p-2 text-xs text-red-400 whitespace-pre-wrap">{aabError}</pre>
          {/if}
        </section>
      {/if}
    </div>

    <div class="flex justify-end gap-2 pt-2">
      {#if onRecheck}
        <Button
          variant="outline"
          onclick={() => {
            open = false;
            onRecheck();
          }}>{t("setup.recheck")}</Button
        >
      {/if}
      <Button onclick={() => (open = false)}>{t("common.close")}</Button>
    </div>
  </DialogContent>
</Dialog>
