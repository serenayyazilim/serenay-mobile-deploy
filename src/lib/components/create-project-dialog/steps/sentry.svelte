<script lang="ts">
  import { LoaderCircle, Check, CircleAlert, TriangleAlert } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { inputClass, labelClass } from "../shared";
  import ModeToggle from "../mode-toggle.svelte";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();

  $effect(() => {
    if (wizard.form.sentryAutoCreate) wizard.checkSentryAuth();
  });
</script>

<div class="space-y-4">
  <ModeToggle
    bind:checked={wizard.form.sentryAutoCreate}
    onLabel={t("createProject.autoCreate")}
    offLabel={t("createProject.manual")}
  />

  {#if wizard.form.sentryAutoCreate}
    {#if wizard.sentryCheckLoading}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-secondary/40 text-muted-foreground">
        <LoaderCircle class="w-4 h-4 animate-spin" />
        {t("createProject.sentryChecking")}
      </div>
    {:else if wizard.sentryAuth && !wizard.sentryAuth.authenticated}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-amber-500/10 text-amber-600">
        <TriangleAlert class="w-4 h-4 flex-shrink-0" />
        {wizard.sentryAuth.message ?? t("createProject.sentryNotAuthenticated")}
      </div>
    {:else if wizard.sentryAuth?.authenticated}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-green-500/10 text-green-600">
        <Check class="w-4 h-4 flex-shrink-0" />
        {t("createProject.sentryAuthenticated")}
      </div>
    {/if}

    <div class="space-y-1">
      <label class={labelClass} for="cp-sentry-org">{t("createProject.sentryOrgSlug")}</label>
      <input id="cp-sentry-org" bind:value={wizard.form.sentryOrgSlug} class={inputClass} placeholder="my-org" />
    </div>

    <button
      type="button"
      onclick={() => wizard.handleSentryAutoCreate()}
      disabled={wizard.sentryLoading || !wizard.sentryAuth?.authenticated || !wizard.form.sentryOrgSlug || !wizard.form.projectId}
      class="w-full h-9 px-4 rounded-lg bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 flex items-center justify-center gap-2"
    >
      {#if wizard.sentryLoading}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
      {t("createProject.sentryCreateButton")}
    </button>

    {#if wizard.sentryError}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-600">
        <CircleAlert class="w-4 h-4 flex-shrink-0" />
        {wizard.sentryError}
      </div>
    {/if}

    {#if wizard.form.sentryDsn}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-green-500/10 text-green-600 font-mono break-all">
        <Check class="w-4 h-4 flex-shrink-0" />
        {wizard.form.sentryDsn}
      </div>
    {/if}
  {:else}
    <div class="space-y-1">
      <label class={labelClass} for="cp-sentry">Sentry DSN</label>
      <input id="cp-sentry" bind:value={wizard.form.sentryDsn} class={inputClass} placeholder="https://...@sentry.io/..." />
    </div>
  {/if}
</div>
