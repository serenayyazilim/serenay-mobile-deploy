<script lang="ts">
  import { untrack } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { LoaderCircle, CircleAlert, Check, ArrowLeft, Trash2, Plus, ImagePlus, Send, Calendar, Clock, Timer } from "@lucide/svelte";
  import {
    EVENT_STATE_LABELS, LOCALE_SUGGESTIONS, PURCHASE_REQUIREMENT_OPTIONS, fromDatetimeLocal,
    combineDateTime, splitDatetimeLocal, MIN_EVENT_DURATION_MS, MAX_EVENT_DURATION_MS,
  } from "$lib/appstoreconnect/labels";
  import { i18n, t } from "$lib/i18n/index.svelte";

  function formatDuration(ms: number): string {
    const totalMinutes = Math.round(ms / 60000);
    const days = Math.floor(totalMinutes / 1440);
    const hours = Math.floor((totalMinutes % 1440) / 60);
    const minutes = totalMinutes % 60;
    const parts: string[] = [];
    if (days) parts.push(t("eventForm.durationDays", { count: days }));
    if (hours) parts.push(t("eventForm.durationHours", { count: hours }));
    if (minutes || parts.length === 0) parts.push(t("eventForm.durationMinutes", { count: minutes }));
    return parts.join(" ");
  }

  const ASSET_IMAGE_REQUIREMENTS: Record<"EVENT_CARD" | "EVENT_DETAILS_PAGE", {
    minWidth: number; minHeight: number; maxWidth: number; maxHeight: number; ratio: number; ratioLabel: string;
  }> = {
    EVENT_CARD: { minWidth: 1920, minHeight: 1080, maxWidth: 3840, maxHeight: 2160, ratio: 16 / 9, ratioLabel: "16:9" },
    EVENT_DETAILS_PAGE: { minWidth: 1080, minHeight: 1920, maxWidth: 2160, maxHeight: 3840, ratio: 9 / 16, ratioLabel: "9:16" },
  };

  function requirementHint(assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE"): string {
    const r = ASSET_IMAGE_REQUIREMENTS[assetType];
    return t("eventEditor.imageRequirementHint", {
      ratio: r.ratioLabel,
      min: `${r.minWidth}x${r.minHeight}`,
      max: `${r.maxWidth}x${r.maxHeight}`,
    });
  }

  function readImageDimensions(filePath: string): Promise<{ width: number; height: number }> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
      img.onerror = () => reject(new Error(t("eventEditor.imageReadError")));
      img.src = convertFileSrc(filePath);
    });
  }

  function validateImageDimensions(
    assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE",
    dims: { width: number; height: number }
  ): string | null {
    const r = ASSET_IMAGE_REQUIREMENTS[assetType];
    const ratioOk = Math.abs(dims.width / dims.height - r.ratio) < 0.02;
    const sizeOk =
      dims.width >= r.minWidth && dims.height >= r.minHeight && dims.width <= r.maxWidth && dims.height <= r.maxHeight;
    if (ratioOk && sizeOk) return null;
    return t("eventEditor.imageDimensionError", {
      width: dims.width,
      height: dims.height,
      ratio: r.ratioLabel,
      min: `${r.minWidth}x${r.minHeight}`,
      max: `${r.maxWidth}x${r.maxHeight}`,
    });
  }

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

  // ── Schedule editing ──
  let schedTerritories = $state("");
  let schedPublishStartDate = $state("");
  let schedPublishStartTime = $state("");
  let schedEventStartDate = $state("");
  let schedEventStartTime = $state("");
  let schedEventEndDate = $state("");
  let schedEventEndTime = $state("");
  let schedDeepLink = $state("");
  let schedPurchaseRequirement = $state("");
  let schedOriginal = $state("");
  let schedSaving = $state(false);
  let schedError = $state<string | null>(null);

  const schedPublishStart = $derived(combineDateTime(schedPublishStartDate, schedPublishStartTime));
  const schedEventStart = $derived(combineDateTime(schedEventStartDate, schedEventStartTime));
  const schedEventEnd = $derived(combineDateTime(schedEventEndDate, schedEventEndTime));

  const schedDurationMs = $derived.by(() => {
    if (!schedEventStart || !schedEventEnd) return null;
    const start = new Date(schedEventStart).getTime();
    const end = new Date(schedEventEnd).getTime();
    if (isNaN(start) || isNaN(end)) return null;
    return end - start;
  });

  const schedDurationError = $derived(
    schedDurationMs === null
      ? null
      : schedDurationMs < MIN_EVENT_DURATION_MS
        ? t("eventForm.durationTooShort")
        : schedDurationMs > MAX_EVENT_DURATION_MS
          ? t("eventForm.durationTooLong")
          : null
  );

  const schedDirty = $derived(
    JSON.stringify([
      schedTerritories.trim(), schedPublishStart, schedEventStart, schedEventEnd,
      schedDeepLink.trim(), schedPurchaseRequirement,
    ]) !== schedOriginal
  );

  const schedCanSave = $derived(
    !!(schedTerritories.trim() && schedPublishStart && schedEventStart && schedEventEnd && !schedDurationError && schedDirty)
  );

  // Apple requires these before a review submission will be accepted, even though they're
  // optional while the event is still a draft — surface that proactively instead of letting
  // the user hit the opaque API error on submit.
  const submitBlockers = $derived.by(() => {
    if (!event) return [];
    const blockers: string[] = [];
    if (!schedDeepLink.trim()) blockers.push(t("eventEditor.blockerDeepLink"));
    if (!schedPurchaseRequirement) blockers.push(t("eventEditor.blockerPurchaseRequirement"));
    const now = Date.now();
    if (schedPublishStart && new Date(schedPublishStart).getTime() <= now) blockers.push(t("eventEditor.blockerPublishStartPast"));
    if (schedEventStart && new Date(schedEventStart).getTime() <= now) blockers.push(t("eventEditor.blockerEventStartPast"));
    if (schedDirty) blockers.push(t("eventEditor.blockerUnsavedChanges"));
    return blockers;
  });

  function applyScheduleFromEvent(ev: any) {
    const sched = ev?.attributes?.territorySchedules?.[0];
    const ps = splitDatetimeLocal(sched?.publishStart);
    const es = splitDatetimeLocal(sched?.eventStart);
    const ee = splitDatetimeLocal(sched?.eventEnd);
    schedTerritories = (sched?.territories || []).join(", ");
    schedPublishStartDate = ps.date;
    schedPublishStartTime = ps.time;
    schedEventStartDate = es.date;
    schedEventStartTime = es.time;
    schedEventEndDate = ee.date;
    schedEventEndTime = ee.time;
    schedDeepLink = ev?.attributes?.deepLink || "";
    schedPurchaseRequirement = ev?.attributes?.purchaseRequirement || "";
    schedOriginal = JSON.stringify([
      schedTerritories.trim(),
      combineDateTime(ps.date, ps.time),
      combineDateTime(es.date, es.time),
      combineDateTime(ee.date, ee.time),
      schedDeepLink.trim(),
      schedPurchaseRequirement,
    ]);
  }

  async function saveSchedule() {
    if (!schedCanSave) return;
    schedSaving = true;
    schedError = null;
    try {
      await invoke("asc_event_update", {
        workspace: workspacePath,
        id: eventId,
        attributes: {
          deepLink: schedDeepLink.trim() || null,
          purchaseRequirement: schedPurchaseRequirement || null,
          territorySchedules: [
            {
              territories: schedTerritories.split(",").map((v) => v.trim().toUpperCase()).filter(Boolean),
              eventStart: fromDatetimeLocal(schedEventStart),
              eventEnd: fromDatetimeLocal(schedEventEnd),
              publishStart: fromDatetimeLocal(schedPublishStart),
            },
          ],
        },
      });
      await load();
    } catch (e) {
      schedError = String(e);
    } finally {
      schedSaving = false;
    }
  }

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
      applyScheduleFromEvent(event);
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
      const exists = untrack(() => !!editState[loc.id]);
      if (!exists) {
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

  async function uploadAssetFromPath(loc: Localization, assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE", filePath: string) {
    uploading[loc.id] = assetType;
    cardErrors[loc.id] = null;
    try {
      if (!/\.(png|jpe?g)$/i.test(filePath)) {
        cardErrors[loc.id] = t("eventEditor.imageFormatError");
        return;
      }

      const dims = await readImageDimensions(filePath);
      const dimensionError = validateImageDimensions(assetType, dims);
      if (dimensionError) {
        cardErrors[loc.id] = dimensionError;
        return;
      }

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

  async function uploadAsset(loc: Localization, assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE") {
    const filePath = await open({ filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }] });
    if (!filePath || Array.isArray(filePath)) return;
    await uploadAssetFromPath(loc, assetType, filePath);
  }

  let refreshingShot = $state<Record<string, string | null>>({});

  async function refreshShotStatus(loc: Localization, assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE") {
    refreshingShot[loc.id] = assetType;
    try {
      await fetchScreenshots(loc.id);
    } finally {
      refreshingShot[loc.id] = null;
    }
  }

  async function discardStuckAsset(loc: Localization, assetType: "EVENT_CARD" | "EVENT_DETAILS_PAGE", shotId: string) {
    cardErrors[loc.id] = null;
    try {
      await invoke("asc_screenshot_delete", { workspace: workspacePath, id: shotId });
      await fetchScreenshots(loc.id);
    } catch (e) {
      cardErrors[loc.id] = String(e);
    }
  }

  // ── Drag & drop image upload ──
  let dragOverKey = $state<string | null>(null);

  function dropZoneAt(physicalX: number, physicalY: number): HTMLElement | null {
    const scale = window.devicePixelRatio || 1;
    const el = document.elementFromPoint(physicalX / scale, physicalY / scale) as HTMLElement | null;
    return el?.closest<HTMLElement>("[data-drop-zone]") ?? null;
  }

  $effect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          const zone = dropZoneAt(event.payload.position.x, event.payload.position.y);
          dragOverKey = zone ? `${zone.dataset.locId}:${zone.dataset.assetType}` : null;
        } else if (event.payload.type === "drop") {
          const zone = dropZoneAt(event.payload.position.x, event.payload.position.y);
          dragOverKey = null;
          const path = event.payload.paths?.[0];
          const loc = zone && localizations.find((l) => l.id === zone.dataset.locId);
          const assetType = zone?.dataset.assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE" | undefined;
          if (path && loc && assetType) {
            uploadAssetFromPath(loc, assetType, path);
          }
        } else {
          dragOverKey = null;
        }
      })
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

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

    <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
      <div>
        <p class="text-sm font-semibold">{event.attributes.referenceName}</p>
        <p class="text-xs text-muted-foreground mt-1">
          {event.attributes.badge} · {event.attributes.priority} · {t("eventEditor.primaryLocale")}: {event.attributes.primaryLocale}
        </p>
      </div>

      {#if canSubmit}
        <div class="pt-2 border-t border-border/30 space-y-2">
          <div class="space-y-1">
            <label class={labelClass} for="sched-territories">{t("eventForm.territories")}</label>
            <input id="sched-territories" bind:value={schedTerritories} class={inputClass} />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1">
              <label class={labelClass} for="sched-deep-link">{t("eventEditor.deepLinkLabel")}</label>
              <input id="sched-deep-link" bind:value={schedDeepLink} class={inputClass} placeholder="myapp://event/summer" />
            </div>
            <div class="space-y-1">
              <label class={labelClass} for="sched-purchase-req">{t("eventEditor.purchaseRequirementLabel")}</label>
              <select id="sched-purchase-req" bind:value={schedPurchaseRequirement} class={inputClass}>
                <option value="">{t("eventForm.purchaseRequirementUnset")}</option>
                {#each PURCHASE_REQUIREMENT_OPTIONS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
              </select>
            </div>
          </div>
          <p class="text-[10px] text-muted-foreground/70 -mt-1">{t("eventForm.purchaseRequirementHint")} {t("eventEditor.requiredForSubmit")}</p>

          <div class="space-y-1">
            <label class={labelClass} for="sched-publish-date">{t("eventForm.publishStart")}</label>
            <div class="grid grid-cols-[1fr_120px] gap-2">
              <div class="relative">
                <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-publish-date" type="date" bind:value={schedPublishStartDate} class={`${inputClass} pl-8`} />
              </div>
              <div class="relative">
                <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-publish-time" type="time" bind:value={schedPublishStartTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.publishStart")} />
              </div>
            </div>
          </div>

          <div class="space-y-1">
            <label class={labelClass} for="sched-start-date">{t("eventForm.eventStart")}</label>
            <div class="grid grid-cols-[1fr_120px] gap-2">
              <div class="relative">
                <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-start-date" type="date" bind:value={schedEventStartDate} class={`${inputClass} pl-8`} />
              </div>
              <div class="relative">
                <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-start-time" type="time" bind:value={schedEventStartTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.eventStart")} />
              </div>
            </div>
          </div>

          <div class="space-y-1">
            <label class={labelClass} for="sched-end-date">{t("eventForm.eventEnd")}</label>
            <div class="grid grid-cols-[1fr_120px] gap-2">
              <div class="relative">
                <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-end-date" type="date" bind:value={schedEventEndDate} min={schedEventStartDate || undefined} class={`${inputClass} pl-8`} />
              </div>
              <div class="relative">
                <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
                <input id="sched-end-time" type="time" bind:value={schedEventEndTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.eventEnd")} />
              </div>
            </div>
          </div>

          {#if schedDurationMs !== null}
            <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs ${schedDurationError ? "bg-red-500/10 text-red-600" : "bg-primary/10 text-primary"}`}>
              {#if schedDurationError}<CircleAlert class="w-3.5 h-3.5 flex-shrink-0" />{:else}<Timer class="w-3.5 h-3.5 flex-shrink-0" />{/if}
              <span>
                {#if schedDurationError}
                  {schedDurationError}
                {:else}
                  {t("eventForm.durationLabel", { duration: formatDuration(schedDurationMs) })}
                {/if}
              </span>
            </div>
          {/if}

          {#if schedError}
            <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs bg-red-500/10 text-red-600">
              <CircleAlert class="w-3.5 h-3.5 flex-shrink-0" />
              {schedError}
            </div>
          {/if}

          {#if schedDirty}
            <div class="flex justify-end">
              <button onclick={saveSchedule} disabled={schedSaving || !schedCanSave}
                class={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold transition-all ${schedCanSave ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}>
                {#if schedSaving}<LoaderCircle class="w-3 h-3 animate-spin" />{:else}<Check class="w-3 h-3" />{/if}
                {t("common.save")}
              </button>
            </div>
          {/if}
        </div>
      {:else if event.attributes.territorySchedules?.[0]}
        {@const sched = event.attributes.territorySchedules[0]}
        <p class="text-xs text-muted-foreground">
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
                  <div
                    data-drop-zone
                    data-loc-id={loc.id}
                    data-asset-type={assetType}
                    class={`relative w-full rounded-lg bg-secondary/40 ring-1 ring-border/40 flex items-center justify-center overflow-hidden transition-colors ${assetType === "EVENT_DETAILS_PAGE" ? "aspect-[9/16]" : "aspect-video"} ${dragOverKey === `${loc.id}:${assetType}` ? "ring-2 ring-primary bg-primary/10" : ""}`}
                  >
                    {#if shotState === "COMPLETE" && imgUrl}
                      <img src={imgUrl} alt={label as string} class="w-full h-full object-cover" />
                      <span class="absolute bottom-0 inset-x-0 px-1.5 py-1 text-[10px] text-center text-white bg-black/55 backdrop-blur-sm">
                        {requirementHint(assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE")}
                      </span>
                    {:else if shot}
                      {@const stuck = shotState === "AWAITING_UPLOAD" || shotState === "FAILED"}
                      <div class="flex flex-col items-center gap-1.5 px-2 text-center">
                        {#if stuck}
                          <CircleAlert class="w-4 h-4 text-amber-500" />
                        {:else}
                          <LoaderCircle class="w-4 h-4 animate-spin text-muted-foreground" />
                        {/if}
                        <span class="text-xs text-muted-foreground">
                          {stuck ? t("eventEditor.uploadStuck") : t("eventEditor.processing")}
                        </span>
                        <span class="text-[10px] text-muted-foreground/60 font-mono">{shotState}</span>
                        <div class="flex items-center gap-2 mt-0.5">
                          {#if !stuck}
                            <button onclick={() => refreshShotStatus(loc, assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE")}
                              disabled={refreshingShot[loc.id] === assetType}
                              class="text-[10px] text-primary hover:underline disabled:opacity-50">
                              {refreshingShot[loc.id] === assetType ? t("eventEditor.refreshing") : t("eventEditor.refreshStatus")}
                            </button>
                          {/if}
                          <button onclick={() => discardStuckAsset(loc, assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE", (shot as any).id)}
                            class="text-[10px] text-red-600 hover:underline">
                            {t("eventEditor.discardAndRetry")}
                          </button>
                        </div>
                      </div>
                    {:else if uploading[loc.id] !== assetType}
                      <button onclick={() => uploadAsset(loc, assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE")}
                        class="flex flex-col items-center gap-1 text-xs text-muted-foreground hover:text-foreground px-2 text-center">
                        <ImagePlus class="w-5 h-5" />
                        <span>{t("eventEditor.uploadImage")}</span>
                        <span class="text-[10px] text-muted-foreground/50">{t("eventEditor.orDragDrop")}</span>
                        <span class="text-[10px] text-muted-foreground/70">{requirementHint(assetType as "EVENT_CARD" | "EVENT_DETAILS_PAGE")}</span>
                      </button>
                    {/if}

                    {#if uploading[loc.id] === assetType}
                      <div class="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-background/85 backdrop-blur-sm">
                        <LoaderCircle class="w-6 h-6 animate-spin text-primary" />
                        <span class="text-xs font-medium text-foreground">{t("eventEditor.uploading")}</span>
                      </div>
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

    {#if canSubmit && submitBlockers.length > 0}
      <div class="p-3 rounded-lg text-xs bg-amber-500/10 text-amber-700 space-y-1">
        <p class="font-medium flex items-center gap-1.5"><CircleAlert class="w-3.5 h-3.5 flex-shrink-0" /> {t("eventEditor.notReadyForSubmit")}</p>
        <ul class="list-disc list-inside space-y-0.5 pl-1">
          {#each submitBlockers as blocker (blocker)}
            <li>{blocker}</li>
          {/each}
        </ul>
      </div>
    {/if}

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
      <button onclick={handleSubmitForReview} disabled={submitting || !canSubmit || submitBlockers.length > 0}
        class={`flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold transition-all ${canSubmit && submitBlockers.length === 0 ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}>
        {#if submitting}<LoaderCircle class="w-3.5 h-3.5 animate-spin" />{:else}<Send class="w-3.5 h-3.5" />{/if}
        {t("eventEditor.submitForReview")}
      </button>
    </div>
  </div>
{/if}
