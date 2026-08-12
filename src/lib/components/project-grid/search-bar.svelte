<script lang="ts">
  import { Search } from "@lucide/svelte";
  import { Input } from "$lib/components/ui/input";
  import { t } from "$lib/i18n/index.svelte";

  let { value = $bindable("") }: { value: string } = $props();
</script>

<div class="relative max-w-md">
  <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
  <Input
    type="text"
    bind:value
    placeholder={t("common.search")}
    class="pl-11 h-12 rounded-2xl bg-secondary/30 border-0 focus-visible:ring-1 focus-visible:ring-primary/50"
  />
</div>
