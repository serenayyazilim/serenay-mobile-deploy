<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  let { class: className, children, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div
  data-slot="card"
  class={cn("glass text-card-foreground flex flex-col gap-6 rounded-[28px] py-6", className)}
  {...restProps}
>
  {@render children?.()}
</div>
