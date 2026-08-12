<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Settings2, LoaderCircle, Check, CircleAlert } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

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
  }

  const EMPTY_FORM: FormState = {
    bundleIdPrefix: "", keystoreAliasPrefix: "", keystorePassword: "", keystoreCommonName: "",
    keystoreOrgUnit: "", keystoreOrgName: "", keystoreLocality: "", keystoreState: "", keystoreCountry: "",
  };

  let configured = $state<boolean | null>(null);
  let showDialog = $state(false);
  let form = $state<FormState>({ ...EMPTY_FORM });
  let saving = $state(false);
  let result = $state<{ success: boolean; message: string } | null>(null);

  async function checkConfig() {
    try {
      const data = await invoke<{ configured: boolean; config: FormState | null }>("workspace_config_get", { workspace: workspacePath });
      configured = data.configured;
      if (data.config) form = { ...EMPTY_FORM, ...data.config };
    } catch {
      configured = false;
    }
  }

  $effect(() => {
    if (workspacePath) checkConfig();
  });

  const canSave = $derived(Object.values(form).every((v) => v.trim().length > 0));

  async function handleSave() {
    saving = true;
    result = null;
    try {
      await invoke("workspace_config_save", { workspace: workspacePath, config: form });
      result = { success: true, message: "Kaydedildi" };
      configured = true;
      setTimeout(() => (showDialog = false), 900);
    } catch (e) {
      result = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }
</script>

<button
  onclick={() => { result = null; showDialog = true; }}
  title="Workspace Ayarları"
  class={`flex items-center gap-2 px-3.5 py-2 rounded-full text-sm font-medium ring-1 transition-all ${
    configured ? "bg-secondary/50 text-foreground ring-border/50 hover:bg-secondary" : "bg-orange-500/10 text-orange-600 ring-orange-500/20 hover:bg-orange-500/20"
  }`}
>
  <Settings2 class="w-4 h-4" />
  <span class="hidden sm:inline">Workspace Ayarları</span>
  {#if configured === false}<span class="w-1.5 h-1.5 rounded-full bg-orange-500"></span>{/if}
</button>

<Dialog bind:open={showDialog}>
  <DialogContent class="sm:max-w-md max-h-[85vh] overflow-y-auto">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Settings2 class="w-5 h-5" />
        Workspace Ayarları
      </DialogTitle>
      <DialogDescription>
        Bu workspace'teki tüm alt-uygulamalar için ortak Bundle ID öneki ve Android keystore imzalama kimliği.
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-3 py-2">
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="bundle-id-prefix">Bundle ID Öneki</label>
        <input id="bundle-id-prefix" bind:value={form.bundleIdPrefix} placeholder="com.sirketiniz" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      </div>
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="keystore-alias-prefix">Keystore Alias Öneki</label>
        <input id="keystore-alias-prefix" bind:value={form.keystoreAliasPrefix} placeholder="app" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      </div>
      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="keystore-password">Keystore Şifresi</label>
        <input id="keystore-password" bind:value={form.keystorePassword} placeholder="güçlü bir şifre" class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      </div>

      <p class="text-xs text-muted-foreground pt-1">Keystore imzalama kimliği (keytool -dname)</p>
      <div class="grid grid-cols-2 gap-2">
        <input bind:value={form.keystoreCommonName} placeholder="Ad Soyad (CN)" class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
        <input bind:value={form.keystoreOrgUnit} placeholder="Birim (OU)" class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
        <input bind:value={form.keystoreOrgName} placeholder="Şirket (O)" class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
        <input bind:value={form.keystoreLocality} placeholder="Şehir (L)" class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
        <input bind:value={form.keystoreState} placeholder="İl/Eyalet (S)" class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
        <input bind:value={form.keystoreCountry} placeholder="Ülke kodu (C, ör. TR)" maxlength={2} class="h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none" />
      </div>

      {#if result}
        <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${result.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}>
          {#if result.success}<Check class="w-4 h-4 flex-shrink-0" />{:else}<CircleAlert class="w-4 h-4 flex-shrink-0" />{/if}
          {result.message}
        </div>
      {/if}
    </div>

    <div class="flex gap-3">
      <Button variant="outline" class="flex-1" onclick={() => (showDialog = false)}>Kapat</Button>
      <Button class="flex-1 gap-2" onclick={handleSave} disabled={saving || !canSave}>
        {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
        Kaydet
      </Button>
    </div>
  </DialogContent>
</Dialog>
