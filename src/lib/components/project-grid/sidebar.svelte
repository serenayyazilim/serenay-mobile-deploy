<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { FolderOpen, LogOut, Settings, CalendarClock, Home } from "@lucide/svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
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
    `w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 transition-all ${
      active ? "bg-primary/10 ring-primary/30 text-primary" : "ring-border/50 bg-secondary/50 hover:bg-secondary text-foreground"
    }`;

  const workspaceName = $derived(workspaceState.path?.split("/").filter(Boolean).pop() ?? "");

  let showSettings = $state(false);
</script>

<aside class="w-64 shrink-0 h-screen sticky top-0 flex flex-col bg-secondary/20 border-r border-border/50 p-4">
  <div class="px-2 pt-2 pb-6">
    <h1 class="text-lg font-semibold tracking-tight">Serenay Mobile Deploy</h1>
  </div>

  <nav class="flex-1 flex flex-col items-start gap-2 overflow-y-auto overflow-x-hidden px-0.5 pt-0.5">
    <button onclick={onHome} class={navButtonClass(currentView === "projects")}>
      <Home class="w-4 h-4" />
      {t("sidebar.home")}
    </button>

    <button onclick={onInAppEvents} class={navButtonClass(currentView === "events")}>
      <CalendarClock class="w-4 h-4" />
      {t("sidebar.inAppEvents")}
    </button>
  </nav>

  <div class="pt-3 border-t border-border/50 space-y-1">
    <div class="flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-muted-foreground">
      <FolderOpen class="w-4 h-4 shrink-0" />
      <span class="truncate">{workspaceName}</span>
    </div>
    <button
      onclick={() => (showSettings = true)}
      class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
    >
      <Settings class="w-4 h-4" />
      {t("common.settings")}
    </button>
    <button
      onclick={() => workspaceState.clear()}
      class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
    >
      <LogOut class="w-4 h-4" />
      {t("sidebar.switchWorkspace")}
    </button>
    {#if appVersion}
      <p class="px-3 pt-1 text-[10px] text-muted-foreground/60">v{appVersion}</p>
    {/if}
  </div>
</aside>

<SettingsDialog bind:open={showSettings} workspacePath={workspaceState.path ?? ""} showWorkspaceTab={supportsMultipleProjects} />
