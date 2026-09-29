<script lang="ts" module>
  export interface Slice {
    key: string;
    name: string;
    color: string;
    value: number;
  }
</script>

<script lang="ts">
  import { money, percent } from "$lib/format";

  let {
    slices,
    centerLabel,
    active = $bindable(null),
    size = 240,
  }: { slices: Slice[]; centerLabel: string; active?: string | null; size?: number } = $props();

  const total = $derived(slices.reduce((s, x) => s + x.value, 0));
  const R = $derived(size / 2 - 6);
  const r = $derived(R * 0.64);
  const c = $derived(size / 2);

  let tip = $state<{ x: number; y: number } | null>(null);
  let wrap: HTMLDivElement;

  function arc(a0: number, a1: number, outer: number, inner: number): string {
    // Pełne koło jako jedna ścieżka: odrobinę krótsze niż 2π.
    if (a1 - a0 >= Math.PI * 2 - 1e-6) a1 = a0 + Math.PI * 2 - 1e-4;
    const large = a1 - a0 > Math.PI ? 1 : 0;
    const p = (a: number, rad: number) => `${c + rad * Math.sin(a)} ${c - rad * Math.cos(a)}`;
    return [
      `M ${p(a0, outer)}`,
      `A ${outer} ${outer} 0 ${large} 1 ${p(a1, outer)}`,
      `L ${p(a1, inner)}`,
      `A ${inner} ${inner} 0 ${large} 0 ${p(a0, inner)}`,
      "Z",
    ].join(" ");
  }

  const arcs = $derived.by(() => {
    let a = 0;
    return slices.map((s) => {
      const a0 = a;
      a += total ? (s.value / total) * Math.PI * 2 : 0;
      return { ...s, a0, a1: a };
    });
  });

  const activeSlice = $derived(slices.find((s) => s.key === active) ?? null);

  function move(e: PointerEvent, key: string) {
    active = key;
    const b = wrap.getBoundingClientRect();
    tip = { x: e.clientX - b.left, y: e.clientY - b.top };
  }
  function leave() {
    active = null;
    tip = null;
  }
</script>

<div class="wrap" bind:this={wrap} style:width="{size}px" style:height="{size}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" role="img" aria-label="Wykres kołowy: {centerLabel}">
    {#if total === 0}
      <circle cx={c} cy={c} r={(R + r) / 2} fill="none" stroke="var(--surface-2)" stroke-width={R - r} />
    {:else}
      {#each arcs as s (s.key)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <path
          d={arc(s.a0, s.a1, active === s.key ? R + 4 : R, r)}
          fill={s.color}
          stroke="var(--surface)"
          stroke-width="2"
          stroke-linejoin="round"
          class:dim={active !== null && active !== s.key}
          onpointermove={(e) => move(e, s.key)}
          onpointerleave={leave}
        />
      {/each}
    {/if}
  </svg>

  <div class="center">
    {#if activeSlice}
      <span class="c-label">{activeSlice.name}</span>
      <strong class="num">{money(activeSlice.value)}</strong>
      <span class="c-sub">{percent(activeSlice.value, total)}</span>
    {:else}
      <span class="c-label">{centerLabel}</span>
      <strong class="num">{money(total)}</strong>
    {/if}
  </div>

  {#if tip && activeSlice}
    <div class="tooltip" style:left="{tip.x}px" style:top="{tip.y}px">
      <span class="dot" style:background={activeSlice.color}></span>
      <span>{activeSlice.name}</span>
      <strong class="num">{money(activeSlice.value)}</strong>
      <span class="muted">{percent(activeSlice.value, total)}</span>
    </div>
  {/if}
</div>

<style>
  .wrap {
    position: relative;
    flex: none;
  }
  path {
    cursor: pointer;
    transition:
      opacity 0.15s,
      d 0.15s;
  }
  path.dim {
    opacity: 0.35;
  }
  .center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    text-align: center;
    padding: 0 22%;
  }
  .c-label {
    font-size: 12px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .center strong {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .c-sub {
    font-size: 12px;
    color: var(--muted);
  }
  .tooltip {
    position: absolute;
    z-index: 5;
    transform: translate(12px, -50%);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    white-space: nowrap;
    pointer-events: none;
    font-size: 13px;
  }
</style>
