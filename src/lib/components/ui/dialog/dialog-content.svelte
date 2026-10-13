<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { X } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  let { class: className, children, ...restProps }: DialogPrimitive.ContentProps = $props();
</script>

<DialogPrimitive.Portal>
  <DialogPrimitive.Overlay
    class="fade-pop fixed inset-0 z-50 bg-black/25 backdrop-blur-[3px]"
  />
  <DialogPrimitive.Content
    class={cn(
      "glass-strong glass-pop fixed top-[50%] left-[50%] z-50 grid w-full max-w-md translate-x-[-50%] translate-y-[-50%] gap-4 rounded-[28px] p-6",
      className
    )}
    {...restProps}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class="pressable absolute top-4 right-4 grid size-7 place-items-center rounded-full bg-secondary text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus:outline-none disabled:pointer-events-none"
    >
      <X class="size-3.5" />
      <span class="sr-only">Kapat</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
