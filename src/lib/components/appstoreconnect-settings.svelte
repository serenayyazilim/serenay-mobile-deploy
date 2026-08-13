<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Check, CircleAlert, KeyRound, Pencil, CalendarClock } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let { workspacePath }: { workspacePath: string } = $props();

  let configured = $state<boolean | null>(null);
  let editing = $state(false);
  let issuerId = $state("");
  let keyId = $state("");
  let privateKey = $state("");
  let saving = $state(false);
  let result = $state<{ success: boolean; message: string } | null>(null);

  async function checkConfig() {
    try {
      const data = await invoke<{ configured: boolean; issuerId?: string; keyId?: string }>("asc_config_get", {
        workspace: workspacePath,
      });
      configured = data.configured;
      if (data.configured) {
        issuerId = data.issuerId || "";
        keyId = data.keyId || "";
      } else {
        editing = true;
      }
    } catch {
      configured = false;
      editing = true;
    }
  }

  $effect(() => {
    if (workspacePath) checkConfig();
  });

  function startEditing() {
    result = null;
    privateKey = "";
    editing = true;
  }

  function cancelEditing() {
    result = null;
    privateKey = "";
    editing = false;
  }

  async function handleSave() {
    saving = true;
    result = null;
    try {
      await invoke("asc_config_save", { workspace: workspacePath, issuerId, keyId, privateKey });
      result = { success: true, message: t("ascSettings.verifiedAndSaved") };
      configured = true;
      privateKey = "";
      editing = false;
    } catch (error) {
      result = { success: false, message: String(error) };
    } finally {
      saving = false;
    }
  }

  async function handleRemove() {
    saving = true;
    try {
      await invoke("asc_config_delete", { workspace: workspacePath });
      configured = false;
      issuerId = "";
      keyId = "";
      privateKey = "";
      result = null;
      editing = true;
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-sm font-semibold">{t("settings.appStoreConnectTab")}</h3>
      <p class="text-xs text-muted-foreground mt-0.5">{t("ascSettings.description")}</p>
    </div>
    {#if configured !== null}
      <span
        class={`shrink-0 flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ring-1 ${
          configured
            ? "bg-green-500/10 text-green-700 dark:text-green-400 ring-green-500/20"
            : "bg-orange-500/10 text-orange-600 ring-orange-500/20"
        }`}
      >
        <span class={`w-1.5 h-1.5 rounded-full ${configured ? "bg-green-500" : "bg-orange-500"}`}></span>
        {configured ? t("ascSettings.connected") : t("ascSettings.notConnected")}
      </span>
    {/if}
  </div>

  {#if configured && !editing}
    <div class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-green-500/20 bg-green-500/10">
      <CalendarClock class="w-4 h-4 shrink-0 text-green-600" />
      <div class="flex-1 min-w-0">
        <p class="text-green-700 dark:text-green-400 font-mono text-xs truncate">Issuer: {issuerId}</p>
        <p class="text-green-700 dark:text-green-400 font-mono text-xs truncate">Key ID: {keyId}</p>
      </div>
      <button
        title={t("common.edit")}
        onclick={startEditing}
        class="shrink-0 p-1.5 rounded-md text-green-700/70 dark:text-green-400/70 hover:text-green-700 dark:hover:text-green-400 hover:bg-green-500/20 transition-colors"
      >
        <Pencil class="w-4 h-4" />
      </button>
    </div>
  {:else}
    <div class="space-y-3">
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="asc-issuer-id">Issuer ID</label>
        <input
          id="asc-issuer-id"
          bind:value={issuerId}
          placeholder="69a6de8f-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
          class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
        />
      </div>
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="asc-key-id">Key ID</label>
        <input
          id="asc-key-id"
          bind:value={keyId}
          placeholder="ABCD123456"
          class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
        />
      </div>
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="asc-private-key">
          {t("ascSettings.privateKey")} {#if configured}<span class="text-green-600">{t("ascSettings.privateKeyStored")}</span>{/if}
        </label>
        <textarea
          id="asc-private-key"
          bind:value={privateKey}
          placeholder={"-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"}
          rows={5}
          class="w-full px-3 py-2 text-xs font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none resize-none"
        ></textarea>
      </div>

      {#if result}
        <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${result.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}>
          {#if result.success}
            <Check class="w-4 h-4 flex-shrink-0" />
          {:else}
            <CircleAlert class="w-4 h-4 flex-shrink-0" />
          {/if}
          {result.message}
        </div>
      {/if}

      <div class="flex gap-3">
        {#if configured}
          <Button variant="outline" class="text-red-600 hover:text-red-700" onclick={handleRemove} disabled={saving}>
            {t("common.remove")}
          </Button>
          <Button variant="outline" onclick={cancelEditing} disabled={saving}>
            {t("common.cancel")}
          </Button>
        {/if}
        <Button class="flex-1 gap-2" onclick={handleSave} disabled={saving || !issuerId || !keyId || (!privateKey && !configured)}>
          {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{:else}<KeyRound class="w-4 h-4" />{/if}
          {t("ascSettings.saveAndVerify")}
        </Button>
      </div>
    </div>
  {/if}
</div>
