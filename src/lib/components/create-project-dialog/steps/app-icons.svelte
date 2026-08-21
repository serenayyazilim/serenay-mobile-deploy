<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import IconPicker from "../icon-picker.svelte";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();
</script>

<div class="grid grid-cols-2 gap-3">
  <IconPicker
    bind:value={wizard.form.icon512Path}
    label={t("createProject.androidIcon")}
    selectedLabel={t("createProject.selected512")}
  />
  <IconPicker
    bind:value={wizard.form.icon1024Path}
    label={t("createProject.iosIcon")}
    selectedLabel={t("createProject.selected1024")}
  />
  <IconPicker
    bind:value={wizard.form.appLogoPath}
    label={t("createProject.inAppLogo")}
    selectedLabel={t("createProject.selectedLogo")}
  />
</div>
