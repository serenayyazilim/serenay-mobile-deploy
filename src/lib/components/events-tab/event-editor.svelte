<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { LoaderCircle, CircleAlert, Check, ArrowLeft, Trash2, Plus, ImagePlus, Send } from "@lucide/svelte";
  import { EVENT_STATE_LABELS, LOCALE_SUGGESTIONS } from "$lib/appstoreconnect/labels";
  import { i18n, t } from "$lib/i18n/index.svelte";

  let { workspacePath, appId, eventId, onBack, onDeleted }: {
    workspacePath: string;
    appId: string;
    eventId: string;
    onBack: () => void;
    onDeleted: () => void;
  } = $props();

  const inputClass = "w-full h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none";
  const labelClass = "text-xs text-muted-foreground";

  interface Localization {
    id: string;
    attributes: { locale: string; name?: string; shortDescription?: string; longDescription?: string };
  }

  let event = $state<any>(null);
  let localizations = $state<Localization[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let addingLocalization = $state(false);
  let submitting = $state(false);
  let submitResult = $state<{ success: boolean; message: string } | null>(null);
  let deleting = $state(false);

  async function load() {
    loading = true;
    error = null;
    try {
      const data = await invoke<{ event: any; included: Localization[] }>("asc_event_get", {
        workspace: workspacePath,
        id: eventId,
      });
      event = data.event;
      localizations = data.included || [];
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  async function handleDeleteEvent() {
    if (!confirm(t("eventEditor.confirmDeleteEvent"))) return;
    deleting = true;
    try {
      await invoke("asc_event_delete", { workspace: workspacePath, id: eventId });
      onDeleted();
    } finally {
      deleting = false;
    }
  }

  async function handleSubmitForReview() {
    submitting = true;
    submitResult = null;
    try {
      await invoke("asc_event_submit", { workspace: workspacePath, id: eventId, appId });
      submitResult = { success: true, message: t("eventEditor.submittedForReview") };
      load();
    } catch (e) {
      submitResult = { success: false, message: String(e) };
    } finally {
      submitting = false;
    }
  }

  // ── Add localization ──
  let newLocale = $state("");
  let newName = $state("");
  let newShort = $state("");
  let newLong = $state("");
  let addSaving = $state(false);
  let addError = $state<string | null>(null);

  async function handleAddLocalization() {
    if (!newLocale.trim() || !newName.trim()) return;
    addSaving = true;
    addError = null;
    try {
      await invoke("asc_localization_create", {
        workspace: workspacePath,
        appEventId: eventId,
        attributes: {
          locale: newLocale.trim(),
          name: newName.trim(),
          shortDescription: newShort.trim() || undefined,
          longDescription: newLong.trim() || undefined,
        },
      });
      addingLocalization = false;
      newLocale = newName = newShort = newLong = "";
      load();
    } catch (e) {
      addError = String(e);
    } finally {
      addSaving = false;
    }
  }

  // ── Localization card state (keyed by loc.id) ──
  let editState = $state<Record<string, { name: string; short: string; long: string; saving: boolean }>>({});
  let screenshots = $state<Record<string, any[]>>({});
  let uploading = $state<Record<string, string | null>>({});
  let cardErrors = $state<Record<string, string | null>>({});

  $effect(() => {
    for (const loc of localizations) {
      if (!editState[loc.id]) {
        editState[loc.id] = {
          name: loc.attributes.name || "",
          short: loc.attributes.shortDescription || "",
          long: loc.attributes.longDescription || "",
          saving: false,
        };
        fetchScreenshots(loc.id);
      }
    }
  });

  async function fetchScreenshots(locId: string) {
    try {
      screenshots[locId] = await invoke<any[]>("asc_localization_screenshots", { workspace: workspacePath, id: locId });
    } catch {
      screenshots[locId] = [];
    }
  }

  function isChanged(loc: Localization) {
    const s = editState[loc.id];
    if (!s) return false;
    return (
      s.name !== (loc.attributes.name || "") ||
      s.short !== (loc.attributes.shortDescription || "") ||
      s.long !== (loc.attributes.longDescription || "")
    );
  }

  async function saveLocalization(loc: Localization) {
    const s = editState[loc.id];
    s.saving = true;
    try {
      await invoke("asc_localization_update", {
        workspace: workspacePath,
        id: loc.id,
        attributes: { name: s.name, shortDescription: s.short, longDescription: s.long },
      });
      load();
    } finally {
      s.saving = false;
    }
  }

  async function deleteLocalization(loc: Localization) {
    if (!confirm(t("eventEditor.confirmDeleteLocalization", { locale: loc.attributes.locale }))) return;
    await invoke("asc_localization_delete", { workspace: workspacePath, id: loc.id });
    load();
  }

  async function uploadAsset(loc: Localization, assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE") {
    const filePath = await open({ filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }] });
    if (!filePath || Array.isArray(filePath)) return;

    uploading[loc.id] = assetType;
    cardErrors[loc.id] = null;
    try {
      await invoke("asc_screenshot_upload", {
        workspace: workspacePath,
        localizationId: loc.id,
        assetType,
        filePath,
      });
      await fetchScreenshots(loc.id);
    } catch (e) {
      cardErrors[loc.id] = String(e);
    } finally {
      uploading[loc.id] = null;
    }
  }

  function assetImageUrl(shot: any): string | undefined {
    const url = shot?.attributes?.imageAsset?.templateUrl;
    if (!url) return undefined;
    return url.replace("{w}", "300").replace("{h}", "300").replace("{f}", "png");
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-20">
    <LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" />
  </div>
{:else if error || !event}
  <div class="space-y-3">
    <button onclick={onBack} class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
      <ArrowLeft class="w-3.5 h-3.5" /> {t("eventEditor.backToList")}
    </button>
    <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-600">
      <CircleAlert class="w-4 h-4 flex-shrink-0" />
      {error || t("eventEditor.eventNotFound")}
    </div>
  </div>
{:else}
  {@const state = event.attributes.eventState}
  {@const stateInfo = EVENT_STATE_LABELS[state] || { label: state, className: "bg-secondary text-muted-foreground" }}
  {@const canSubmit = state === "DRAFT" || state === "REJECTED"}
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <button onclick={onBack} class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
        <ArrowLeft class="w-3.5 h-3.5" /> {t("eventEditor.backToList")}
      </button>
      <span class={`text-xs font-medium px-2.5 py-1 rounded-full ${stateInfo.className}`}>{stateInfo.label}</span>
    </div>

    <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30">
      <p class="text-sm font-semibold">{event.attributes.referenceName}</p>
      <p class="text-xs text-muted-foreground mt-1">
        {event.attributes.badge} · {event.attributes.priority} · {t("eventEditor.primaryLocale")}: {event.attributes.primaryLocale}
      </p>
      {#if event.attributes.territorySchedules?.[0]}
        {@const sched = event.attributes.territorySchedules[0]}
        <p class="text-xs text-muted-foreground mt-1">
          {new Date(sched.eventStart).toLocaleString(i18n.locale === "tr" ? "tr-TR" : "en-US")} → {new Date(sched.eventEnd).toLocaleString(i18n.locale === "tr" ? "tr-TR" : "en-US")}
          · {sched.territories.join(", ")}
        </p>
      {/if}
    </div>

    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <p class="text-xs font-medium text-muted-foreground">{t("eventEditor.localizations")}</p>
        <button onclick={() => (addingLocalization = true)} class="flex items-center gap-1 text-xs text-primary hover:underline">
          <Plus class="w-3.5 h-3.5" /> {t("common.add")}
        </button>
      </div>

      {#if addingLocalization}
        <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
          <p class="text-xs font-medium">{t("eventEditor.newLocalization")}</p>
          <div class="space-y-1">
            <label class={labelClass} for="new-locale">Locale</label>
            <input id="new-locale" bind:value={newLocale} class={inputClass} list="asc-locale-suggestions-edit" />
            <datalist id="asc-locale-suggestions-edit">
              {#each LOCALE_SUGGESTIONS as l (l)}<option value={l}></option>{/each}
            </datalist>
          </div>
          <div class="space-y-1">
            <label class={labelClass} for="new-name">{t("eventForm.headline")}</label>
            <input id="new-name" bind:value={newName} class={inputClass} maxlength={30} />
          </div>
          <div class="space-y-1">
            <label class={labelClass} for="new-short">{t("eventForm.shortDescription")}</label>
            <input id="new-short" bind:value={newShort} class={inputClass} maxlength={50} />
          </div>
          <div class="space-y-1">
            <label class={labelClass} for="new-long">{t("eventForm.longDescription")}</label>
            <textarea id="new-long" bind:value={newLong} rows={2} maxlength={500}
              class="w-full px-3 py-2 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none resize-none"></textarea>
          </div>
          {#if addError}<div class="text-xs text-red-600">{addError}</div>{/if}
          <div class="flex gap-2 justify-end">
            <button onclick={() => (addingLocalization = false)} class="px-3 py-1.5 rounded-full text-xs font-medium text-muted-foreground hover:bg-secondary">{t("common.cancel")}</button>
            <button onclick={handleAddLocalization} disabled={addSaving || !newLocale.trim() || !newName.trim()}
              class="flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50">
              {#if addSaving}<LoaderCircle class="w-3 h-3 animate-spin" />{:else}{t("common.add")}{/if}
            </button>
          </div>
        </div>
      {/if}

      {#each localizations as loc (loc.id)}
        {@const s = editState[loc.id]}
        {#if s}
          {@const cardShot = (screenshots[loc.id] || []).find((x) => x.attributes?.appEventAssetType === "EVENT_CARD")}
          {@const detailShot = (screenshots[loc.id] || []).find((x) => x.attributes?.appEventAssetType === "EVENT_DETAILS_PAGE")}
          <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-xs font-mono font-semibold px-2 py-1 rounded-md bg-background/60">{loc.attributes.locale}</span>
              <button onclick={() => deleteLocalization(loc)} class="text-muted-foreground hover:text-red-600 transition-colors">
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>

            <div class="space-y-1">
              <label class={labelClass} for="loc-{loc.id}-name">{t("eventForm.headline")}</label>
              <input id="loc-{loc.id}-name" bind:value={s.name} class={inputClass} maxlength={30} />
            </div>
            <div class="space-y-1">
              <label class={labelClass} for="loc-{loc.id}-short">{t("eventForm.shortDescription")}</label>
              <input id="loc-{loc.id}-short" bind:value={s.short} class={inputClass} maxlength={50} />
            </div>
            <div class="space-y-1">
              <label class={labelClass} for="loc-{loc.id}-long">{t("eventForm.longDescription")}</label>
              <textarea id="loc-{loc.id}-long" bind:value={s.long} rows={2} maxlength={500}
                class="w-full px-3 py-2 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none resize-none"></textarea>
            </div>

            <div class="grid grid-cols-2 gap-3">
              {#each [["EVENT_CARD", t("eventEditor.cardImage"), cardShot], ["EVENT_DETAILS_PAGE", t("eventEditor.detailPageImage"), detailShot]] as [assetType, label, shot] (assetType)}
                {@const shotState = (shot as any)?.attributes?.assetDeliveryState?.state}
                {@const imgUrl = assetImageUrl(shot)}
                <div class="space-y-1">
                  <p class={labelClass}>{label}</p>
                  <div class="relative w-full aspect-video rounded-lg bg-secondary/40 ring-1 ring-border/40 flex items-center justify-center overflow-hidden">
                    {#if uploading[loc.id] === assetType}
                      <LoaderCircle class="w-5 h-5 animate-spin text-muted-foreground" />
                    {:else if shotState === "COMPLETE" && imgUrl}
                      <img src={imgUrl} alt={label as string} class="w-full h-full object-cover" />
                    {:else if shot}
                      <span class="text-xs text-muted-foreground">{t("eventEditor.processing")} ({shotState})</span>
                    {:else}
                      <button onclick={() => uploadAsset(loc, assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE")}
                        class="flex flex-col items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
                        <ImagePlus class="w-5 h-5" />
                        {t("eventEditor.uploadImage")}
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>

            {#if cardErrors[loc.id]}
              <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs bg-red-500/10 text-red-600">
                <CircleAlert class="w-3.5 h-3.5 flex-shrink-0" />
                {cardErrors[loc.id]}
              </div>
            {/if}

            {#if isChanged(loc)}
              <div class="flex justify-end">
                <button onclick={() => saveLocalization(loc)} disabled={s.saving}
                  class="flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold bg-primary text-primary-foreground hover:bg-primary/90">
                  {#if s.saving}<LoaderCircle class="w-3 h-3 animate-spin" />{:else}<Check class="w-3 h-3" />{/if}
                  {t("common.save")}
                </button>
              </div>
            {/if}
          </div>
        {/if}
      {/each}
    </div>

    {#if submitResult}
      <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${submitResult.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}>
        {#if submitResult.success}<Check class="w-4 h-4 flex-shrink-0" />{:else}<CircleAlert class="w-4 h-4 flex-shrink-0" />{/if}
        {submitResult.message}
      </div>
    {/if}

    <div class="flex items-center justify-between pt-2 border-t border-border/50">
      <button onclick={handleDeleteEvent} disabled={deleting}
        class="flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold text-red-600 hover:bg-red-500/10 transition-colors">
        {#if deleting}<LoaderCircle class="w-3.5 h-3.5 animate-spin" />{:else}<Trash2 class="w-3.5 h-3.5" />{/if}
        {t("eventEditor.deleteEvent")}
      </button>
      <button onclick={handleSubmitForReview} disabled={submitting || !canSubmit}
        class={`flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold transition-all ${canSubmit ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}>
        {#if submitting}<LoaderCircle class="w-3.5 h-3.5 animate-spin" />{:else}<Send class="w-3.5 h-3.5" />{/if}
        {t("eventEditor.submitForReview")}
      </button>
    </div>
  </div>
{/if}
