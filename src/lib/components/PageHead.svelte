<script lang="ts">
  import type { Snippet } from "svelte";
  import MonthPicker from "./MonthPicker.svelte";

  let {
    title,
    subtitle,
    monthPicker = true,
    actions,
  }: { title: string; subtitle?: string; monthPicker?: boolean; actions?: Snippet } = $props();
</script>

<div class="page-head">
  <div>
    <h1>{title}</h1>
    {#if subtitle}<p>{subtitle}</p>{/if}
  </div>
  <div class="right">
    {#if monthPicker}<MonthPicker />{/if}
    {@render actions?.()}
  </div>
</div>

<style>
  .right {
    display: flex;
    align-items: center;
    gap: 10px;
  }
</style>
