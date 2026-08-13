<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { User, LogIn, LogOut, LoaderCircle, Terminal, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";
  import { t } from "$lib/i18n/index.svelte";

  let currentAccount = $state<string | null>(null);
  let loading = $state(true);
  let showLoginDialog = $state(false);

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

{#if loading}
  <button class="w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-border/50 bg-secondary/50 opacity-70 cursor-default" disabled>
    <LoaderCircle class="w-4 h-4 shrink-0 animate-spin" />
    <span class="truncate">{t("common.loading")}</span>
  </button>
{:else if currentAccount}
  <div class="w-full flex items-center gap-2 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-green-500/20 bg-green-500/10">
    <User class="w-4 h-4 shrink-0 text-green-600" />
    <span class="truncate text-green-700 dark:text-green-400 flex-1">{currentAccount.split("@")[0]}</span>
    <button
      title={t("firebase.logout")}
      onclick={handleLogout}
      class="shrink-0 p-1 -m-1 rounded-md text-green-700/70 dark:text-green-400/70 hover:text-green-700 dark:hover:text-green-400 hover:bg-green-500/20 transition-colors"
    >
      <LogOut class="w-4 h-4" />
    </button>
  </div>
{:else}
  <button
    onclick={() => (showLoginDialog = true)}
    class="w-full flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-sm font-medium ring-1 ring-orange-500/20 bg-orange-500/10 text-orange-600 hover:bg-orange-500/20 transition-all"
  >
    <LogIn class="w-4 h-4 shrink-0" />
    <span class="truncate">{t("firebase.login")}</span>
  </button>

  <Dialog bind:open={showLoginDialog}>
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Terminal class="w-5 h-5" />
          {t("firebase.loginRequired")}
        </DialogTitle>
        <DialogDescription>{t("firebase.loginRequiredDescription")}</DialogDescription>
      </DialogHeader>

      <div class="space-y-4 py-4">
        <div class="p-4 bg-secondary rounded-xl">
          <p class="text-sm text-muted-foreground mb-3">{t("firebase.runInTerminal")}</p>
          <code class="block p-3 bg-black text-green-400 rounded-lg text-sm font-mono">firebase login</code>
        </div>
        <p class="text-sm text-muted-foreground">{t("firebase.checkAfterLogin")}</p>
      </div>

      <div class="flex gap-3">
        <Button variant="outline" class="flex-1" onclick={() => (showLoginDialog = false)}>{t("common.close")}</Button>
        <Button class="flex-1 gap-2" onclick={() => { showLoginDialog = false; checkCurrentAccount(); }}>
          <RefreshCw class="w-4 h-4" />
          {t("firebase.checkStatus")}
        </Button>
      </div>
    </DialogContent>
  </Dialog>
{/if}
