<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Rocket } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";

  type DeployPlatform = "ios" | "android" | "all";

  let { open = $bindable(false), project, workspacePath, onConfirm, onCancel }: {
    open: boolean;
    project: WorkspaceProject | null;
    workspacePath: string;
    onConfirm: (whatsNew: string, platform: DeployPlatform) => void;
    onCancel: () => void;
  } = $props();

  const FLAG_TO_LANG: Record<string, { flag: string; label: string }> = {
    ENGLISH: { flag: "🇺🇸", label: "İngilizce" },
    RUSSIAN: { flag: "🇷🇺", label: "Rusça" },
    FRENCH: { flag: "🇫🇷", label: "Fransızca" },
    ITALIAN: { flag: "🇮🇹", label: "İtalyanca" },
    ARABIC: { flag: "🇸🇦", label: "Arapça" },
    SPANISH: { flag: "🇪🇸", label: "İspanyolca" },
    KAZAKH: { flag: "🇰🇿", label: "Kazakça" },
  };

  let text = $state("");
  let platform = $state<DeployPlatform>("all");
  let languages = $state<{ flag: string; label: string }[]>([]);

  $effect(() => {
    if (!open || !project || !workspacePath) return;
    languages = [];
    invoke<Record<string, string | number | boolean>>("config_serconf_get", { workspace: workspacePath, projectId: project.id })
      .then((config) => {
        const result = [{ flag: "🇹🇷", label: "Türkçe" }];
        for (const [key, meta] of Object.entries(FLAG_TO_LANG)) {
          if (config[key] === true) result.push(meta);
        }
        languages = result;
      })
      .catch(() => {});
  });

  function handleConfirm() {
    onConfirm(text.trim() || "Hata düzeltmeleri ve performans iyileştirmeleri.", platform);
    text = "";
    platform = "all";
  }

  function handleCancel() {
    text = "";
    platform = "all";
    onCancel();
  }

  const platformOptions: { value: DeployPlatform; label: string }[] = [
    { value: "ios", label: "Sadece iOS" },
    { value: "android", label: "Sadece Android" },
    { value: "all", label: "İkisi de" },
  ];
</script>

<Dialog {open} onOpenChange={(o: boolean) => { if (!o) handleCancel(); }}>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Rocket class="w-5 h-5" />
        {project?.appName} — Deploy
      </DialogTitle>
    </DialogHeader>

    <div class="space-y-3 py-1">
      <p class="text-sm text-muted-foreground">
        Bu sürümdeki yenilikleri girin. Metin, mağazada aktif olan dillere otomatik çevrilecek ve her iki store'a gönderilecek.
      </p>

      <textarea
        placeholder="Hata düzeltmeleri ve performans iyileştirmeleri."
        bind:value={text}
        rows={4}
        class="placeholder:text-muted-foreground border-input w-full rounded-md border bg-transparent px-3 py-2 text-sm shadow-xs outline-none resize-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]"
      ></textarea>

      {#if languages.length > 0}
        <div class="rounded-md bg-muted/50 px-3 py-2 space-y-1">
          <p class="text-xs text-muted-foreground font-medium">Tahmini diller (mağazadan deploy sırasında güncellenir):</p>
          <div class="flex flex-wrap gap-x-3 gap-y-1">
            {#each languages as lang (lang.label)}
              <span class="text-xs text-foreground">{lang.flag} {lang.label}</span>
            {/each}
          </div>
        </div>
      {/if}

      <div class="space-y-1.5">
        <p class="text-xs text-muted-foreground font-medium">Platform:</p>
        <div class="flex gap-2">
          {#each platformOptions as opt (opt.value)}
            <button
              type="button"
              onclick={() => (platform = opt.value)}
              class={`flex-1 rounded-md border px-3 py-1.5 text-xs font-medium transition-colors ${
                platform === opt.value ? "border-primary bg-primary text-primary-foreground" : "border-input bg-transparent text-foreground hover:bg-muted"
              }`}
            >
              {opt.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-2 pt-2">
      <Button variant="outline" onclick={handleCancel}>İptal</Button>
      <Button onclick={handleConfirm}>Deploy Başlat</Button>
    </div>
  </DialogContent>
</Dialog>
