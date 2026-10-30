<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Check, CircleAlert } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let { workspacePath }: { workspacePath: string } = $props();

  interface FormState {
    bundleIdPrefix: string;
    keystoreAliasPrefix: string;
    keystorePassword: string;
    keystoreCommonName: string;
    keystoreOrgUnit: string;
    keystoreOrgName: string;
    keystoreLocality: string;
    keystoreState: string;
    keystoreCountry: string;
    minVersion: string;
  }

  const EMPTY_FORM: FormState = {
    bundleIdPrefix: "", keystoreAliasPrefix: "", keystorePassword: "", keystoreCommonName: "",
    keystoreOrgUnit: "", keystoreOrgName: "", keystoreLocality: "", keystoreState: "", keystoreCountry: "", minVersion: "19.0.0",
  };

  let form = $state<FormState>({ ...EMPTY_FORM });
  let saving = $state(false);
  let result = $state<{ success: boolean; message: string } | null>(null);

  async function checkConfig() {
    try {
      const data = await invoke<{ configured: boolean; config: FormState | null }>("workspace_config_get", { workspace: workspacePath });
      if (data.config) form = { ...EMPTY_FORM, ...data.config };
    } catch {
      // no config saved yet
    }
  }

  $effect(() => {
    if (workspacePath) checkConfig();
  });

  // The minimum version is the only optional field: leaving it empty turns it off.
  const canSave = $derived(Object.entries(form).every(([key, v]) => key === "minVersion" || v.trim().length > 0));

  async function handleSave() {
    saving = true;
    result = null;
    try {
      await invoke("workspace_config_save", { workspace: workspacePath, config: { ...form, minVersion: form.minVersion.trim() } });
      result = { success: true, message: t("projectSettings.saved") };
    } catch (e) {
      result = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-sm font-semibold">{t("workspaceSettings.title")}</h3>
    <p class="text-xs text-muted-foreground mt-0.5">{t("workspaceSettings.description")}</p>
  </div>

  <div class="space-y-3">
    <div class="space-y-1">
      <label class="text-xs text-muted-foreground" for="bundle-id-prefix">{t("workspaceSettings.bundleIdPrefix")}</label>
      <input id="bundle-id-prefix" bind:value={form.bundleIdPrefix} placeholder="com.yourcompany" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
    </div>
    <div class="space-y-1">
      <label class="text-xs text-muted-foreground" for="keystore-alias-prefix">{t("workspaceSettings.keystoreAliasPrefix")}</label>
      <input id="keystore-alias-prefix" bind:value={form.keystoreAliasPrefix} placeholder="app" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
    </div>
    <div class="space-y-1">
      <label class="text-xs text-muted-foreground" for="keystore-password">{t("workspaceSettings.keystorePassword")}</label>
      <input id="keystore-password" bind:value={form.keystorePassword} placeholder={t("workspaceSettings.strongPassword")} class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
    </div>

    <p class="text-xs text-muted-foreground pt-1">{t("workspaceSettings.signingIdentity")}</p>
    <div class="grid grid-cols-2 gap-2">
      <input bind:value={form.keystoreCommonName} placeholder={t("workspaceSettings.fullName")} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <input bind:value={form.keystoreOrgUnit} placeholder={t("workspaceSettings.orgUnit")} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <input bind:value={form.keystoreOrgName} placeholder={t("workspaceSettings.orgName")} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <input bind:value={form.keystoreLocality} placeholder={t("workspaceSettings.locality")} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <input bind:value={form.keystoreState} placeholder={t("workspaceSettings.state")} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <input bind:value={form.keystoreCountry} placeholder={t("workspaceSettings.country")} maxlength={2} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
    </div>

    <div class="space-y-1 pt-1">
      <label class="text-xs text-muted-foreground" for="min-version">{t("workspaceSettings.minVersion")}</label>
      <input id="min-version" bind:value={form.minVersion} placeholder="19.0.0" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      <p class="text-xs text-muted-foreground/80">{t("workspaceSettings.minVersionHint")}</p>
    </div>

    {#if result}
      <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${result.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}>
        {#if result.success}<Check class="w-4 h-4 flex-shrink-0" />{:else}<CircleAlert class="w-4 h-4 flex-shrink-0" />{/if}
        {result.message}
      </div>
    {/if}
  </div>

  <Button class="w-full gap-2" onclick={handleSave} disabled={saving || !canSave}>
    {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
    {t("common.save")}
  </Button>
</div>
