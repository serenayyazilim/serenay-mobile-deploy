<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let { class: className, children, ...restProps }: DialogPrimitive.DescriptionProps = $props();
</script>

<DialogPrimitive.Description class={cn("text-muted-foreground text-sm", className)} {...restProps}>
  {@render children?.()}
</DialogPrimitive.Description>
