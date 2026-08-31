<script lang="ts">
  import { onMount } from "svelte";
  import { LoaderCircle } from "@lucide/svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import { projectsState, type WorkspaceProject } from "$lib/stores/projects.svelte";
  import { deployState } from "$lib/stores/deploy.svelte";
  import { buildState } from "$lib/stores/build.svelte";
  import SearchBar from "./search-bar.svelte";
  import ProjectCard from "./project-card.svelte";
  import Sidebar from "./sidebar.svelte";
  import ProjectSettingsDialog from "$lib/components/project-settings-dialog/index.svelte";
  import CreateProjectDialog from "$lib/components/create-project-dialog";
  import DeviceSelectorDialog from "$lib/components/device-selector-dialog.svelte";
  import WhatsNewDialog from "./dialogs/whats-new-dialog.svelte";
  import TwoFactorDialog from "./dialogs/two-factor-dialog.svelte";
  import ErrorDialog from "./dialogs/error-dialog.svelte";
  import BuildLogsDialog from "./dialogs/build-logs-dialog.svelte";
  import SyncVersionsDialog from "./dialogs/sync-versions-dialog.svelte";
  import { t } from "$lib/i18n/index.svelte";

  const supportsMultipleProjects = $derived(workspaceState.mode === "sermobileboss");

  let settingsDialogOpen = $state(false);
  let settingsProject = $state<WorkspaceProject | null>(null);
  let createDialogOpen = $state(false);
  let syncDialogOpen = $state(false);

  function openSettings(project: WorkspaceProject) {
    settingsProject = project;
    settingsDialogOpen = true;
  }

  onMount(() => {
    if (workspaceState.path) projectsState.load(workspaceState.path);
  });
</script>

{#if projectsState.loading}
  <div class="flex items-center justify-center h-96">
    <LoaderCircle class="w-8 h-8 animate-spin text-muted-foreground" />
  </div>
{:else}
  <div class="flex">
    <Sidebar
      projectCount={projectsState.projects.length}
      {supportsMultipleProjects}
      onCreateProject={() => (createDialogOpen = true)}
      onSyncVersions={() => (syncDialogOpen = true)}
    />

    <div class="flex-1 min-w-0 space-y-6 p-8">
      <SearchBar bind:value={projectsState.searchQuery} />

      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-5">
        {#each projectsState.filtered as project (project.id)}
          <ProjectCard
            {project}
            workspacePath={workspaceState.path ?? ""}
            version={projectsState.versions[project.id]}
            onSettings={openSettings}
          />
        {/each}
      </div>

      {#if projectsState.filtered.length === 0}
        <div class="text-center py-20 text-muted-foreground">
          {projectsState.searchQuery ? t("projectGrid.noResults") : t("projectGrid.noProjects")}
        </div>
      {/if}
    </div>
  </div>

  <ProjectSettingsDialog
    bind:open={settingsDialogOpen}
    project={settingsProject}
    workspacePath={workspaceState.path ?? ""}
    workspaceMode={workspaceState.mode}
    onProjectRenamed={() => workspaceState.path && projectsState.fetchProjects(workspaceState.path)}
    onVersionSaved={() => workspaceState.path && projectsState.fetchVersions(workspaceState.path)}
  />

  <CreateProjectDialog
    bind:open={createDialogOpen}
    workspacePath={workspaceState.path ?? ""}
    onProjectCreated={() => workspaceState.path && projectsState.load(workspaceState.path)}
  />

  <SyncVersionsDialog bind:open={syncDialogOpen} workspacePath={workspaceState.path ?? ""} />

  <DeviceSelectorDialog
    bind:open={buildState.deviceDialogOpen}
    onSelect={(device) => workspaceState.path && buildState.handleDeviceSelect(workspaceState.path, device)}
    projectName={buildState.selectedProject?.appName ?? ""}
  />

  <WhatsNewDialog
    bind:open={deployState.whatsNewDialogOpen}
    project={deployState.pendingProject}
    workspacePath={workspaceState.path ?? ""}
    onConfirm={(whatsNew, platform, bumpVersion) =>
      workspaceState.path &&
      deployState.confirmDeploy(
        workspaceState.path,
        projectsState.versions,
        () => projectsState.fetchVersions(workspaceState.path!),
        whatsNew,
        platform,
        bumpVersion
      )}
    onCancel={() => deployState.cancelDeploy()}
  />

  <TwoFactorDialog
    bind:open={deployState.twoFactorOpen}
    prompt={deployState.twoFactorPrompt}
    onSubmit={(code) => deployState.submitTwoFactor(code)}
    onCancel={() => deployState.cancelTwoFactor()}
  />

  <ErrorDialog bind:open={deployState.errorDialogOpen} title={deployState.errorTitle} logs={deployState.errorLogs} onClose={() => deployState.resetDeploy()} />
  <ErrorDialog bind:open={buildState.errorDialogOpen} title={buildState.errorTitle} logs={buildState.errorLogs} onClose={() => buildState.resetBuild()} />
  <BuildLogsDialog
    bind:open={buildState.logsDialogOpen}
    title={t("projectCard.buildLogsTitle", { name: buildState.selectedProject?.appName ?? "" })}
    logs={buildState.buildLogs}
    status={buildState.buildStatus}
  />
{/if}
