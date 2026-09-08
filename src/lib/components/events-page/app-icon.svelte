<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { workspacePath, projectId, bundleId, name }: {
    workspacePath: string;
    projectId: string | undefined;
    bundleId: string;
    name: string;
  } = $props();

  const initials = $derived(name.slice(0, 2).toUpperCase());
  let iconDataUrl = $state<string | null>(null);

  // Apple doesn't expose app icons through the App Store Connect API, so we source
  // them elsewhere: prefer the icon from the matching local project (if any), and
  // fall back to the public App Store lookup for apps that are already published there.
  async function fetchStoreIcon(id: string): Promise<string | null> {
    for (const country of ["us", "tr"]) {
      try {
        const res = await fetch(`https://itunes.apple.com/lookup?bundleId=${encodeURIComponent(id)}&country=${country}`);
        if (!res.ok) continue;
        const data = await res.json();
        const artwork = data?.results?.[0]?.artworkUrl512 || data?.results?.[0]?.artworkUrl100;
        if (artwork) return artwork;
      } catch {
        // try next country / give up
      }
    }
    return null;
  }

  $effect(() => {
    iconDataUrl = null;
    const id = projectId;
    const bid = bundleId;

    (async () => {
      if (id) {
        try {
          const url = await invoke<string | null>("project_icon", { workspace: workspacePath, projectId: id });
          if (url) {
            iconDataUrl = url;
            return;
          }
        } catch {
          // fall through to the App Store lookup
        }
      }
      if (bid) iconDataUrl = await fetchStoreIcon(bid);
    })();
  });
</script>

<div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-secondary to-secondary/50 shadow-md shadow-black/10 flex items-center justify-center overflow-hidden shrink-0">
  {#if iconDataUrl}
    <img src={iconDataUrl} alt={name} class="w-full h-full object-cover" />
  {:else}
    <span class="text-lg font-semibold text-muted-foreground">{initials}</span>
  {/if}
</div>
