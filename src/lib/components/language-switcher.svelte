<script lang="ts">
  import { Languages } from "@lucide/svelte";
  import { i18n, locales, type Locale } from "$lib/i18n/index.svelte";

  function toggle() {
    const next: Locale = i18n.locale === "en" ? "tr" : "en";
    i18n.setLocale(next);
  }
</script>

<button
  onclick={toggle}
  class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
  title={locales[i18n.locale]}
>
  <Languages class="w-4 h-4 shrink-0" />
  <span class="truncate">{locales[i18n.locale]}</span>
</button>
