<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";

  let { colors, onUpdateColor, splashColor, onUpdateSplashColor }: {
    colors: Record<string, string>;
    onUpdateColor: (key: string, value: string) => void;
    splashColor?: string;
    onUpdateSplashColor?: (value: string) => void;
  } = $props();

  const rows = $derived([
    { key: "fallbackPrimary", label: t("colorsTab.primaryColor"), description: t("colorsTab.primaryColorDescription") },
    { key: "fallbackAccent", label: t("colorsTab.accentColor"), description: t("colorsTab.accentColorDescription") },
    { key: "fallbackDark", label: t("colorsTab.darkColor"), description: t("colorsTab.darkColorDescription") },
  ]);

  function handleTextChange(key: string, raw: string) {
    let v = raw.toUpperCase();
    if (!v.startsWith("#")) v = "#" + v;
    if (!/^#[0-9A-F]{0,6}$/.test(v) || v.length !== 7) return;
    if (key === "__splash") onUpdateSplashColor?.(v);
    else onUpdateColor(key, v);
  }
</script>

<div class="space-y-3">
  <p class="text-xs text-muted-foreground pb-1">{t("colorsTab.description")}</p>

  {#each rows as row (row.key)}
    {@const value = colors[row.key] || "#FFFFFF"}
    <div class="p-4 rounded-xl ring-1 bg-secondary/30 ring-border/30 transition-colors">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm">{row.label}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{row.description}</p>
        </div>
        <div class="flex items-center gap-2 flex-shrink-0">
          <div class="w-8 h-8 rounded-lg ring-1 ring-border/50 flex-shrink-0" style={`background-color: ${value}`}></div>
          <input
            type="color"
            value={value}
            oninput={(e) => onUpdateColor(row.key, (e.currentTarget as HTMLInputElement).value.toUpperCase())}
            class="w-10 h-10 rounded-xl ring-1 ring-border/50 cursor-pointer bg-secondary/30"
          />
          <input
            type="text"
            {value}
            oninput={(e) => handleTextChange(row.key, (e.currentTarget as HTMLInputElement).value)}
            placeholder="#FFFFFF"
            maxlength={7}
            class="w-24 h-9 px-3 text-xs font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
          />
        </div>
      </div>
    </div>
  {/each}

  {#if onUpdateSplashColor}
    {@const sValue = splashColor || "#FFFFFF"}
    <div class="p-4 rounded-xl ring-1 bg-secondary/30 ring-border/30 transition-colors">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm">{t("colorsTab.splashColor")}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{t("colorsTab.splashColorDescription")}</p>
        </div>
        <div class="flex items-center gap-2 flex-shrink-0">
          <div class="w-8 h-8 rounded-lg ring-1 ring-border/50 flex-shrink-0" style={`background-color: ${sValue}`}></div>
          <input
            type="color"
            value={sValue}
            oninput={(e) => onUpdateSplashColor((e.currentTarget as HTMLInputElement).value.toUpperCase())}
            class="w-10 h-10 rounded-xl ring-1 ring-border/50 cursor-pointer bg-secondary/30"
          />
          <input
            type="text"
            value={sValue}
            oninput={(e) => handleTextChange("__splash", (e.currentTarget as HTMLInputElement).value)}
            placeholder="#FFFFFF"
            maxlength={7}
            class="w-24 h-9 px-3 text-xs font-mono rounded-lg bg-background/50 ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 outline-none"
          />
        </div>
      </div>
    </div>
  {/if}

  <div class="p-4 rounded-xl bg-secondary/30 ring-1 ring-border/30">
    <p class="text-xs text-muted-foreground mb-3">{t("colorsTab.preview")}</p>
    <div class="flex gap-3">
      {#each rows as row (row.key)}
        <div class="flex-1 flex flex-col items-center gap-1.5">
          <div class="w-full h-10 rounded-lg ring-1 ring-border/30" style={`background-color: ${colors[row.key] || "#FFFFFF"}`}></div>
          <span class="text-[10px] text-muted-foreground font-mono">{row.label.split(" ")[0]}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
