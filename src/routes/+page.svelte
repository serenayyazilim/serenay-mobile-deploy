<script lang="ts">
  import { onMount } from "svelte";
  import WorkspaceSelector from "$lib/components/workspace-selector.svelte";
  import ProjectGrid from "$lib/components/project-grid/project-grid.svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import { i18n, t } from "$lib/i18n/index.svelte";

  onMount(() => {
    workspaceState.init();
    i18n.init();
  });
</script>

{#if !workspaceState.loaded}
  <div class="min-h-screen flex items-center justify-center text-muted-foreground">{t("common.loading")}</div>
{:else if !workspaceState.path}
  <WorkspaceSelector />
{:else}
  <ProjectGrid />
{/if}
