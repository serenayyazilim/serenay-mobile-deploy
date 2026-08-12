<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Pencil, Check, X } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogTitle } from "$lib/components/ui/dialog";
  import { SERCONF_CATEGORIES, getFieldsByCategory } from "$lib/serconf-schema";
  import CategoryPill from "./category-pill.svelte";
  import ColorsTab from "./colors-tab.svelte";
  import SettingsContent from "./settings-content.svelte";
  import EventsTab from "$lib/components/events-tab/index.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { open = $bindable(false), project, workspacePath, workspaceMode, onProjectRenamed }: {
    open: boolean;
    project: WorkspaceProject | null;
    workspacePath: string;
    workspaceMode: string | null;
    onProjectRenamed?: () => void;
  } = $props();

  const supportsTenantConfig = $derived(workspaceMode !== "generic");
  let activeCategory = $state("colors");

  let editingName = $state(false);
  let nameValue = $state("");
  let nameSaving = $state(false);

  let config = $state<Record<string, string | number | boolean>>({});
  let originalConfig = $state<Record<string, string | number | boolean>>({});
  let configLoading = $state(true);
  let configSaving = $state(false);
  let configResult = $state<{ success: boolean; message: string } | null>(null);

  let colors = $state<Record<string, string>>({});
  let originalColors = $state<Record<string, string>>({});
  let colorsLoading = $state(true);
  let colorsSaving = $state(false);
  let colorsResult = $state<{ success: boolean; message: string } | null>(null);

  async function loadAll() {
    if (!project) return;
    configLoading = true;
    colorsLoading = true;
    try {
      if (supportsTenantConfig) {
        const data = await invoke<Record<string, string | number | boolean>>("config_serconf_get", {
          workspace: workspacePath,
          projectId: project.id,
        });
        config = { ...data };
        originalConfig = { ...data };
      }
    } catch {
      config = {};
      originalConfig = {};
    } finally {
      configLoading = false;
    }

    try {
      const data = await invoke<Record<string, string>>("config_colors_get", { workspace: workspacePath, projectId: project.id });
      colors = { ...data };
      originalColors = { ...data };
    } catch {
      colors = {};
      originalColors = {};
    } finally {
      colorsLoading = false;
    }
  }

  $effect(() => {
    if (open && project) {
      activeCategory = "colors";
      nameValue = project.appName;
      editingName = false;
      configResult = null;
      colorsResult = null;
      loadAll();
    }
  });

  function updateField(key: string, value: string | number | boolean) {
    config = { ...config, [key]: value };
  }

  function updateColor(key: string, value: string) {
    colors = { ...colors, [key]: value };
  }

  const configHasChanges = $derived(JSON.stringify(config) !== JSON.stringify(originalConfig));
  const colorsHasChanges = $derived(JSON.stringify(colors) !== JSON.stringify(originalColors));

  async function handleSaveConfig() {
    if (!project) return;
    configSaving = true;
    configResult = null;
    try {
      await invoke("config_serconf_save", { workspace: workspacePath, projectId: project.id, config });
      originalConfig = { ...config };
      configResult = { success: true, message: t("projectSettings.saved") };
    } catch (e) {
      configResult = { success: false, message: String(e) };
    } finally {
      configSaving = false;
    }
  }

  async function handleSaveColors() {
    if (!project) return;
    colorsSaving = true;
    colorsResult = null;
    try {
      await invoke("config_colors_save", { workspace: workspacePath, projectId: project.id, colors });
      originalColors = { ...colors };
      colorsResult = { success: true, message: t("projectSettings.colorsSaved") };
    } catch (e) {
      colorsResult = { success: false, message: String(e) };
    } finally {
      colorsSaving = false;
    }
  }

  async function handleNameSave() {
    if (!project || !nameValue.trim() || nameValue === project.appName) {
      editingName = false;
      return;
    }
    nameSaving = true;
    try {
      await invoke("projects_rename", { workspace: workspacePath, projectId: project.id, appName: nameValue.trim() });
      onProjectRenamed?.();
    } finally {
      nameSaving = false;
      editingName = false;
    }
  }

  const allCategories = $derived([
    ...(supportsTenantConfig ? [{ id: "colors", label: t("projectSettings.colors") }] : []),
    { id: "events", label: "In-App Events" },
    ...(supportsTenantConfig ? SERCONF_CATEGORIES : []),
  ]);

  const currentFields = $derived(getFieldsByCategory(activeCategory));
</script>

<Dialog bind:open>
  <DialogContent class="max-w-2xl p-0 gap-0 overflow-hidden rounded-3xl border-0 shadow-2xl max-h-[85vh] flex flex-col">
    <div class="px-6 pt-6 pb-4">
      <div class="flex items-center gap-3">
        <div class="flex-1 min-w-0">
          {#if editingName}
            <div class="flex items-center gap-2">
              <input
                bind:value={nameValue}
                onkeydown={(e) => {
                  if (e.key === "Enter") handleNameSave();
                  if (e.key === "Escape") { nameValue = project?.appName ?? ""; editingName = false; }
                }}
                class="text-xl font-semibold bg-transparent border-b border-primary outline-none w-full min-w-0"
                disabled={nameSaving}
              />
              <button onclick={handleNameSave} disabled={nameSaving || !nameValue.trim()} class="text-green-500 hover:text-green-600 shrink-0 disabled:opacity-50">
                {#if nameSaving}<LoaderCircle class="w-4 h-4 animate-spin" />{:else}<Check class="w-4 h-4" />{/if}
              </button>
              <button onclick={() => { nameValue = project?.appName ?? ""; editingName = false; }} class="text-muted-foreground hover:text-foreground shrink-0">
                <X class="w-4 h-4" />
              </button>
            </div>
          {:else}
            <div class="flex items-center gap-2 group">
              <DialogTitle class="text-xl font-semibold truncate">{project?.appName}</DialogTitle>
              <button onclick={() => (editingName = true)} class="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-foreground shrink-0">
                <Pencil class="w-3.5 h-3.5" />
              </button>
            </div>
          {/if}
          <p class="text-sm text-muted-foreground">{t("projectSettings.title")}</p>
        </div>
      </div>
    </div>

    <div class="px-6 pb-4">
      <div class="flex gap-2 overflow-x-auto pb-1">
        {#each allCategories as cat (cat.id)}
          <CategoryPill label={cat.label} active={activeCategory === cat.id} onClick={() => (activeCategory = cat.id)} />
        {/each}
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 pb-4 min-h-[300px]">
      {#if activeCategory === "colors"}
        {#if colorsLoading}
          <div class="flex items-center justify-center py-20"><LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" /></div>
        {:else}
          <ColorsTab {colors} onUpdateColor={updateColor} />
        {/if}
      {:else if activeCategory === "events" && project}
        <EventsTab {workspacePath} bundleId={project.bundleId} />
      {:else if configLoading}
        <div class="flex items-center justify-center py-20"><LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" /></div>
      {:else}
        <SettingsContent fields={currentFields} {config} onUpdateField={updateField} />
      {/if}
    </div>

    {#if activeCategory === "events"}
      <div class="px-6 py-4 bg-secondary/30 border-t border-border/50 flex justify-end">
        <button onclick={() => (open = false)} class="px-5 py-2.5 rounded-full text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors">
          {t("common.close")}
        </button>
      </div>
    {:else}
      {@const isColors = activeCategory === "colors"}
      {@const hasChanges = isColors ? colorsHasChanges : configHasChanges}
      {@const saving = isColors ? colorsSaving : configSaving}
      {@const result = isColors ? colorsResult : configResult}
      <div class="px-6 py-4 bg-secondary/30 border-t border-border/50 flex items-center justify-between gap-3">
        <div class="text-sm">
          {#if result}
            <span class={result.success ? "text-green-600" : "text-red-600"}>{result.message}</span>
          {/if}
        </div>
        <div class="flex gap-2">
          <button onclick={() => (open = false)} class="px-5 py-2.5 rounded-full text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors">
            Kapat
          </button>
          <button
            onclick={isColors ? handleSaveColors : handleSaveConfig}
            disabled={!hasChanges || saving}
            class={`flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-semibold transition-all ${hasChanges ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}
          >
            {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
            {t("common.save")}
          </button>
        </div>
      </div>
    {/if}
  </DialogContent>
</Dialog>
