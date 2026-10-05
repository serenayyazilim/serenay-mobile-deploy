<script lang="ts">
  import { ChevronDown, ChevronUp, Search } from "@lucide/svelte";
  import { t } from "$lib/i18n/index.svelte";

  let { logs, logClass }: { logs: string[]; logClass: (log: string) => string } = $props();

  let scrollEl: HTMLDivElement | undefined;
  let inputEl: HTMLInputElement | undefined;
  let query = $state("");
  let current = $state(0);

  const pattern = $derived(query ? new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "i") : null);
  const matches = $derived(pattern ? logs.flatMap((log, i) => (pattern.test(log) ? [i] : [])) : []);

  // Follow new output unless the user is searching.
  $effect(() => {
    logs.length;
    if (scrollEl && !pattern) scrollEl.scrollTop = scrollEl.scrollHeight;
  });

  function jump(step: number) {
    if (!matches.length) return;
    current = (current + step + matches.length) % matches.length;
    scrollEl?.querySelector(`[data-line="${matches[current]}"]`)?.scrollIntoView({ block: "center" });
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      inputEl?.focus();
      inputEl?.select();
    }
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="flex items-center gap-2 rounded-md border px-2 h-9">
  <Search class="w-4 h-4 text-muted-foreground shrink-0" />
  <input
    bind:this={inputEl}
    bind:value={query}
    oninput={() => ((current = -1), jump(1))}
    onkeydown={(e) => e.key === "Enter" && (e.preventDefault(), jump(e.shiftKey ? -1 : 1))}
    placeholder={t("common.search")}
    class="flex-1 bg-transparent text-sm outline-none"
  />
  {#if query}
    <span class="text-xs text-muted-foreground tabular-nums">{matches.length ? current + 1 : 0}/{matches.length}</span>
    <button type="button" class="p-1 rounded hover:bg-secondary" onclick={() => jump(-1)}><ChevronUp class="w-4 h-4" /></button>
    <button type="button" class="p-1 rounded hover:bg-secondary" onclick={() => jump(1)}><ChevronDown class="w-4 h-4" /></button>
  {/if}
</div>

<div bind:this={scrollEl} class="flex-1 overflow-auto bg-zinc-950 rounded-lg p-4 font-mono text-sm">
  {#each logs as log, i (i)}
    <div data-line={i} class={`py-0.5 whitespace-pre-wrap break-all ${logClass(log)} ${matches[current] === i ? "bg-yellow-500/20" : ""}`}>
      {#if pattern && pattern.test(log)}
        {#each log.split(new RegExp(pattern.source, "gi")) as part, j}
          {#if j % 2}<mark class="bg-yellow-400 text-black rounded-sm">{part}</mark>{:else}{part}{/if}
        {/each}
      {:else}{log}{/if}
    </div>
  {/each}
</div>
