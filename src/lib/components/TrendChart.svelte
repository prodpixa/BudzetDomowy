<script lang="ts">
  import type { TrendPoint } from "$lib/api";
  import { money, monthLabel, monthShort } from "$lib/format";

  let { points, current }: { points: TrendPoint[]; current: string } = $props();

  let width = $state(600);
  const height = 220;
  const pad = { top: 12, right: 8, bottom: 28, left: 64 };
  const BAR = 20;
  const GAP = 2;

  const innerW = $derived(Math.max(0, width - pad.left - pad.right));
  const innerH = height - pad.top - pad.bottom;

  // „Ładna” skala osi Y: 1/2/2.5/5 × 10^n
  const ticks = $derived.by(() => {
    const max = Math.max(1, ...points.flatMap((p) => [p.income, p.expense]));
    const raw = max / 4;
    const mag = Math.pow(10, Math.floor(Math.log10(raw)));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw)!;
    const n = Math.ceil(max / step);
    return Array.from({ length: n + 1 }, (_, i) => i * step);
  });
  const yMax = $derived(ticks[ticks.length - 1] || 1);
  const y = (v: number) => pad.top + innerH - (v / yMax) * innerH;
  const band = $derived(points.length ? innerW / points.length : 0);

  function bar(x: number, v: number): string {
    const h = Math.max(0, (v / yMax) * innerH);
    if (h < 0.5) return "";
    const top = pad.top + innerH - h;
    const base = pad.top + innerH;
    const rr = Math.min(4, h, BAR / 2);
    return `M ${x} ${base} L ${x} ${top + rr} Q ${x} ${top} ${x + rr} ${top} L ${x + BAR - rr} ${top} Q ${x + BAR} ${top} ${x + BAR} ${top + rr} L ${x + BAR} ${base} Z`;
  }

  function axisLabel(v: number): string {
    const zl = v / 100;
    if (zl >= 1000) return `${(zl / 1000).toLocaleString("pl-PL", { maximumFractionDigits: 1 })} tys.`;
    return `${zl.toLocaleString("pl-PL")}`;
  }

  let hover = $state<number | null>(null);
</script>

<div class="legend">
  <span><i style:background="var(--chart-income)"></i>Przychody</span>
  <span><i style:background="var(--chart-expense)"></i>Wydatki</span>
</div>

<div class="chart" bind:clientWidth={width}>
  <svg {width} {height} role="img" aria-label="Przychody i wydatki w ostatnich miesiącach">
    {#each ticks as t (t)}
      <line
        x1={pad.left}
        x2={width - pad.right}
        y1={y(t)}
        y2={y(t)}
        stroke={t === 0 ? "var(--baseline)" : "var(--grid)"}
        stroke-width="1"
      />
      <text x={pad.left - 10} y={y(t)} class="tick" text-anchor="end" dominant-baseline="middle">
        {axisLabel(t)}
      </text>
    {/each}

    {#each points as p, i (p.month)}
      {@const cx = pad.left + band * i + band / 2}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect
        x={pad.left + band * i}
        y={pad.top}
        width={band}
        height={innerH}
        fill={hover === i ? "var(--surface-2)" : "transparent"}
        rx="6"
        onpointerenter={() => (hover = i)}
        onpointerleave={() => (hover = null)}
      />
      <path d={bar(cx - BAR - GAP / 2, p.income)} fill="var(--chart-income)" pointer-events="none" />
      <path d={bar(cx + GAP / 2, p.expense)} fill="var(--chart-expense)" pointer-events="none" />
      <text x={cx} y={height - 8} text-anchor="middle" class="tick" class:current={p.month === current}>
        {monthShort(p.month)}
      </text>
    {/each}
  </svg>

  {#if hover !== null && points[hover]}
    {@const p = points[hover]}
    {@const left = pad.left + band * hover + band / 2}
    <div class="tooltip" style:left="{left}px" class:flip={left > width - 200}>
      <strong>{monthLabel(p.month)}</strong>
      <div><i style:background="var(--chart-income)"></i>Przychody <b class="num">{money(p.income)}</b></div>
      <div><i style:background="var(--chart-expense)"></i>Wydatki <b class="num">{money(p.expense)}</b></div>
      <div class="sep">
        Bilans <b class="num" class:pos={p.income - p.expense > 0} class:neg={p.income - p.expense < 0}>
          {money(p.income - p.expense, { sign: true })}
        </b>
      </div>
    </div>
  {/if}
</div>

<style>
  .legend {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--text-2);
    margin-bottom: 8px;
  }
  .legend span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  i {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 3px;
  }
  .chart {
    position: relative;
    width: 100%;
  }
  svg {
    display: block;
    overflow: visible;
  }
  .tick {
    font-size: 11px;
    fill: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .tick.current {
    fill: var(--text);
    font-weight: 650;
  }
  .tooltip {
    position: absolute;
    top: 0;
    transform: translateX(16px);
    min-width: 200px;
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    pointer-events: none;
    font-size: 13px;
    z-index: 5;
  }
  .tooltip.flip {
    transform: translateX(calc(-100% - 16px));
  }
  .tooltip div {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
  }
  .tooltip b {
    margin-left: auto;
    padding-left: 12px;
    color: var(--text);
    font-weight: 600;
  }
  .tooltip b.pos {
    color: var(--income);
  }
  .tooltip b.neg {
    color: var(--expense);
  }
  .sep {
    border-top: 1px solid var(--border);
    padding-top: 5px;
  }
</style>
