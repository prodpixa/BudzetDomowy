<script lang="ts">
  import { Pencil, Plus, Receipt, Search, Trash2, Wallet, X } from "@lucide/svelte";
  import { api, type Expense, type Income } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { dayLabel, defaultDate, money, monthLabel, percent, plural } from "$lib/format";
  import { categoryIcon, slotColor } from "$lib/icons";
  import PageHead from "$lib/components/PageHead.svelte";

  let { kind }: { kind: "expense" | "income" } = $props();

  interface Row {
    id: number;
    name: string;
    amount: number;
    date: string;
    note: string;
    groupId: number | null;
    raw: Expense | Income;
  }

  const isExpense = $derived(kind === "expense");
  let rows = $state<Row[]>([]);
  let search = $state("");
  // undefined = wszystkie, null = bez kategorii/źródła
  let filter = $state<number | null | undefined>(undefined);

  $effect(() => {
    const month = app.month;
    const k = kind;
    void app.version;
    const load =
      k === "expense"
        ? api.listExpenses(month).then((list) =>
            list.map((e) => ({ ...e, id: e.id!, groupId: e.categoryId, raw: e })),
          )
        : api.listIncomes(month).then((list) =>
            list.map((e) => ({ ...e, id: e.id!, groupId: e.sourceId, raw: e })),
          );
    load.then((r) => (rows = r)).catch((e) => app.error(e));
  });

  // Po zmianie zakładki czyścimy filtry.
  $effect(() => {
    void kind;
    search = "";
    filter = undefined;
  });

  function group(id: number | null) {
    if (isExpense) {
      const c = app.categoryById.get(id);
      return {
        name: c?.name ?? "Bez kategorii",
        color: slotColor(c?.color ?? 0),
        icon: categoryIcon(c?.icon),
      };
    }
    const s = app.sourceById.get(id);
    return { name: s?.name ?? "Bez źródła", color: "var(--income)", icon: Wallet };
  }

  const total = $derived(rows.reduce((s, r) => s + r.amount, 0));

  const groups = $derived.by(() => {
    const m = new Map<number | null, number>();
    for (const r of rows) m.set(r.groupId, (m.get(r.groupId) ?? 0) + r.amount);
    return [...m.entries()].map(([id, sum]) => ({ id, sum, ...group(id) })).sort((a, b) => b.sum - a.sum);
  });

  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return rows.filter(
      (r) =>
        (filter === undefined || r.groupId === filter) &&
        (!q || r.name.toLowerCase().includes(q) || r.note.toLowerCase().includes(q)),
    );
  });
  const filteredTotal = $derived(filtered.reduce((s, r) => s + r.amount, 0));

  const byDay = $derived.by(() => {
    const days: { date: string; items: Row[]; sum: number }[] = [];
    for (const r of filtered) {
      let d = days[days.length - 1];
      if (!d || d.date !== r.date) days.push((d = { date: r.date, items: [], sum: 0 }));
      d.items.push(r);
      d.sum += r.amount;
    }
    return days;
  });

  function add() {
    const base = { id: null, name: "", amount: 0, date: defaultDate(app.month), note: "" };
    app.editor = isExpense
      ? { kind: "expense", item: { ...base, categoryId: filter ?? null } }
      : { kind: "income", item: { ...base, sourceId: filter ?? null } };
  }

  function edit(r: Row) {
    app.editor = isExpense
      ? { kind: "expense", item: { ...(r.raw as Expense) } }
      : { kind: "income", item: { ...(r.raw as Income) } };
  }

  async function remove(r: Row) {
    const ok = await app.confirm(
      isExpense ? "Usunąć wydatek?" : "Usunąć przychód?",
      `„${r.name}” na kwotę ${money(r.amount)} zostanie trwale usunięty.`,
    );
    if (!ok) return;
    await app.mutate(
      () => (isExpense ? api.deleteExpense(r.id) : api.deleteIncome(r.id)),
      "Usunięto",
    );
  }
</script>

<div class="page">
  <PageHead
    title={isExpense ? "Wydatki" : "Przychody"}
    subtitle={isExpense ? "Ewidencja realnych wydatków z podziałem na kategorie" : "Realne wpływy, od których odchodzą wydatki"}
  >
    {#snippet actions()}
      <button class="btn primary" onclick={add}>
        <Plus size={17} />
        {isExpense ? "Dodaj wydatek" : "Dodaj przychód"}
      </button>
    {/snippet}
  </PageHead>

  <div class="layout">
    <section class="card list">
      <div class="toolbar">
        <label class="search">
          <Search size={16} />
          <input placeholder="Szukaj po nazwie lub notatce…" bind:value={search} />
          {#if search}
            <button class="icon-btn" onclick={() => (search = "")} aria-label="Wyczyść"><X size={14} /></button>
          {/if}
        </label>
        {#if filter !== undefined || search}
          <span class="filtered">
            Wyniki: <strong class="num">{money(filteredTotal)}</strong>
          </span>
        {/if}
      </div>

      {#if filter !== undefined}
        {@const g = group(filter)}
        <div class="active-filter">
          <span class="dot" style:background={g.color}></span>
          Filtr: <strong>{g.name}</strong>
          <button class="icon-btn" onclick={() => (filter = undefined)} aria-label="Usuń filtr"><X size={14} /></button>
        </div>
      {/if}

      {#if byDay.length === 0}
        <div class="empty">
          {#if isExpense}<Receipt size={32} strokeWidth={1.5} />{:else}<Wallet size={32} strokeWidth={1.5} />{/if}
          {rows.length === 0
            ? `Brak ${isExpense ? "wydatków" : "przychodów"} w miesiącu: ${monthLabel(app.month)}`
            : "Nic nie pasuje do filtrów"}
          {#if rows.length === 0}
            <button class="btn sm" onclick={add}><Plus size={15} /> Dodaj</button>
          {/if}
        </div>
      {:else}
        {#each byDay as day (day.date)}
          <div class="day">
            <div class="day-head">
              <span>{dayLabel(day.date)}</span>
              <span class="num">{money(day.sum)}</span>
            </div>
            {#each day.items as r (r.id)}
              {@const g = group(r.groupId)}
              <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
              <div class="row" onclick={() => edit(r)}>
                <span class="bubble" style:--c={g.color}><g.icon size={16} /></span>
                <div class="info">
                  <span class="name">{r.name}</span>
                  <span class="meta">
                    {g.name}{#if r.note}<span class="note"> · {r.note}</span>{/if}
                  </span>
                </div>
                <span class="amount num" class:pos={!isExpense}>
                  {isExpense ? "−" : "+"}{money(r.amount)}
                </span>
                <div class="actions">
                  <button class="icon-btn" onclick={(e) => { e.stopPropagation(); edit(r); }} aria-label="Edytuj">
                    <Pencil size={15} />
                  </button>
                  <button class="icon-btn danger" onclick={(e) => { e.stopPropagation(); remove(r); }} aria-label="Usuń">
                    <Trash2 size={15} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/each}
      {/if}
    </section>

    <aside class="card side">
      <span class="side-label">Suma w miesiącu</span>
      <strong class="side-total num" class:neg={isExpense} class:pos={!isExpense}>{money(total)}</strong>
      <span class="muted small">{plural(rows.length, "wpis", "wpisy", "wpisów")}</span>

      <div class="groups">
        <h3>{isExpense ? "Według kategorii" : "Według źródła"}</h3>
        {#each groups as g (g.id)}
          <button class="grp" class:selected={filter === g.id} onclick={() => (filter = filter === g.id ? undefined : g.id)}>
            <div class="grp-row">
              <span class="grp-name"><span class="dot" style:background={g.color}></span>{g.name}</span>
              <span class="num">{money(g.sum)}</span>
            </div>
            <div class="bar">
              <div style:width={percent(g.sum, total).replace(",", ".")} style:background={g.color}></div>
            </div>
          </button>
        {:else}
          <span class="muted small">Brak danych</span>
        {/each}
      </div>
    </aside>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 16px;
    align-items: start;
  }
  .list {
    padding: 12px 12px 16px;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 4px 8px;
  }
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 6px 0 12px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--muted);
  }
  .search input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
  }
  .filtered {
    color: var(--text-2);
    font-size: 13px;
  }
  .active-filter {
    display: inline-flex;
    align-self: flex-start;
    align-items: center;
    gap: 6px;
    margin: 0 4px 6px;
    padding: 2px 4px 2px 10px;
    border-radius: 99px;
    background: var(--accent-soft);
    font-size: 13px;
  }
  .day {
    margin-top: 10px;
  }
  .day-head {
    display: flex;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background 0.1s;
  }
  .row:hover {
    background: var(--surface-2);
  }
  .bubble {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    flex: none;
    background: color-mix(in srgb, var(--c) 16%, transparent);
    color: var(--c);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    font-weight: 550;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    font-size: 12px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .amount {
    font-weight: 650;
    font-size: 15px;
  }
  .actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .row:hover .actions {
    opacity: 1;
  }

  .side {
    position: sticky;
    top: 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .side-label {
    color: var(--text-2);
    font-weight: 550;
  }
  .side-total {
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .small {
    font-size: 12px;
  }
  .groups {
    margin-top: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .groups h3 {
    color: var(--text-2);
    margin-bottom: 6px;
  }
  .grp {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    margin: 0 -10px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    text-align: left;
  }
  .grp:hover {
    background: var(--surface-2);
  }
  .grp.selected {
    background: var(--accent-soft);
  }
  .grp-row {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .grp-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }
  .bar {
    height: 4px;
    border-radius: 99px;
    background: var(--surface-2);
  }
  .bar div {
    height: 100%;
    border-radius: 99px;
  }

  @media (max-width: 1000px) {
    .layout {
      grid-template-columns: 1fr;
    }
    .side {
      position: static;
      order: -1;
    }
  }
  @media (max-width: 760px) {
    .side-total {
      font-size: 24px;
    }
    .list {
      padding: 8px 6px 12px;
    }
    .row {
      padding: 9px 8px;
      gap: 10px;
    }
  }
</style>
