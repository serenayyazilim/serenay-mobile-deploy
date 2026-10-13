<script lang="ts">
  let {
    checked = $bindable(false),
    onLabel,
    offLabel,
  }: {
    checked: boolean;
    onLabel: string;
    offLabel: string;
  } = $props();
</script>

<div class="inline-flex items-center rounded-full bg-secondary ring-1 ring-border/30 p-0.5 text-xs">
  <button
    type="button"
    onclick={() => (checked = true)}
    class={`px-3 py-1.5 rounded-full transition-colors ${checked ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"}`}
  >
    {onLabel}
  </button>
  <button
    type="button"
    onclick={() => (checked = false)}
    class={`px-3 py-1.5 rounded-full transition-colors ${!checked ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"}`}
  >
    {offLabel}
  </button>
</div>
