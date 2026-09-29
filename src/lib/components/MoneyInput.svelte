<script lang="ts">
  import { untrack } from "svelte";
  import { moneyInput, parseMoney } from "$lib/format";

  // `value` w groszach; null gdy pole jest puste lub niepoprawne.
  let {
    value = $bindable(),
    placeholder = "0,00",
    autofocus = false,
    id,
  }: { value: number | null; placeholder?: string; autofocus?: boolean; id?: string } = $props();

  let text = $state("");
  const invalid = $derived(text.trim() !== "" && parseMoney(text) === null);

  // Zmiana z zewnątrz (np. wyczyszczenie formularza) → aktualizuj tekst pola.
  $effect.pre(() => {
    const v = value;
    untrack(() => {
      if (v !== parseMoney(text)) text = v ? moneyInput(v) : "";
    });
  });
</script>

<div class="wrap">
  <!-- svelte-ignore a11y_autofocus -->
  <input
    {id}
    class="input amount"
    class:invalid
    inputmode="decimal"
    autocomplete="off"
    {placeholder}
    {autofocus}
    value={text}
    oninput={(e) => {
      text = e.currentTarget.value;
      value = parseMoney(text);
    }}
  />
  <span class="unit">zł</span>
</div>

<style>
  .wrap {
    position: relative;
  }
  .input {
    padding-right: 34px;
  }
  .unit {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--muted);
    pointer-events: none;
  }
  .invalid,
  .invalid:focus {
    border-color: var(--expense);
    box-shadow: 0 0 0 3px var(--expense-soft);
  }
</style>
