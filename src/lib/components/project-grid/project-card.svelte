<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Rocket, LoaderCircle, CircleCheck, CircleX, Play, Settings, Clock, ScrollText } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { formatDuration } from "$lib/deploy-utils";
  import { deployState } from "$lib/stores/deploy.svelte";
  import { buildState } from "$lib/stores/build.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { project, workspacePath, version, onSettings }: {
    project: WorkspaceProject;
    workspacePath: string;
    version: string | undefined;
    onSettings: (project: WorkspaceProject) => void;
  } = $props();

  const initials = $derived(project.appName.slice(0, 2).toUpperCase());
  let iconDataUrl = $state<string | null>(null);

  $effect(() => {
    iconDataUrl = null;
    invoke<string | null>("project_icon", { workspace: workspacePath, projectId: project.id })
      .then((url) => (iconDataUrl = url))
      .catch(() => (iconDataUrl = null));
  });

  const isDeploying = $derived(deployState.deployingProjectId === project.id);
  const isDeploySuccess = $derived(isDeploying && deployState.deployStatus === "success");
  const isDeployError = $derived(isDeploying && deployState.deployStatus === "error");
  const isDeployProcessing = $derived(isDeploying && !isDeploySuccess && !isDeployError);

  const isBuilding = $derived(buildState.buildingProjectId === project.id);
  const isBuildSuccess = $derived(isBuilding && buildState.buildStatus === "success");
  const isBuildError = $derived(isBuilding && buildState.buildStatus === "error");
  const isBuildRunning = $derived(isBuilding && buildState.buildStatus === "running");
</script>

<div
  class={`group relative flex flex-col items-center p-5 rounded-3xl transition-all duration-300 ${
    isDeploying ? "bg-primary/5 ring-2 ring-primary/20" : isBuilding ? "bg-green-500/5 ring-2 ring-green-500/20" : "bg-card dark:bg-transparent hover:bg-secondary/50 hover:shadow-lg hover:shadow-black/5 hover:-translate-y-1"
  }`}
>
  {#if (buildState.buildingProjectId === null || isBuilding) && !isDeploying}
    <button
      onclick={() => buildState.handleBuildClick(project)}
      disabled={buildState.buildingProjectId !== null || deployState.deployingProjectId !== null}
      title={t("projectCard.buildAndRun")}
      class={`absolute top-3 left-3 p-2 rounded-full transition-all duration-200 disabled:opacity-50 ${
        isBuildRunning || isBuildSuccess ? "bg-green-500 text-white" : isBuildError ? "bg-red-500 text-white" : "opacity-0 group-hover:opacity-100 bg-green-500 text-white hover:scale-110"
      }`}
    >
      {#if isBuildRunning}<LoaderCircle class="w-4 h-4 animate-spin" />
      {:else if isBuildSuccess}<CircleCheck class="w-4 h-4" />
      {:else if isBuildError}<CircleX class="w-4 h-4" />
      {:else}<Play class="w-4 h-4" />{/if}
    </button>
  {/if}

  {#if (deployState.deployingProjectId === null || isDeploying) && !isBuilding}
    <button
      onclick={() => deployState.handleDeploy(project)}
      disabled={deployState.deployingProjectId !== null || buildState.buildingProjectId !== null}
      title={t("projectCard.deploy")}
      class={`absolute top-3 right-3 p-2 rounded-full transition-all duration-200 disabled:opacity-50 ${
        isDeployProcessing ? "bg-primary text-primary-foreground" : isDeploySuccess ? "bg-green-500 text-white" : isDeployError ? "bg-red-500 text-white" : "opacity-0 group-hover:opacity-100 bg-primary text-primary-foreground hover:scale-110"
      }`}
    >
      {#if isDeployProcessing}<LoaderCircle class="w-4 h-4 animate-spin" />
      {:else if isDeploySuccess}<CircleCheck class="w-4 h-4" />
      {:else if isDeployError}<CircleX class="w-4 h-4" />
      {:else}<Rocket class="w-4 h-4" />{/if}
    </button>
  {/if}

  {#if !isDeploying && !isBuilding}
    <button
      onclick={(e) => { e.stopPropagation(); onSettings(project); }}
      class="absolute bottom-3 right-3 p-2 rounded-full opacity-0 group-hover:opacity-100 bg-secondary hover:bg-secondary/80 transition-all duration-200"
      title={t("common.settings")}
    >
      <Settings class="w-4 h-4 text-muted-foreground" />
    </button>
  {/if}

  <div class="w-20 h-20 rounded-[22px] bg-gradient-to-br from-secondary to-secondary/50 shadow-lg shadow-black/10 flex items-center justify-center overflow-hidden mb-4">
    {#if iconDataUrl}
      <img src={iconDataUrl} alt={project.appName} class="w-full h-full object-cover" />
    {:else}
      <span class="text-2xl font-semibold text-muted-foreground">{initials}</span>
    {/if}
  </div>

  <h3 class="font-medium text-sm text-center leading-tight line-clamp-2">{project.appName}</h3>

  {#if version && !isDeploying}
    <p class="text-[10px] text-muted-foreground mt-1">v{version.split("+")[0]}</p>
  {/if}

  {#if isDeploying}
    <div class="w-full mt-3 space-y-2">
      {#if isDeployProcessing}
        <div class="w-full h-1.5 bg-secondary rounded-full overflow-hidden">
          <div class="h-full bg-primary rounded-full transition-all duration-500 ease-out" style={`width: ${deployState.deployProgress}%`}></div>
        </div>
      {/if}

      {#if deployState.deployElapsed > 0}
        <div class="flex items-center justify-center gap-1 text-[10px] text-muted-foreground">
          <Clock class="w-3 h-3" />
          <span>{formatDuration(deployState.deployElapsed)}</span>
        </div>
      {/if}

      <p class={`text-xs text-center truncate max-w-full ${isDeploySuccess ? "text-green-600" : isDeployError ? "text-red-600" : "text-muted-foreground"}`}>
        {deployState.deployMessage}
        {#if isDeployProcessing && deployState.deployProgress > 0}
          <span class="ml-1 text-primary font-medium">%{deployState.deployProgress}</span>
        {/if}
      </p>

      {#if isDeployProcessing}
        <p class="text-[10px] text-center text-amber-600 dark:text-amber-500">{t("projectCard.deployStopsOnClose")}</p>
      {/if}
    </div>
  {/if}

  {#if isDeploySuccess || isDeployError}
    <Button size="sm" variant="ghost" class="mt-3 h-7 text-xs rounded-full" onclick={() => deployState.resetDeploy()}>{t("common.close")}</Button>
  {/if}

  {#if isBuilding}
    <div class="w-full mt-3 space-y-2">
      {#if isBuildRunning && buildState.buildLogs.length > 0}
        <div class="w-full bg-secondary/50 rounded-lg p-2 overflow-hidden">
          {#each buildState.buildLogs.slice(-2) as log, i (i)}
            <p class="text-[10px] text-muted-foreground truncate">{log}</p>
          {/each}
          <button
            onclick={(e) => { e.stopPropagation(); buildState.openLogsDialog(); }}
            class="mt-1 flex w-full items-center justify-center gap-1.5 rounded-md bg-background/80 hover:bg-background text-muted-foreground hover:text-foreground py-1.5 transition-colors"
          >
            <ScrollText class="w-3.5 h-3.5" />
            <span class="text-[10px] font-medium">{t("projectCard.viewAllLogs")}</span>
          </button>
        </div>
      {/if}

      <p class={`text-xs text-center truncate max-w-full ${isBuildSuccess ? "text-green-600" : isBuildError ? "text-red-600" : "text-muted-foreground"}`}>
        {#if isBuildRunning}{t("projectCard.buildRunning")}{/if}
        {#if isBuildSuccess}{t("projectCard.appRunning")}{/if}
        {#if isBuildError}{t("projectCard.buildError")}{/if}
      </p>

      {#if isBuildRunning}
        <p class="text-[10px] text-center text-amber-600 dark:text-amber-500">{t("projectCard.buildStopsOnClose")}</p>
      {/if}

      {#if isBuildSuccess || isBuildError}
        <Button size="sm" variant="ghost" class="w-full h-7 text-xs rounded-full" onclick={() => buildState.resetBuild()}>{t("common.close")}</Button>
      {/if}
    </div>
  {/if}
</div>
