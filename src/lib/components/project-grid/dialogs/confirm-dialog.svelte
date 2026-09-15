<script lang="ts">
  import { TriangleAlert } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";
  import { confirmState } from "$lib/stores/confirm.svelte";
  import { t } from "$lib/i18n/index.svelte";
</script>

<Dialog bind:open={confirmState.open} onOpenChange={(value) => !value && confirmState.cancel()}>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-yellow-600">
        <TriangleAlert class="w-5 h-5" />
        {confirmState.title}
      </DialogTitle>
      <DialogDescription class="whitespace-pre-line pt-1">{confirmState.description}</DialogDescription>
    </DialogHeader>

    <div class="flex justify-end gap-2 pt-4">
      <Button variant="outline" onclick={() => confirmState.cancel()}>{confirmState.cancelLabel ?? t("common.cancel")}</Button>
      <Button onclick={() => confirmState.confirm()}>{confirmState.confirmLabel ?? t("common.continue")}</Button>
    </div>
  </DialogContent>
</Dialog>
