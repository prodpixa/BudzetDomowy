<script lang="ts">
  import "@fontsource-variable/inter";
  import "../app.css";
  import { onMount } from "svelte";
  import { api, UnauthorizedError } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import Login from "$lib/components/Login.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Overlays from "$lib/components/Overlays.svelte";
  import EntryEditor from "$lib/components/EntryEditor.svelte";
  import Dashboard from "$lib/views/Dashboard.svelte";
  import Entries from "$lib/views/Entries.svelte";
  import Planned from "$lib/views/Planned.svelte";
  import Vouchers from "$lib/views/Vouchers.svelte";
  import Settings from "$lib/views/Settings.svelte";

  let main = $state<HTMLElement>();

  onMount(async () => {
    try {
      const me = await api.me();
      await app.loadDictionaries();
      app.user = me.username;
    } catch (e) {
      if (!(e instanceof UnauthorizedError)) app.error(e);
      app.user = null;
    }
  });

  // Nowa zakładka zaczyna się od góry.
  $effect(() => {
    void app.view;
    main?.scrollTo({ top: 0 });
  });
</script>

{#if app.user === null}
  <Login />
{:else if app.user}
  <div class="shell">
    <Sidebar />
    <main bind:this={main}>
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
    </main>
  </div>
{/if}

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
    height: 100dvh;
    overflow: hidden;
  }
  main {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
  /* Telefon: nawigacja na dole, treść nad nią. */
  @media (max-width: 760px) {
    .shell {
      flex-direction: column-reverse;
    }
  }
</style>
