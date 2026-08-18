<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { Plus, RefreshCw, FolderOpen, LogOut, Settings } from "@lucide/svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import SettingsDialog from "$lib/components/settings-dialog.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let appVersion = $state("");
  getVersion().then((v) => (appVersion = v));

  let { projectCount, supportsMultipleProjects, onCreateProject, onSyncVersions }: {
    projectCount: number;
    supportsMultipleProjects: boolean;
    onCreateProject: () => void;
    onSyncVersions: () => void;
  } = $props();

  const workspaceName = $derived(workspaceState.path?.split("/").filter(Boolean).pop() ?? "");

  let showSettings = $state(false);
</script>

<aside class="w-64 shrink-0 h-screen sticky top-0 flex flex-col bg-secondary/20 border-r border-border/50 p-4">
  <div class="px-2 pt-2 pb-6">
    <h1 class="text-lg font-semibold tracking-tight">Serenay Mobile Deploy</h1>
    <p class="text-xs text-muted-foreground mt-0.5">{t("sidebar.projectCount", { count: projectCount })}</p>
  </div>

  <nav class="flex-1 flex flex-col items-start gap-2 overflow-y-auto">
    {#if supportsMultipleProjects}
      <button
        onclick={onCreateProject}
        class="w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl font-semibold text-sm bg-primary text-primary-foreground hover:bg-primary/90 transition-all mb-1"
      >
        <Plus class="w-4 h-4" /> {t("sidebar.newProject")}
      </button>

      <button
        onclick={onSyncVersions}
        class="w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-border/50 bg-secondary/50 hover:bg-secondary transition-all"
      >
        <RefreshCw class="w-4 h-4 text-muted-foreground" />
        {t("sidebar.syncVersions")}
      </button>
    {/if}
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
