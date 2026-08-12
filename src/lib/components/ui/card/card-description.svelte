<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  let { class: className, children, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div data-slot="card-description" class={cn("text-muted-foreground text-sm", className)} {...restProps}>
  {@render children?.()}
</div>
