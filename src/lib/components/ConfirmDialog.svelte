<script lang="ts">
  // The shared confirmation dialog. Driven by confirmStore.ask().
  import Modal from "./Modal.svelte";
  import Button from "./Button.svelte";
  import { confirmStore } from "../stores/confirm.svelte";
</script>

<Modal open={confirmStore.open} title={confirmStore.title} onclose={() => confirmStore.cancel()}>
  <p class="body">{confirmStore.body}</p>
  <div class="actions">
    <Button variant="ghost" onclick={() => confirmStore.cancel()}>Cancel</Button>
    <Button variant={confirmStore.variant} onclick={() => confirmStore.confirm()}>
      {confirmStore.confirmLabel}
    </Button>
  </div>
</Modal>

<style>
  .body {
    margin: 0 0 1.25rem;
    font-size: 0.92rem;
    line-height: 1.5;
    color: var(--muted);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
