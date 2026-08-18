<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Tag, Check, CircleAlert } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";

  let { project, workspacePath, onVersionSaved }: {
    project: WorkspaceProject;
    workspacePath: string;
    onVersionSaved?: () => void;
  } = $props();

  function parseParts(v: string) {
    const [semver = "", build = ""] = v.split("+");
    const [major = "0", minor = "0", patch = "0"] = semver.split(".");
    return { major, minor, patch, build };
  }

  function compose(major: string, minor: string, patch: string, build: string) {
    const b = build.trim();
    return b ? `${major}.${minor}.${patch}+${b}` : `${major}.${minor}.${patch}`;
  }

  function autoBuild(major: string, minor: string, patch: string) {
    const ma = parseInt(major) || 0;
    const mi = parseInt(minor) || 0;
    const pa = parseInt(patch) || 0;
    return String(ma * 10_000_000 + mi * 100_000 + pa * 1_000);
  }

  let loading = $state(true);
  let saving = $state(false);
  let saveResult = $state<{ success: boolean; message: string } | null>(null);
  let original = $state("");
  let major = $state("1");
  let minor = $state("0");
  let patch = $state("0");
  let build = $state("");

  async function load() {
    loading = true;
    saveResult = null;
    try {
      const data = await invoke<{ versions: Record<string, string> }>("projects_versions", { workspace: workspacePath });
      const v = data.versions?.[project.id] || "1.0.0+1";
      original = v;
      const p = parseParts(v);
      major = p.major;
      minor = p.minor;
      patch = p.patch;
      build = p.build || autoBuild(p.major, p.minor, p.patch);
    } catch {
      original = "1.0.0+1";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    project.id;
    workspacePath;
    load();
  });

  function updateVersion(ma: string, mi: string, pa: string) {
    build = autoBuild(ma, mi, pa);
  }

  const current = $derived(compose(major, minor, patch, build));
  const hasChanges = $derived(current !== original);

  async function handleSave() {
    saving = true;
    saveResult = null;
    try {
      await invoke("projects_version_set", { workspace: workspacePath, projectId: project.id, version: current });
      original = current;
      saveResult = { success: true, message: `${t("versionTab.saved")}: ${current}` };
      onVersionSaved?.();
    } catch (e) {
      saveResult = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-20"><LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" /></div>
{:else}
  <div class="space-y-4">
    <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 flex items-center gap-3">
      <Tag class="w-4 h-4 text-muted-foreground shrink-0" />
      <div>
        <p class="text-xs text-muted-foreground">{t("versionTab.currentVersion")}</p>
        <p class="text-sm font-mono font-semibold">{original}</p>
      </div>
    </div>

    <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30 space-y-4">
      <p class="text-sm font-medium">{t("versionTab.editVersion")}</p>

      <div class="space-y-1">
        <p class="text-xs text-muted-foreground">{t("versionTab.versionNumber")}</p>
        <div class="flex items-center gap-2">
          <div class="flex-1">
            <label class="text-[10px] text-muted-foreground mb-1 block" for="version-major">Major</label>
            <input
              id="version-major"
              type="number"
              min="0"
              value={major}
              oninput={(e) => { major = e.currentTarget.value; updateVersion(major, minor, patch); saveResult = null; }}
              class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
            />
          </div>
          <span class="text-muted-foreground mt-5">.</span>
          <div class="flex-1">
            <label class="text-[10px] text-muted-foreground mb-1 block" for="version-minor">Minor</label>
            <input
              id="version-minor"
              type="number"
              min="0"
              value={minor}
              oninput={(e) => { minor = e.currentTarget.value; updateVersion(major, minor, patch); saveResult = null; }}
              class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
            />
          </div>
          <span class="text-muted-foreground mt-5">.</span>
          <div class="flex-1">
            <label class="text-[10px] text-muted-foreground mb-1 block" for="version-patch">Patch</label>
            <input
              id="version-patch"
              type="number"
              min="0"
              value={patch}
              oninput={(e) => { patch = e.currentTarget.value; updateVersion(major, minor, patch); saveResult = null; }}
              class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
            />
          </div>
        </div>
      </div>

      <div class="space-y-1">
        <label class="text-xs text-muted-foreground" for="version-build">{t("versionTab.buildNumber")}</label>
        <input
          id="version-build"
          type="text"
          value={build}
          oninput={(e) => { build = e.currentTarget.value; saveResult = null; }}
          placeholder="1000"
          class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
        />
      </div>

      <div class="flex items-center justify-between pt-1">
        <div class="text-xs text-muted-foreground">
          {t("versionTab.preview")}:
          <span class={`font-mono font-semibold ${hasChanges ? "text-amber-600" : "text-foreground"}`}>{current}</span>
        </div>
        <button
          onclick={handleSave}
          disabled={saving || !hasChanges}
          class={`flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold transition-all ${hasChanges ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-secondary text-muted-foreground cursor-not-allowed"}`}
        >
          {#if saving}<LoaderCircle class="w-3 h-3 animate-spin" />{/if}
          {t("common.save")}
        </button>
      </div>
    </div>

    {#if saveResult}
      <div class={`flex items-center gap-2 px-4 py-3 rounded-xl text-sm ${saveResult.success ? "bg-green-500/10 text-green-600 ring-1 ring-green-500/20" : "bg-red-500/10 text-red-600 ring-1 ring-red-500/20"}`}>
        {#if saveResult.success}<Check class="w-4 h-4 shrink-0" />{:else}<CircleAlert class="w-4 h-4 shrink-0" />{/if}
        {saveResult.message}
      </div>
    {/if}

    <p class="text-xs text-muted-foreground px-1">{t("versionTab.hint")}</p>
  </div>
{/if}
