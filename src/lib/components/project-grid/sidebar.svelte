<script lang="ts">
  import { Plus, RefreshCw, FolderOpen, LogOut } from "@lucide/svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import AppStoreConnectSettings from "$lib/components/appstoreconnect-settings.svelte";
  import FirebaseAccountSelector from "$lib/components/firebase-account-selector.svelte";
  import SermobilebossWorkspaceSettings from "$lib/components/sermobileboss-workspace-settings.svelte";
  import LanguageSwitcher from "$lib/components/language-switcher.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { projectCount, supportsMultipleProjects, onCreateProject, onSyncVersions }: {
    projectCount: number;
    supportsMultipleProjects: boolean;
    onCreateProject: () => void;
    onSyncVersions: () => void;
  } = $props();

  const workspaceName = $derived(workspaceState.path?.split("/").filter(Boolean).pop() ?? "");
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

      <SermobilebossWorkspaceSettings workspacePath={workspaceState.path ?? ""} />

      <div class="h-px bg-border/50 w-full my-2"></div>
    {/if}

    <FirebaseAccountSelector />
    <AppStoreConnectSettings workspacePath={workspaceState.path ?? ""} />
  </nav>

  <div class="pt-3 border-t border-border/50 space-y-1">
    <div class="flex items-center gap-2 px-3 py-2 rounded-xl text-sm text-muted-foreground">
      <FolderOpen class="w-4 h-4 shrink-0" />
      <span class="truncate">{workspaceName}</span>
    </div>
    <LanguageSwitcher />
    <button
      onclick={() => workspaceState.clear()}
      class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
    >
      <LogOut class="w-4 h-4" />
      {t("sidebar.switchWorkspace")}
    </button>
  </div>
</aside>
