<script lang="ts">
  import { getBasicFields } from "$lib/serconf-schema";
  import { inputClass, labelClass } from "../shared";
  import type { CreateProjectWizard } from "../use-create-project.svelte";

  let { wizard }: { wizard: CreateProjectWizard } = $props();

  const basicFields = getBasicFields();
</script>

<div class="space-y-3">
  {#each basicFields as field (field.key)}
    {@const value = wizard.form.serconf[field.key] ?? field.defaultValue}
    <div class="space-y-1">
      <label class={labelClass} for="cp-{field.key}">{field.label}</label>
      {#if field.type === "boolean"}
        <input
          id="cp-{field.key}"
          type="checkbox"
          checked={Boolean(value)}
          onchange={(e) => (wizard.form.serconf = { ...wizard.form.serconf, [field.key]: (e.currentTarget as HTMLInputElement).checked })}
          class="w-5 h-5 rounded accent-primary cursor-pointer"
        />
      {:else if field.type === "enum"}
        <select
          id="cp-{field.key}"
          value={value}
          onchange={(e) => (wizard.form.serconf = { ...wizard.form.serconf, [field.key]: (e.currentTarget as HTMLSelectElement).value })}
          class={inputClass}
        >
          {#each field.enumOptions || [] as opt (opt.value)}<option value={opt.value}>{opt.label}</option>{/each}
        </select>
      {:else}
        <input
          id="cp-{field.key}"
          value={value}
          oninput={(e) => (wizard.form.serconf = { ...wizard.form.serconf, [field.key]: (e.currentTarget as HTMLInputElement).value })}
          class={inputClass}
        />
      {/if}
    </div>
  {/each}
</div>
