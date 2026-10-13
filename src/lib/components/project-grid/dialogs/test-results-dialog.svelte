<script lang="ts">
  import { FlaskConical, LoaderCircle, CircleCheck, CircleX, CircleMinus } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";
  import { workspaceState } from "$lib/stores/workspace.svelte";
  import { testsState } from "$lib/stores/tests.svelte";
  import LogView from "./log-view.svelte";

  let showLogs = $state(false);

  const order = { failed: 0, passed: 1, skipped: 2 };
  const sorted = $derived([...testsState.results].sort((a, b) => order[a.status] - order[b.status]));
  const count = (status: string) => testsState.results.filter((r) => r.status === status).length;

  // Show the logs while running, or when the run produced no results to look at.
  $effect(() => {
    showLogs = testsState.status === "running" || (testsState.status !== "idle" && testsState.results.length === 0);
  });

  function logClass(log: string): string {
    if (/^(✗|✕|FAIL\b)|\s✕ /.test(log)) return "text-red-400 log-light:text-red-600";
    if (/^(✅|PASS\b)|^\s*✓ /.test(log)) return "text-emerald-300 log-light:text-emerald-700";
    if (log.startsWith("▸ ")) return "text-zinc-400 log-light:text-zinc-500";
    if (/\b(error|failed)\b|❌/i.test(log)) return "text-red-400 log-light:text-red-600";
    if (/warning|⚠/i.test(log)) return "text-yellow-400 log-light:text-amber-600";
    return "text-zinc-300 log-light:text-zinc-800";
  }
</script>

<Dialog bind:open={testsState.dialogOpen}>
  <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        {#if testsState.status === "running"}<LoaderCircle class="w-5 h-5 animate-spin text-primary" />
        {:else if testsState.status === "success"}<CircleCheck class="w-5 h-5 text-green-500" />
        {:else if testsState.status === "error"}<CircleX class="w-5 h-5 text-red-500" />
        {:else}<FlaskConical class="w-5 h-5 text-muted-foreground" />{/if}
        {t("tests.title", { name: testsState.project?.appName ?? "" })}
      </DialogTitle>
    </DialogHeader>

    <div class="flex items-center gap-3 text-sm">
      {#if testsState.status === "running"}
        <span class="text-muted-foreground">{t("tests.running")}</span>
      {:else if testsState.results.length > 0}
        <span class="text-green-600">{t("tests.passed", { count: count("passed") })}</span>
        <span class="text-red-600">{t("tests.failed", { count: count("failed") })}</span>
        <span class="text-muted-foreground">{t("tests.skipped", { count: count("skipped") })}</span>
      {/if}
      <div class="ml-auto flex gap-1">
        <Button size="sm" variant={showLogs ? "ghost" : "secondary"} onclick={() => (showLogs = false)}>{t("tests.showResults")}</Button>
        <Button size="sm" variant={showLogs ? "secondary" : "ghost"} onclick={() => (showLogs = true)}>{t("tests.showLogs")}</Button>
      </div>
    </div>

    {#if showLogs}
      <LogView logs={testsState.logs} {logClass} />
    {:else if sorted.length === 0}
      <p class="py-8 text-center text-sm text-muted-foreground">{t("tests.noResults")}</p>
    {:else}
      <div class="flex-1 overflow-auto rounded-lg border divide-y">
        {#each sorted as test, i (i)}
          <details class="group px-3 py-2" open={test.status === "failed" && count("failed") <= 3}>
            <summary class={`flex items-center gap-2 text-sm ${test.message ? "cursor-pointer" : "list-none"}`}>
              {#if test.status === "passed"}<CircleCheck class="w-4 h-4 shrink-0 text-green-500" />
              {:else if test.status === "failed"}<CircleX class="w-4 h-4 shrink-0 text-red-500" />
              {:else}<CircleMinus class="w-4 h-4 shrink-0 text-muted-foreground" />{/if}
              <span class="break-all">{test.name}</span>
            </summary>
            {#if test.message}
              <pre class="mt-2 overflow-auto rounded bg-zinc-950 p-3 text-xs text-red-300 whitespace-pre-wrap">{test.message}</pre>
            {/if}
          </details>
        {/each}
      </div>
    {/if}

    <div class="flex justify-end gap-2 pt-4">
      {#if testsState.status !== "running" && testsState.project && workspaceState.path}
        <Button variant="outline" onclick={() => testsState.run(testsState.project!, workspaceState.path!)}>{t("tests.rerun")}</Button>
      {/if}
      <Button onclick={() => (testsState.dialogOpen = false)}>{t("common.close")}</Button>
    </div>
  </DialogContent>
</Dialog>
