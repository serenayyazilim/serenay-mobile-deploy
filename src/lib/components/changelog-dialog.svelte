<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Sparkles, Plus, Zap, Wrench, ExternalLink, ArrowRight } from "@lucide/svelte";
  import { Dialog, DialogContent, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { changelogState, type ChangeKind } from "$lib/stores/changelog.svelte";
  import { i18n, t } from "$lib/i18n/index.svelte";

  const CHANGELOG_URL = "https://github.com/serenayyazilim/serenay-mobile-deploy/blob/main/CHANGELOG.md";

  const kinds: { kind: ChangeKind; labelKey: string; icon: typeof Plus; badge: string; dot: string }[] = [
    { kind: "new", labelKey: "changelog.new", icon: Plus, badge: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400", dot: "bg-emerald-500" },
    { kind: "improved", labelKey: "changelog.improved", icon: Zap, badge: "bg-sky-500/10 text-sky-600 dark:text-sky-400", dot: "bg-sky-500" },
    { kind: "fixed", labelKey: "changelog.fixed", icon: Wrench, badge: "bg-amber-500/10 text-amber-600 dark:text-amber-400", dot: "bg-amber-500" },
  ];

  let active = $state(0);
  $effect(() => {
    if (changelogState.open) active = 0;
  });

  const release = $derived(changelogState.shown[active] ?? null);
  const groups = $derived(
    kinds
      .map((k) => ({ ...k, changes: release?.changes.filter((c) => c.kind === k.kind) ?? [] }))
      .filter((g) => g.changes.length)
  );

  function formatDate(date: string): string {
    const d = new Date(`${date}T00:00:00`);
    return isNaN(d.getTime()) ? date : new Intl.DateTimeFormat(i18n.locale, { dateStyle: "long" }).format(d);
  }
</script>

{#snippet rich(text: string)}
  {#each text.split("`") as part, i}
    {#if i % 2}<code class="rounded-[5px] border border-border/60 bg-muted px-1 py-px font-mono text-[0.8em] text-foreground">{part}</code>{:else}{part}{/if}
  {/each}
{/snippet}

<Dialog open={changelogState.open} onOpenChange={(o: boolean) => { if (!o) changelogState.dismiss(); }}>
  <DialogContent
    class="max-w-[600px] gap-0 overflow-hidden rounded-2xl p-0 shadow-2xl"
    onOpenAutoFocus={(e: Event) => {
      e.preventDefault();
      (e.target as HTMLElement).querySelector<HTMLElement>("[data-autofocus]")?.focus();
    }}
  >
    {#if release}
      <header class="relative overflow-hidden border-b border-border/60 px-7 pt-8 pb-6">
        <div
          class="pointer-events-none absolute inset-0 [background-image:radial-gradient(var(--border)_1px,transparent_1px)] [background-size:14px_14px] [mask-image:linear-gradient(to_bottom,black,transparent_85%)]"
        ></div>
        <div class="pointer-events-none absolute -top-28 -left-16 h-56 w-80 rounded-full bg-violet-500/20 blur-3xl"></div>

        <div class="relative flex items-center gap-4">
          <img src="/app-icon.png" alt="" class="size-14 shrink-0 rounded-[14px] shadow-lg shadow-violet-500/20 ring-1 ring-black/5" />
          <div class="min-w-0">
            <p class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-violet-600 dark:text-violet-400">
              <Sparkles class="size-3" />
              {t("changelog.eyebrow")}
            </p>
            <DialogTitle class="mt-1 text-2xl font-semibold tracking-tight">
              {t("changelog.title", { version: release.version })}
            </DialogTitle>
            <DialogDescription class="mt-0.5 text-sm text-muted-foreground">
              {t("changelog.releasedOn", { date: formatDate(release.date) })}
            </DialogDescription>
          </div>
        </div>

        <div class="relative mt-5 flex flex-wrap items-center gap-2">
          {#each groups as g (g.kind)}
            <span class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium {g.badge}">
              <span class="size-1.5 rounded-full {g.dot}"></span>
              {g.changes.length} {t(g.labelKey)}
            </span>
          {/each}

          {#if changelogState.shown.length > 1}
            <div class="ml-auto flex rounded-lg bg-muted p-0.5" role="tablist">
              {#each changelogState.shown as r, i (r.version)}
                <button
                  type="button"
                  role="tab"
                  aria-selected={active === i}
                  onclick={() => (active = i)}
                  class="rounded-md px-2.5 py-1 text-xs font-medium tabular-nums transition-all {active === i
                    ? 'bg-background text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'}"
                >
                  v{r.version}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </header>

      <div class="max-h-[min(52vh,480px)] overflow-y-auto px-7 pt-6 pb-10 [mask-image:linear-gradient(to_bottom,black_calc(100%-40px),transparent)]">
        {#key release.version}
          <div class="space-y-7">
            {#each groups as g, gi (g.kind)}
              {@const Icon = g.icon}
              <section in:fly={{ y: 8, duration: 320, delay: gi * 60, easing: cubicOut }}>
                <h3 class="mb-3 flex items-center gap-2.5 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  <span class="grid size-6 place-items-center rounded-md {g.badge}">
                    <Icon class="size-3.5" strokeWidth={2.5} />
                  </span>
                  {t(g.labelKey)}
                </h3>
                <ul class="ml-3 space-y-4 border-l border-border pl-6">
                  {#each g.changes as change, i}
                    <li class="relative" in:fly={{ y: 6, duration: 320, delay: gi * 60 + i * 35, easing: cubicOut }}>
                      <span class="absolute top-[7px] -left-[27.5px] size-1.5 rounded-full ring-4 ring-background {g.dot}"></span>
                      {#if change.title}
                        <p class="text-sm font-medium leading-snug text-foreground">{@render rich(change.title)}</p>
                        <p class="mt-1 text-[13px] leading-relaxed text-muted-foreground">{@render rich(change.body)}</p>
                      {:else}
                        <p class="text-sm leading-relaxed text-foreground/85">{@render rich(change.body)}</p>
                      {/if}
                    </li>
                  {/each}
                </ul>
              </section>
            {/each}
          </div>
        {/key}
      </div>

      <footer class="flex items-center justify-between gap-3 border-t border-border/60 bg-muted/30 px-7 py-4">
        <Button variant="ghost" size="sm" class="text-muted-foreground" onclick={() => openUrl(CHANGELOG_URL)}>
          {t("changelog.fullNotes")}
          <ExternalLink />
        </Button>
        <Button data-autofocus onclick={() => changelogState.dismiss()}>
          {t("changelog.continue")}
          <ArrowRight />
        </Button>
      </footer>
    {/if}
  </DialogContent>
</Dialog>
