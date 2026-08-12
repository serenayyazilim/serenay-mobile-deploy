<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { User, LogIn, LogOut, LoaderCircle, Terminal, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";

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
  <Button variant="outline" class="gap-2 h-9 px-3 rounded-full" disabled>
    <LoaderCircle class="w-4 h-4 animate-spin" />
    <span class="text-sm">Yükleniyor...</span>
  </Button>
{:else if currentAccount}
  <div class="flex items-center gap-2">
    <div class="flex items-center gap-2 px-3 py-2 rounded-full bg-green-500/10 text-sm border border-green-500/20">
      <User class="w-4 h-4 text-green-600" />
      <span class="max-w-32 truncate text-green-700 dark:text-green-400 font-medium">{currentAccount.split("@")[0]}</span>
    </div>
    <Button variant="outline" size="sm" class="gap-2 h-9 px-3 rounded-full" onclick={handleLogout}>
      <LogOut class="w-4 h-4" />
      <span class="text-sm">Çıkış</span>
    </Button>
  </div>
{:else}
  <Button variant="outline" class="gap-2 h-9 px-3 rounded-full border-orange-500/50 text-orange-600 hover:bg-orange-500/10" onclick={() => (showLoginDialog = true)}>
    <LogIn class="w-4 h-4" />
    <span class="text-sm">Firebase Giriş</span>
  </Button>

  <Dialog bind:open={showLoginDialog}>
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Terminal class="w-5 h-5" />
          Firebase Giriş Gerekli
        </DialogTitle>
        <DialogDescription>Firebase CLI ile terminal üzerinden giriş yapmanız gerekiyor.</DialogDescription>
      </DialogHeader>

      <div class="space-y-4 py-4">
        <div class="p-4 bg-secondary rounded-xl">
          <p class="text-sm text-muted-foreground mb-3">Terminal'de şu komutu çalıştırın:</p>
          <code class="block p-3 bg-black text-green-400 rounded-lg text-sm font-mono">firebase login</code>
        </div>
        <p class="text-sm text-muted-foreground">Giriş yaptıktan sonra aşağıdaki butona tıklayarak durumu kontrol edin.</p>
      </div>

      <div class="flex gap-3">
        <Button variant="outline" class="flex-1" onclick={() => (showLoginDialog = false)}>Kapat</Button>
        <Button class="flex-1 gap-2" onclick={() => { showLoginDialog = false; checkCurrentAccount(); }}>
          <RefreshCw class="w-4 h-4" />
          Kontrol Et
        </Button>
      </div>
    </DialogContent>
  </Dialog>
{/if}
