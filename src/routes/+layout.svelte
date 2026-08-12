<script lang="ts">
  import "../app.css";

  let { children } = $props();
</script>

<svelte:head>
  <title>Serenay Mobile Deploy</title>
</svelte:head>

{@render children()}
