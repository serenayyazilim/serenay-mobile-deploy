<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { LoaderCircle, Check, CircleAlert, ImagePlus, Sparkles } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { getBasicFields } from "$lib/serconf-schema";

  let { open: dialogOpen = $bindable(false), workspacePath, onProjectCreated }: {
    open: boolean;
    workspacePath: string;
    onProjectCreated?: () => void;
  } = $props();

  const basicFields = getBasicFields();

  let projectId = $state("");
  let appName = $state("");
  let bgColor = $state("#FFFFFF");
  let icon512Path = $state<string | null>(null);
  let icon1024Path = $state<string | null>(null);
  let appLogoPath = $state<string | null>(null);
  let splashPath = $state<string | null>(null);
  let firebaseAndroidPath = $state<string | null>(null);
  let firebaseIosPath = $state<string | null>(null);
  let sentryDsn = $state("");
  let serconf = $state<Record<string, string | number | boolean>>({});

  const canSave = $derived(/^[a-z0-9]+$/.test(projectId) && appName.trim().length > 0);

  let saving = $state(false);
  let result = $state<{ success: boolean; message: string } | null>(null);

  function resetForm() {
    projectId = "";
    appName = "";
    bgColor = "#FFFFFF";
    icon512Path = icon1024Path = appLogoPath = splashPath = firebaseAndroidPath = firebaseIosPath = null;
    sentryDsn = "";
    serconf = {};
    result = null;
  }

  $effect(() => {
    if (dialogOpen) resetForm();
  });

  async function pickImage(setter: (path: string) => void) {
    const path = await open({ filters: [{ name: "Görsel", extensions: ["png", "jpg", "jpeg"] }] });
    if (path && !Array.isArray(path)) setter(path);
  }

  async function pickJson(setter: (path: string) => void) {
    const path = await open({ filters: [{ name: "Dosya", extensions: ["json", "plist"] }] });
    if (path && !Array.isArray(path)) setter(path);
  }

  async function handleCreate() {
    if (!canSave) return;
    saving = true;
    result = null;
    try {
      const message = await invoke<string>("project_create", {
        params: {
          workspacePath,
          projectId,
          appName: appName.trim(),
          bgColor,
          icon512Path,
          icon1024Path,
          appLogoPath,
          splashPath,
          serconf,
          firebaseAndroidPath,
          firebaseIosPath,
          sentryDsn: sentryDsn.trim() || null,
          resume: false,
        },
      });
      result = { success: true, message };
      onProjectCreated?.();
      setTimeout(() => (dialogOpen = false), 1200);
    } catch (e) {
      result = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }

  const inputClass = "w-full h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none";
  const labelClass = "text-xs text-muted-foreground";
</script>

<Dialog bind:open={dialogOpen}>
  <DialogContent class="sm:max-w-lg max-h-[85vh] overflow-y-auto">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2"><Sparkles class="w-5 h-5" /> Yeni Alt-Uygulama</DialogTitle>
      <DialogDescription>Bu workspace'e yeni bir white-label alt-uygulama ekle.</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-2">
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label class={labelClass} for="cp-id">Proje ID (küçük harf/rakam) *</label>
          <input id="cp-id" bind:value={projectId} placeholder="myapp" class="{inputClass} font-mono" />
        </div>
        <div class="space-y-1">
          <label class={labelClass} for="cp-name">Uygulama Adı *</label>
          <input id="cp-name" bind:value={appName} placeholder="My App" class={inputClass} />
        </div>
      </div>

      <div class="space-y-1">
        <label class={labelClass} for="cp-bg">Marka Rengi (renk/splash arka planı)</label>
        <div class="flex items-center gap-2">
          <input id="cp-bg" type="color" bind:value={bgColor} class="w-10 h-10 rounded-xl border-0 cursor-pointer bg-transparent" />
          <input bind:value={bgColor} class="{inputClass} font-mono w-28" maxlength={7} />
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <button onclick={() => pickImage((p) => (icon512Path = p))} class="flex flex-col items-center gap-1 p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
          <ImagePlus class="w-5 h-5" />
          {icon512Path ? "512x512 seçildi ✓" : "Android İkon (512x512)"}
        </button>
        <button onclick={() => pickImage((p) => (icon1024Path = p))} class="flex flex-col items-center gap-1 p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
          <ImagePlus class="w-5 h-5" />
          {icon1024Path ? "1024x1024 seçildi ✓" : "iOS İkon (1024x1024)"}
        </button>
        <button onclick={() => pickImage((p) => (appLogoPath = p))} class="flex flex-col items-center gap-1 p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
          <ImagePlus class="w-5 h-5" />
          {appLogoPath ? "Logo seçildi ✓" : "Uygulama İçi Logo (ops.)"}
        </button>
        <button onclick={() => pickImage((p) => (splashPath = p))} class="flex flex-col items-center gap-1 p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
          <ImagePlus class="w-5 h-5" />
          {splashPath ? "Splash seçildi ✓" : "Splash Görseli"}
        </button>
      </div>

      <div class="pt-2 border-t border-border/50 space-y-3">
        <p class="text-xs font-medium text-muted-foreground">Temel Ayarlar</p>
        {#each basicFields as field (field.key)}
          {@const value = serconf[field.key] ?? field.defaultValue}
          <div class="space-y-1">
            <label class={labelClass} for="cp-{field.key}">{field.label}</label>
            {#if field.type === "boolean"}
              <input id="cp-{field.key}" type="checkbox" checked={Boolean(value)}
                onchange={(e) => (serconf = { ...serconf, [field.key]: (e.currentTarget as HTMLInputElement).checked })}
                class="w-5 h-5 rounded accent-primary cursor-pointer" />
            {:else if field.type === "enum"}
              <select id="cp-{field.key}" value={value} onchange={(e) => (serconf = { ...serconf, [field.key]: (e.currentTarget as HTMLSelectElement).value })} class={inputClass}>
                {#each field.enumOptions || [] as opt (opt.value)}<option value={opt.value}>{opt.label}</option>{/each}
              </select>
            {:else}
              <input id="cp-{field.key}" value={value} oninput={(e) => (serconf = { ...serconf, [field.key]: (e.currentTarget as HTMLInputElement).value })} class={inputClass} />
            {/if}
          </div>
        {/each}
      </div>

      <div class="pt-2 border-t border-border/50 space-y-3">
        <p class="text-xs font-medium text-muted-foreground">Firebase &amp; Sentry (opsiyonel)</p>
        <div class="grid grid-cols-2 gap-3">
          <button onclick={() => pickJson((p) => (firebaseAndroidPath = p))} class="flex flex-col items-center gap-1 p-3 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
            {firebaseAndroidPath ? "google-services.json ✓" : "google-services.json"}
          </button>
          <button onclick={() => pickJson((p) => (firebaseIosPath = p))} class="flex flex-col items-center gap-1 p-3 rounded-xl bg-secondary/30 ring-1 ring-border/30 hover:ring-border/60 text-xs text-muted-foreground">
            {firebaseIosPath ? "GoogleService-Info.plist ✓" : "GoogleService-Info.plist"}
          </button>
        </div>
        <div class="space-y-1">
          <label class={labelClass} for="cp-sentry">Sentry DSN</label>
          <input id="cp-sentry" bind:value={sentryDsn} class={inputClass} placeholder="https://...@sentry.io/..." />
        </div>
      </div>

      {#if result}
        <div class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${result.success ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"}`}>
          {#if result.success}<Check class="w-4 h-4 flex-shrink-0" />{:else}<CircleAlert class="w-4 h-4 flex-shrink-0" />{/if}
          {result.message}
        </div>
      {/if}
    </div>

    <div class="flex gap-3">
      <Button variant="outline" class="flex-1" onclick={() => (dialogOpen = false)}>Vazgeç</Button>
      <Button class="flex-1 gap-2" onclick={handleCreate} disabled={saving || !canSave}>
        {#if saving}<LoaderCircle class="w-4 h-4 animate-spin" />{/if}
        Oluştur
      </Button>
    </div>
  </DialogContent>
</Dialog>
