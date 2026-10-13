<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  let { class: className, value = $bindable(), ...restProps }: HTMLInputAttributes = $props();
</script>

<input
  data-slot="input"
  class={cn(
    "file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground border-input h-9 w-full min-w-0 rounded-xl border bg-background/40 px-3 py-1 text-base shadow-[inset_0_1px_2px_rgb(0_0_0/0.06)] transition-[color,box-shadow,background-color] focus-visible:bg-background/70 outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
    "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
    "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
    className
  )}
  bind:value
  {...restProps}
/>
