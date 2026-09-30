<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FolderOpen, History, CircleCheck, CircleAlert, LoaderCircle } from "@lucide/svelte";
  import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { workspaceState, type WorkspaceMode } from "$lib/stores/workspace.svelte";
  import { i18n, t } from "$lib/i18n/index.svelte";
  import ProjectSetupDialog from "$lib/components/project-setup/project-setup-dialog.svelte";
  import type { ProjectKind } from "$lib/components/project-setup/types";

  interface RecentWorkspace {
    path: string;
    name: string;
    lastUsed: string;
  }

  interface ValidationResult {
    valid: boolean;
    message: string;
    mode?: WorkspaceMode;
    kind?: ProjectKind;
    projectName?: string;
    projectCount?: number;
  }

  let inputPath = $state("");
  let recentWorkspaces = $state<RecentWorkspace[]>([]);
  let validating = $state(false);
  let validationResult = $state<ValidationResult | null>(null);
  let browsing = $state(false);
  // A single-app workspace goes through the setup wizard before it opens.
  let setupWorkspace = $state<{ path: string; kind: ProjectKind; name?: string } | null>(null);

  $effect(() => {
    fetchRecentWorkspaces();
  });

  async function fetchRecentWorkspaces() {
    try {
      recentWorkspaces = await invoke<RecentWorkspace[]>("workspace_recent_get");
    } catch (error) {
      console.error("Failed to load recent workspaces:", error);
    }
  }

  async function validateAndSetWorkspace(path: string) {
    validating = true;
    validationResult = null;

    try {
      const result = await invoke<ValidationResult>("workspace_validate", { workspacePath: path });
      validationResult = result;

      if (result.valid && result.mode === "generic" && result.kind) {
        setupWorkspace = { path, kind: result.kind, name: result.projectName };
      } else if (result.valid && result.mode) {
        await openWorkspace(path, result.mode, result.projectName);
      }
    } catch {
      validationResult = { valid: false, message: t("workspaceSelector.validationError") };
    } finally {
      validating = false;
    }
  }

  async function openWorkspace(path: string, mode: WorkspaceMode, name?: string) {
    await invoke("workspace_recent_add", { path, name });
    await workspaceState.setWorkspace(path, mode);
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (inputPath.trim()) {
      validateAndSetWorkspace(inputPath.trim());
    }
  }

  async function handleBrowse() {
    browsing = true;
    try {
      const path = await invoke<string | null>("workspace_browse");
      if (path) {
        inputPath = path;
        await validateAndSetWorkspace(path);
      }
    } catch (error) {
      console.error("Failed to select folder:", error);
    } finally {
      browsing = false;
    }
  }

  function handleRecentClick(workspace: RecentWorkspace) {
    inputPath = workspace.path;
    validateAndSetWorkspace(workspace.path);
  }

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString(i18n.locale === "tr" ? "tr-TR" : "en-US", {
      day: "numeric",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<div class="min-h-screen bg-background flex items-center justify-center p-4">
  <div class="w-full max-w-2xl space-y-6">
    <div class="text-center space-y-2">
      <h1 class="text-3xl font-bold text-foreground">Serenay Mobile Deploy</h1>
      <p class="text-muted-foreground">{t("workspaceSelector.subtitle")}</p>
    </div>

    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <FolderOpen class="w-5 h-5" />
          {t("workspaceSelector.projectFolder")}
        </CardTitle>
        <CardDescription>
          {t("workspaceSelector.projectFolderDescription")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onsubmit={handleSubmit} class="space-y-4">
          <Button
            type="button"
            variant="outline"
            size="lg"
            onclick={handleBrowse}
            disabled={browsing || validating}
            class="w-full h-24 border-2 border-dashed hover:border-primary hover:bg-primary/5 transition-all"
          >
            {#if browsing}
              <div class="flex flex-col items-center gap-2">
                <LoaderCircle class="w-8 h-8 animate-spin text-muted-foreground" />
                <span class="text-sm text-muted-foreground">{t("workspaceSelector.selectingFolder")}</span>
              </div>
            {:else}
              <div class="flex flex-col items-center gap-2">
                <FolderOpen class="w-8 h-8 text-muted-foreground" />
                <span class="text-sm font-medium">{t("workspaceSelector.selectFolder")}</span>
              </div>
            {/if}
          </Button>

          <div class="relative">
            <div class="absolute inset-0 flex items-center">
              <span class="w-full border-t"></span>
            </div>
            <div class="relative flex justify-center text-xs uppercase">
              <span class="bg-card px-2 text-muted-foreground">{t("workspaceSelector.orEnterPath")}</span>
            </div>
          </div>

          <div class="flex gap-2">
            <Input
              type="text"
              bind:value={inputPath}
              oninput={() => (validationResult = null)}
              placeholder="/Users/username/projects/my-app"
              class="flex-1 font-mono text-sm"
              disabled={validating || browsing}
            />
            <Button type="submit" disabled={validating || browsing || !inputPath.trim()}>
              {#if validating}
                <LoaderCircle class="w-4 h-4 animate-spin" />
              {:else}
                {t("workspaceSelector.open")}
              {/if}
            </Button>
          </div>

          {#if validationResult}
            <div
              class={`flex items-start gap-3 p-3 rounded-lg ${
                validationResult.valid
                  ? "bg-green-500/10 border border-green-500/30"
                  : "bg-red-500/10 border border-red-500/30"
              }`}
            >
              {#if validationResult.valid}
                <CircleCheck class="w-5 h-5 text-green-500 shrink-0 mt-0.5" />
              {:else}
                <CircleAlert class="w-5 h-5 text-red-500 shrink-0 mt-0.5" />
              {/if}
              <div class="flex-1">
                <p class={`font-medium ${validationResult.valid ? "text-green-600" : "text-red-600"}`}>
                  {validationResult.message}
                </p>
                {#if validationResult.valid && validationResult.projectName}
                  <p class="text-sm text-muted-foreground mt-1">
                    {t("workspaceSelector.projectsFound", {
                      name: validationResult.projectName,
                      count: validationResult.projectCount ?? 0,
                    })}
                  </p>
                {/if}
              </div>
            </div>
          {/if}
        </form>
      </CardContent>
    </Card>

    {#if recentWorkspaces.length > 0}
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <History class="w-5 h-5" />
            {t("workspaceSelector.recentlyUsed")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div class="space-y-2">
            {#each recentWorkspaces as workspace (workspace.path)}
              <button
                onclick={() => handleRecentClick(workspace)}
                disabled={validating}
                class="w-full flex items-center gap-4 p-3 rounded-lg border border-border
                       hover:border-primary hover:bg-primary/5 transition-all text-left
                       disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <FolderOpen class="w-5 h-5 text-muted-foreground shrink-0" />
                <div class="flex-1 min-w-0">
                  <p class="font-medium text-foreground truncate">{workspace.name}</p>
                  <p class="text-xs text-muted-foreground font-mono truncate">{workspace.path}</p>
                </div>
                <span class="text-xs text-muted-foreground shrink-0">{formatDate(workspace.lastUsed)}</span>
              </button>
            {/each}
          </div>
        </CardContent>
      </Card>
    {/if}

    <p class="text-center text-sm text-muted-foreground">{t("workspaceSelector.supportedProjects")}</p>
  </div>
</div>

{#if setupWorkspace}
  {@const setup = setupWorkspace}
  <ProjectSetupDialog
    workspacePath={setup.path}
    kind={setup.kind}
    onDone={() => {
      // `setup` tracks `setupWorkspace`, so read it before clearing that.
      const { path, name } = setup;
      setupWorkspace = null;
      openWorkspace(path, "generic", name).catch(
        (error) => (validationResult = { valid: false, message: String(error) })
      );
    }}
    onCancel={() => (setupWorkspace = null)}
  />
{/if}
