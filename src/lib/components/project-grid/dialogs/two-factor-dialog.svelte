<script lang="ts">
  import { ShieldCheck } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";

  let { open = $bindable(false), prompt, onSubmit, onCancel }: {
    open: boolean;
    prompt: string;
    onSubmit: (code: string) => void;
    onCancel: () => void;
  } = $props();

  let digits = $state<string[]>(["", "", "", "", "", ""]);
  let inputEls: (HTMLInputElement | null)[] = [];

  $effect(() => {
    if (open) {
      digits = ["", "", "", "", "", ""];
      setTimeout(() => inputEls[0]?.focus(), 50);
    }
  });

  function handleInput(i: number, e: Event) {
    const value = (e.currentTarget as HTMLInputElement).value;
    const digit = value.replace(/\D/g, "").slice(-1);
    digits[i] = digit;
    if (digit && i < 5) inputEls[i + 1]?.focus();
  }

  function handleKeydown(i: number, e: KeyboardEvent) {
    if (e.key === "Backspace" && !digits[i] && i > 0) {
      inputEls[i - 1]?.focus();
    } else if (e.key === "Enter") {
      const code = digits.join("");
      if (code.length === 6) handleSubmit();
    }
  }

  function handlePaste(e: ClipboardEvent) {
    e.preventDefault();
    const pasted = (e.clipboardData?.getData("text") || "").replace(/\D/g, "").slice(0, 6);
    if (!pasted) return;
    for (let i = 0; i < 6; i++) digits[i] = pasted[i] ?? "";
    inputEls[Math.min(pasted.length, 5)]?.focus();
  }

  function handleSubmit() {
    const code = digits.join("");
    if (code.length !== 6) return;
    onSubmit(code);
    digits = ["", "", "", "", "", ""];
  }

  const code = $derived(digits.join(""));
</script>

<Dialog {open} onOpenChange={(o: boolean) => { if (!o) onCancel(); }}>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <ShieldCheck class="w-5 h-5 text-blue-500" />
        Apple Kimlik Doğrulama
      </DialogTitle>
    </DialogHeader>

    <div class="space-y-5 py-1">
      <p class="text-sm text-muted-foreground">
        Apple hesabınız için 6 haneli doğrulama kodunu girin. Kodu cihazınızdan veya SMS'ten alabilirsiniz.
      </p>

      {#if prompt}
        <p class="text-xs text-muted-foreground bg-muted/50 rounded-md px-3 py-2 font-mono">{prompt}</p>
      {/if}

      <div class="flex gap-2 justify-center">
        {#each digits as digit, i (i)}
          <input
            bind:this={inputEls[i]}
            type="text"
            inputmode="numeric"
            maxlength={1}
            value={digit}
            oninput={(e) => handleInput(i, e)}
            onkeydown={(e) => handleKeydown(i, e)}
            onpaste={handlePaste}
            class="w-11 h-13 text-center text-xl font-semibold rounded-lg border border-input bg-background shadow-xs outline-none focus:border-ring focus:ring-ring/50 focus:ring-[3px] transition-all caret-transparent"
          />
        {/each}
      </div>
    </div>

    <div class="flex justify-end gap-2 pt-2">
      <Button variant="outline" onclick={onCancel}>İptal</Button>
      <Button onclick={handleSubmit} disabled={code.length !== 6}>Gönder</Button>
    </div>
  </DialogContent>
</Dialog>
