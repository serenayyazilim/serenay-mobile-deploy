<script lang="ts">
  import { Sun, Moon, MonitorCog, Check, Palette, Flame, CalendarClock, Settings2, MessageSquare, Info, Volume2 } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { i18n, locales, type Locale } from "$lib/i18n/index.svelte";
  import { themeState, type ThemeMode } from "$lib/stores/theme.svelte";
  import { videoState, youtubeId } from "$lib/stores/video.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import FirebaseSettings from "$lib/components/firebase-account-selector.svelte";
  import AppStoreConnectSettings from "$lib/components/appstoreconnect-settings.svelte";
  import SlackSettings from "$lib/components/slack-settings.svelte";
  import SermobilebossWorkspaceSettings from "$lib/components/sermobileboss-workspace-settings.svelte";
  import AboutSettings from "$lib/components/about-settings.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false), workspacePath = "", showWorkspaceTab = false, onRestartTour, onShowChangelog }: {
    open?: boolean;
    workspacePath?: string;
    showWorkspaceTab?: boolean;
    onRestartTour?: () => void;
    onShowChangelog?: () => void;
  } = $props();

  type Tab = "general" | "firebase" | "appstoreconnect" | "slack" | "workspace" | "about";
  let activeTab = $state<Tab>("general");

  const tabs = $derived(
    [
      { id: "general" as const, labelKey: "settings.general", icon: Palette },
      { id: "firebase" as const, labelKey: "settings.firebaseTab", icon: Flame },
      { id: "appstoreconnect" as const, labelKey: "settings.appStoreConnectTab", icon: CalendarClock },
      { id: "slack" as const, labelKey: "settings.slackTab", icon: MessageSquare },
      ...(showWorkspaceTab ? [{ id: "workspace" as const, labelKey: "workspaceSettings.title", icon: Settings2 }] : []),
      { id: "about" as const, labelKey: "settings.aboutTab", icon: Info },
    ]
  );

  $effect(() => {
    if (activeTab === "workspace" && !showWorkspaceTab) activeTab = "general";
  });

  let videoUrl = $state(videoState.url);
  $effect(() => {
    if (open) videoUrl = videoState.url;
  });
  const videoUrlInvalid = $derived(videoUrl.trim() !== "" && !youtubeId(videoUrl));
  const videoUrlDirty = $derived(videoUrl.trim() !== videoState.url);
  let videoSaved = $state(false);

  async function saveVideoUrl() {
    if (videoUrlInvalid || !videoUrlDirty) return;
    await videoState.save({ url: videoUrl.trim() });
    videoSaved = true;
    setTimeout(() => (videoSaved = false), 2000);
  }

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
              class={`pressable w-full flex items-center gap-2.5 px-3 py-2 rounded-full text-sm font-medium ${
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

              <div class="space-y-2">
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <p class="text-xs font-medium text-muted-foreground">{t("settings.videoPlayer")}</p>
                    <p class="text-xs text-muted-foreground/80 mt-0.5">{t("settings.videoPlayerHint")}</p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    aria-checked={videoState.enabled}
                    aria-label={t("settings.videoPlayer")}
                    onclick={() => videoState.save({ enabled: !videoState.enabled })}
                    class={`relative h-6 w-10 shrink-0 rounded-full transition-colors duration-300 ${videoState.enabled ? "bg-green-500" : "bg-secondary ring-1 ring-border"}`}
                  >
                    <span
                      class={`absolute top-0.5 left-0.5 size-5 rounded-full bg-white shadow-md transition-transform duration-300 ease-(--ease-spring) ${videoState.enabled ? "translate-x-4" : ""}`}
                    ></span>
                  </button>
                </div>
                <div class="flex gap-2">
                  <Input
                    bind:value={videoUrl}
                    disabled={!videoState.enabled}
                    placeholder="https://www.youtube.com/watch?v=…"
                    aria-invalid={videoUrlInvalid}
                    onkeydown={(e) => e.key === "Enter" && saveVideoUrl()}
                  />
                  <Button disabled={!videoState.enabled || videoUrlInvalid || !videoUrlDirty} onclick={saveVideoUrl}>
                    {#if videoSaved}<Check class="w-4 h-4" />{t("settings.saved")}{:else}{t("common.save")}{/if}
                  </Button>
                </div>
                {#if videoUrlInvalid}
                  <p class="text-xs text-red-600">{t("settings.videoUrlInvalid")}</p>
                {/if}
                <label class="flex items-center gap-3 pt-1 text-xs text-muted-foreground">
                  <Volume2 class="w-4 h-4 shrink-0" />
                  <span class="shrink-0">{t("settings.videoVolume")}</span>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="5"
                    value={videoState.volume}
                    disabled={!videoState.enabled}
                    oninput={(e) => (videoState.volume = +e.currentTarget.value)}
                    onchange={(e) => videoState.save({ volume: +e.currentTarget.value })}
                    class="flex-1 accent-primary disabled:opacity-50"
                  />
                  <span class="w-9 text-right tabular-nums text-foreground">%{videoState.volume}</span>
                </label>
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
          {:else if activeTab === "about"}
            <AboutSettings {onRestartTour} {onShowChangelog} />
          {/if}
        </div>
      </div>
    </div>
  </DialogContent>
</Dialog>
