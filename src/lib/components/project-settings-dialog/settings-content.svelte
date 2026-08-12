<script lang="ts">
  import type { SerConfField } from "$lib/serconf-schema";

  let { fields, config, onUpdateField }: {
    fields: SerConfField[];
    config: Record<string, string | number | boolean>;
    onUpdateField: (key: string, value: string | number | boolean) => void;
  } = $props();

  const inputClass = "w-full h-9 px-3 text-sm rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none";
</script>

<div class="space-y-3">
  {#each fields as field (field.key)}
    {@const value = config[field.key] ?? field.defaultValue}
    <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm">{field.label}{field.required ? " *" : ""}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{field.description}</p>
        </div>
        <div class="flex-shrink-0 w-40">
          {#if field.type === "boolean"}
            <label class="flex items-center justify-end">
              <input
                type="checkbox"
                checked={Boolean(value)}
                onchange={(e) => onUpdateField(field.key, (e.currentTarget as HTMLInputElement).checked)}
                class="w-5 h-5 rounded accent-primary cursor-pointer"
              />
            </label>
          {:else if field.type === "number"}
            <input
              type="number"
              value={value}
              oninput={(e) => onUpdateField(field.key, Number((e.currentTarget as HTMLInputElement).value))}
              class={inputClass}
            />
          {:else if field.type === "enum"}
            <select
              value={value}
              onchange={(e) => onUpdateField(field.key, (e.currentTarget as HTMLSelectElement).value)}
              class={inputClass}
            >
              {#each field.enumOptions || [] as opt (opt.value)}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          {:else}
            <input
              type="text"
              value={value}
              oninput={(e) => onUpdateField(field.key, (e.currentTarget as HTMLInputElement).value)}
              class={inputClass}
            />
          {/if}
        </div>
      </div>
    </div>
  {/each}
</div>
