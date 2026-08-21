<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import { inputClass, labelClass } from "../shared";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();
</script>

<div class="space-y-4">
  <div class="space-y-1">
    <label class={labelClass} for="cp-id">{t("createProject.projectId")}</label>
    <input
      id="cp-id"
      value={wizard.form.projectId}
      oninput={(e) => wizard.setProjectId((e.currentTarget as HTMLInputElement).value)}
      placeholder="myapp"
      class="{inputClass} font-mono"
    />
    {#if wizard.projectIdError}
      <p class="text-xs text-red-500">{wizard.projectIdError}</p>
    {/if}
  </div>
  <div class="space-y-1">
    <label class={labelClass} for="cp-name">{t("createProject.appName")}</label>
    <input id="cp-name" bind:value={wizard.form.appName} placeholder="My App" class={inputClass} />
  </div>
  {#if wizard.bundleIdPrefix && wizard.form.projectId}
    <p class="text-xs text-muted-foreground font-mono">{wizard.bundleId}</p>
  {/if}
</div>
