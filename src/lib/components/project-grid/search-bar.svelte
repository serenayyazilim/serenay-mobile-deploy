<script lang="ts">
  import { Search } from "@lucide/svelte";
  import { Input } from "$lib/components/ui/input";
  import { t } from "$lib/i18n/index.svelte";
  import { cn } from "$lib/utils";

  let { value = $bindable(""), class: className = "" }: { value: string; class?: string } = $props();
</script>

<div class={cn("relative max-w-md", className)}>
  <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
  <Input
    type="text"
    bind:value
    placeholder={t("common.search")}
    class="pl-9 h-9 rounded-xl bg-secondary/30 border-0 focus-visible:ring-1 focus-visible:ring-primary/50"
  />
</div>
