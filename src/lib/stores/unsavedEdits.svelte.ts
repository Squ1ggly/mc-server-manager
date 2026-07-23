// Tracks in-progress edits that navigating away would throw away.
//
// Editors live several components deep and are destroyed outright when the
// route or the selected tab changes, so they can't intercept that themselves.
// Each registers what's unsaved — and how to save it — here instead, and every
// place navigation happens asks before going anywhere.

import { confirmStore } from "./confirm.svelte";

export interface UnsavedGuard {
  /** What is unsaved, woven into the prompt — e.g. "the file server.properties"
   *  or "this server's settings". */
  description: string;
  /** Persists the pending edits. Resolves `true` once saved, or `false` when
   *  the save failed (the page surfaces its own error). A failed save cancels
   *  the navigation so nothing is lost. */
  save: () => Promise<boolean>;
}

class UnsavedEditsStore {
  private guard = $state<UnsavedGuard | null>(null);

  /** Whether an editor currently has unsaved changes. */
  get hasUnsaved(): boolean {
    return this.guard !== null;
  }

  /** Called by an editor while its content diverges from what's saved. */
  register(guard: UnsavedGuard): void {
    this.guard = guard;
  }

  /** Drops the record without asking — after a save, an agreed discard, or
   *  when the editor is torn down. */
  clear(): void {
    this.guard = null;
  }

  /**
   * Whether it's safe to navigate away, blocking on a Save / Don't save /
   * Cancel prompt when there are unsaved edits. Save persists them (staying
   * put if that fails), Don't save discards them, Cancel aborts the move.
   */
  async confirmLeave(): Promise<boolean> {
    const guard = this.guard;
    if (guard === null) {
      return true;
    }

    const choice = await confirmStore.ask({
      title: "Unsaved changes",
      body: `You've made changes to ${guard.description} that haven't been saved yet.`,
      confirmLabel: "Save",
      variant: "primary",
      secondaryLabel: "Don't save",
      secondaryVariant: "danger",
    });

    if (choice === "cancel") {
      return false;
    }
    if (choice === "secondary") {
      this.clear();
      return true;
    }

    const saved = await guard.save();
    if (!saved) {
      // The save failed (the page has already shown why) — stay put so the
      // edits aren't thrown away.
      return false;
    }
    this.clear();
    return true;
  }
}

export const unsavedEditsStore = new UnsavedEditsStore();
