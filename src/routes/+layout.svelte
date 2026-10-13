<script lang="ts">
  import "../app.css";

  let { children } = $props();

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
