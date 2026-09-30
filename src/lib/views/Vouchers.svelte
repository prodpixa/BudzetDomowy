<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight, Pencil, Plus, Store, Ticket, Trash2 } from "@lucide/svelte";
  import { api, type Voucher, type VoucherKind, type VoucherSummary } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { dayLabel, defaultDate, money, monthLabel } from "$lib/format";
  import PageHead from "$lib/components/PageHead.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import MoneyInput from "$lib/components/MoneyInput.svelte";

  let entries = $state<Voucher[]>([]);
  let summary = $state<VoucherSummary | null>(null);

  $effect(() => {
    const month = app.month;
    void app.version;
    Promise.all([api.listVouchers(month), api.voucherSummary(month)])
      .then(([e, s]) => {
        entries = e;
        summary = s;
      })
      .catch((e) => app.error(e));
  });

  const columns: { kind: VoucherKind; title: string; placeLabel: string; placeholder: string }[] = [
    { kind: "topup", title: "Wpływy na kartę", placeLabel: "Opis", placeholder: "np. Doładowanie od pracodawcy" },
    { kind: "spend", title: "Wydatki z karty", placeLabel: "Gdzie", placeholder: "np. Lidl, Żabka…" },
  ];

  // Szybkie dodawanie – osobny stan dla każdej kolumny.
  let forms = $state<Record<VoucherKind, { place: string; amount: number | null; date: string }>>({
    topup: { place: "", amount: null, date: "" },
    spend: { place: "", amount: null, date: "" },
  });
  $effect(() => {
    const d = defaultDate(app.month);
    forms.topup.date = d;
    forms.spend.date = d;
  });

  let editing = $state<Voucher | null>(null);
  let editAmount = $state<number | null>(null);

  async function add(kind: VoucherKind, e: Event) {
    e.preventDefault();
    const f = forms[kind];
    if (!f.place.trim() || !f.amount) return;
    const ok = await app.mutate(
      () => api.saveVoucher({ id: null, kind, place: f.place.trim(), amount: f.amount!, date: f.date, note: "" }),
    );
    if (ok !== undefined) {
      f.place = "";
      f.amount = null;
    }
  }

  function edit(v: Voucher) {
    editing = { ...v };
    editAmount = v.amount;
  }

  async function saveEdit(e: Event) {
    e.preventDefault();
    if (!editing || !editing.place.trim() || !editAmount) return;
    const item = { ...editing, amount: editAmount };
    const ok = await app.mutate(() => api.saveVoucher(item), "Zapisano zmiany");
    if (ok !== undefined) editing = null;
  }

  async function remove(v: Voucher) {
    const ok = await app.confirm("Usunąć wpis?", `„${v.place}” na kwotę ${money(v.amount)} zostanie usunięty.`);
    if (ok) await app.mutate(() => api.deleteVoucher(v.id!), "Usunięto");
  }
</script>

<div class="page">
  <PageHead title="Bony żywnościowe" subtitle="Osobna karta – nie wpływa na główny budżet" />

  {#if summary}
    <div class="kpis">
      <div class="card kpi hero">
        <div class="kpi-head"><span class="kpi-icon"><Ticket size={16} /></span>Saldo na karcie</div>
        <strong class="num" class:neg={summary.balance < 0}>{money(summary.balance)}</strong>
        <span class="sub">stan na koniec miesiąca: {monthLabel(app.month)}</span>
      </div>
      <div class="card kpi">
        <div class="kpi-head">Przeniesione</div>
        <strong class="num">{money(summary.carriedOver)}</strong>
        <span class="sub">saldo z poprzednich miesięcy</span>
      </div>
      <div class="card kpi">
        <div class="kpi-head"><span class="kpi-icon inc"><ArrowDownLeft size={16} /></span>Wpływy w miesiącu</div>
        <strong class="num pos">{money(summary.monthTopups)}</strong>
      </div>
      <div class="card kpi">
        <div class="kpi-head"><span class="kpi-icon exp"><ArrowUpRight size={16} /></span>Wydatki w miesiącu</div>
        <strong class="num neg">{money(summary.monthSpends)}</strong>
      </div>
    </div>
  {/if}

  <div class="cols">
    {#each columns as col (col.kind)}
      {@const list = entries.filter((e) => e.kind === col.kind)}
      {@const f = forms[col.kind]}
      <section class="card">
        <div class="card-head">
          <h2>{col.title}</h2>
          <span class="num col-sum" class:pos={col.kind === "topup"} class:neg={col.kind === "spend"}>
            {money(list.reduce((s, e) => s + e.amount, 0))}
          </span>
        </div>

        <form class="add" onsubmit={(e) => add(col.kind, e)}>
          <input class="input" bind:value={f.place} placeholder={col.placeholder} aria-label={col.placeLabel} />
          <div class="amt"><MoneyInput bind:value={f.amount} /></div>
          <input class="input date" type="date" bind:value={f.date} aria-label="Data" />
          <button class="btn primary" type="submit" disabled={!f.place.trim() || !f.amount} aria-label="Dodaj">
            <Plus size={17} />
          </button>
        </form>

        <ul>
          {#each list as v (v.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
            <li onclick={() => edit(v)}>
              <span class="bubble" class:in={v.kind === "topup"}>
                {#if v.kind === "topup"}<ArrowDownLeft size={16} />{:else}<Store size={16} />{/if}
              </span>
              <div class="info">
                <span class="name">{v.place}</span>
                <span class="meta">{dayLabel(v.date)}{v.note ? ` · ${v.note}` : ""}</span>
              </div>
              <span class="num amount" class:pos={v.kind === "topup"}>
                {v.kind === "topup" ? "+" : "−"}{money(v.amount)}
              </span>
              <div class="actions">
                <button class="icon-btn" onclick={(e) => { e.stopPropagation(); edit(v); }} aria-label="Edytuj"><Pencil size={15} /></button>
                <button class="icon-btn danger" onclick={(e) => { e.stopPropagation(); remove(v); }} aria-label="Usuń"><Trash2 size={15} /></button>
              </div>
            </li>
          {:else}
            <li class="empty">Brak wpisów w tym miesiącu</li>
          {/each}
        </ul>
      </section>
    {/each}
  </div>
</div>

{#if editing}
  <Modal title="Edytuj wpis bonów" onclose={() => (editing = null)} width={460}>
    <form id="voucher-form" onsubmit={saveEdit}>
      <div class="kind-toggle">
        <button type="button" class:active={editing.kind === "topup"} onclick={() => (editing!.kind = "topup")}>Wpływ</button>
        <button type="button" class:active={editing.kind === "spend"} onclick={() => (editing!.kind = "spend")}>Wydatek</button>
      </div>
      <label class="field">
        <span>{editing.kind === "spend" ? "Gdzie" : "Opis"}</span>
        <input class="input" bind:value={editing.place} />
      </label>
      <div class="row">
        <label class="field">
          <span>Kwota</span>
          <MoneyInput bind:value={editAmount} />
        </label>
        <label class="field">
          <span>Data</span>
          <input class="input" type="date" bind:value={editing.date} />
        </label>
      </div>
      <label class="field">
        <span>Notatka</span>
        <input class="input" bind:value={editing.note} placeholder="opcjonalnie" />
      </label>
    </form>
    {#snippet footer()}
      <button class="btn" onclick={() => (editing = null)}>Anuluj</button>
      <button class="btn primary" type="submit" form="voucher-form">Zapisz</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .kpis {
    display: grid;
    grid-template-columns: 1.3fr 1fr 1fr 1fr;
    gap: 16px;
  }
  .kpi {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 18px 20px;
  }
  .kpi-head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
    color: var(--text-2);
    font-weight: 550;
  }
  .kpi-icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--warning-soft);
    color: var(--warning);
  }
  .kpi-icon.inc {
    background: var(--income-soft);
    color: var(--income);
  }
  .kpi-icon.exp {
    background: var(--expense-soft);
    color: var(--expense);
  }
  .kpi strong {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .kpi.hero {
    background: linear-gradient(135deg, var(--warning-soft), transparent 70%), var(--surface);
  }
  .kpi.hero strong {
    font-size: 30px;
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }
  .col-sum {
    font-weight: 700;
    font-size: 16px;
  }
  .add {
    display: flex;
    gap: 6px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 6px;
  }
  .add > .input:first-child {
    flex: 1;
    min-width: 0;
  }
  .amt {
    width: 120px;
    flex: none;
  }
  .date {
    width: 150px;
    flex: none;
  }
  .add .btn {
    width: 38px;
    height: 38px;
    padding: 0;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 10px;
    border-radius: var(--radius);
    cursor: pointer;
  }
  li:hover:not(.empty) {
    background: var(--surface-2);
  }
  .bubble {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: var(--warning-soft);
    color: var(--warning);
    flex: none;
  }
  .bubble.in {
    background: var(--income-soft);
    color: var(--income);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    font-weight: 550;
  }
  .meta {
    font-size: 12px;
    color: var(--muted);
  }
  .amount {
    font-weight: 650;
  }
  .actions {
    display: flex;
    opacity: 0;
  }
  li:hover .actions {
    opacity: 1;
  }
  .empty {
    justify-content: center;
    padding: 24px;
    color: var(--muted);
    cursor: default;
  }
  form#voucher-form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .kind-toggle {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .kind-toggle button {
    height: 32px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    font-weight: 550;
    cursor: pointer;
  }
  .kind-toggle button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }
  @media (max-width: 1100px) {
    .kpis {
      grid-template-columns: 1fr 1fr;
    }
    .cols {
      grid-template-columns: 1fr;
    }
  }
  @media (max-width: 760px) {
    .kpis {
      gap: 10px;
    }
    .kpi {
      padding: 14px;
    }
    .kpi strong,
    .kpi.hero strong {
      font-size: 19px;
    }
    .kpi-head {
      font-size: 12px;
    }
    .add {
      flex-wrap: wrap;
    }
    .add > .input:first-child {
      flex-basis: 100%;
    }
    .amt,
    .date {
      flex: 1;
      width: auto;
    }
  }
</style>
