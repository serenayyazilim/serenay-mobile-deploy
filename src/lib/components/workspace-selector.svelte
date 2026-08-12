<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FolderOpen, History, CircleCheck, CircleAlert, LoaderCircle } from "@lucide/svelte";
  import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { workspaceState, type WorkspaceMode } from "$lib/stores/workspace.svelte";

  interface RecentWorkspace {
    path: string;
    name: string;
    lastUsed: string;
  }

  interface ValidationResult {
    valid: boolean;
    message: string;
    mode?: WorkspaceMode;
    projectName?: string;
    projectCount?: number;
  }

  let inputPath = $state("");
  let recentWorkspaces = $state<RecentWorkspace[]>([]);
  let validating = $state(false);
  let validationResult = $state<ValidationResult | null>(null);
  let browsing = $state(false);

  $effect(() => {
    fetchRecentWorkspaces();
  });

  async function fetchRecentWorkspaces() {
    try {
      recentWorkspaces = await invoke<RecentWorkspace[]>("workspace_recent_get");
    } catch (error) {
      console.error("Recent workspaces yüklenemedi:", error);
    }
  }

  async function validateAndSetWorkspace(path: string) {
    validating = true;
    validationResult = null;

    try {
      const result = await invoke<ValidationResult>("workspace_validate", { workspacePath: path });
      validationResult = result;

      if (result.valid) {
        await invoke("workspace_recent_add", { path, name: result.projectName });
        if (result.mode) await workspaceState.setWorkspace(path, result.mode);
      }
    } catch {
      validationResult = { valid: false, message: "Doğrulama hatası" };
    } finally {
      validating = false;
    }
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
      console.error("Klasör seçme hatası:", error);
    } finally {
      browsing = false;
    }
  }

  function handleRecentClick(workspace: RecentWorkspace) {
    inputPath = workspace.path;
    validateAndSetWorkspace(workspace.path);
  }

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString("tr-TR", {
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
      <p class="text-muted-foreground">Flutter projenizi seçerek başlayın</p>
    </div>

    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <FolderOpen class="w-5 h-5" />
          Proje Klasörü
        </CardTitle>
        <CardDescription>
          Flutter projesinin kök dizinini girin (pubspec.yaml'ın bulunduğu klasör)
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
                <span class="text-sm text-muted-foreground">Klasör seçiliyor...</span>
              </div>
            {:else}
              <div class="flex flex-col items-center gap-2">
                <FolderOpen class="w-8 h-8 text-muted-foreground" />
                <span class="text-sm font-medium">Klasör Seç</span>
              </div>
            {/if}
          </Button>

          <div class="relative">
            <div class="absolute inset-0 flex items-center">
              <span class="w-full border-t"></span>
            </div>
            <div class="relative flex justify-center text-xs uppercase">
              <span class="bg-card px-2 text-muted-foreground">veya yol girin</span>
            </div>
          </div>

          <div class="flex gap-2">
            <Input
              type="text"
              bind:value={inputPath}
              oninput={() => (validationResult = null)}
              placeholder="/Users/username/projects/my-flutter-app"
              class="flex-1 font-mono text-sm"
              disabled={validating || browsing}
            />
            <Button type="submit" disabled={validating || browsing || !inputPath.trim()}>
              {#if validating}
                <LoaderCircle class="w-4 h-4 animate-spin" />
              {:else}
                Aç
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
                    {validationResult.projectName} - {validationResult.projectCount} proje
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
            Son Kullanılanlar
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

    <p class="text-center text-sm text-muted-foreground">
      Proje klasörünüz <code class="bg-muted px-1.5 py-0.5 rounded text-xs">pubspec.yaml</code> ve
      <code class="bg-muted px-1.5 py-0.5 rounded text-xs">sermobileboss_projects.json</code>
      dosyalarını içermelidir.
    </p>
  </div>
</div>
