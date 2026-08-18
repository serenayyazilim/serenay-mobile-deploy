<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ImageIcon, Palette, CircleCheck, CircleAlert, Flame, FileJson, FileText } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import AssetUploadCard from "./asset-upload-card.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";

  let { project, workspacePath, generic }: {
    project: WorkspaceProject;
    workspacePath: string;
    generic: boolean;
  } = $props();

  let uploadingAsset = $state<string | null>(null);
  let assetResult = $state<{ success: boolean; message: string } | null>(null);
  let refreshKey = $state(0);
  let previews = $state<Record<string, string | null>>({});

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

  async function handleUpload(assetType: string, file: File) {
    uploadingAsset = assetType;
    assetResult = null;
    try {
      const buffer = new Uint8Array(await file.arrayBuffer());
      await invoke("project_asset_upload", { workspace: workspacePath, projectId: project.id, assetType, data: Array.from(buffer) });
      assetResult = { success: true, message: t("assetsTab.uploaded") };
      refreshKey++;
    } catch (e) {
      assetResult = { success: false, message: String(e) };
    } finally {
      uploadingAsset = null;
    }
  }
</script>

<div class="space-y-3">
  {#if !generic}
    <AssetUploadCard
      label={t("assetsTab.logo")}
      description={t("assetsTab.logoDescription")}
      icon={ImageIcon}
      preview={previews.logo}
      uploading={uploadingAsset === "logo"}
      onUpload={(f) => handleUpload("logo", f)}
    />
  {/if}

  <AssetUploadCard
    label={t("assetsTab.splash")}
    description={t("assetsTab.splashDescription")}
    icon={Palette}
    preview={previews.splash}
    uploading={uploadingAsset === "splash"}
    onUpload={(f) => handleUpload("splash", f)}
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
        preview={previews.icon}
        uploading={uploadingAsset === "icon"}
        onUpload={(f) => handleUpload("icon", f)}
      />
    {:else}
      <div class="grid grid-cols-2 gap-3">
        <AssetUploadCard
          label="Android"
          description="512x512"
          icon={ImageIcon}
          preview={previews.icon512}
          uploading={uploadingAsset === "icon512"}
          onUpload={(f) => handleUpload("icon512", f)}
        />
        <AssetUploadCard
          label="iOS"
          description="1024x1024"
          icon={ImageIcon}
          preview={previews.icon1024}
          uploading={uploadingAsset === "icon1024"}
          onUpload={(f) => handleUpload("icon1024", f)}
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
        uploading={uploadingAsset === "firebaseAndroid"}
        onUpload={(f) => handleUpload("firebaseAndroid", f)}
        accept=".json"
      />
      <AssetUploadCard
        label="iOS"
        description="GoogleService-Info.plist"
        icon={FileText}
        uploading={uploadingAsset === "firebaseIos"}
        onUpload={(f) => handleUpload("firebaseIos", f)}
        accept=".plist"
      />
    </div>
  </div>

  {#if assetResult}
    <div class={`flex items-center gap-2 px-4 py-3 rounded-xl text-sm ${assetResult.success ? "bg-green-500/10 text-green-600 ring-1 ring-green-500/20" : "bg-red-500/10 text-red-600 ring-1 ring-red-500/20"}`}>
      {#if assetResult.success}<CircleCheck class="w-4 h-4 shrink-0" />{:else}<CircleAlert class="w-4 h-4 shrink-0" />{/if}
      {assetResult.message}
    </div>
  {/if}
</div>
