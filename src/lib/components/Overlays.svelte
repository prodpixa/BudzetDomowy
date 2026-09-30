<script lang="ts">
  import { CircleCheck, Info, TriangleAlert } from "@lucide/svelte";
  import { app } from "$lib/state.svelte";
  import Modal from "./Modal.svelte";

  function answer(ok: boolean) {
    app.confirmRequest?.resolve(ok);
    app.confirmRequest = null;
  }
</script>

{#if app.confirmRequest}
  {@const req = app.confirmRequest}
  <Modal title={req.title} onclose={() => answer(false)} width={420}>
    <p class="text">{req.text}</p>
    {#snippet footer()}
      <button class="btn" onclick={() => answer(false)}>Anuluj</button>
      <!-- svelte-ignore a11y_autofocus -->
      <button class="btn danger" autofocus onclick={() => answer(true)}>{req.confirmLabel}</button>
    {/snippet}
  </Modal>
{/if}

<div class="toasts" aria-live="polite">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}">
      {#if t.kind === "ok"}
        <CircleCheck size={17} />
      {:else if t.kind === "error"}
        <TriangleAlert size={17} />
      {:else}
        <Info size={17} />
      {/if}
      <span>{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .text {
    margin: 0;
    color: var(--text-2);
  }
  .toasts {
    position: fixed;
    right: 20px;
    bottom: 20px;
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 240px;
    max-width: 420px;
    padding: 12px 16px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    animation: slide 0.2s ease-out;
    font-weight: 500;
  }
  .toast.ok :global(svg) {
    color: var(--income);
  }
  .toast.error :global(svg) {
    color: var(--expense);
  }
  .toast.info :global(svg) {
    color: var(--accent);
  }
  @media (max-width: 760px) {
    .toasts {
      left: 12px;
      right: 12px;
      bottom: calc(76px + env(safe-area-inset-bottom));
    }
    .toast {
      min-width: 0;
      max-width: none;
    }
  }
  @keyframes slide {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
</style>
