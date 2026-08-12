<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CalendarClock, LoaderCircle, Check, CircleAlert, KeyRound } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  let { workspacePath }: { workspacePath: string } = $props();

  let configured = $state<boolean | null>(null);
  let showDialog = $state(false);
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
      }
    } catch {
      configured = false;
    }
  }

  $effect(() => {
    if (workspacePath) checkConfig();
  });

  async function handleSave() {
    saving = true;
    result = null;
    try {
      await invoke("asc_config_save", { workspace: workspacePath, issuerId, keyId, privateKey });
      result = { success: true, message: "Bağlantı doğrulandı ve kaydedildi" };
      configured = true;
      privateKey = "";
      setTimeout(() => (showDialog = false), 900);
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
    } finally {
      saving = false;
    }
  }
</script>

<button
  onclick={() => {
    result = null;
    showDialog = true;
  }}
  title="App Store Connect API"
  class={`flex items-center gap-2 px-3.5 py-2 rounded-full text-sm font-medium ring-1 transition-all ${
    configured
      ? "bg-green-500/10 text-green-700 dark:text-green-400 ring-green-500/20 hover:bg-green-500/20"
      : "bg-orange-500/10 text-orange-600 ring-orange-500/20 hover:bg-orange-500/20"
  }`}
>
  <CalendarClock class="w-4 h-4" />
  <span class="hidden sm:inline">App Store Connect</span>
  {#if configured === false}
    <span class="w-1.5 h-1.5 rounded-full bg-orange-500"></span>
  {/if}
</button>

<Dialog bind:open={showDialog}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <KeyRound class="w-5 h-5" />
        App Store Connect API
      </DialogTitle>
      <DialogDescription>
        In-App Events (uygulama içi etkinlik) yönetimi için App Store Connect API anahtarınızı girin.
        Users and Access &gt; Integrations &gt; App Store Connect API üzerinden oluşturabilirsiniz.
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-3 py-2">
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
          Private Key (.p8 içeriği) {#if configured}<span class="text-green-600">— kayıtlı, değiştirmek için yapıştırın</span>{/if}
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
    </div>

    <div class="flex gap-3">
      {#if configured}
        <Button variant="outline" class="text-red-600 hover:text-red-700" onclick={handleRemove} disabled={saving}>
          Kaldır
        </Button>
      {/if}
      <Button variant="outline" class="flex-1" onclick={() => (showDialog = false)}>Kapat</Button>
      <Button class="flex-1 gap-2" onclick={handleSave} disabled={saving || !issuerId || !keyId || (!privateKey && !configured)}>
        {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
        Kaydet ve Doğrula
      </Button>
    </div>
  </DialogContent>
</Dialog>
