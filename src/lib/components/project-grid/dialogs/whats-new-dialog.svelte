<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Rocket } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import type { WorkspaceProject } from "$lib/stores/projects.svelte";
  import { t } from "$lib/i18n/index.svelte";

  type DeployPlatform = "ios" | "android" | "all";

  let { open = $bindable(false), project, workspacePath, onConfirm, onCancel }: {
    open: boolean;
    project: WorkspaceProject | null;
    workspacePath: string;
    onConfirm: (whatsNew: string, platform: DeployPlatform, bumpVersion: boolean) => void;
    onCancel: () => void;
  } = $props();

  const FLAG_TO_LANG: Record<string, { flag: string; labelKey: string }> = {
    ENGLISH: { flag: "🇺🇸", labelKey: "language.english" },
    RUSSIAN: { flag: "🇷🇺", labelKey: "language.russian" },
    FRENCH: { flag: "🇫🇷", labelKey: "language.french" },
    ITALIAN: { flag: "🇮🇹", labelKey: "language.italian" },
    ARABIC: { flag: "🇸🇦", labelKey: "language.arabic" },
    SPANISH: { flag: "🇪🇸", labelKey: "language.spanish" },
    KAZAKH: { flag: "🇰🇿", labelKey: "language.kazakh" },
  };

  let text = $state("");
  let platform = $state<DeployPlatform>("all");
  let languages = $state<{ flag: string; labelKey: string }[]>([]);
  let bumpVersion = $state(true);
  let currentVersion = $state("");

  $effect(() => {
    if (!open || !project || !workspacePath) return;
    languages = [];
    invoke<Record<string, string | number | boolean>>("config_serconf_get", { workspace: workspacePath, projectId: project.id })
      .then((config) => {
        const result = [{ flag: "🇹🇷", labelKey: "language.turkish" }];
        for (const [key, meta] of Object.entries(FLAG_TO_LANG)) {
          if (config[key] === true) result.push(meta);
        }
        languages = result;
      })
      .catch(() => {});

    currentVersion = "";
    invoke<{ versions: Record<string, string> }>("projects_versions", { workspace: workspacePath })
      .then((data) => (currentVersion = data.versions?.[project.id] ?? ""))
      .catch(() => {});
  });

  function nextVersion(current: string): string {
    const [semver] = current.split("+");
    let [major, minor, patch] = semver.split(".").map((n) => parseInt(n, 10) || 0);
    patch += 1;
    if (patch > 9) { patch = 0; minor += 1; }
    if (minor > 9) { minor = 0; major += 1; }
    const build = major * 10_000_000 + minor * 100_000 + patch * 1_000;
    return `${major}.${minor}.${patch}+${build}`;
  }

  function handleConfirm() {
    onConfirm(text.trim() || t("whatsNew.defaultText"), platform, bumpVersion);
    text = "";
    platform = "all";
    bumpVersion = true;
  }

  function handleCancel() {
    text = "";
    platform = "all";
    bumpVersion = true;
    onCancel();
  }

  const platformOptions: { value: DeployPlatform; labelKey: string }[] = [
    { value: "ios", labelKey: "whatsNew.iosOnly" },
    { value: "android", labelKey: "whatsNew.androidOnly" },
    { value: "all", labelKey: "whatsNew.both" },
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
        {t("whatsNew.description")}
      </p>

      <textarea
        placeholder={t("whatsNew.defaultText")}
        bind:value={text}
        rows={4}
        class="placeholder:text-muted-foreground border-input w-full rounded-md border bg-transparent px-3 py-2 text-sm shadow-xs outline-none resize-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]"
      ></textarea>

      {#if languages.length > 0}
        <div class="rounded-md bg-muted/50 px-3 py-2 space-y-1">
          <p class="text-xs text-muted-foreground font-medium">{t("whatsNew.estimatedLanguages")}</p>
          <div class="flex flex-wrap gap-x-3 gap-y-1">
            {#each languages as lang (lang.labelKey)}
              <span class="text-xs text-foreground">{lang.flag} {t(lang.labelKey)}</span>
            {/each}
          </div>
        </div>
      {/if}

      <div class="space-y-1.5">
        <p class="text-xs text-muted-foreground font-medium">{t("whatsNew.platform")}</p>
        <div class="flex gap-2">
          {#each platformOptions as opt (opt.value)}
            <button
              type="button"
              onclick={() => (platform = opt.value)}
              class={`flex-1 rounded-md border px-3 py-1.5 text-xs font-medium transition-colors ${
                platform === opt.value ? "border-primary bg-primary text-primary-foreground" : "border-input bg-transparent text-foreground hover:bg-muted"
              }`}
            >
              {t(opt.labelKey)}
            </button>
          {/each}
        </div>
      </div>

      <label class="flex items-center gap-2.5 rounded-md border border-input px-3 py-2.5 cursor-pointer">
        <input type="checkbox" bind:checked={bumpVersion} class="w-4 h-4 accent-primary shrink-0" />
        <span class="flex-1 text-xs">
          <span class="font-medium text-foreground">{t("whatsNew.bumpVersion")}</span>
          {#if currentVersion}
            <span class="text-muted-foreground">
              {" — "}{bumpVersion ? `${currentVersion} → ${nextVersion(currentVersion)}` : t("whatsNew.bumpVersionOffHint", { version: currentVersion })}
            </span>
          {/if}
        </span>
      </label>
    </div>

    <div class="flex justify-end gap-2 pt-2">
      <Button variant="outline" onclick={handleCancel}>{t("common.cancel")}</Button>
      <Button onclick={handleConfirm}>{t("whatsNew.startDeploy")}</Button>
    </div>
  </DialogContent>
</Dialog>
