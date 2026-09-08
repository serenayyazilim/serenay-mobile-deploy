<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CalendarClock, CircleAlert, LoaderCircle, Search } from "@lucide/svelte";
  import { Input } from "$lib/components/ui/input";
  import { Dialog, DialogContent, DialogTitle } from "$lib/components/ui/dialog";
  import { projectsState } from "$lib/stores/projects.svelte";
  import EventsTab from "$lib/components/events-tab/index.svelte";
  import AppIcon from "./app-icon.svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { workspacePath }: { workspacePath: string } = $props();

  interface AscApp {
    id: string;
    name: string;
    bundleId: string;
  }

  let configured = $state<boolean | null>(null);
  let apps = $state<AscApp[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let search = $state("");

  const localProjectByBundleId = $derived(
    new Map(projectsState.projects.map((p) => [p.bundleId, p.id])),
  );

  const filteredApps = $derived(
    (() => {
      const q = search.trim().toLowerCase();
      if (!q) return apps;
      return apps.filter((a) => a.name.toLowerCase().includes(q) || a.bundleId.toLowerCase().includes(q));
    })(),
  );

  async function load() {
    loading = true;
    error = null;
    try {
      const configData = await invoke<{ configured: boolean }>("asc_config_get", { workspace: workspacePath });
      if (!configData.configured) {
        configured = false;
        return;
      }
      configured = true;
      apps = await invoke<AscApp[]>("asc_apps_list", { workspace: workspacePath });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  let selectedApp = $state<AscApp | null>(null);
  let dialogOpen = $state(false);

  function openApp(app: AscApp) {
    selectedApp = app;
    dialogOpen = true;
  }
</script>

<div class="flex-1 min-w-0 h-screen flex flex-col">
  {#if loading}
    <div class="flex-1 flex items-center justify-center h-96">
      <LoaderCircle class="w-8 h-8 animate-spin text-muted-foreground" />
    </div>
  {:else if configured === false}
    <div class="flex-1 flex flex-col items-center justify-center text-center text-muted-foreground p-8">
      <CalendarClock class="w-10 h-10 mx-auto mb-3 opacity-30" />
      <p class="text-sm">{t("eventsPage.notConfigured")}</p>
      <p class="text-xs mt-1">{t("eventsPage.notConfiguredHint")}</p>
    </div>
  {:else if error}
    <div class="p-8">
      <div class="flex items-center gap-2 px-4 py-3 rounded-xl text-sm bg-red-500/10 text-red-600 max-w-md">
        <CircleAlert class="w-4 h-4 shrink-0" />
        {t("eventsPage.loadFailed")}: {error}
      </div>
    </div>
  {:else}
    <div class="shrink-0 flex items-center gap-4 p-8 pb-6">
      <div class="relative max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <Input
          type="text"
          bind:value={search}
          placeholder={t("eventsPage.searchPlaceholder")}
          class="pl-11 h-12 rounded-2xl bg-secondary/30 border-0 focus-visible:ring-1 focus-visible:ring-primary/50"
        />
      </div>
      <p class="text-base font-medium text-muted-foreground whitespace-nowrap">
        {t("eventsPage.appCount", { count: apps.length })}
      </p>
    </div>

    <div class="flex-1 min-h-0 overflow-y-auto px-8 pb-8">
      {#if filteredApps.length === 0}
        <div class="text-center py-20 text-muted-foreground">
          <CalendarClock class="w-10 h-10 mx-auto mb-3 opacity-30" />
          <p class="text-sm">{t("eventsPage.noApps")}</p>
        </div>
      {:else}
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-5">
          {#each filteredApps as app (app.id)}
            <button
              onclick={() => openApp(app)}
              class="group flex flex-col items-center gap-3 p-5 rounded-3xl bg-card dark:bg-transparent hover:bg-secondary/50 hover:shadow-lg hover:shadow-black/5 hover:-translate-y-1 transition-all duration-300 text-center"
            >
              <AppIcon {workspacePath} projectId={localProjectByBundleId.get(app.bundleId)} bundleId={app.bundleId} name={app.name} />
              <div class="min-w-0 w-full">
                <p class="font-medium text-sm truncate">{app.name}</p>
                <p class="text-[10px] text-muted-foreground truncate">{app.bundleId}</p>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<Dialog bind:open={dialogOpen}>
  <DialogContent class="max-w-2xl p-0 gap-0 overflow-hidden rounded-3xl border-0 shadow-2xl max-h-[85vh] flex flex-col">
    <div class="px-6 pt-6 pb-4">
      <DialogTitle class="text-xl font-semibold truncate">{selectedApp?.name}</DialogTitle>
      <p class="text-sm text-muted-foreground">{selectedApp?.bundleId}</p>
    </div>
    <div class="flex-1 overflow-y-auto px-6 pb-6 min-h-[300px]">
      {#if selectedApp}
        <EventsTab {workspacePath} bundleId={selectedApp.bundleId} />
      {/if}
    </div>
  </DialogContent>
</Dialog>
