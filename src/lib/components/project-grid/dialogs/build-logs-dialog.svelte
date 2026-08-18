<script lang="ts">
  import { Terminal, LoaderCircle, CircleCheck, CircleX, RefreshCw, RotateCcw, Square } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";
  import { buildState, type BuildStatus } from "$lib/stores/build.svelte";

  let { open = $bindable(false), title, logs, status }: { open: boolean; title: string; logs: string[]; status: BuildStatus } = $props();

  let scrollEl: HTMLDivElement | undefined;

  $effect(() => {
    logs.length;
    if (open && scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
  });

  function copyLogs() {
    navigator.clipboard.writeText(logs.join("\n"));
  }

  function logClass(log: string): string {
    if (/error|❌|failed/i.test(log)) return "text-red-400";
    if (/warning|⚠/i.test(log)) return "text-yellow-400";
    return "text-zinc-300";
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        {#if status === "running"}<LoaderCircle class="w-5 h-5 animate-spin text-primary" />
        {:else if status === "success"}<CircleCheck class="w-5 h-5 text-green-500" />
        {:else if status === "error"}<CircleX class="w-5 h-5 text-red-500" />
        {:else}<Terminal class="w-5 h-5 text-muted-foreground" />{/if}
        {title}
      </DialogTitle>
    </DialogHeader>

    <div bind:this={scrollEl} class="flex-1 overflow-auto bg-zinc-950 rounded-lg p-4 font-mono text-sm">
      {#each logs as log, i (i)}
        <div class={`py-0.5 whitespace-pre-wrap break-all ${logClass(log)}`}>{log}</div>
      {/each}
    </div>

    <div class="flex items-center justify-between gap-2 pt-4">
      <div class="flex gap-2">
        {#if status === "running"}
          <Button variant="outline" size="sm" disabled={buildState.isReloading} onclick={() => buildState.hotReload()}>
            <RefreshCw class={`w-4 h-4 mr-1.5 ${buildState.isReloading ? "animate-spin" : ""}`} />
            {t("build.hotReload")}
          </Button>
          <Button variant="outline" size="sm" disabled={buildState.isRestarting} onclick={() => buildState.hotRestart()}>
            <RotateCcw class={`w-4 h-4 mr-1.5 ${buildState.isRestarting ? "animate-spin" : ""}`} />
            {t("build.hotRestart")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={buildState.isStopping}
            class="text-red-600 hover:text-red-600 border-red-200 hover:bg-red-50 dark:border-red-900 dark:hover:bg-red-950"
            onclick={() => buildState.stopBuild()}
          >
            {#if buildState.isStopping}<LoaderCircle class="w-4 h-4 mr-1.5 animate-spin" />
            {:else}<Square class="w-4 h-4 mr-1.5" />{/if}
            {t("build.stop")}
          </Button>
        {/if}
      </div>

      <div class="flex gap-2">
        <Button variant="outline" onclick={copyLogs}>{t("common.copy")}</Button>
        <Button onclick={() => (open = false)}>{t("common.close")}</Button>
      </div>
    </div>
  </DialogContent>
</Dialog>
