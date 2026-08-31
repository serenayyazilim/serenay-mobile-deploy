<script lang="ts">
  import { CircleX } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false), title, logs, onClose }: { open: boolean; title: string; logs: string[]; onClose?: () => void } = $props();

  function copyLogs() {
    navigator.clipboard.writeText(logs.join("\n"));
  }

  function logClass(log: string): string {
    if (/error|❌|failed/i.test(log)) return "text-red-400";
    if (/warning|⚠/i.test(log)) return "text-yellow-400";
    return "text-zinc-300";
  }

  function handleOpenChange(value: boolean) {
    open = value;
    if (!value) onClose?.();
  }
</script>

<Dialog bind:open onOpenChange={handleOpenChange}>
  <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
    <DialogHeader>
      <DialogTitle class="text-red-600 flex items-center gap-2">
        <CircleX class="w-5 h-5" />
        {title}
      </DialogTitle>
    </DialogHeader>

    <div class="flex-1 overflow-auto bg-zinc-950 rounded-lg p-4 font-mono text-sm">
      {#each logs as log, i (i)}
        <div class={`py-0.5 whitespace-pre-wrap break-all ${logClass(log)}`}>{log}</div>
      {/each}
    </div>

    <div class="flex justify-end gap-2 pt-4">
      <Button variant="outline" onclick={copyLogs}>{t("common.copy")}</Button>
      <Button onclick={() => handleOpenChange(false)}>{t("common.close")}</Button>
    </div>
  </DialogContent>
</Dialog>
