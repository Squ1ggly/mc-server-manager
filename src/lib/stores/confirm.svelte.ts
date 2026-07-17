// A promise-based confirmation dialog for destructive actions that happen
// outside a view with room for an inline "Sure?" step — the right-click menu,
// mainly. `ask()` resolves true only when the user confirms.

export interface ConfirmRequest {
  title: string;
  body: string;
  confirmLabel: string;
  variant?: "primary" | "soft" | "danger";
}

class ConfirmStore {
  open = $state(false);
  title = $state("");
  body = $state("");
  confirmLabel = $state("");
  variant = $state<"primary" | "soft" | "danger">("primary");

  private resolver: ((confirmed: boolean) => void) | null = null;

  ask(request: ConfirmRequest): Promise<boolean> {
    this.title = request.title;
    this.body = request.body;
    this.confirmLabel = request.confirmLabel;
    this.variant = request.variant ?? "primary";
    this.open = true;
    return new Promise((resolve) => {
      this.resolver = resolve;
    });
  }

  confirm(): void {
    this.finish(true);
  }

  /** Cancel — also used when the dialog is dismissed any other way. */
  cancel(): void {
    this.finish(false);
  }

  private finish(confirmed: boolean): void {
    this.open = false;
    const resolve = this.resolver;
    this.resolver = null;
    resolve?.(confirmed);
  }
}

export const confirmStore = new ConfirmStore();
