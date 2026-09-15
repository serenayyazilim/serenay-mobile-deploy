class ConfirmState {
  open = $state(false);
  title = $state("");
  description = $state("");
  confirmLabel = $state<string | undefined>(undefined);
  cancelLabel = $state<string | undefined>(undefined);

  private resolve: ((value: boolean) => void) | null = null;

  ask(title: string, description: string, options?: { confirmLabel?: string; cancelLabel?: string }): Promise<boolean> {
    this.title = title;
    this.description = description;
    this.confirmLabel = options?.confirmLabel;
    this.cancelLabel = options?.cancelLabel;
    this.open = true;

    return new Promise<boolean>((resolve) => {
      this.resolve = resolve;
    });
  }

  confirm() {
    this.open = false;
    this.resolve?.(true);
    this.resolve = null;
  }

  cancel() {
    this.open = false;
    this.resolve?.(false);
    this.resolve = null;
  }
}

export const confirmState = new ConfirmState();
