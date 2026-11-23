<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Copy, Check, Eye, EyeOff, RefreshCw, Trash2, Link, Plus } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { t } from "$lib/i18n/index.svelte";

  type Token = { name: string; token: string; permissions: string[] };
  type Data = { permissions: string[]; tokens: Token[] };

  let data = $state<Data>({ permissions: [], tokens: [] });
  let edits = $state<Record<string, string[]>>({});
  let revealed = $state<Record<string, boolean>>({});
  let copied = $state("");
  let error = $state("");
  let newName = $state("");
  let newPerms = $state<string[]>(["read"]);
  let link = $state("");
  let enabled = $state(true);
  let password = $state("");
  let showPassword = $state(false);

  async function setEnabled(on: boolean) {
    enabled = on;
    await invoke("mcp_enabled_set", { on }).catch((e) => (error = String(e)));
  }

  function load(d: Data) {
    data = d;
    edits = Object.fromEntries(d.tokens.map((x) => [x.name, [...(x.permissions ?? [])]]));
  }

  async function apply(action: string, name: string, permissions: string[] = []) {
    error = "";
    try {
      load(await invoke<Data>("mcp_tokens_apply", { action, name, permissions }));
      return true;
    } catch (e) {
      error = String(e);
      return false;
    }
  }

  function toggle(list: string[], p: string) {
    return list.includes(p) ? list.filter((x) => x !== p) : [...list, p];
  }

  async function copy(text: string, key: string) {
    await navigator.clipboard.writeText(text).catch(() => {});
    copied = key;
    setTimeout(() => (copied = ""), 1500);
  }

  async function create() {
    if (await apply("create", newName, newPerms)) newName = "";
  }

  async function makeLink() {
    link = await invoke<string>("mcp_tokens_link");
    await copy(link, "link");
  }

  $effect(() => {
    invoke<string>("mcp_main_password").then((v) => (password = v)).catch(() => {});
    invoke<boolean>("mcp_enabled_get").then((v) => (enabled = v)).catch(() => {});
    invoke<Data>("mcp_tokens_list").then(load).catch((e) => (error = String(e)));
  });
</script>

<div class="space-y-4">
  <label class="flex items-center justify-between rounded-lg border p-3">
    <span>
      <span class="block text-sm font-medium">{t("mcpTokens.enabled")}</span>
      <span class="block text-xs text-muted-foreground">{enabled ? t("mcpTokens.on") : t("mcpTokens.off")}</span>
    </span>
    <input type="checkbox" class="w-5 h-5" checked={enabled} onchange={(e) => setEnabled(e.currentTarget.checked)} />
  </label>

  <p class="text-xs text-muted-foreground">{t("mcpTokens.description")}</p>

  {#each data.tokens as tk (tk.name)}
    {@const perms = edits[tk.name] ?? []}
    <div class="rounded-lg border p-3 space-y-2">
      <div class="flex items-center gap-2">
        <span class="font-medium text-sm">{tk.name}</span>
        <code class="text-xs text-muted-foreground break-all flex-1">{revealed[tk.name] ? tk.token : "•".repeat(12)}</code>
        <Button variant="ghost" size="icon" onclick={() => (revealed[tk.name] = !revealed[tk.name])}>
          {#if revealed[tk.name]}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
        </Button>
        <Button variant="ghost" size="icon" onclick={() => copy(tk.token, tk.name)}>
          {#if copied === tk.name}<Check class="w-4 h-4" />{:else}<Copy class="w-4 h-4" />{/if}
        </Button>
      </div>
      <div class="flex flex-wrap gap-x-4 gap-y-1">
        {#each data.permissions as p}
          <label class="flex items-center gap-1 text-xs">
            <input type="checkbox" checked={perms.includes(p)} onchange={() => (edits[tk.name] = toggle(perms, p))} />
            {p}
          </label>
        {/each}
      </div>
      <div class="flex gap-2">
        <Button size="sm" onclick={() => apply("update", tk.name, perms)}>{t("common.save")}</Button>
        <Button size="sm" variant="outline" onclick={() => apply("regenerate", tk.name)}><RefreshCw class="w-3.5 h-3.5 mr-1" />{t("mcpTokens.newSecret")}</Button>
        <Button size="sm" variant="outline" onclick={() => apply("delete", tk.name)}><Trash2 class="w-3.5 h-3.5 mr-1" />{t("mcpTokens.delete")}</Button>
      </div>
    </div>
  {/each}

  <div class="rounded-lg border p-3 space-y-2">
    <p class="text-sm font-medium">{t("mcpTokens.new")}</p>
    <Input bind:value={newName} placeholder={t("mcpTokens.name")} />
    <div class="flex flex-wrap gap-x-4 gap-y-1">
      {#each data.permissions as p}
        <label class="flex items-center gap-1 text-xs">
          <input type="checkbox" checked={newPerms.includes(p)} onchange={() => (newPerms = toggle(newPerms, p))} />
          {p}
        </label>
      {/each}
    </div>
    <Button size="sm" disabled={!newName.trim()} onclick={create}><Plus class="w-3.5 h-3.5 mr-1" />{t("mcpTokens.create")}</Button>
  </div>

  <div class="rounded-lg border p-3 space-y-2">
    <p class="text-xs text-muted-foreground">{t("mcpTokens.linkHint")}</p>
    <div class="flex items-center gap-2">
      <span class="text-xs">{t("mcpTokens.mainPassword")}</span>
      <code class="text-xs text-muted-foreground break-all flex-1">{showPassword ? password : "•".repeat(12)}</code>
      <Button variant="ghost" size="icon" onclick={() => (showPassword = !showPassword)}>
        {#if showPassword}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
      </Button>
      <Button variant="ghost" size="icon" onclick={() => copy(password, "password")}>
        {#if copied === "password"}<Check class="w-4 h-4" />{:else}<Copy class="w-4 h-4" />{/if}
      </Button>
    </div>
    <div class="flex gap-2">
      <Button size="sm" variant="outline" onclick={makeLink}><Link class="w-3.5 h-3.5 mr-1" />{t("mcpTokens.link")}</Button>
      {#if link}<Button size="sm" variant="outline" onclick={() => openUrl(link)}>{t("mcpTokens.open")}</Button>{/if}
    </div>
    {#if link}<code class="block text-xs break-all text-muted-foreground">{link}</code>{/if}
  </div>

  {#if error}<p class="text-xs text-red-600">{error}</p>{/if}
</div>
