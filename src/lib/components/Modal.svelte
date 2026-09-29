<script lang="ts">
  import type { Snippet } from "svelte";
  import { X } from "@lucide/svelte";

  let {
    title,
    onclose,
    width = 480,
    children,
    footer,
  }: {
    title: string;
    onclose: () => void;
    width?: number;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onmousedown={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" role="dialog" aria-modal="true" aria-label={title} style:width="{width}px">
    <header>
      <h2>{title}</h2>
      <button class="icon-btn" onclick={onclose} aria-label="Zamknij"><X size={18} /></button>
    </header>
    <div class="body">
      {@render children()}
    </div>
    {#if footer}
      <footer>{@render footer()}</footer>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.36);
    backdrop-filter: blur(3px);
    animation: fade 0.12s ease-out;
  }
  .modal {
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 48px);
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    animation: pop 0.16s cubic-bezier(0.2, 0.9, 0.3, 1.2);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px 6px;
  }
  header h2 {
    font-size: 17px;
  }
  .body {
    padding: 14px 20px 20px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 14px 20px;
    border-top: 1px solid var(--border);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.98);
    }
  }
</style>
