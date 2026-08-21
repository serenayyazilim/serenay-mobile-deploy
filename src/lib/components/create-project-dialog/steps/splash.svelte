<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import { inputClass, labelClass } from "../shared";
  import IconPicker from "../icon-picker.svelte";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();
</script>

<div class="space-y-4">
  <IconPicker
    bind:value={wizard.form.splashPath}
    label={t("createProject.splashImage")}
    selectedLabel={t("createProject.selectedSplash")}
  />
  <div class="space-y-1">
    <label class={labelClass} for="cp-bg">{t("createProject.brandColor")}</label>
    <div class="flex items-center gap-2">
      <input id="cp-bg" type="color" bind:value={wizard.form.bgColor} class="w-10 h-10 rounded-xl ring-1 ring-border/50 cursor-pointer bg-secondary/30" />
      <input bind:value={wizard.form.bgColor} class="{inputClass} font-mono w-28" maxlength={7} />
    </div>
  </div>
</div>
