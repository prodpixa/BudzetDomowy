<script lang="ts">
  import { untrack } from "svelte";
  import { Trash2 } from "@lucide/svelte";
  import { api } from "$lib/api";
  import { money } from "$lib/format";
  import { app, type EntryEditor } from "$lib/state.svelte";
  import { categoryIcon, slotColor } from "$lib/icons";
  import Modal from "./Modal.svelte";
  import MoneyInput from "./MoneyInput.svelte";

  let { editor }: { editor: EntryEditor } = $props();

  // Komponent jest tworzony od nowa dla każdego edytora ({#key}), więc wartość startowa wystarczy.
  // Formularz pracuje na kopii – zmiany trafiają do bazy dopiero po zapisaniu.
  const { kind, item, onSaved } = untrack(() => editor);
  const isExpense = kind === "expense";
  let name = $state(item.name);
  let amount = $state<number | null>(item.amount || null);
  let date = $state(item.date);
  let note = $state(item.note);
  let groupId = $state<number | null>("categoryId" in item ? item.categoryId : item.sourceId);
  let saving = $state(false);

  const isNew = item.id === null;
  const title = `${isNew ? "Nowy" : "Edytuj"} ${isExpense ? "wydatek" : "przychód"}`;
  const canSave = $derived(name.trim() !== "" && !!amount && amount > 0 && date !== "");

  async function remove() {
    const id = item.id;
    if (id === null) return;
    const ok = await app.confirm(
      isExpense ? "Usunąć wydatek?" : "Usunąć przychód?",
      `„${item.name}” na kwotę ${money(item.amount)} zostanie trwale usunięty.`,
    );
    if (!ok) return;
    const done = await app.mutate(
      () => (isExpense ? api.deleteExpense(id) : api.deleteIncome(id)),
      "Usunięto",
    );
    if (done !== undefined) app.editor = null;
  }

  async function save(e?: Event) {
    e?.preventDefault();
    if (!canSave || saving) return;
    saving = true;
    const base = { id: item.id, name: name.trim(), amount: amount!, date, note };
    const ok = await app.mutate(
      () =>
        isExpense
          ? api.saveExpense({ ...base, categoryId: groupId })
          : api.saveIncome({ ...base, sourceId: groupId }),
      isNew ? (isExpense ? "Dodano wydatek" : "Dodano przychód") : "Zapisano zmiany",
    );
    saving = false;
    if (ok !== undefined) {
      onSaved?.();
      app.editor = null;
    }
  }
</script>

<Modal {title} onclose={() => (app.editor = null)} width={520}>
  <form id="entry-form" onsubmit={save}>
    <div class="row">
      <label class="field grow">
        <span>{isExpense ? "Na co" : "Skąd"}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="input"
          bind:value={name}
          placeholder={isExpense ? "np. Zakupy w Biedronce" : "np. Wypłata"}
          autofocus={!name}
        />
      </label>
      <label class="field amount">
        <span>Kwota</span>
        <MoneyInput bind:value={amount} autofocus={!!name && !amount} />
      </label>
    </div>

    <label class="field">
      <span>Data</span>
      <input class="input" type="date" bind:value={date} required />
    </label>

    <div class="field">
      <span>{isExpense ? "Kategoria" : "Źródło"}</span>
      <div class="chips">
        {#if isExpense}
          {#each app.categories as c (c.id)}
            {@const Icon = categoryIcon(c.icon)}
            <button
              type="button"
              class="chip"
              class:selected={groupId === c.id}
              style:--c={slotColor(c.color)}
              onclick={() => (groupId = groupId === c.id ? null : c.id)}
            >
              <Icon size={15} />
              {c.name}
            </button>
          {/each}
        {:else}
          {#each app.sources as s (s.id)}
            <button
              type="button"
              class="chip"
              class:selected={groupId === s.id}
              style:--c="var(--income)"
              onclick={() => (groupId = groupId === s.id ? null : s.id)}
            >
              {s.name}
            </button>
          {/each}
        {/if}
      </div>
    </div>

    <label class="field">
      <span>Notatka <em class="muted">(opcjonalnie)</em></span>
      <textarea class="input" rows="2" bind:value={note}></textarea>
    </label>
  </form>

  {#snippet footer()}
    {#if !isNew}
      <button class="btn ghost delete" type="button" onclick={remove}><Trash2 size={16} /> Usuń</button>
    {/if}
    <button class="btn" type="button" onclick={() => (app.editor = null)}>Anuluj</button>
    <button class="btn primary" type="submit" form="entry-form" disabled={!canSave || saving}>
      {isNew ? "Dodaj" : "Zapisz"}
    </button>
  {/snippet}
</Modal>

<style>
  .delete {
    margin-right: auto;
    color: var(--expense);
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .row {
    display: flex;
    gap: 12px;
  }
  .grow {
    flex: 1;
  }
  .amount {
    width: 150px;
  }
  @media (max-width: 760px) {
    .row {
      flex-direction: column;
    }
    .amount {
      width: 100%;
    }
  }
  em {
    font-style: normal;
    font-weight: 400;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 99px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-2);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s;
  }
  .chip :global(svg) {
    color: var(--c);
  }
  .chip:hover {
    background: var(--surface-hover);
  }
  .chip.selected {
    border-color: var(--c);
    background: color-mix(in srgb, var(--c) 14%, var(--surface));
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--c);
  }
</style>
