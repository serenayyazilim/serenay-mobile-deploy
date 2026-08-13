<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { User, LogOut, LoaderCircle, Terminal, RefreshCw, CircleAlert } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { t } from "$lib/i18n/index.svelte";

  let currentAccount = $state<string | null>(null);
  let loading = $state(true);

  async function checkCurrentAccount() {
    loading = true;
    try {
      const data = await invoke<{ currentAccount: string | null }>("firebase_accounts");
      currentAccount = data.currentAccount;
    } catch {
      currentAccount = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    checkCurrentAccount();
  });

  async function handleLogout() {
    loading = true;
    try {
      await invoke("firebase_logout");
      currentAccount = null;
    } finally {
      loading = false;
    }
  }
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-sm font-semibold">{t("settings.firebaseTab")}</h3>
    <p class="text-xs text-muted-foreground mt-0.5">{t("firebase.description")}</p>
  </div>

  {#if loading}
    <div class="flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-border/50 bg-secondary/50">
      <LoaderCircle class="w-4 h-4 shrink-0 animate-spin" />
      {t("common.loading")}
    </div>
  {:else if currentAccount}
    <div class="flex items-center gap-2 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-green-500/20 bg-green-500/10">
      <User class="w-4 h-4 shrink-0 text-green-600" />
      <span class="truncate text-green-700 dark:text-green-400 flex-1">{currentAccount}</span>
      <button
        title={t("firebase.logout")}
        onclick={handleLogout}
        class="shrink-0 p-1 -m-1 rounded-md text-green-700/70 dark:text-green-400/70 hover:text-green-700 dark:hover:text-green-400 hover:bg-green-500/20 transition-colors"
      >
        <LogOut class="w-4 h-4" />
      </button>
    </div>
  {:else}
    <div class="flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-orange-500/20 bg-orange-500/10 text-orange-600">
      <CircleAlert class="w-4 h-4 shrink-0" />
      {t("firebase.loginRequired")}
    </div>

    <div class="p-4 bg-secondary rounded-xl space-y-3">
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <Terminal class="w-4 h-4 shrink-0" />
        {t("firebase.runInTerminal")}
      </div>
      <code class="block p-3 bg-black text-green-400 rounded-lg text-sm font-mono">firebase login</code>
      <p class="text-xs text-muted-foreground">{t("firebase.checkAfterLogin")}</p>
    </div>

    <Button class="w-full gap-2" onclick={checkCurrentAccount}>
      <RefreshCw class="w-4 h-4" />
      {t("firebase.checkStatus")}
    </Button>
  {/if}
</div>
