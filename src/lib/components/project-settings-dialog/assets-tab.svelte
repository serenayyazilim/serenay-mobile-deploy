<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ImageIcon, Palette, Flame, FileJson, FileText } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import AssetUploadCard from "./asset-upload-card.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";

  let {
    project,
    workspacePath,
    generic,
    hasChanges = $bindable(false),
    saving = $bindable(false),
    result = $bindable(null),
  }: {
    project: WorkspaceProject;
    workspacePath: string;
    generic: boolean;
    hasChanges?: boolean;
    saving?: boolean;
    result?: { success: boolean; message: string } | null;
  } = $props();

  let refreshKey = $state(0);
  let previews = $state<Record<string, string | null>>({});
  let pending = $state<Record<string, File>>({});
  let localPreviews = $state<Record<string, string>>({});

  async function loadPreview(iconType: string) {
    try {
      previews[iconType] = await invoke<string | null>("project_icon", { workspace: workspacePath, projectId: project.id, iconType });
    } catch {
      previews[iconType] = null;
    }
  }

  const previewTypes = $derived(generic ? ["icon", "splash"] : ["logo", "splash", "icon512", "icon1024"]);

  $effect(() => {
    refreshKey;
    project.id;
    for (const type of previewTypes) loadPreview(type);
  });

  $effect(() => {
    project.id;
    for (const url of Object.values(localPreviews)) URL.revokeObjectURL(url);
    pending = {};
    localPreviews = {};
    result = null;
  });

  $effect(() => {
    hasChanges = Object.keys(pending).length > 0;
  });

  function handleStage(assetType: string, file: File) {
    if (localPreviews[assetType]) URL.revokeObjectURL(localPreviews[assetType]);
    pending = { ...pending, [assetType]: file };
    localPreviews = { ...localPreviews, [assetType]: URL.createObjectURL(file) };
    result = null;
  }

  export async function save() {
    if (!project || Object.keys(pending).length === 0) return;
    saving = true;
    result = null;
    try {
      for (const [assetType, file] of Object.entries(pending)) {
        const buffer = new Uint8Array(await file.arrayBuffer());
        await invoke("project_asset_upload", { workspace: workspacePath, projectId: project.id, assetType, data: Array.from(buffer) });
      }
      for (const url of Object.values(localPreviews)) URL.revokeObjectURL(url);
      pending = {};
      localPreviews = {};
      refreshKey++;
      result = { success: true, message: t("assetsTab.uploaded") };
    } catch (e) {
      result = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-3">
  {#if !generic}
    <AssetUploadCard
      label={t("assetsTab.logo")}
      description={t("assetsTab.logoDescription")}
      icon={ImageIcon}
      preview={localPreviews.logo ?? previews.logo}
      uploading={saving && "logo" in pending}
      pending={"logo" in pending}
      onUpload={(f) => handleStage("logo", f)}
    />
  {/if}

  <AssetUploadCard
    label={t("assetsTab.splash")}
    description={t("assetsTab.splashDescription")}
    icon={Palette}
    preview={localPreviews.splash ?? previews.splash}
    uploading={saving && "splash" in pending}
    pending={"splash" in pending}
    onUpload={(f) => handleStage("splash", f)}
  />

  <div class="p-4 rounded-2xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
    <div>
      <p class="font-medium text-sm">{t("assetsTab.appIcon")}</p>
      <p class="text-xs text-muted-foreground mt-0.5">{t("assetsTab.appIconHint")}</p>
    </div>

    {#if generic}
      <AssetUploadCard
        label={t("assetsTab.appIcon")}
        description="1024x1024"
        icon={ImageIcon}
        preview={localPreviews.icon ?? previews.icon}
        uploading={saving && "icon" in pending}
        pending={"icon" in pending}
        onUpload={(f) => handleStage("icon", f)}
      />
    {:else}
      <div class="grid grid-cols-2 gap-3">
        <AssetUploadCard
          label="Android"
          description="512x512"
          icon={ImageIcon}
          preview={localPreviews.icon512 ?? previews.icon512}
          uploading={saving && "icon512" in pending}
          pending={"icon512" in pending}
          onUpload={(f) => handleStage("icon512", f)}
        />
        <AssetUploadCard
          label="iOS"
          description="1024x1024"
          icon={ImageIcon}
          preview={localPreviews.icon1024 ?? previews.icon1024}
          uploading={saving && "icon1024" in pending}
          pending={"icon1024" in pending}
          onUpload={(f) => handleStage("icon1024", f)}
        />
      </div>
    {/if}
  </div>

  <div class="p-4 rounded-2xl bg-secondary/30 ring-1 ring-border/30 space-y-3">
    <div class="flex items-center gap-2">
      <Flame class="w-4 h-4 text-orange-500" />
      <div>
        <p class="font-medium text-sm">{t("assetsTab.firebaseConfig")}</p>
        <p class="text-xs text-muted-foreground mt-0.5">{t("assetsTab.firebaseConfigHint")}</p>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <AssetUploadCard
        label="Android"
        description="google-services.json"
        icon={FileJson}
        uploading={saving && "firebaseAndroid" in pending}
        pending={"firebaseAndroid" in pending}
        onUpload={(f) => handleStage("firebaseAndroid", f)}
        accept=".json"
      />
      <AssetUploadCard
        label="iOS"
        description="GoogleService-Info.plist"
        icon={FileText}
        uploading={saving && "firebaseIos" in pending}
        pending={"firebaseIos" in pending}
        onUpload={(f) => handleStage("firebaseIos", f)}
        accept=".plist"
      />
    </div>
  </div>
</div>
