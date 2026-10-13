<script lang="ts">
  import { fly } from "svelte/transition";
  import { backOut } from "svelte/easing";
  import { X } from "@lucide/svelte";
  import { buildState } from "$lib/stores/build.svelte";
  import { deployState } from "$lib/stores/deploy.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { videoState } from "$lib/stores/video.svelte";

  const busy = $derived(
    buildState.buildStatus === "running" || deployState.deployStatus === "activating" || deployState.deployStatus === "deploying"
  );

  let open = $state(false);
  let frame = $state<HTMLIFrameElement>();

  // Starts muted, then once the player reports ready it's unmuted at the set volume, so it never
  // blasts at 100%. Talks to the player through YouTube's iframe postMessage API (`enablejsapi=1`).
  let ready = $state(false);
  const command = (func: string, args: unknown[] = []) =>
    frame?.contentWindow?.postMessage(JSON.stringify({ event: "command", func, args, id: 1, channel: "widget" }), "*");

  // Follow the volume slider in Settings while the video plays.
  $effect(() => {
    const volume = videoState.volume;
    if (ready) command("setVolume", [volume]);
  });

  // Ask the player to report its events (onReady) to this page.
  function onFrameLoad() {
    ready = false;
    frame?.contentWindow?.postMessage(JSON.stringify({ event: "listening", id: 1, channel: "widget" }), "*");
  }

  function onPlayerMessage(e: MessageEvent) {
    if (!frame || e.source !== frame.contentWindow || typeof e.data !== "string") return;
    try {
      if (JSON.parse(e.data).event === "onReady") {
        ready = true;
        command("setVolume", [videoState.volume]);
        command("unMute");
      }
    } catch {
      // Not a player message.
    }
  }
  // Open each time a build or deploy starts; closing keeps it closed until the next one.
  $effect(() => {
    if (busy) open = true;
  });
</script>

<svelte:window onmessage={onPlayerMessage} />

{#if videoState.enabled && videoState.videoId && open}
  <div
    class="glass-strong fixed right-5 bottom-5 z-40 w-96 overflow-hidden rounded-3xl p-1.5"
    in:fly={{ y: 160, duration: 650, easing: backOut }}
    out:fly={{ y: 160, duration: 300 }}
  >
    <div class="relative aspect-video overflow-hidden rounded-[18px] bg-black">
      <iframe
        bind:this={frame}
        onload={onFrameLoad}
        class="absolute inset-0 size-full"
        src={`https://www.youtube-nocookie.com/embed/${videoState.videoId}?autoplay=1&mute=1&playsinline=1&rel=0&enablejsapi=1`}
        title="YouTube"
        allow="autoplay; encrypted-media; picture-in-picture"
        referrerpolicy="strict-origin-when-cross-origin"
        allowfullscreen
      ></iframe>
    </div>
    <button
      type="button"
      onclick={() => (open = false)}
      title={t("common.close")}
      class="pressable absolute top-3 right-3 grid size-7 place-items-center rounded-full bg-black/55 text-white backdrop-blur-md hover:bg-black/75"
    >
      <X class="size-3.5" />
    </button>
  </div>
{/if}
