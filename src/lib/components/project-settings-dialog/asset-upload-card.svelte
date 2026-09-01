<script lang="ts">
  import { LoaderCircle, Upload, ImageIcon } from "@lucide/svelte";

  let { label, description, icon: Icon = ImageIcon, preview, uploading, pending = false, onUpload, accept = "image/png" }: {
    label: string;
    description: string;
    icon?: typeof ImageIcon;
    preview?: string | null;
    uploading: boolean;
    pending?: boolean;
    onUpload: (file: File) => void;
    accept?: string;
  } = $props();

  let inputEl = $state<HTMLInputElement | null>(null);

  function handleChange(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (file) onUpload(file);
    (e.currentTarget as HTMLInputElement).value = "";
  }
</script>

<div
  role="button"
  tabindex="0"
  onclick={() => !uploading && inputEl?.click()}
  onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); !uploading && inputEl?.click(); } }}
  class={`flex items-center gap-3 p-3 rounded-xl bg-secondary/30 ring-1 cursor-pointer hover:bg-secondary/50 transition-colors group ${pending ? "ring-2 ring-amber-500/50" : "ring-border/30"}`}
>
  <input bind:this={inputEl} type="file" {accept} class="hidden" onchange={handleChange} />
  <div class="w-12 h-12 rounded-xl overflow-hidden bg-background/70 ring-1 ring-border/30 flex items-center justify-center shrink-0">
    {#if preview}
      <img src={preview} alt={label} class="w-full h-full object-cover" />
    {:else}
      <Icon class="w-5 h-5 text-muted-foreground" />
    {/if}
  </div>
  <div class="flex-1 min-w-0">
    <p class="text-sm font-medium">{label}</p>
    <p class="text-xs text-muted-foreground truncate">{description}</p>
  </div>
  {#if uploading}
    <LoaderCircle class="w-4 h-4 animate-spin text-muted-foreground shrink-0" />
  {:else}
    <Upload class="w-4 h-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity shrink-0" />
  {/if}
</div>
