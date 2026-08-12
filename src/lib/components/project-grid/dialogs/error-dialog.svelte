<script lang="ts">
  import { CircleX } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";

  let { open = $bindable(false), title, logs }: { open: boolean; title: string; logs: string[] } = $props();

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
      <Button variant="outline" onclick={copyLogs}>Kopyala</Button>
      <Button onclick={() => (open = false)}>Kapat</Button>
    </div>
  </DialogContent>
</Dialog>
