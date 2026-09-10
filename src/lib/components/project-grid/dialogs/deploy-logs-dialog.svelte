<script lang="ts">
  import { Terminal, LoaderCircle } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false), title, logs, isRunning }: { open: boolean; title: string; logs: string[]; isRunning: boolean } = $props();

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
    if (/✅|success/i.test(log)) return "text-green-400";
    return "text-zinc-300";
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        {#if isRunning}<LoaderCircle class="w-5 h-5 animate-spin text-primary" />
        {:else}<Terminal class="w-5 h-5 text-muted-foreground" />{/if}
        {title}
      </DialogTitle>
    </DialogHeader>

    <div bind:this={scrollEl} class="flex-1 overflow-auto bg-zinc-950 rounded-lg p-4 font-mono text-sm">
      {#each logs as log, i (i)}
        <div class={`py-0.5 whitespace-pre-wrap break-all ${logClass(log)}`}>{log}</div>
      {/each}
    </div>

    <div class="flex justify-end gap-2 pt-4">
      <Button variant="outline" onclick={copyLogs}>{t("common.copy")}</Button>
      <Button onclick={() => (open = false)}>{t("common.close")}</Button>
    </div>
  </DialogContent>
</Dialog>
