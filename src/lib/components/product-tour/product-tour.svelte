<script lang="ts">
  import { tick } from "svelte";
  import { ChevronLeft, ChevronRight, MousePointer2, X } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { onboardingState } from "$lib/stores/onboarding.svelte";
  import { tourSteps, type TourStep } from "./steps";
  import { t } from "$lib/i18n/index.svelte";

  type Rect = { top: number; left: number; width: number; height: number };

  // Local, component-owned visual state. None of this is ever read back by
  // the effect below, and the effect never writes to onboardingState — only
  // the Next/Back/Skip click handlers do. This keeps the reactive graph
  // one-directional and avoids Svelte's "effect reads and writes the same
  // state" infinite-loop guard.
  let steps = $state<TourStep[]>([]);
  let rect = $state<Rect | null>(null);
  let cursor = $state<{ x: number; y: number }>({ x: 0, y: 0 });
  let phase = $state<"moving" | "clicked" | "idle">("idle");
  let cardStyle = $state("");
  let ready = $state(false);

  const current = $derived(steps[onboardingState.stepIndex] ?? null);
  const isLast = $derived(onboardingState.stepIndex >= steps.length - 1);
  const isFirst = $derived(onboardingState.stepIndex === 0);

  const PADDING = 10;

  function findTarget(id: string): HTMLElement | null {
    return document.querySelector<HTMLElement>(`[data-tour="${id}"]`);
  }

  function rectOf(el: HTMLElement): Rect {
    const r = el.getBoundingClientRect();
    return { top: r.top, left: r.left, width: r.width, height: r.height };
  }

  function center(r: Rect): { x: number; y: number } {
    return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
  }

  function placeCard(target: Rect, placement: TourStep["placement"]) {
    const cardW = 320;
    const cardH = 160;
    const gap = 18;
    let top = 0;
    let left = 0;

    if (placement === "right") {
      top = target.top + target.height / 2 - cardH / 2;
      left = target.left + target.width + gap;
    } else if (placement === "left") {
      top = target.top + target.height / 2 - cardH / 2;
      left = target.left - cardW - gap;
    } else if (placement === "bottom") {
      top = target.top + target.height + gap;
      left = target.left + target.width / 2 - cardW / 2;
    } else {
      top = target.top - cardH - gap;
      left = target.left + target.width / 2 - cardW / 2;
    }

    top = Math.min(Math.max(top, 12), window.innerHeight - cardH - 12);
    left = Math.min(Math.max(left, 12), window.innerWidth - cardW - 12);

    cardStyle = `top:${top}px; left:${left}px; width:${cardW}px;`;
  }

  let runToken = 0;

  async function playStep(stepIndexAtStart: number) {
    const step = steps[stepIndexAtStart];
    const token = ++runToken;
    ready = false;
    rect = null;
    if (!step) return;

    const el = findTarget(step.id);
    if (!el) return;

    el.scrollIntoView({ block: "center", behavior: stepIndexAtStart === 0 ? "auto" : "smooth" });
    await tick();
    await new Promise((r) => setTimeout(r, 260));
    if (token !== runToken) return;

    const targetRect = rectOf(el);
    rect = {
      top: targetRect.top - PADDING,
      left: targetRect.left - PADDING,
      width: targetRect.width + PADDING * 2,
      height: targetRect.height + PADDING * 2,
    };

    phase = "moving";
    cursor = center(targetRect);
    await new Promise((r) => setTimeout(r, 550));
    if (token !== runToken) return;
    phase = "clicked";
    await new Promise((r) => setTimeout(r, 260));
    if (token !== runToken) return;
    phase = "idle";

    placeCard(targetRect, step.placement);
    ready = true;
  }

  function advance() {
    if (onboardingState.stepIndex < steps.length - 1) {
      onboardingState.goTo(onboardingState.stepIndex + 1);
    } else {
      onboardingState.finish();
    }
  }

  function goBack() {
    if (onboardingState.stepIndex > 0) {
      onboardingState.goTo(onboardingState.stepIndex - 1);
    }
  }

  // Builds the filtered step list exactly once per tour run (when `active`
  // flips on). Writes only local state, never onboardingState.
  $effect(() => {
    if (onboardingState.active) {
      const found = tourSteps.filter((s) => findTarget(s.id));
      steps = found;
      if (found.length === 0) {
        // Defer to a microtask so this write doesn't happen inside the
        // same synchronous pass that read `active`.
        queueMicrotask(() => onboardingState.finish());
      }
    } else {
      steps = [];
      rect = null;
      ready = false;
      runToken++;
    }
  });

  // Drives the cursor/spotlight/tooltip animation for the current step.
  // Depends on stepIndex (written only by advance()/goBack(), i.e. user
  // clicks) and steps (written only by the effect above) — never writes
  // either of those itself.
  $effect(() => {
    const index = onboardingState.stepIndex;
    if (onboardingState.active && steps.length > 0) {
      playStep(index);
    }
  });

  function handleResize() {
    if (!onboardingState.active || !current || !rect) return;
    const el = findTarget(current.id);
    if (!el) return;
    const targetRect = rectOf(el);
    rect = {
      top: targetRect.top - PADDING,
      left: targetRect.left - PADDING,
      width: targetRect.width + PADDING * 2,
      height: targetRect.height + PADDING * 2,
    };
    placeCard(targetRect, current.placement);
  }
</script>

<svelte:window onresize={handleResize} />

{#if onboardingState.active && rect}
  <div class="fixed inset-0 z-[100]" aria-hidden="true">
    <!-- dimming panels around the spotlight -->
    <div class="absolute bg-black/60 transition-all duration-300" style={`top:0; left:0; right:0; height:${Math.max(rect.top, 0)}px;`}></div>
    <div
      class="absolute bg-black/60 transition-all duration-300"
      style={`top:${rect.top + rect.height}px; left:0; right:0; bottom:0;`}
    ></div>
    <div
      class="absolute bg-black/60 transition-all duration-300"
      style={`top:${rect.top}px; left:0; width:${Math.max(rect.left, 0)}px; height:${rect.height}px;`}
    ></div>
    <div
      class="absolute bg-black/60 transition-all duration-300"
      style={`top:${rect.top}px; left:${rect.left + rect.width}px; right:0; height:${rect.height}px;`}
    ></div>

    <!-- block real interaction with the spotlighted element -->
    <div
      class="absolute"
      style={`top:${rect.top}px; left:${rect.left}px; width:${rect.width}px; height:${rect.height}px;`}
    ></div>

    <!-- spotlight ring -->
    <div
      class="absolute rounded-2xl ring-2 ring-primary shadow-[0_0_0_4px_rgba(255,255,255,0.15)] transition-all duration-300 pointer-events-none"
      style={`top:${rect.top}px; left:${rect.left}px; width:${rect.width}px; height:${rect.height}px;`}
    ></div>
  </div>

  <!-- fake cursor -->
  <div
    class="fixed z-[102] pointer-events-none transition-all ease-out"
    style={`top:${cursor.y}px; left:${cursor.x}px; transition-duration:${phase === "moving" ? "550ms" : "0ms"};`}
  >
    <div class="relative -translate-x-1 -translate-y-1">
      {#if phase === "clicked"}
        <span class="absolute inset-0 -m-2 rounded-full bg-primary/40 animate-ping"></span>
      {/if}
      <MousePointer2 class="w-6 h-6 text-white drop-shadow-[0_1px_3px_rgba(0,0,0,0.7)]" fill="black" />
    </div>
  </div>

  <!-- tooltip card -->
  {#if ready && current}
    <div
      class="fixed z-[103] rounded-2xl bg-popover text-popover-foreground ring-1 ring-border/60 shadow-2xl p-4 animate-in fade-in-0 zoom-in-95 duration-200"
      style={cardStyle}
    >
      <button
        onclick={() => onboardingState.skip()}
        class="absolute top-2.5 right-2.5 p-1 rounded-full text-muted-foreground hover:bg-secondary hover:text-foreground transition-colors"
        title={t("tour.skip")}
      >
        <X class="w-3.5 h-3.5" />
      </button>

      <h4 class="text-sm font-semibold pr-5">{t(current.titleKey)}</h4>
      <p class="text-xs text-muted-foreground mt-1.5 leading-relaxed">{t(current.descKey)}</p>

      <div class="flex items-center justify-between mt-4">
        <div class="flex items-center gap-1">
          {#each steps as _s, i (i)}
            <span class={`h-1.5 rounded-full transition-all ${i === onboardingState.stepIndex ? "w-4 bg-primary" : "w-1.5 bg-border"}`}></span>
          {/each}
        </div>

        <div class="flex items-center gap-1.5">
          {#if !isFirst}
            <Button size="sm" variant="ghost" class="h-7 px-2 text-xs gap-1" onclick={goBack}>
              <ChevronLeft class="w-3.5 h-3.5" />
              {t("tour.back")}
            </Button>
          {/if}
          <Button size="sm" class="h-7 px-3 text-xs gap-1" onclick={advance}>
            {isLast ? t("tour.finish") : t("tour.next")}
            {#if !isLast}<ChevronRight class="w-3.5 h-3.5" />{/if}
          </Button>
        </div>
      </div>
    </div>
  {/if}
{/if}
