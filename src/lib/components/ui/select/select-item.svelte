<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";
  import { Check } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    value,
    label,
    children: children_,
    ...restProps
  }: SelectPrimitive.ItemProps = $props();
</script>

<SelectPrimitive.Item
  bind:ref
  data-slot="select-item"
  {value}
  class={cn(
    "focus:bg-secondary focus:text-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex w-full cursor-pointer items-center gap-2 rounded-lg py-1.5 pr-8 pl-2 text-sm outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className
  )}
  {...restProps}
>
  {#snippet children(itemProps)}
    <span class="flex flex-1 items-center gap-2">
      {#if children_}
        {@render children_(itemProps)}
      {:else}
        {label ?? value}
      {/if}
    </span>
    {#if itemProps.selected}
      <span class="absolute right-2 flex size-3.5 items-center justify-center">
        <Check class="size-4" />
      </span>
    {/if}
  {/snippet}
</SelectPrimitive.Item>
