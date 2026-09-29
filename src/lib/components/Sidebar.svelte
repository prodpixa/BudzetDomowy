<script lang="ts">
  import {
    LayoutDashboard,
    ListTodo,
    Monitor,
    Moon,
    Receipt,
    Settings,
    Sun,
    Ticket,
    Wallet,
  } from "@lucide/svelte";
  import { app, type Theme, type View } from "$lib/state.svelte";

  const items: { id: View; label: string; icon: typeof Wallet }[] = [
    { id: "dashboard", label: "Pulpit", icon: LayoutDashboard },
    { id: "expenses", label: "Wydatki", icon: Receipt },
    { id: "incomes", label: "Przychody", icon: Wallet },
    { id: "planned", label: "Planowane", icon: ListTodo },
  ];

  const themes: { id: Theme; label: string; icon: typeof Sun }[] = [
    { id: "light", label: "Jasny", icon: Sun },
    { id: "system", label: "Systemowy", icon: Monitor },
    { id: "dark", label: "Ciemny", icon: Moon },
  ];
</script>

<aside>
  <div class="brand">
    <div class="logo"><Wallet size={18} strokeWidth={2.2} /></div>
    <div>
      <strong>Budżet Domowy</strong>
    </div>
  </div>

  <nav>
    <span class="section">Budżet</span>
    {#each items as it (it.id)}
      <button class="nav-item" class:active={app.view === it.id} onclick={() => (app.view = it.id)}>
        <it.icon size={18} />
        {it.label}
      </button>
    {/each}

    <span class="section">Osobno</span>
    <button class="nav-item" class:active={app.view === "vouchers"} onclick={() => (app.view = "vouchers")}>
      <Ticket size={18} />
      Bony żywnościowe
    </button>
  </nav>

  <div class="bottom">
    <button class="nav-item" class:active={app.view === "settings"} onclick={() => (app.view = "settings")}>
      <Settings size={18} />
      Ustawienia
    </button>
    <div class="theme" role="radiogroup" aria-label="Motyw">
      {#each themes as t (t.id)}
        <button
          role="radio"
          aria-checked={app.theme === t.id}
          class:active={app.theme === t.id}
          title={t.label}
          onclick={() => app.setTheme(t.id)}
        >
          <t.icon size={15} />
        </button>
      {/each}
    </div>
  </div>
</aside>

<style>
  aside {
    width: 232px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 20px 14px 16px;
    background: var(--surface);
    border-right: 1px solid var(--border);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px 18px;
    font-size: 15px;
  }
  .logo {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: linear-gradient(135deg, var(--accent), #1baf7a);
    color: #fff;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .section {
    padding: 14px 10px 6px;
    font-size: 11px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 11px;
    height: 38px;
    padding: 0 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    font-weight: 550;
    cursor: pointer;
    text-align: left;
    transition:
      background 0.12s,
      color 0.12s;
  }
  .nav-item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .bottom {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .theme {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .theme button {
    display: grid;
    place-items: center;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .theme button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }
</style>
