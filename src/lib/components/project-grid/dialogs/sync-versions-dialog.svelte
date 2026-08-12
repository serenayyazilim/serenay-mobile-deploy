<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { RefreshCw } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface SyncResult {
    updated: { id: string; old: string | null; new: string }[];
    skipped: string[];
    failed: string[];
  }

  interface SyncEvent {
    type: "log" | "error" | "result" | "done";
    message?: string;
    result?: SyncResult;
    success?: boolean;
  }

  let { open = $bindable(false), workspacePath }: { open: boolean; workspacePath: string } = $props();

  let running = $state(false);
  let logs = $state<{ type: string; message: string }[]>([]);
  let result = $state<SyncResult | null>(null);

  async function handleStart() {
    running = true;
    logs = [];
    result = null;

    let unlisten: UnlistenFn | null = null;
    try {
      const jobId = await invoke<string>("sync_versions_start", { workspacePath });
      await new Promise<void>((resolve) => {
        listen<SyncEvent>(`sync-versions-event-${jobId}`, (event) => {
          const data = event.payload;
          if (data.type === "result" && data.result) {
            result = data.result;
          } else if (data.type === "done") {
            resolve();
          } else {
            logs = [...logs, { type: data.type, message: data.message || "" }];
          }
        }).then((fn) => (unlisten = fn));
      });
    } catch (e) {
      logs = [...logs, { type: "error", message: String(e) }];
    } finally {
      running = false;
      (unlisten as UnlistenFn | null)?.();
    }
  }

  function handleClose() {
    if (running) return;
    logs = [];
    result = null;
    open = false;
  }
</script>

<Dialog {open} onOpenChange={(o: boolean) => { if (!o) handleClose(); }}>
  <DialogContent class="max-w-2xl max-h-[80vh] flex flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <RefreshCw class="w-5 h-5" />
        Mağaza Versiyonlarını Senkronize Et
      </DialogTitle>
    </DialogHeader>

    <p class="text-sm text-muted-foreground">
      Tüm projelerin mevcut mağaza versiyonlarını Google Play ve App Store'dan çekip
      <code class="text-xs bg-muted px-1 py-0.5 rounded">version.json</code> dosyalarını günceller.
    </p>

    {#if logs.length > 0}
      <div class="flex-1 overflow-y-auto rounded-md bg-black/90 p-3 font-mono text-xs min-h-0 max-h-80">
        {#each logs as log, i (i)}
          <div class={log.type === "error" ? "text-red-400" : log.message.includes("✅") ? "text-green-400" : log.message.includes("⚠️") ? "text-yellow-400" : "text-gray-300"}>
            {log.message}
          </div>
        {/each}
      </div>
    {/if}

    {#if result}
      <div class="rounded-md border bg-muted/30 px-4 py-3 space-y-2 text-sm">
        <p class="font-medium">Sonuç</p>
        <div class="grid grid-cols-3 gap-3 text-center">
          <div class="rounded-md bg-green-500/10 border border-green-500/20 py-2">
            <p class="text-lg font-bold text-green-500">{result.updated.length}</p>
            <p class="text-xs text-muted-foreground">Güncellendi</p>
          </div>
          <div class="rounded-md bg-muted/50 border py-2">
            <p class="text-lg font-bold">{result.skipped.length}</p>
            <p class="text-xs text-muted-foreground">Değişmedi</p>
          </div>
          <div class="rounded-md bg-red-500/10 border border-red-500/20 py-2">
            <p class="text-lg font-bold text-red-500">{result.failed.length}</p>
            <p class="text-xs text-muted-foreground">Başarısız</p>
          </div>
        </div>
        {#if result.updated.length > 0}
          <div class="space-y-1 max-h-40 overflow-y-auto">
            {#each result.updated as u (u.id)}
              <div class="flex items-center gap-2 text-xs text-muted-foreground">
                <span class="font-medium text-foreground w-32 shrink-0">{u.id}</span>
                <span class="text-muted-foreground/60">{u.old || "?"}</span>
                <span>→</span>
                <span class="text-green-500">{u.new}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="outline" onclick={handleClose} disabled={running}>Kapat</Button>
      <Button onclick={handleStart} disabled={running}>
        <RefreshCw class={`w-4 h-4 mr-2 ${running ? "animate-spin" : ""}`} />
        {running ? "Senkronize ediliyor..." : "Başlat"}
      </Button>
    </div>
  </DialogContent>
</Dialog>
