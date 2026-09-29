<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight, ChartPie, ListTodo, PiggyBank, Scale } from "@lucide/svelte";
  import { api, type MonthSummary, type TrendPoint } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { money, percent } from "$lib/format";
  import { categoryIcon, slotColor } from "$lib/icons";
  import PageHead from "$lib/components/PageHead.svelte";
  import DonutChart, { type Slice } from "$lib/components/DonutChart.svelte";
  import TrendChart from "$lib/components/TrendChart.svelte";

  let summary = $state<MonthSummary | null>(null);
  let trend = $state<TrendPoint[]>([]);
  let active = $state<string | null>(null);

  $effect(() => {
    const month = app.month;
    void app.version;
    Promise.all([api.monthSummary(month), api.monthTrend(month, 6)])
      .then(([s, t]) => {
        summary = s;
        trend = t;
      })
      .catch((e) => app.error(e));
  });

  const slices = $derived<Slice[]>(
    (summary?.byCategory ?? []).map((c) => ({
      key: String(c.id),
      name: c.name,
      color: slotColor(c.color),
      value: c.total,
    })),
  );

  const plannedBalance = $derived(
    summary ? summary.plannedIncomes.total - summary.plannedExpenses.total : 0,
  );
</script>

<div class="page">
  <PageHead title="Pulpit" subtitle="Podsumowanie budżetu w wybranym miesiącu" />

  {#if summary}
    <div class="kpis">
      <div class="card kpi">
        <div class="kpi-head"><span class="kpi-icon inc"><ArrowDownLeft size={16} /></span>Przychody</div>
        <strong class="num">{money(summary.incomeTotal)}</strong>
        <span class="sub">plan: <span class="num">{money(summary.plannedIncomes.total)}</span></span>
      </div>
      <div class="card kpi">
        <div class="kpi-head"><span class="kpi-icon exp"><ArrowUpRight size={16} /></span>Wydatki</div>
        <strong class="num">{money(summary.expenseTotal)}</strong>
        <span class="sub">plan: <span class="num">{money(summary.plannedExpenses.total)}</span></span>
      </div>
      <div class="card kpi">
        <div class="kpi-head"><span class="kpi-icon"><Scale size={16} /></span>Bilans miesiąca</div>
        <strong class="num" class:pos={summary.balance > 0} class:neg={summary.balance < 0}>
          {money(summary.balance, { sign: true })}
        </strong>
        <span class="sub">przychody − wydatki</span>
      </div>
      <div class="card kpi hero">
        <div class="kpi-head"><span class="kpi-icon acc"><PiggyBank size={16} /></span>Saldo na koniec miesiąca</div>
        <strong class="num" class:neg={summary.endBalance < 0}>{money(summary.endBalance)}</strong>
        <span class="sub">
          saldo przeniesione:
          <span class="num" class:neg={summary.carriedOver < 0}>{money(summary.carriedOver)}</span>
        </span>
      </div>
    </div>

    <div class="grid">
      <section class="card">
        <div class="card-head">
          <h2><ChartPie size={17} /> Wydatki według kategorii</h2>
        </div>
        {#if summary.byCategory.length === 0}
          <div class="empty">
            <ChartPie size={32} strokeWidth={1.5} />
            Brak wydatków w tym miesiącu
            <button class="btn sm" onclick={() => (app.view = "expenses")}>Dodaj pierwszy wydatek</button>
          </div>
        {:else}
          <div class="pie">
            <DonutChart {slices} centerLabel="Wydatki razem" bind:active />
            <table class="legend">
              <thead>
                <tr><th>Kategoria</th><th class="r">Kwota</th><th class="r">Udział</th></tr>
              </thead>
              <tbody>
                {#each summary.byCategory as c (c.id)}
                  {@const Icon = categoryIcon(c.icon)}
                  <tr
                    class:active={active === String(c.id)}
                    class:dim={active !== null && active !== String(c.id)}
                    onpointerenter={() => (active = String(c.id))}
                    onpointerleave={() => (active = null)}
                  >
                    <td>
                      <span class="cat">
                        <span class="dot" style:background={slotColor(c.color)}></span>
                        <Icon size={15} />
                        {c.name}
                      </span>
                    </td>
                    <td class="r num">{money(c.total)}</td>
                    <td class="r num muted">{percent(c.total, summary.expenseTotal)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>

      <section class="card plan">
        <div class="card-head">
          <h2><ListTodo size={17} /> Plan na ten miesiąc</h2>
          <button class="btn sm ghost" onclick={() => (app.view = "planned")}>Otwórz</button>
        </div>

        {#each [{ label: "Planowane wydatki", t: summary.plannedExpenses, left: "Do zapłacenia" }, { label: "Planowane przychody", t: summary.plannedIncomes, left: "Do otrzymania" }] as block (block.label)}
          <div class="plan-block">
            <div class="plan-row">
              <span>{block.label}</span>
              <strong class="num">{money(block.t.total)}</strong>
            </div>
            <div class="progress">
              <div style:width={block.t.total ? `${((block.t.total - block.t.remaining) / block.t.total) * 100}%` : "0%"}></div>
            </div>
            <div class="plan-row small">
              <span class="muted">{block.t.doneCount} z {block.t.count} odhaczone</span>
              <span>{block.left}: <strong class="num">{money(block.t.remaining)}</strong></span>
            </div>
          </div>
        {/each}

        <div class="plan-row total">
          <span>Planowany bilans</span>
          <strong class="num" class:pos={plannedBalance > 0} class:neg={plannedBalance < 0}>
            {money(plannedBalance, { sign: true })}
          </strong>
        </div>
      </section>
    </div>

    <section class="card">
      <div class="card-head"><h2>Ostatnie 6 miesięcy</h2></div>
      <TrendChart points={trend} current={app.month} />
    </section>
  {/if}
</div>

<style>
  .kpis {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
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
    color: var(--text-2);
    font-weight: 550;
  }
  .kpi-icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--surface-2);
    color: var(--text-2);
  }
  .kpi-icon.inc {
    background: var(--income-soft);
    color: var(--income);
  }
  .kpi-icon.exp {
    background: var(--expense-soft);
    color: var(--expense);
  }
  .kpi-icon.acc {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .kpi strong {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: normal;
  }
  .kpi.hero {
    background: linear-gradient(135deg, var(--accent-soft), transparent 70%), var(--surface);
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(280px, 1fr);
    gap: 16px;
    align-items: start;
  }

  .pie {
    display: flex;
    align-items: center;
    gap: 32px;
    flex-wrap: wrap;
  }
  .legend {
    flex: 1;
    min-width: 260px;
    border-collapse: collapse;
  }
  .legend th {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    text-align: left;
    padding: 0 8px 8px;
  }
  .legend td {
    padding: 8px;
    border-top: 1px solid var(--border);
    transition: opacity 0.15s;
  }
  .legend tr.active td {
    background: var(--surface-2);
  }
  .legend tr.dim td {
    opacity: 0.45;
  }
  .legend .r {
    text-align: right;
  }
  .cat {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }
  .cat :global(svg) {
    color: var(--text-2);
  }

  .plan {
    display: flex;
    flex-direction: column;
  }
  .plan .card-head {
    margin-bottom: 8px;
  }
  .plan-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
  }
  .plan-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .plan-row.small {
    font-size: 12px;
  }
  .plan-row.total {
    padding-top: 14px;
    font-weight: 600;
  }
  .plan-row.total strong {
    font-size: 17px;
  }

  @media (max-width: 1100px) {
    .kpis {
      grid-template-columns: repeat(2, 1fr);
    }
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
