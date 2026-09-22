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
      "bg-popover text-popover-foreground relative z-50 min-w-[8rem] overflow-hidden rounded-xl border border-border/50 shadow-md p-1",
      "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
      className
    )}
    {...restProps}
  >
    {@render children?.()}
  </SelectPrimitive.Content>
</SelectPrimitive.Portal>
