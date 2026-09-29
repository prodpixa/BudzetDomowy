<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight, Check, Copy, Pencil, Plus, StickyNote, Trash2 } from "@lucide/svelte";
  import { api, type Planned, type PlannedKind } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { defaultDate, money, monthLabel, plural } from "$lib/format";
  import MoneyInput from "./MoneyInput.svelte";

  let { kind, items }: { kind: PlannedKind; items: Planned[] } = $props();

  const isExpense = $derived(kind === "expense");
  const total = $derived(items.reduce((s, i) => s + i.amount, 0));
  const remaining = $derived(items.filter((i) => !i.done).reduce((s, i) => s + i.amount, 0));
  const doneCount = $derived(items.filter((i) => i.done).length);

  // formularz dodawania
  let newName = $state("");
  let newAmount = $state<number | null>(null);
  let newNote = $state("");
  let nameInput: HTMLInputElement;

  // edycja w miejscu
  let editId = $state<number | null>(null);
  let editName = $state("");
  let editAmount = $state<number | null>(null);
  let editNote = $state("");

  async function add(e: Event) {
    e.preventDefault();
    if (!newName.trim() || !newAmount) return;
    const ok = await app.mutate(() =>
      api.savePlanned({
        id: null,
        kind,
        month: app.month,
        name: newName.trim(),
        amount: newAmount!,
        note: newNote.trim(),
        done: false,
      }),
    );
    if (ok !== undefined) {
      newName = "";
      newAmount = null;
      newNote = "";
      nameInput.focus();
    }
  }

  function startEdit(i: Planned) {
    editId = i.id;
    editName = i.name;
    editAmount = i.amount;
    editNote = i.note;
  }

  async function saveEdit(i: Planned, e?: Event) {
    e?.preventDefault();
    if (!editName.trim() || !editAmount) return;
    const ok = await app.mutate(() =>
      api.savePlanned({ ...i, name: editName.trim(), amount: editAmount!, note: editNote.trim() }),
    );
    if (ok !== undefined) editId = null;
  }

  async function toggle(i: Planned) {
    const done = !i.done;
    const ok = await app.mutate(() => api.setPlannedDone(i.id!, done));
    if (ok === undefined || !done) return;
    // Po odhaczeniu proponujemy dodanie realnego wpisu – kwotę można zmienić lub anulować.
    const base = { id: null, name: i.name, amount: i.amount, date: defaultDate(app.month), note: i.note };
    app.editor = isExpense
      ? { kind: "expense", item: { ...base, categoryId: null } }
      : { kind: "income", item: { ...base, sourceId: null } };
  }

  async function remove(i: Planned) {
    const ok = await app.confirm("Usunąć pozycję?", `„${i.name}” zniknie z listy planowanych.`);
    if (ok) await app.mutate(() => api.deletePlanned(i.id!), "Usunięto");
  }

  async function copyPrevious() {
    const r = await app.mutate(() => api.copyPlannedFromPrevious(app.month, kind));
    if (!r) return;
    if (!r.fromMonth) app.toast("Brak pozycji w poprzednich miesiącach", "info");
    else if (r.copied === 0) app.toast("Wszystkie pozycje są już na liście", "info");
    else
      app.toast(
        `Skopiowano ${plural(r.copied, "pozycję", "pozycje", "pozycji")} z miesiąca: ${monthLabel(r.fromMonth)}`,
      );
  }
</script>

<section class="card col" class:income={!isExpense}>
  <div class="card-head">
    <h2>
      <span class="kind-icon">
        {#if isExpense}<ArrowUpRight size={16} />{:else}<ArrowDownLeft size={16} />{/if}
      </span>
      {isExpense ? "Planowane wydatki i rachunki" : "Planowane przychody"}
    </h2>
    <button class="btn sm" onclick={copyPrevious} title="Kopiuje pozycje z ostatniego miesiąca, w którym coś zaplanowano">
      <Copy size={14} /> Skopiuj z poprzedniego
    </button>
  </div>

  <div class="sums">
    <div>
      <span>Zaplanowano</span>
      <strong class="num">{money(total)}</strong>
    </div>
    <div>
      <span>{isExpense ? "Pozostało do zapłacenia" : "Pozostało do otrzymania"}</span>
      <strong class="num accent">{money(remaining)}</strong>
    </div>
  </div>
  <div class="progress">
    <div style:width={total ? `${((total - remaining) / total) * 100}%` : "0%"}></div>
  </div>
  <span class="count muted">{doneCount} z {items.length} odhaczone</span>

  <form class="add" onsubmit={add}>
    <div class="add-row">
      <input
        class="input"
        bind:this={nameInput}
        bind:value={newName}
        placeholder={isExpense ? "np. Czynsz, Internet, Prezent…" : "np. Wypłata, Premia…"}
      />
      <div class="amt"><MoneyInput bind:value={newAmount} /></div>
      <button class="btn primary" type="submit" disabled={!newName.trim() || !newAmount} aria-label="Dodaj">
        <Plus size={17} />
      </button>
    </div>
    <input class="input note-input" bind:value={newNote} placeholder="Notatka (opcjonalnie)" />
  </form>

  <ul>
    {#each items as i (i.id)}
      <li class:done={i.done}>
        {#if editId === i.id}
          <form class="edit" onsubmit={(e) => saveEdit(i, e)}>
            <div class="add-row">
              <!-- svelte-ignore a11y_autofocus -->
              <input class="input" bind:value={editName} autofocus />
              <div class="amt"><MoneyInput bind:value={editAmount} /></div>
            </div>
            <input class="input note-input" bind:value={editNote} placeholder="Notatka (opcjonalnie)" />
            <div class="edit-actions">
              <button class="btn sm" type="button" onclick={() => (editId = null)}>Anuluj</button>
              <button class="btn sm primary" type="submit" disabled={!editName.trim() || !editAmount}>Zapisz</button>
            </div>
          </form>
        {:else}
          <button
            class="check"
            role="checkbox"
            aria-checked={i.done}
            aria-label="Odhacz {i.name}"
            onclick={() => toggle(i)}
          >
            {#if i.done}<Check size={14} strokeWidth={3} />{/if}
          </button>
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="info" onclick={() => startEdit(i)}>
            <span class="name">{i.name}</span>
            {#if i.note}
              <span class="note"><StickyNote size={12} /> {i.note}</span>
            {/if}
          </div>
          <span class="amount num">{money(i.amount)}</span>
          <div class="actions">
            <button class="icon-btn" onclick={() => startEdit(i)} aria-label="Edytuj"><Pencil size={15} /></button>
            <button class="icon-btn danger" onclick={() => remove(i)} aria-label="Usuń"><Trash2 size={15} /></button>
          </div>
        {/if}
      </li>
    {:else}
      <li class="empty">
        Lista jest pusta. Dodaj pozycję powyżej albo skopiuj ją z poprzedniego miesiąca.
      </li>
    {/each}
  </ul>
</section>

<style>
  .col {
    display: flex;
    flex-direction: column;
    gap: 10px;
    --kind: var(--expense);
    --kind-soft: var(--expense-soft);
  }
  .col.income {
    --kind: var(--income);
    --kind-soft: var(--income-soft);
  }
  .card-head {
    margin-bottom: 4px;
  }
  .kind-icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--kind-soft);
    color: var(--kind);
  }
  .sums {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .sums div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--surface-2);
  }
  .sums span {
    font-size: 12px;
    color: var(--text-2);
  }
  .sums strong {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .sums .accent {
    color: var(--kind);
  }
  .progress > div {
    background: var(--kind);
  }
  .count {
    font-size: 12px;
  }

  .add,
  .edit {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .add {
    padding: 12px 0 4px;
    border-top: 1px solid var(--border);
    margin-top: 4px;
  }
  .add-row {
    display: flex;
    gap: 6px;
  }
  .add-row > .input {
    flex: 1;
    min-width: 0;
  }
  .amt {
    width: 130px;
    flex: none;
  }
  .add .btn {
    width: 38px;
    height: 38px;
    padding: 0;
  }
  .note-input {
    height: 32px;
    font-size: 13px;
  }
  .edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: var(--radius);
    transition: background 0.1s;
  }
  li:hover:not(.empty) {
    background: var(--surface-2);
  }
  li:has(.edit) {
    background: var(--surface-2);
    display: block;
  }
  .check {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    flex: none;
    border-radius: 7px;
    border: 2px solid var(--border-strong);
    background: var(--surface);
    color: #fff;
    cursor: pointer;
    transition: all 0.15s;
  }
  .check:hover {
    border-color: var(--kind);
  }
  li.done .check {
    background: var(--kind);
    border-color: var(--kind);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    cursor: pointer;
  }
  .name {
    font-weight: 550;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
  }
  li.done .name {
    text-decoration: line-through;
    color: var(--muted);
  }
  .amount {
    font-weight: 650;
  }
  li.done .amount {
    color: var(--muted);
  }
  .actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  li:hover .actions {
    opacity: 1;
  }
  .empty {
    justify-content: center;
    padding: 24px 12px;
    color: var(--muted);
    text-align: center;
  }
</style>
