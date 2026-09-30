<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open as openFile } from "@tauri-apps/plugin-dialog";
  import { CircleCheck, CircleAlert, TriangleAlert, LoaderCircle, Wrench } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import AppStoreConnectSettings from "$lib/components/appstoreconnect-settings.svelte";
  import FirstReleaseDialog from "./first-release-dialog.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { KIND_LABELS, STORE_NAMES, type ProjectKind, type Readiness, type StoreChecks, type StorePlatform } from "./types";

  let {
    workspacePath,
    kind: detectedKind,
    onDone,
    onCancel,
  }: { workspacePath: string; kind: ProjectKind; onDone: () => void; onCancel: () => void } = $props();

  type Step = "loading" | "expo" | "prebuild" | "prebuildFailed" | "readiness";
  type Status = "ok" | "warn" | "missing" | "loading";

  let open = $state(true);
  let step = $state<Step>("loading");
  // The dialog is mounted fresh for each setup, and a successful prebuild changes the kind.
  // svelte-ignore state_referenced_locally
  let kind = $state<ProjectKind>(detectedKind);
  let error = $state("");

  let expoConfig = $state<{ dynamic: boolean; bundleIdentifier: string | null; package: string | null } | null>(null);
  let bundleIdentifier = $state("");
  let androidPackage = $state("");
  let prebuildLogs = $state<string[]>([]);

  let generated = $state<Partial<Record<StorePlatform, boolean>>>({});
  let readiness = $state<Readiness | null>(null);
  let ascConfigured = $state(false);
  let store = $state<StoreChecks | null>(null);
  let teamId = $state("");
  let actionError = $state("");
  let showAscSettings = $state(false);
  let firstReleaseOpen = $state(false);
  let firstReleaseShown = false;

  let missingStores = $derived(
    store ? (Object.keys(store) as StorePlatform[]).filter((p) => store?.[p]?.status === "missing") : []
  );
  let expoIdsMissing = $derived(
    !!expoConfig &&
      !expoConfig.dynamic &&
      ((!expoConfig.bundleIdentifier && !bundleIdentifier.trim()) || (!expoConfig.package && !androidPackage.trim()))
  );

  onMount(async () => {
    if (kind !== "expo") return applySetup();
    try {
      expoConfig = await invoke("setup_expo_config", { workspacePath });
      step = "expo";
    } catch (e) {
      error = String(e);
      step = "expo";
    }
  });

  async function runPrebuild() {
    step = "prebuild";
    prebuildLogs = [];
    const ids =
      expoConfig && !expoConfig.dynamic
        ? {
            bundleIdentifier: expoConfig.bundleIdentifier ? null : bundleIdentifier.trim(),
            package: expoConfig.package ? null : androidPackage.trim(),
          }
        : { bundleIdentifier: null, package: null };

    try {
      const processId = await invoke<string>("setup_expo_prebuild", { workspacePath, ...ids });
      const unlisten = await listen<{ type: string; message?: string; prompt?: string; success?: boolean }>(
        `setup-event-${processId}`,
        async ({ payload }) => {
          if (payload.type !== "done") {
            prebuildLogs = [...prebuildLogs, payload.message ?? payload.prompt ?? ""];
            return;
          }
          unlisten();
          if (payload.success) {
            kind = "reactNative";
            await applySetup();
          } else {
            step = "prebuildFailed";
          }
        }
      );
    } catch (e) {
      prebuildLogs = [...prebuildLogs, String(e)];
      step = "prebuildFailed";
    }
  }

  async function applySetup() {
    step = "readiness";
    error = "";
    try {
      const applied = await invoke<{ platform: StorePlatform; generated: boolean }[]>("setup_apply", { workspacePath });
      generated = Object.fromEntries(applied.map((a) => [a.platform, a.generated]));
      await recheck();
    } catch (e) {
      error = String(e);
    }
  }

  async function loadReadiness() {
    readiness = await invoke<Readiness>("setup_readiness", { workspacePath });
    teamId = readiness.ios?.teamId ?? "";
    if (readiness.ios) {
      ascConfigured = (await invoke<{ configured: boolean }>("asc_config_get", { workspace: workspacePath })).configured;
    }
  }

  async function recheck() {
    actionError = "";
    try {
      await loadReadiness();
      store = null;
      store = await invoke<StoreChecks>("setup_store_check", { workspacePath });
      if (!firstReleaseShown && missingStores.length > 0) {
        firstReleaseShown = true;
        firstReleaseOpen = true;
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function runAction(action: () => Promise<unknown>) {
    actionError = "";
    try {
      await action();
      await loadReadiness();
    } catch (e) {
      actionError = String(e);
    }
  }

  let completing = $state(false);

  // Adds the missing lanes / Appfile settings; the store check reruns since it uses those lanes.
  async function completeFastlane() {
    completing = true;
    actionError = "";
    try {
      await invoke("setup_complete_fastlane", { workspacePath });
      await recheck();
    } catch (e) {
      actionError = String(e);
    } finally {
      completing = false;
    }
  }

  const saveTeamId = () => runAction(() => invoke("setup_set_team_id", { workspacePath, teamId }));

  const choosePlayKey = () =>
    runAction(async () => {
      const file = await openFile({ filters: [{ name: "JSON", extensions: ["json"] }] });
      if (typeof file === "string") await invoke("setup_set_play_key", { workspacePath, sourcePath: file });
    });

  function storeStatus(platform: StorePlatform): Status {
    const record = store?.[platform];
    if (!record) return "loading";
    return record.status === "found" ? "ok" : record.status === "missing" ? "missing" : "warn";
  }

  function handleOpenChange(value: boolean) {
    if (!value) onCancel();
  }
</script>

{#snippet statusIcon(status: Status)}
  {#if status === "ok"}
    <CircleCheck class="w-4 h-4 text-green-500 shrink-0 mt-0.5" />
  {:else if status === "warn"}
    <TriangleAlert class="w-4 h-4 text-yellow-500 shrink-0 mt-0.5" />
  {:else if status === "missing"}
    <CircleAlert class="w-4 h-4 text-red-500 shrink-0 mt-0.5" />
  {:else}
    <LoaderCircle class="w-4 h-4 animate-spin text-muted-foreground shrink-0 mt-0.5" />
  {/if}
{/snippet}

<Dialog bind:open onOpenChange={handleOpenChange}>
  <DialogContent
    class={`max-w-xl max-h-[85vh] overflow-y-auto ${step === "prebuild" ? "[&>button:last-child]:hidden" : ""}`}
    interactOutsideBehavior={step === "prebuild" ? "ignore" : "close"}
    escapeKeydownBehavior={step === "prebuild" ? "ignore" : "close"}
  >
    {#if step === "loading"}
      <div class="flex justify-center py-8"><LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" /></div>
    {:else if step === "expo"}
      <DialogHeader>
        <DialogTitle>{t("setup.expo.title")}</DialogTitle>
        <DialogDescription class="pt-1">{t("setup.expo.description")}</DialogDescription>
      </DialogHeader>

      {#if expoConfig?.dynamic}
        <p class="text-sm rounded-lg bg-yellow-500/10 border border-yellow-500/30 p-3">{t("setup.expo.dynamicHint")}</p>
      {:else if expoConfig && (!expoConfig.bundleIdentifier || !expoConfig.package)}
        <div class="space-y-3">
          <p class="text-sm text-muted-foreground">{t("setup.expo.idsHint")}</p>
          {#if !expoConfig.bundleIdentifier}
            <label class="block space-y-1 text-sm">
              <span>{t("setup.expo.bundleIdentifier")}</span>
              <Input bind:value={bundleIdentifier} placeholder="com.company.app" class="font-mono" />
            </label>
          {/if}
          {#if !expoConfig.package}
            <label class="block space-y-1 text-sm">
              <span>{t("setup.expo.package")}</span>
              <Input bind:value={androidPackage} placeholder="com.company.app" class="font-mono" />
            </label>
          {/if}
        </div>
      {/if}
      {#if error}<p class="text-sm text-red-600">{error}</p>{/if}

      <div class="flex justify-end gap-2">
        <Button variant="outline" onclick={onCancel}>{t("common.cancel")}</Button>
        <!-- Prebuild is asked for again when the project is deployed. -->
        <Button variant="outline" onclick={onDone}>{t("setup.skipForNow")}</Button>
        <Button onclick={runPrebuild} disabled={expoIdsMissing}>{t("setup.expo.confirm")}</Button>
      </div>
    {:else if step === "prebuild" || step === "prebuildFailed"}
      <DialogHeader>
        <DialogTitle class={step === "prebuildFailed" ? "text-red-600" : ""}>
          {step === "prebuild" ? t("setup.prebuild.title") : t("setup.prebuild.failed")}
        </DialogTitle>
        {#if step === "prebuild"}
          <DialogDescription class="pt-1">{t("setup.prebuild.description")}</DialogDescription>
        {/if}
      </DialogHeader>

      {#if step === "prebuild"}
        <div class="relative h-2 overflow-hidden rounded-full bg-muted" role="progressbar" aria-label={t("setup.prebuild.title")}>
          <div class="absolute inset-y-0 w-1/3 rounded-full bg-primary animate-indeterminate motion-reduce:animate-none"></div>
        </div>
        <p class="truncate font-mono text-xs text-muted-foreground">{prebuildLogs.at(-1) ?? "npx expo prebuild"}</p>
      {:else}
        <div class="max-h-72 overflow-auto rounded-lg bg-zinc-950 p-3 font-mono text-xs text-zinc-300">
          {#each prebuildLogs as log, i (i)}
            <div class="whitespace-pre-wrap break-all">{log}</div>
          {/each}
        </div>
        <div class="flex justify-end gap-2">
          <Button variant="outline" onclick={onCancel}>{t("common.close")}</Button>
          <Button onclick={runPrebuild}>{t("common.retry")}</Button>
        </div>
      {/if}
    {:else}
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Wrench class="w-5 h-5" />
          {t("setup.readiness.title")}
        </DialogTitle>
        <DialogDescription class="pt-1">{t("setup.readiness.description")}</DialogDescription>
      </DialogHeader>

      {#if !readiness}
        {#if error}
          <p class="text-sm text-red-600">{error}</p>
        {:else}
          <div class="flex justify-center py-8"><LoaderCircle class="w-6 h-6 animate-spin text-muted-foreground" /></div>
        {/if}
      {:else}
        <ul class="divide-y rounded-lg border text-sm">
          <li class="flex gap-3 p-3">
            {@render statusIcon("ok")}
            <span class="flex-1">{t("setup.kind")}</span>
            <span class="text-muted-foreground">{KIND_LABELS[kind]}</span>
          </li>

          {#each [["ios", readiness.ios], ["android", readiness.android]] as const as [platform, info] (platform)}
            {#if info}
              <li class="flex gap-3 p-3">
                {@render statusIcon(info.fastlaneMissing.length ? "warn" : "ok")}
                <div class="flex-1 space-y-1">
                  <p>{t("setup.fastlane", { platform: platform === "ios" ? "iOS" : "Android" })}</p>
                  <p class="text-xs text-muted-foreground">
                    {generated[platform] ? t("setup.fastlane.generated") : t("setup.fastlane.existing")}
                  </p>
                  {#if info.fastlaneMissing.length}
                    <p class="text-xs">{t("setup.fastlane.missing")}</p>
                    <ul class="space-y-0.5">
                      {#each info.fastlaneMissing as item (item)}
                        <li><code class="rounded bg-muted px-1.5 py-0.5 text-xs">{item}</code></li>
                      {/each}
                    </ul>
                    <div class="flex items-center gap-2 pt-1">
                      <Button variant="outline" size="sm" onclick={completeFastlane} disabled={completing}>
                        {#if completing}<LoaderCircle class="animate-spin" />{/if}
                        {t("setup.fastlane.complete")}
                      </Button>
                      <span class="text-xs text-muted-foreground">{t("setup.fastlane.completeHint")}</span>
                    </div>
                  {/if}
                </div>
              </li>
            {/if}
          {/each}

          {#if readiness.ios}
            <li class="flex gap-3 p-3">
              {@render statusIcon(ascConfigured ? "ok" : "missing")}
              <div class="flex-1 space-y-2">
                <div class="flex items-center gap-2">
                  <span class="flex-1">{t("setup.ascKey")}</span>
                  <Button variant="outline" size="sm" onclick={() => (showAscSettings = !showAscSettings)}>
                    {showAscSettings ? t("setup.hide") : t("setup.configure")}
                  </Button>
                </div>
                {#if showAscSettings}
                  <AppStoreConnectSettings {workspacePath} />
                {/if}
              </div>
            </li>

            <li class="flex gap-3 p-3">
              {@render statusIcon(readiness.ios.teamId ? "ok" : "missing")}
              <div class="flex-1 space-y-2">
                <span>{t("setup.teamId")}</span>
                <div class="flex gap-2">
                  <Input bind:value={teamId} placeholder="ABCDE12345" class="h-8 font-mono" />
                  <Button size="sm" onclick={saveTeamId} disabled={!teamId.trim() || teamId === readiness.ios.teamId}>
                    {t("common.save")}
                  </Button>
                </div>
                {#if !readiness.ios.teamId}
                  <p class="text-xs text-muted-foreground">{t("setup.teamIdHint")}</p>
                {/if}
              </div>
            </li>
          {/if}

          {#if readiness.android}
            <li class="flex gap-3 p-3">
              {@render statusIcon(readiness.android.playKey === false ? "missing" : "ok")}
              <div class="flex-1 space-y-1">
                <div class="flex items-center gap-2">
                  <span class="flex-1">{t("setup.playKey")}</span>
                  {#if readiness.android.playKey !== null}
                    <Button variant="outline" size="sm" onclick={choosePlayKey}>{t("setup.chooseJson")}</Button>
                  {/if}
                </div>
                {#if readiness.android.playKey === null}
                  <p class="text-xs text-muted-foreground">{t("setup.playKeyExternal")}</p>
                {/if}
              </div>
            </li>

            <li class="flex gap-3 p-3">
              {@render statusIcon(readiness.android.releaseSigning ? "ok" : "warn")}
              <div class="flex-1 space-y-1">
                <span>{t("setup.releaseSigning")}</span>
                {#if !readiness.android.releaseSigning}
                  <p class="text-xs text-muted-foreground">{t("setup.releaseSigningHint")}</p>
                {/if}
              </div>
            </li>
          {/if}

          {#each (["ios", "android"] as const).filter((p) => readiness?.[p]) as platform (platform)}
            {@const record = store?.[platform]}
            <li class="flex gap-3 p-3">
              {@render statusIcon(storeStatus(platform))}
              <div class="flex-1 space-y-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="flex-1">{t("setup.store", { store: STORE_NAMES[platform] })}</span>
                  {#if record?.status === "missing"}
                    <Button variant="outline" size="sm" onclick={() => (firstReleaseOpen = true)}>{t("setup.store.howTo")}</Button>
                  {/if}
                </div>
                <p class="text-xs text-muted-foreground break-words">
                  {#if !record}
                    {t("setup.store.checking")}
                  {:else if record.status === "found"}
                    {t("setup.store.found", { count: record.locales.length })} · {record.locales.join(", ")}
                  {:else if record.status === "missing"}
                    {t("setup.store.missing")}
                  {:else}
                    {record.message}
                  {/if}
                </p>
              </div>
            </li>
          {/each}
        </ul>

        {#if actionError || error}
          <p class="text-sm text-red-600">{actionError || error}</p>
        {/if}

        <div class="flex justify-end gap-2">
          <Button variant="outline" onclick={recheck} disabled={!store}>{t("setup.recheck")}</Button>
          <Button onclick={onDone}>{t("common.continue")}</Button>
        </div>
      {/if}
    {/if}
  </DialogContent>
</Dialog>

<FirstReleaseDialog bind:open={firstReleaseOpen} stores={missingStores} {workspacePath} onRecheck={recheck} />
