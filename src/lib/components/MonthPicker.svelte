<script lang="ts">
  import { ChevronLeft, ChevronRight } from "@lucide/svelte";
  import { app } from "$lib/state.svelte";
  import { currentMonth, monthLabel, shiftMonth } from "$lib/format";

  const isCurrent = $derived(app.month === currentMonth());
</script>

<div class="picker">
  <button class="icon-btn" onclick={() => (app.month = shiftMonth(app.month, -1))} aria-label="Poprzedni miesiąc">
    <ChevronLeft size={18} />
  </button>
  <span class="label">{monthLabel(app.month)}</span>
  <button class="icon-btn" onclick={() => (app.month = shiftMonth(app.month, 1))} aria-label="Następny miesiąc">
    <ChevronRight size={18} />
  </button>
  {#if !isCurrent}
    <button class="btn sm ghost today" onclick={() => (app.month = currentMonth())}>Bieżący</button>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-sm);
  }
  .picker {
    justify-content: space-between;
  }
  .label {
    min-width: 150px;
    text-align: center;
    font-weight: 650;
    font-size: 15px;
  }
  .today {
    color: var(--accent);
  }
</style>
