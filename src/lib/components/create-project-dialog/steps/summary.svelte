<script lang="ts">
  import { Check, CircleAlert } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();
</script>

{#if wizard.result}
  <div
    class={`flex items-center gap-2 px-3 py-3 rounded-lg text-sm ${wizard.result.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}
  >
    {#if wizard.result.success}<Check class="w-4 h-4 flex-shrink-0" />{:else}<CircleAlert class="w-4 h-4 flex-shrink-0" />{/if}
    {wizard.result.message}
  </div>
{:else}
  <div class="space-y-3">
    <div class="rounded-xl bg-secondary/30 ring-1 ring-border/30 p-4 flex items-center gap-3">
      <div class="w-12 h-12 rounded-xl" style={`background-color: ${wizard.form.bgColor}`}></div>
      <div>
        <p class="text-sm font-medium">{wizard.form.appName}</p>
        <p class="text-xs text-muted-foreground font-mono">{wizard.bundleId}</p>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-2 text-xs">
      <div class="rounded-lg bg-secondary/30 ring-1 ring-border/30 p-2">
        <p class={"text-muted-foreground"}>{t("createProject.projectId")}</p>
        <p class="font-mono">{wizard.form.projectId}</p>
      </div>
      <div class="rounded-lg bg-secondary/30 ring-1 ring-border/30 p-2">
        <p class="text-muted-foreground">API URL</p>
        <p class="font-mono truncate">{wizard.form.serconf["API_URL"] ?? "-"}</p>
      </div>
      <div class="rounded-lg bg-secondary/30 ring-1 ring-border/30 p-2">
        <p class="text-muted-foreground">Firebase</p>
        <p>{wizard.form.firebaseCreated || wizard.form.firebaseAndroidPath ? "✓" : "-"}</p>
      </div>
      <div class="rounded-lg bg-secondary/30 ring-1 ring-border/30 p-2">
        <p class="text-muted-foreground">Sentry</p>
        <p>{wizard.form.sentryDsn ? "✓" : "-"}</p>
      </div>
    </div>
  </div>
{/if}
