<script lang="ts">
  import { Sun, Moon, MonitorCog, Check } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { i18n, locales, type Locale } from "$lib/i18n/index.svelte";
  import { themeState, type ThemeMode } from "$lib/stores/theme.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  const themeOptions: { value: ThemeMode; labelKey: string; icon: typeof Sun }[] = [
    { value: "light", labelKey: "settings.light", icon: Sun },
    { value: "dark", labelKey: "settings.dark", icon: Moon },
    { value: "system", labelKey: "settings.system", icon: MonitorCog },
  ];
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>{t("common.settings")}</DialogTitle>
      <DialogDescription>{t("settings.description")}</DialogDescription>
    </DialogHeader>

    <div class="space-y-5 py-2">
      <div class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground">{t("settings.appearance")}</p>
        <div class="grid grid-cols-3 gap-2">
          {#each themeOptions as option (option.value)}
            {@const Icon = option.icon}
            <button
              onclick={() => themeState.setMode(option.value)}
              class={`flex flex-col items-center gap-1.5 px-2 py-3 rounded-xl text-xs font-medium ring-1 transition-all ${
                themeState.mode === option.value
                  ? "bg-primary text-primary-foreground ring-primary"
                  : "bg-secondary/50 text-foreground ring-border/50 hover:bg-secondary"
              }`}
            >
              <Icon class="w-4 h-4" />
              {t(option.labelKey)}
            </button>
          {/each}
        </div>
      </div>

      <div class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground">{t("settings.language")}</p>
        <div class="grid grid-cols-2 gap-2">
          {#each Object.entries(locales) as [code, label] (code)}
            <button
              onclick={() => i18n.setLocale(code as Locale)}
              class={`flex items-center justify-between gap-2 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 transition-all ${
                i18n.locale === code
                  ? "bg-primary text-primary-foreground ring-primary"
                  : "bg-secondary/50 text-foreground ring-border/50 hover:bg-secondary"
              }`}
            >
              {label}
              {#if i18n.locale === code}<Check class="w-4 h-4" />{/if}
            </button>
          {/each}
        </div>
      </div>
    </div>
  </DialogContent>
</Dialog>
