<script lang="ts">
  import { changelogState } from "$lib/stores/changelog.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { fly } from "svelte/transition";
  import { FolderOpen, LogOut, Settings, CalendarClock, Home } from "@lucide/svelte";
  import FlutterIcon from "$lib/components/icons/flutter-icon.svelte";
  import GooglePlayIcon from "$lib/components/icons/google-play-icon.svelte";
  import AppStoreIcon from "$lib/components/icons/app-store-icon.svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import { onboardingState } from "$lib/stores/onboarding.svelte";
  import SettingsDialog from "$lib/components/settings-dialog.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let appVersion = $state("");
  getVersion().then((v) => (appVersion = v));

  let { supportsMultipleProjects, currentView, onHome, onInAppEvents }: {
    supportsMultipleProjects: boolean;
    currentView: "projects" | "events";
    onHome: () => void;
    onInAppEvents: () => void;
  } = $props();

  const navButtonClass = (active: boolean) =>
    `pressable w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-full text-sm font-medium ${
      active ? "bg-primary/12 text-primary shadow-[inset_0_1px_0_var(--glass-highlight)]" : "text-foreground hover:bg-secondary"
    }`;

  const workspaceName = $derived(workspaceState.path?.split("/").filter(Boolean).pop() ?? "");

  let showSettings = $state(false);

  const promoLinkClass =
    "pressable w-full flex items-center gap-2.5 px-3 py-2 rounded-full text-sm font-medium text-white shadow-[inset_0_1px_0_rgb(255_255_255/0.35)] hover:brightness-110";

  const promoItems = $derived([
    {
      icon: FlutterIcon,
      label: t("sidebar.pubdevPackage"),
      url: "https://pub.dev/packages/serenay_ecommerce_widgets",
      gradient: "bg-gradient-to-r from-violet-500 to-blue-500",
    },
    {
      icon: GooglePlayIcon,
      label: t("sidebar.playStoreApps"),
      url: "https://play.google.com/store/apps/dev?id=6871065193664876210&hl=tr",
      gradient: "bg-gradient-to-r from-emerald-500 to-teal-500",
    },
    {
      icon: AppStoreIcon,
      label: t("sidebar.appStoreApps"),
      url: "https://apps.apple.com/tr/developer/serenay-yaz%C4%B1l%C4%B1m/id1682847846",
      gradient: "bg-gradient-to-r from-slate-700 to-slate-900",
    },
  ]);

  let promoIndex = $state(0);
  $effect(() => {
    const id = setInterval(() => {
      promoIndex = (promoIndex + 1) % promoItems.length;
    }, 4000);
    return () => clearInterval(id);
  });
</script>

<!-- Floating glass sidebar; its top holds the window's traffic lights and drags the window. -->
<aside class="glass animate-glass-in w-64 shrink-0 h-[calc(100vh-1rem)] sticky top-2 m-2 mr-0 flex flex-col rounded-[26px] p-4 pt-12">
  <div data-tauri-drag-region class="px-2 pt-2 pb-6">
    <h1 class="text-lg font-semibold tracking-tight">Serenay Mobile Deploy</h1>
  </div>

  <nav class="flex-1 flex flex-col items-start gap-2 overflow-y-auto overflow-x-hidden px-0.5 pt-0.5">
    <button data-tour="tour-home" onclick={onHome} class={navButtonClass(currentView === "projects")}>
      <Home class="w-4 h-4" />
      {t("sidebar.home")}
    </button>

    <button data-tour="tour-events" onclick={onInAppEvents} class={navButtonClass(currentView === "events")}>
      <CalendarClock class="w-4 h-4" />
      {t("sidebar.inAppEvents")}
    </button>
  </nav>

  <div class="border-border/50 space-y-2 pb-4">
    <div class="relative h-9 overflow-hidden rounded-full">
      {#key promoIndex}
        {@const Icon = promoItems[promoIndex].icon}
        <button
          class="{promoLinkClass} {promoItems[promoIndex].gradient} absolute inset-0"
          onclick={() => openUrl(promoItems[promoIndex].url)}
          in:fly={{ x: 24, duration: 300 }}
          out:fly={{ x: -24, duration: 300 }}
        >
          <Icon class="w-4 h-4 shrink-0" />
          <span class="truncate min-w-0 flex-1 text-left">{promoItems[promoIndex].label}</span>
        </button>
      {/key}
    </div>
  </div>

  <div class="pt-3 border-t border-border/50 space-y-1">
    <div class="flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-muted-foreground">
      <FolderOpen class="w-4 h-4 shrink-0" />
      <span class="truncate">{workspaceName}</span>
    </div>
    <button
      data-tour="tour-settings"
      onclick={() => (showSettings = true)}
      class="pressable w-full flex items-center gap-2.5 px-3 py-2 rounded-full text-sm text-muted-foreground hover:bg-secondary hover:text-foreground"
    >
      <Settings class="w-4 h-4" />
      {t("common.settings")}
    </button>
    <button
      data-tour="tour-switch-workspace"
      onclick={() => workspaceState.clear()}
      class="pressable w-full flex items-center gap-2.5 px-3 py-2 rounded-full text-sm text-muted-foreground hover:bg-secondary hover:text-foreground"
    >
      <LogOut class="w-4 h-4" />
      {t("sidebar.switchWorkspace")}
    </button>
    {#if appVersion}
      <p class="px-3 pt-1 text-[10px] text-muted-foreground/60">v{appVersion}</p>
    {/if}
  </div>
</aside>

<SettingsDialog
  bind:open={showSettings}
  workspacePath={workspaceState.path ?? ""}
  showWorkspaceTab={supportsMultipleProjects}
  onRestartTour={() => {
    showSettings = false;
    onboardingState.restart();
  }}
  onShowChangelog={() => {
    showSettings = false;
    changelogState.showCurrent();
  }}
/>
