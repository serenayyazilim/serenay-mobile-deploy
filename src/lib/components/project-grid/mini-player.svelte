<script lang="ts">
  import { untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
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

  // The player page comes from the app's loopback server (see `commands/video.rs`): YouTube
  // rejects embeds from the app's own tauri:// page with error 153.
  let port = $state<number>();
  invoke<number>("video_player_port").then((p) => (port = p)).catch(() => {});

  // Starting volume goes in the URL; later slider changes are posted to the page, so the
  // video doesn't reload.
  const src = $derived(
    port && videoState.videoId
      ? `http://127.0.0.1:${port}/?v=${videoState.videoId}&volume=${untrack(() => videoState.volume)}`
      : null
  );
  $effect(() => {
    const volume = videoState.volume;
    frame?.contentWindow?.postMessage({ volume }, "*");
  });

  // Open each time a build or deploy starts; closing keeps it closed until the next one.
  $effect(() => {
    if (busy) open = true;
  });
</script>

{#if videoState.enabled && src && open}
  <div
    class="glass-strong fixed right-5 bottom-5 z-40 w-96 overflow-hidden rounded-3xl p-1.5"
    in:fly={{ y: 160, duration: 650, easing: backOut }}
    out:fly={{ y: 160, duration: 300 }}
  >
    <div class="relative aspect-video overflow-hidden rounded-[18px] bg-black">
      <iframe
        bind:this={frame}
        class="absolute inset-0 size-full"
        {src}
        title="YouTube"
        allow="autoplay; encrypted-media; picture-in-picture; fullscreen"
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
