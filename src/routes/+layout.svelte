<script lang="ts">
  import "../app.css";
  import { getCurrentWindow, ProgressBarStatus } from "@tauri-apps/api/window";
  import { buildState } from "$lib/stores/build.svelte";
  import { deployState } from "$lib/stores/deploy.svelte";
  import { testsState } from "$lib/stores/tests.svelte";

  let { children } = $props();

  function setDockProgress(status: ProgressBarStatus, progress = 0) {
    getCurrentWindow().setProgressBar({ status, progress: Math.round(progress) }).catch(() => {});
  }

  // Mirrors the running deploy / build / test as a progress bar on the Dock icon.
  $effect(() => {
    const deploy = deployState.deployStatus;
    if (deploy === "activating" || deploy === "deploying") return setDockProgress(ProgressBarStatus.Normal, deployState.deployProgress);
    if (buildState.buildStatus === "running" || testsState.status === "running") {
      // ponytail: build/test report no percentage, so the bar creeps toward 90% over time; parse flutter output if a real number is wanted
      const start = Date.now();
      const tick = () => setDockProgress(ProgressBarStatus.Normal, 90 * (1 - Math.exp(-(Date.now() - start) / 60_000)));
      tick();
      const id = setInterval(tick, 1000);
      return () => clearInterval(id);
    }
    if (deploy === "error") return setDockProgress(ProgressBarStatus.Error, deployState.deployProgress);
    if (buildState.buildStatus === "error" || (testsState.status === "error" && testsState.dialogOpen)) return setDockProgress(ProgressBarStatus.Error, 100);
    setDockProgress(ProgressBarStatus.None);
  });

  // Feeds the pointer position to the glass surface under it, for its moving highlight.
  let lit: HTMLElement | null = null;
  function onPointerMove(e: PointerEvent) {
    const el = (e.target as Element | null)?.closest<HTMLElement>(".glass, .glass-strong") ?? null;
    if (lit && lit !== el) lit.style.removeProperty("--mx"), lit.style.removeProperty("--my");
    lit = el;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    el.style.setProperty("--mx", `${e.clientX - rect.left}px`);
    el.style.setProperty("--my", `${e.clientY - rect.top}px`);
  }
</script>

<svelte:head>
  <title>Serenay Mobile Deploy</title>
</svelte:head>

<svelte:window onpointermove={onPointerMove} />

{@render children()}
