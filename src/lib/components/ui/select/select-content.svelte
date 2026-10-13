<script lang="ts">
  import { Select as SelectPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let {
    ref = $bindable(null),
    class: className,
    sideOffset = 4,
    portalProps,
    children,
    ...restProps
  }: SelectPrimitive.ContentProps & {
    portalProps?: SelectPrimitive.PortalProps;
  } = $props();
</script>

<SelectPrimitive.Portal {...portalProps}>
  <SelectPrimitive.Content
    bind:ref
    data-slot="select-content"
    {sideOffset}
    class={cn(
      "glass-strong glass-pop origin-(--bits-select-content-transform-origin) text-popover-foreground relative z-50 min-w-[8rem] overflow-hidden rounded-2xl p-1.5",
      className
    )}
    {...restProps}
  >
    {@render children?.()}
  </SelectPrimitive.Content>
</SelectPrimitive.Portal>
