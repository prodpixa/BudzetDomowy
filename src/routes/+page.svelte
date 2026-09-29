<script lang="ts">
  import "@fontsource-variable/inter";
  import "../app.css";
  import { onMount } from "svelte";
  import { app } from "$lib/state.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Overlays from "$lib/components/Overlays.svelte";
  import EntryEditor from "$lib/components/EntryEditor.svelte";
  import Dashboard from "$lib/views/Dashboard.svelte";
  import Entries from "$lib/views/Entries.svelte";
  import Planned from "$lib/views/Planned.svelte";
  import Vouchers from "$lib/views/Vouchers.svelte";
  import Settings from "$lib/views/Settings.svelte";

  let ready = $state(false);
  let main: HTMLElement;

  onMount(async () => {
    try {
      await app.loadDictionaries();
    } catch (e) {
      app.error(e);
    }
    ready = true;
  });

  // Nowa zakładka zaczyna się od góry.
  $effect(() => {
    void app.view;
    main?.scrollTo({ top: 0 });
  });
</script>

<div class="shell">
  <Sidebar />
  <main bind:this={main}>
    {#if ready}
      {#if app.view === "dashboard"}
        <Dashboard />
      {:else if app.view === "expenses"}
        <Entries kind="expense" />
      {:else if app.view === "incomes"}
        <Entries kind="income" />
      {:else if app.view === "planned"}
        <Planned />
      {:else if app.view === "vouchers"}
        <Vouchers />
      {:else}
        <Settings />
      {/if}
    {/if}
  </main>
</div>

{#if app.editor}
  {#key app.editor}
    <EntryEditor editor={app.editor} />
  {/key}
{/if}
<Overlays />

<style>
  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
</style>
