<script lang="ts">
  let { label, active, onClick }: { label: string; active: boolean; onClick: () => void } = $props();
</script>

<button
  onclick={onClick}
  class={`px-3.5 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-colors ${
    active ? "bg-primary text-primary-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary"
  }`}
>
  {label}
</button>
