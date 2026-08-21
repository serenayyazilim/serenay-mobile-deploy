<script lang="ts">
  import { Sparkles, ImageIcon, Palette, Globe, Flame, Shield, Check, LoaderCircle } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";
  import { CreateProjectWizard } from "./use-create-project.svelte";
  import type { WizardStep } from "./types";
  import ProjectInfo from "./steps/project-info.svelte";
  import AppIcons from "./steps/app-icons.svelte";
  import Splash from "./steps/splash.svelte";
  import Settings from "./steps/settings.svelte";
  import FirebaseStep from "./steps/firebase.svelte";
  import SentryStep from "./steps/sentry.svelte";
  import Summary from "./steps/summary.svelte";

  let { open: dialogOpen = $bindable(false), workspacePath, onProjectCreated }: {
    open: boolean;
    workspacePath: string;
    onProjectCreated?: () => void;
  } = $props();

  const STEPS = $derived([
    { id: 1 as WizardStep, title: t("createProject.stepInfo"), icon: Sparkles },
    { id: 2 as WizardStep, title: t("createProject.stepIcon"), icon: ImageIcon },
    { id: 3 as WizardStep, title: t("createProject.stepSplash"), icon: Palette },
    { id: 4 as WizardStep, title: t("createProject.stepSettings"), icon: Globe },
    { id: 5 as WizardStep, title: t("createProject.stepFirebase"), icon: Flame },
    { id: 6 as WizardStep, title: t("createProject.stepSentry"), icon: Shield },
    { id: 7 as WizardStep, title: t("createProject.stepSummary"), icon: Check },
  ]);

  let wizard = $state(new CreateProjectWizard(workspacePath, onProjectCreated));

  let wasOpen = false;
  $effect(() => {
    if (dialogOpen && !wasOpen) {
      wizard.workspacePath = workspacePath;
      wizard.onProjectCreated = onProjectCreated;
      wizard.reset();
    }
    wasOpen = dialogOpen;
  });

  async function handleFinalCreate() {
    await wizard.handleCreate();
    if (wizard.result?.success) {
      onProjectCreated?.();
      setTimeout(() => (dialogOpen = false), 1200);
    }
  }
</script>

<Dialog bind:open={dialogOpen}>
  <DialogContent class="sm:max-w-lg max-h-[85vh] overflow-y-auto flex flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2"><Sparkles class="w-5 h-5" /> {t("createProject.title")}</DialogTitle>
      <DialogDescription>{t("createProject.description")}</DialogDescription>
    </DialogHeader>

    <div class="flex items-center justify-center gap-1.5 py-2">
      {#each STEPS as s (s.id)}
        {#if s.id < wizard.step}
          <button
            type="button"
            onclick={() => wizard.goToStep(s.id)}
            class="w-2 h-2 rounded-full bg-primary/50 hover:bg-primary transition-colors"
            aria-label={s.title}
          ></button>
        {:else if s.id === wizard.step}
          <div class="w-6 h-2 rounded-full bg-primary"></div>
        {:else}
          <div class="w-2 h-2 rounded-full bg-muted"></div>
        {/if}
      {/each}
    </div>

    <p class="text-center text-xs font-medium text-muted-foreground -mt-1">{STEPS[wizard.step - 1].title}</p>

    <div class="space-y-4 py-2 flex-1">
      {#if wizard.step === 1}
        <ProjectInfo {wizard} />
      {:else if wizard.step === 2}
        <AppIcons {wizard} />
      {:else if wizard.step === 3}
        <Splash {wizard} />
      {:else if wizard.step === 4}
        <Settings {wizard} />
      {:else if wizard.step === 5}
        <FirebaseStep {wizard} />
      {:else if wizard.step === 6}
        <SentryStep {wizard} />
      {:else if wizard.step === 7}
        <Summary {wizard} />
      {/if}
    </div>

    <div class="flex gap-3">
      {#if wizard.step > 1 && !wizard.result}
        <Button variant="outline" class="flex-1" onclick={() => wizard.goBack()}>{t("createProject.back")}</Button>
      {:else if !wizard.result}
        <Button variant="outline" class="flex-1" onclick={() => (dialogOpen = false)}>{t("common.cancel")}</Button>
      {/if}

      {#if wizard.result?.success}
        <Button class="flex-1" onclick={() => (dialogOpen = false)}>{t("createProject.done")}</Button>
      {:else if wizard.result && !wizard.result.success}
        <Button class="flex-1" onclick={() => (wizard.result = null)}>{t("createProject.retry")}</Button>
      {:else if wizard.step < 7}
        <Button class="flex-1" onclick={() => wizard.goNext()} disabled={!wizard.canGoNext()}>{t("createProject.next")}</Button>
      {:else}
        <Button class="flex-1 gap-2" onclick={handleFinalCreate} disabled={wizard.saving}>
          {#if wizard.saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
          {t("createProject.create")}
        </Button>
      {/if}
    </div>
  </DialogContent>
</Dialog>
