<script lang="ts">
  import { LoaderCircle, Check, CircleAlert, TriangleAlert } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { inputClass, labelClass } from "../shared";
  import ModeToggle from "../mode-toggle.svelte";
  import IconPicker from "../icon-picker.svelte";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();

  $effect(() => {
    if (wizard.form.firebaseAutoCreate) wizard.checkFirebaseLogin();
  });
</script>

<div class="space-y-4">
  <ModeToggle
    bind:checked={wizard.form.firebaseAutoCreate}
    onLabel={t("createProject.autoCreate")}
    offLabel={t("createProject.manual")}
  />

  {#if wizard.form.firebaseAutoCreate}
    {#if wizard.firebaseChecking}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-secondary/40 text-muted-foreground">
        <LoaderCircle class="w-4 h-4 animate-spin" />
        {t("createProject.firebaseChecking")}
      </div>
    {:else if wizard.firebaseLoggedIn === false}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-amber-500/10 text-amber-600">
        <TriangleAlert class="w-4 h-4 flex-shrink-0" />
        {t("createProject.firebaseNotLoggedIn")}
      </div>
    {:else if wizard.firebaseLoggedIn === true}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-green-500/10 text-green-600">
        <Check class="w-4 h-4 flex-shrink-0" />
        {t("createProject.firebaseLoggedIn")}
      </div>
    {/if}

    <div class="rounded-xl bg-secondary/30 ring-1 ring-border/30 p-3 space-y-1 text-xs text-muted-foreground">
      <p>{t("createProject.firebaseProjectPreview")}: <span class="font-mono">sermobilepro-{wizard.form.projectId}</span></p>
      <p>{t("createProject.bundleId")}: <span class="font-mono">{wizard.bundleId}</span></p>
    </div>

    <button
      type="button"
      onclick={() => wizard.handleFirebaseAutoCreate()}
      disabled={wizard.firebaseLoading || wizard.firebaseLoggedIn === false || !wizard.form.projectId}
      class="w-full h-9 px-4 rounded-lg bg-primary text-primary-foreground text-sm font-medium disabled:opacity-50 flex items-center justify-center gap-2"
    >
      {#if wizard.firebaseLoading}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
      {t("createProject.firebaseCreateButton")}
    </button>

    {#if wizard.firebaseSteps.length > 0}
      <div class="space-y-1">
        {#each wizard.firebaseSteps as s (s.step)}
          <div class="flex items-start gap-2 text-xs">
            {#if s.success}
              <Check class="w-3.5 h-3.5 text-green-600 flex-shrink-0 mt-0.5" />
            {:else}
              <CircleAlert class="w-3.5 h-3.5 text-red-500 flex-shrink-0 mt-0.5" />
            {/if}
            <span class="text-muted-foreground">{s.step}: {s.message}</span>
          </div>
        {/each}
      </div>
    {/if}

    {#if wizard.firebaseError}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-600">
        <CircleAlert class="w-4 h-4 flex-shrink-0" />
        {wizard.firebaseError}
      </div>
    {/if}
  {:else}
    <div class="grid grid-cols-2 gap-3">
      <IconPicker
        bind:value={wizard.form.firebaseAndroidPath}
        label="google-services.json"
        selectedLabel="google-services.json ✓"
        extensions={["json"]}
      />
      <IconPicker
        bind:value={wizard.form.firebaseIosPath}
        label="GoogleService-Info.plist"
        selectedLabel="GoogleService-Info.plist ✓"
        extensions={["plist"]}
      />
    </div>
  {/if}
</div>
