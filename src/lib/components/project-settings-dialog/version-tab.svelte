<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { LoaderCircle, Tag } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";

  let {
    project,
    workspacePath,
    onVersionSaved,
    hasChanges = $bindable(false),
    saving = $bindable(false),
    result = $bindable(null),
  }: {
    project: WorkspaceProject;
    workspacePath: string;
    onVersionSaved?: () => void;
    hasChanges?: boolean;
    saving?: boolean;
    result?: { success: boolean; message: string } | null;
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
  let original = $state("");
  let major = $state("1");
  let minor = $state("0");
  let patch = $state("0");
  let build = $state("");

  async function load() {
    loading = true;
    result = null;
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

  $effect(() => {
    hasChanges = current !== original;
  });

  export async function save() {
    saving = true;
    result = null;
    try {
      await invoke("projects_version_set", { workspace: workspacePath, projectId: project.id, version: current });
      original = current;
      result = { success: true, message: `${t("versionTab.saved")}: ${current}` };
      onVersionSaved?.();
    } catch (e) {
      result = { success: false, message: String(e) };
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
              oninput={(e) => { major = e.currentTarget.value; updateVersion(major, minor, patch); result = null; }}
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
              oninput={(e) => { minor = e.currentTarget.value; updateVersion(major, minor, patch); result = null; }}
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
              oninput={(e) => { patch = e.currentTarget.value; updateVersion(major, minor, patch); result = null; }}
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
          oninput={(e) => { build = e.currentTarget.value; result = null; }}
          placeholder="1000"
          class="w-full h-9 px-3 text-sm font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
        />
      </div>

      <div class="pt-1 text-xs text-muted-foreground">
        {t("versionTab.preview")}:
        <span class={`font-mono font-semibold ${hasChanges ? "text-amber-600" : "text-foreground"}`}>{current}</span>
      </div>
    </div>

    <p class="text-xs text-muted-foreground px-1">{t("versionTab.hint")}</p>
  </div>
{/if}
