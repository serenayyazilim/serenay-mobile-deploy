<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, CircleAlert, Plus, CalendarClock, Trash2 } from "@lucide/svelte";
  import { BADGE_OPTIONS, EVENT_STATE_LABELS } from "$lib/appstoreconnect/labels";
  import EventCreateForm from "./event-create-form.svelte";
  import EventEditor from "./event-editor.svelte";
  import { i18n, t } from "$lib/i18n/index.svelte";

  let { workspacePath, bundleId }: { workspacePath: string; bundleId: string } = $props();

  type View = "list" | "create" | { edit: string };

  let configured = $state<boolean | null>(null);
  let appId = $state<string | null>(null);
  let events = $state<any[]>([]);
  let included = $state<any[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let view = $state<View>("list");
  let deletingId = $state<string | null>(null);

  function badgeLabel(value?: string | null) {
    return BADGE_OPTIONS.find((b) => b.value === value)?.label || value || "";
  }

  async function loadEvents() {
    loading = true;
    error = null;
    try {
      const configData = await invoke<{ configured: boolean }>("asc_config_get", { workspace: workspacePath });
      if (!configData.configured) {
        configured = false;
        return;
      }
      configured = true;

      const data = await invoke<{ appId: string; events: any[]; included: any[] }>("asc_events_list", {
        workspace: workspacePath,
        bundleId,
      });
      appId = data.appId;
      events = data.events || [];
      included = data.included || [];
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    view = "list";
    loadEvents();
  });

  async function handleDelete(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (!confirm(t("events.confirmDelete"))) return;
    deletingId = id;
    try {
      await invoke("asc_event_delete", { workspace: workspacePath, id });
      loadEvents();
    } finally {
      deletingId = null;
    }
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-20">
    <LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" />
  </div>
{:else if configured === false}
  <div class="text-center py-16 text-muted-foreground">
    <CalendarClock class="w-10 h-10 mx-auto mb-3 opacity-30" />
    <p class="text-sm">{t("events.notConfigured")}</p>
    <p class="text-xs mt-1">{t("events.notConfiguredHint")}</p>
  </div>
{:else if typeof view === "object" && "edit" in view && appId}
  <EventEditor
    {workspacePath}
    appId={appId}
    eventId={view.edit}
    onBack={() => { view = "list"; loadEvents(); }}
    onDeleted={() => { view = "list"; loadEvents(); }}
  />
{:else if view === "create"}
  <EventCreateForm
    {workspacePath}
    {bundleId}
    onCreated={(eventId) => (view = { edit: eventId })}
    onCancel={() => (view = "list")}
  />
{:else}
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <p class="text-xs font-medium text-muted-foreground">{t("events.count", { count: events.length })}</p>
      <button
        onclick={() => (view = "create")}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
      >
        <Plus class="w-3.5 h-3.5" /> {t("events.newEvent")}
      </button>
    </div>

    {#if error}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-600">
        <CircleAlert class="w-4 h-4 flex-shrink-0" />
        {error}
      </div>
    {/if}

    {#if events.length === 0 && !error}
      <div class="text-center py-16 text-muted-foreground">
        <CalendarClock class="w-10 h-10 mx-auto mb-3 opacity-30" />
        <p class="text-sm">{t("events.noEvents")}</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each events as ev (ev.id)}
          {@const state = ev.attributes.eventState}
          {@const stateInfo = EVENT_STATE_LABELS[state] || { label: state, className: "bg-secondary text-muted-foreground" }}
          {@const localizationIds = (ev.relationships?.localizations?.data || []).map((d: any) => d.id)}
          {@const primaryLoc =
            included.find((l) => localizationIds.includes(l.id) && l.attributes.locale === ev.attributes.primaryLocale) ||
            included.find((l) => localizationIds.includes(l.id))}
          <div
            onclick={() => (view = { edit: ev.id })}
            onkeydown={(e) => e.key === "Enter" && (view = { edit: ev.id })}
            role="button"
            tabindex="0"
            class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 cursor-pointer transition-all flex items-center justify-between gap-3"
          >
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <p class="text-sm font-medium truncate">{primaryLoc?.attributes?.name || ev.attributes.referenceName}</p>
                <span class={`text-[10px] font-medium px-2 py-0.5 rounded-full whitespace-nowrap ${stateInfo.className}`}>{stateInfo.label}</span>
              </div>
              <p class="text-xs text-muted-foreground mt-0.5 truncate">
                {badgeLabel(ev.attributes.badge)}
                {#if ev.attributes.territorySchedules?.[0]}
                  · {new Date(ev.attributes.territorySchedules[0].eventStart).toLocaleDateString(i18n.locale === "tr" ? "tr-TR" : "en-US")} - {new Date(ev.attributes.territorySchedules[0].eventEnd).toLocaleDateString(i18n.locale === "tr" ? "tr-TR" : "en-US")}
                {/if}
              </p>
            </div>
            <button
              onclick={(e) => handleDelete(ev.id, e)}
              disabled={deletingId === ev.id}
              class="p-2 rounded-full text-muted-foreground hover:text-red-600 hover:bg-red-500/10 transition-colors flex-shrink-0"
            >
              {#if deletingId === ev.id}<LoaderCircle class="w-4 h-4 animate-spin" />{:else}<Trash2 class="w-4 h-4" />{/if}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}
