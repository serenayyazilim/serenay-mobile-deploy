<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Smartphone, Monitor, Globe, Cpu, RefreshCw } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import type { FlutterDevice } from "$lib/stores/build.svelte";

  let { open = $bindable(false), onSelect, projectName }: {
    open: boolean;
    onSelect: (device: FlutterDevice | null) => void;
    projectName: string;
  } = $props();

  let devices = $state<FlutterDevice[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function fetchDevices(refresh = false) {
    loading = true;
    error = null;
    try {
      devices = await invoke<FlutterDevice[]>("flutter_devices", { refresh });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) fetchDevices();
  });

  function platformBadge(device: FlutterDevice): string {
    const p = device.platform.toLowerCase();
    if (p.includes("android")) return "Android";
    if (p.includes("ios")) return "iOS";
    if (p.includes("macos")) return "macOS";
    if (p.includes("windows")) return "Windows";
    if (p.includes("web") || p.includes("chrome")) return "Web";
    return device.platform;
  }

  function typeBadge(device: FlutterDevice): string {
    if (device.type === "simulator" || device.platform.toLowerCase().includes("simulator")) return "Simulator";
    if (device.type === "mobile") return "Fiziksel";
    if (device.type === "desktop") return "Desktop";
    return "";
  }
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2"><Smartphone class="w-5 h-5" /> Cihaz Seçin</DialogTitle>
      <p class="text-sm text-muted-foreground">{projectName} için hedef cihazı seçin</p>
    </DialogHeader>

    <div class="space-y-3 mt-4">
      {#if loading}
        <div class="flex items-center justify-center py-8">
          <LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" />
          <span class="ml-2 text-muted-foreground">Cihazlar taranıyor...</span>
        </div>
      {:else if error}
        <div class="text-center py-8">
          <p class="text-red-500 mb-3">{error}</p>
          <Button variant="outline" size="sm" onclick={() => fetchDevices(true)}>
            <RefreshCw class="w-4 h-4 mr-2" /> Tekrar Dene
          </Button>
        </div>
      {:else if devices.length === 0}
        <div class="text-center py-8">
          <p class="text-muted-foreground mb-3">Bağlı cihaz bulunamadı</p>
          <Button variant="outline" size="sm" onclick={() => fetchDevices(true)}>
            <RefreshCw class="w-4 h-4 mr-2" /> Yenile
          </Button>
        </div>
      {:else}
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-muted-foreground">{devices.length} cihaz bulundu</span>
          <Button variant="ghost" size="sm" onclick={() => fetchDevices(true)}><RefreshCw class="w-4 h-4" /></Button>
        </div>

        <div class="space-y-2 max-h-72 overflow-y-auto">
          {#each devices as device (device.id)}
            <button
              onclick={() => { onSelect(device); open = false; }}
              class="w-full flex items-center gap-3 p-3 rounded-xl bg-secondary/50 hover:bg-secondary transition-colors text-left"
            >
              <div class="p-2 rounded-lg bg-background">
                {#if device.platform.toLowerCase().includes("android") || device.platform.toLowerCase().includes("ios")}
                  <Smartphone class="w-5 h-5" />
                {:else if device.platform.toLowerCase().includes("macos") || device.platform.toLowerCase().includes("windows") || device.platform.toLowerCase().includes("linux")}
                  <Monitor class="w-5 h-5" />
                {:else if device.platform.toLowerCase().includes("web") || device.platform.toLowerCase().includes("chrome")}
                  <Globe class="w-5 h-5" />
                {:else}
                  <Cpu class="w-5 h-5" />
                {/if}
              </div>
              <div class="flex-1 min-w-0">
                <p class="font-medium text-sm truncate">{device.name}</p>
                <p class="text-xs text-muted-foreground truncate">{device.id}</p>
              </div>
              <div class="flex flex-col items-end gap-1">
                <span class="text-[10px] px-2 py-0.5 rounded-full bg-primary/10 text-primary font-medium">{platformBadge(device)}</span>
                {#if typeBadge(device)}
                  <span class="text-[10px] px-2 py-0.5 rounded-full bg-secondary text-muted-foreground">{typeBadge(device)}</span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </DialogContent>
</Dialog>
