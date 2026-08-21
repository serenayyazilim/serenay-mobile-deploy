<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { ImagePlus } from "@lucide/svelte";

  let {
    value = $bindable(null),
    label,
    selectedLabel,
    extensions = ["png", "jpg", "jpeg"],
  }: {
    value: string | null;
    label: string;
    selectedLabel: string;
    extensions?: string[];
  } = $props();

  let el: HTMLButtonElement | undefined = $state();
  let dragOver = $state(false);

  const previewSrc = $derived(value ? convertFileSrc(value) : null);

  function matchesExtension(path: string) {
    const ext = path.split(".").pop()?.toLowerCase();
    return ext ? extensions.includes(ext) : false;
  }

  async function pick() {
    const path = await open({ filters: [{ name: "File", extensions }] });
    if (path && !Array.isArray(path)) value = path;
  }

  function isInBounds(x: number, y: number) {
    if (!el) return false;
    const scale = window.devicePixelRatio || 1;
    const rect = el.getBoundingClientRect();
    const logicalX = x / scale;
    const logicalY = y / scale;
    return logicalX >= rect.left && logicalX <= rect.right && logicalY >= rect.top && logicalY <= rect.bottom;
  }

  onMount(() => {
    const unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dragOver = isInBounds(event.payload.position.x, event.payload.position.y);
      } else if (event.payload.type === "drop") {
        const inBounds = isInBounds(event.payload.position.x, event.payload.position.y);
        dragOver = false;
        if (inBounds) {
          const dropped = event.payload.paths.find(matchesExtension) ?? event.payload.paths[0];
          if (dropped) value = dropped;
        }
      } else {
        dragOver = false;
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });
</script>

<button
  bind:this={el}
  type="button"
  onclick={pick}
  class="flex flex-col items-center gap-1 p-3 rounded-xl bg-secondary/30 ring-1 text-xs text-muted-foreground w-full transition-colors {dragOver
    ? 'ring-2 ring-primary bg-primary/10'
    : 'ring-border/30 hover:ring-border/60'}"
>
  {#if value && previewSrc}
    <img src={previewSrc} alt={label} class="w-12 h-12 object-cover rounded-lg ring-1 ring-border/50" />
  {:else}
    <ImagePlus class="w-5 h-5" />
  {/if}
  <span class="truncate max-w-full">{value ? selectedLabel : label}</span>
</button>
