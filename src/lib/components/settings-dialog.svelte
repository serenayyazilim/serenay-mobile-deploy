<script lang="ts">
  import { Sun, Moon, MonitorCog, Check, Palette, Flame, CalendarClock, Settings2, MessageSquare } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { i18n, locales, type Locale } from "$lib/i18n/index.svelte";
  import { themeState, type ThemeMode } from "$lib/stores/theme.svelte";
  import FirebaseSettings from "$lib/components/firebase-account-selector.svelte";
  import AppStoreConnectSettings from "$lib/components/appstoreconnect-settings.svelte";
  import SlackSettings from "$lib/components/slack-settings.svelte";
  import SermobilebossWorkspaceSettings from "$lib/components/sermobileboss-workspace-settings.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false), workspacePath = "", showWorkspaceTab = false }: {
    open?: boolean;
    workspacePath?: string;
    showWorkspaceTab?: boolean;
  } = $props();

  type Tab = "general" | "firebase" | "appstoreconnect" | "slack" | "workspace";
  let activeTab = $state<Tab>("general");

  const tabs = $derived(
    [
      { id: "general" as const, labelKey: "settings.general", icon: Palette },
      { id: "firebase" as const, labelKey: "settings.firebaseTab", icon: Flame },
      { id: "appstoreconnect" as const, labelKey: "settings.appStoreConnectTab", icon: CalendarClock },
      { id: "slack" as const, labelKey: "settings.slackTab", icon: MessageSquare },
      ...(showWorkspaceTab ? [{ id: "workspace" as const, labelKey: "workspaceSettings.title", icon: Settings2 }] : []),
    ]
  );

  $effect(() => {
    if (activeTab === "workspace" && !showWorkspaceTab) activeTab = "general";
  });

  const themeOptions: { value: ThemeMode; labelKey: string; icon: typeof Sun }[] = [
    { value: "light", labelKey: "settings.light", icon: Sun },
    { value: "dark", labelKey: "settings.dark", icon: Moon },
    { value: "system", labelKey: "settings.system", icon: MonitorCog },
  ];
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-3xl p-0 gap-0 overflow-hidden">
    <div class="flex flex-col h-[560px] max-h-[80vh]">
      <DialogHeader class="px-6 pt-6 pb-4 border-b border-border/50">
        <DialogTitle>{t("common.settings")}</DialogTitle>
        <DialogDescription>{t("settings.description")}</DialogDescription>
      </DialogHeader>

      <div class="flex flex-1 min-h-0">
        <nav class="w-56 shrink-0 border-r border-border/50 bg-secondary/20 p-3 space-y-1 overflow-y-auto">
          {#each tabs as tab (tab.id)}
            {@const Icon = tab.icon}
            <button
              onclick={() => (activeTab = tab.id)}
              class={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                activeTab === tab.id ? "bg-primary text-primary-foreground" : "text-foreground hover:bg-secondary"
              }`}
            >
              <Icon class="w-4 h-4 shrink-0" />
              <span class="truncate">{t(tab.labelKey)}</span>
            </button>
          {/each}
        </nav>

        <div class="flex-1 min-w-0 overflow-y-auto p-6">
          {#if activeTab === "general"}
            <div class="space-y-5">
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
          {:else if activeTab === "firebase"}
            <FirebaseSettings />
          {:else if activeTab === "appstoreconnect"}
            <AppStoreConnectSettings {workspacePath} />
          {:else if activeTab === "slack"}
            <SlackSettings {workspacePath} />
          {:else if activeTab === "workspace"}
            <SermobilebossWorkspaceSettings {workspacePath} />
          {/if}
        </div>
      </div>
    </div>
  </DialogContent>
</Dialog>
