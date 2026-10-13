<script lang="ts" module>
  import { type VariantProps, tv } from "tailwind-variants";

  export const buttonVariants = tv({
    base: "pressable inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full text-sm font-medium disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
    variants: {
      variant: {
        default:
          "bg-primary text-primary-foreground hover:brightness-110 shadow-[inset_0_1px_0_rgb(255_255_255/0.35),0_6px_18px_-6px_var(--primary)]",
        destructive:
          "bg-destructive text-white hover:brightness-110 shadow-[inset_0_1px_0_rgb(255_255_255/0.3),0_6px_18px_-6px_var(--destructive)] focus-visible:ring-destructive/30",
        outline: "glass hover:bg-accent hover:text-accent-foreground",
        secondary: "glass text-secondary-foreground",
        ghost: "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2 has-[>svg]:px-3",
        sm: "h-8 gap-1.5 px-3.5 has-[>svg]:px-3",
        lg: "h-11 px-6 has-[>svg]:px-5",
        icon: "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  });

  export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
  export type ButtonSize = VariantProps<typeof buttonVariants>["size"];
</script>

<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: ButtonSize;
    class?: string;
  }

  let {
    class: className,
    variant = "default",
    size = "default",
    type = "button",
    children,
    ...restProps
  }: Props = $props();
</script>

<button
  data-slot="button"
  class={cn(buttonVariants({ variant, size }), className)}
  {type}
  {...restProps}
>
  {@render children?.()}
</button>
