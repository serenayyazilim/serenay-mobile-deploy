<script lang="ts">
  import { onMount } from "svelte";
  import WorkspaceSelector from "$lib/components/workspace-selector.svelte";
  import ProjectGrid from "$lib/components/project-grid/project-grid.svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";

  onMount(() => {
    workspaceState.init();
  });
</script>

{#if !workspaceState.loaded}
  <div class="min-h-screen flex items-center justify-center text-muted-foreground">Yükleniyor...</div>
{:else if !workspaceState.path}
  <WorkspaceSelector />
{:else}
  <ProjectGrid />
{/if}
