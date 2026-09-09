<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, CircleAlert, Calendar, Clock, Timer } from "@lucide/svelte";
  import {
    BADGE_OPTIONS, PURPOSE_OPTIONS, PRIORITY_OPTIONS, PURCHASE_REQUIREMENT_OPTIONS, LOCALE_SUGGESTIONS, fromDatetimeLocal,
    combineDateTime, MIN_EVENT_DURATION_MS, MAX_EVENT_DURATION_MS,
  } from "$lib/appstoreconnect/labels";
  import { t } from "$lib/i18n/index.svelte";

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

  let { workspacePath, bundleId, onCreated, onCancel }: {
    workspacePath: string;
    bundleId: string;
    onCreated: (eventId: string) => void;
    onCancel: () => void;
  } = $props();

  const inputClass = "w-full h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none";
  const labelClass = "text-xs text-muted-foreground";

  let referenceName = $state("");
  let badge = $state("SPECIAL_EVENT");
  let purpose = $state("APPROPRIATE_FOR_ALL_USERS");
  let priority = $state("NORMAL");
  let deepLink = $state("");
  let purchaseRequirement = $state("");
  let primaryLocale = $state("tr");
  let territories = $state("USA");
  let publishStartDate = $state("");
  let publishStartTime = $state("");
  let eventStartDate = $state("");
  let eventStartTime = $state("");
  let eventEndDate = $state("");
  let eventEndTime = $state("");
  let name = $state("");
  let shortDescription = $state("");
  let longDescription = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);

  const publishStart = $derived(combineDateTime(publishStartDate, publishStartTime));
  const eventStart = $derived(combineDateTime(eventStartDate, eventStartTime));
  const eventEnd = $derived(combineDateTime(eventEndDate, eventEndTime));

  const durationMs = $derived.by(() => {
    if (!eventStart || !eventEnd) return null;
    const start = new Date(eventStart).getTime();
    const end = new Date(eventEnd).getTime();
    if (isNaN(start) || isNaN(end)) return null;
    return end - start;
  });

  const durationError = $derived(
    durationMs === null
      ? null
      : durationMs < MIN_EVENT_DURATION_MS
        ? t("eventForm.durationTooShort")
        : durationMs > MAX_EVENT_DURATION_MS
          ? t("eventForm.durationTooLong")
          : null
  );

  const canSave = $derived(
    !!(referenceName.trim() && primaryLocale.trim() && territories.trim() && eventStart && eventEnd && publishStart && name.trim() && !durationError)
  );

  async function handleSubmit() {
    if (!canSave) return;
    saving = true;
    error = null;
    try {
      const result = await invoke<{ event: { id: string } }>("asc_event_create", {
        workspace: workspacePath,
        bundleId,
        attributes: {
          referenceName: referenceName.trim(),
          badge,
          purpose,
          priority,
          deepLink: deepLink.trim() || null,
          purchaseRequirement: purchaseRequirement || null,
          primaryLocale: primaryLocale.trim(),
          territorySchedules: [
            {
              territories: territories.split(",").map((t) => t.trim().toUpperCase()).filter(Boolean),
              eventStart: fromDatetimeLocal(eventStart),
              eventEnd: fromDatetimeLocal(eventEnd),
              publishStart: fromDatetimeLocal(publishStart),
            },
          ],
        },
        primaryLocalization: {
          locale: primaryLocale.trim(),
          name: name.trim(),
          shortDescription: shortDescription.trim() || undefined,
          longDescription: longDescription.trim() || undefined,
        },
      });
      onCreated(result.event.id);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-4">
  <p class="text-sm font-medium">{t("eventForm.newEvent")}</p>

  <div class="space-y-1">
    <label class={labelClass} for="ref-name">{t("eventForm.referenceName")}</label>
    <input id="ref-name" bind:value={referenceName} class={inputClass} placeholder="summer-sale-2026" />
  </div>

  <div class="grid grid-cols-2 gap-3">
    <div class="space-y-1">
      <label class={labelClass} for="badge">{t("eventForm.badge")}</label>
      <select id="badge" bind:value={badge} class={inputClass}>
        {#each BADGE_OPTIONS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
      </select>
    </div>
    <div class="space-y-1">
      <label class={labelClass} for="priority">{t("eventForm.priority")}</label>
      <select id="priority" bind:value={priority} class={inputClass}>
        {#each PRIORITY_OPTIONS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
      </select>
    </div>
  </div>

  <div class="space-y-1">
    <label class={labelClass} for="purpose">{t("eventForm.purpose")}</label>
    <select id="purpose" bind:value={purpose} class={inputClass}>
      {#each PURPOSE_OPTIONS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
    </select>
  </div>

  <div class="grid grid-cols-2 gap-3">
    <div class="space-y-1">
      <label class={labelClass} for="primary-locale">{t("eventForm.primaryLocale")}</label>
      <input id="primary-locale" bind:value={primaryLocale} class={inputClass} list="asc-locale-suggestions" />
      <datalist id="asc-locale-suggestions">
        {#each LOCALE_SUGGESTIONS as l (l)}<option value={l}></option>{/each}
      </datalist>
    </div>
    <div class="space-y-1">
      <label class={labelClass} for="territories">{t("eventForm.territories")}</label>
      <input id="territories" bind:value={territories} class={inputClass} />
    </div>
  </div>

  <div class="space-y-1">
    <label class={labelClass} for="deep-link">{t("eventForm.deepLink")}</label>
    <input id="deep-link" bind:value={deepLink} class={inputClass} placeholder="myapp://event/summer" />
  </div>

  <div class="space-y-1">
    <label class={labelClass} for="purchase-req">{t("eventForm.purchaseRequirement")}</label>
    <select id="purchase-req" bind:value={purchaseRequirement} class={inputClass}>
      <option value="">{t("eventForm.purchaseRequirementUnset")}</option>
      {#each PURCHASE_REQUIREMENT_OPTIONS as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
    </select>
    <p class="text-[10px] text-muted-foreground/70">{t("eventForm.purchaseRequirementHint")}</p>
  </div>

  <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
    <p class="text-xs font-medium text-muted-foreground">{t("eventForm.scheduling")}</p>

    <div class="space-y-1">
      <label class={labelClass} for="publish-start-date">{t("eventForm.publishStart")}</label>
      <div class="grid grid-cols-[1fr_120px] gap-2">
        <div class="relative">
          <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="publish-start-date" type="date" bind:value={publishStartDate} class={`${inputClass} pl-8`} />
        </div>
        <div class="relative">
          <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="publish-start-time" type="time" bind:value={publishStartTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.publishStart")} />
        </div>
      </div>
    </div>

    <div class="space-y-1">
      <label class={labelClass} for="event-start-date">{t("eventForm.eventStart")}</label>
      <div class="grid grid-cols-[1fr_120px] gap-2">
        <div class="relative">
          <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="event-start-date" type="date" bind:value={eventStartDate} class={`${inputClass} pl-8`} />
        </div>
        <div class="relative">
          <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="event-start-time" type="time" bind:value={eventStartTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.eventStart")} />
        </div>
      </div>
    </div>

    <div class="space-y-1">
      <label class={labelClass} for="event-end-date">{t("eventForm.eventEnd")}</label>
      <div class="grid grid-cols-[1fr_120px] gap-2">
        <div class="relative">
          <Calendar class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="event-end-date" type="date" bind:value={eventEndDate} min={eventStartDate || undefined} class={`${inputClass} pl-8`} />
        </div>
        <div class="relative">
          <Clock class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
          <input id="event-end-time" type="time" bind:value={eventEndTime} class={`${inputClass} pl-8`} aria-label={t("eventForm.eventEnd")} />
        </div>
      </div>
    </div>

    {#if durationMs !== null}
      <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs ${durationError ? "bg-red-500/10 text-red-600" : "bg-primary/10 text-primary"}`}>
        {#if durationError}<CircleAlert class="w-3.5 h-3.5 flex-shrink-0" />{:else}<Timer class="w-3.5 h-3.5 flex-shrink-0" />{/if}
        <span>
          {#if durationError}
            {durationError}
          {:else}
            {t("eventForm.durationLabel", { duration: formatDuration(durationMs) })}
          {/if}
        </span>
      </div>
    {/if}
  </div>

  <div class="pt-2 border-t border-border/50 space-y-3">
    <p class="text-xs font-medium text-muted-foreground">{t("eventForm.storeTextForPrimaryLocale")}</p>
    <div class="space-y-1">
      <label class={labelClass} for="loc-name">{t("eventForm.headline")}</label>
      <input id="loc-name" bind:value={name} class={inputClass} maxlength={30} />
    </div>
    <div class="space-y-1">
      <label class={labelClass} for="loc-short">{t("eventForm.shortDescription")}</label>
      <input id="loc-short" bind:value={shortDescription} class={inputClass} maxlength={50} />
    </div>
    <div class="space-y-1">
      <label class={labelClass} for="loc-long">{t("eventForm.longDescription")}</label>
      <textarea id="loc-long" bind:value={longDescription} rows={3} maxlength={500}
        class="w-full px-3 py-2 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none resize-none"></textarea>
    </div>
  </div>

  {#if error}
    <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-600">
      <CircleAlert class="w-4 h-4 flex-shrink-0" />
      {error}
    </div>
  {/if}

  <div class="flex gap-2 justify-end pt-2">
    <button onclick={onCancel} class="px-4 py-2 rounded-full text-xs font-medium text-muted-foreground hover:bg-secondary transition-colors">
      {t("common.cancel")}
    </button>
    <button
      onclick={handleSubmit}
      disabled={saving || !canSave}
      class={`flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold transition-all ${canSave ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}
    >
      {#if saving}<LoaderCircle class="w-3 h-3 animate-spin" /> {t("eventForm.creating")}{:else}{t("eventForm.createEvent")}{/if}
    </button>
  </div>
</div>
