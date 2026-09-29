<script lang="ts">
  import { Scale } from "@lucide/svelte";
  import { api, type Planned } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import { money } from "$lib/format";
  import PageHead from "$lib/components/PageHead.svelte";
  import PlannedList from "$lib/components/PlannedList.svelte";

  let incomes = $state<Planned[]>([]);
  let expenses = $state<Planned[]>([]);

  $effect(() => {
    const month = app.month;
    void app.version;
    Promise.all([api.listPlanned(month, "income"), api.listPlanned(month, "expense")])
      .then(([i, e]) => {
        incomes = i;
        expenses = e;
      })
      .catch((e) => app.error(e));
  });

  const sum = (list: Planned[], onlyOpen = false) =>
    list.filter((i) => !onlyOpen || !i.done).reduce((s, i) => s + i.amount, 0);
  const plannedBalance = $derived(sum(incomes) - sum(expenses));
</script>

<div class="page">
  <PageHead
    title="Planowane"
    subtitle="Lista ToDo planowanych przychodów, wydatków i stałych rachunków"
  />

  <div class="balance card">
    <span class="icon"><Scale size={18} /></span>
    <div class="b-item">
      <span>Planowane przychody</span>
      <strong class="num pos">{money(sum(incomes))}</strong>
    </div>
    <span class="op">−</span>
    <div class="b-item">
      <span>Planowane wydatki</span>
      <strong class="num neg">{money(sum(expenses))}</strong>
    </div>
    <span class="op">=</span>
    <div class="b-item">
      <span>Planowany bilans</span>
      <strong class="num" class:pos={plannedBalance > 0} class:neg={plannedBalance < 0}>
        {money(plannedBalance, { sign: true })}
      </strong>
    </div>
  </div>

  <div class="cols">
    <PlannedList kind="income" items={incomes} />
    <PlannedList kind="expense" items={expenses} />
  </div>
</div>

<style>
  .balance {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 16px 24px;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .b-item {
    display: flex;
    flex-direction: column;
  }
  .b-item span {
    font-size: 12px;
    color: var(--text-2);
  }
  .b-item strong {
    font-size: 20px;
    font-weight: 700;
  }
  .op {
    font-size: 20px;
    color: var(--muted);
  }
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }
  @media (max-width: 1050px) {
    .cols {
      grid-template-columns: 1fr;
    }
  }
</style>
