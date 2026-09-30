<script lang="ts">
  import {
    LayoutDashboard,
    ListTodo,
    LogOut,
    Monitor,
    Moon,
    Receipt,
    Settings,
    Sun,
    Ticket,
    Wallet,
  } from "@lucide/svelte";
  import { api } from "$lib/api";
  import { app, type Theme, type View } from "$lib/state.svelte";

  // `short` – krótka etykieta w dolnej nawigacji na telefonie.
  const items: { id: View; label: string; short: string; icon: typeof Wallet }[] = [
    { id: "dashboard", label: "Pulpit", short: "Pulpit", icon: LayoutDashboard },
    { id: "expenses", label: "Wydatki", short: "Wydatki", icon: Receipt },
    { id: "incomes", label: "Przychody", short: "Przychody", icon: Wallet },
    { id: "planned", label: "Planowane", short: "Plan", icon: ListTodo },
  ];
  const vouchers = { id: "vouchers" as View, label: "Bony żywnościowe", short: "Bony", icon: Ticket };
  const settings = { id: "settings" as View, label: "Ustawienia", short: "Więcej", icon: Settings };

  const themes: { id: Theme; label: string; icon: typeof Sun }[] = [
    { id: "light", label: "Jasny", icon: Sun },
    { id: "system", label: "Systemowy", icon: Monitor },
    { id: "dark", label: "Ciemny", icon: Moon },
  ];

  async function logout() {
    try {
      await api.logout();
    } finally {
      app.user = null;
    }
  }
</script>

{#snippet navItem(it: { id: View; label: string; short: string; icon: typeof Wallet })}
  <button
    class="nav-item"
    class:active={app.view === it.id}
    aria-current={app.view === it.id ? "page" : undefined}
    onclick={() => (app.view = it.id)}
  >
    <it.icon size={18} />
    <span class="long">{it.label}</span>
    <span class="short">{it.short}</span>
  </button>
{/snippet}

<aside>
  <div class="brand">
    <img src="/favicon.png" alt="" width="34" height="34" />
    <strong>Budżet Domowy</strong>
  </div>

  <nav>
    <span class="section">Budżet</span>
    {#each items as it (it.id)}
      {@render navItem(it)}
    {/each}

    <span class="section">Osobno</span>
    {@render navItem(vouchers)}
    <span class="mobile-only">{@render navItem(settings)}</span>
  </nav>

  <div class="bottom">
    {@render navItem(settings)}
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
    <div class="user">
      <span class="avatar">{app.user?.slice(0, 1).toUpperCase()}</span>
      <span class="name">{app.user}</span>
      <button class="icon-btn" onclick={logout} title="Wyloguj" aria-label="Wyloguj"><LogOut size={16} /></button>
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
  .brand img {
    border-radius: 10px;
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
  .short,
  .mobile-only {
    display: none;
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
  .user {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 4px 0 8px;
    border-top: 1px solid var(--border);
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 700;
    font-size: 13px;
  }
  .user .name {
    flex: 1;
    font-weight: 550;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ----- Telefon: dolna nawigacja z ikonami ----- */
  @media (max-width: 760px) {
    aside {
      width: 100%;
      flex-direction: row;
      padding: 4px 4px calc(4px + env(safe-area-inset-bottom));
      border-right: none;
      border-top: 1px solid var(--border);
    }
    .brand,
    .section,
    .bottom,
    .long {
      display: none;
    }
    .short {
      display: inline;
    }
    .mobile-only {
      display: contents;
    }
    nav {
      flex: 1;
      flex-direction: row;
      justify-content: space-around;
    }
    .nav-item {
      flex: 1;
      flex-direction: column;
      justify-content: center;
      gap: 3px;
      height: 54px;
      padding: 0 2px;
      font-size: 11px;
      text-align: center;
      min-width: 0;
    }
    .nav-item.active {
      background: transparent;
    }
    .nav-item.active :global(svg) {
      background: var(--accent-soft);
      border-radius: 99px;
      padding: 3px 12px;
      box-sizing: content-box;
    }
  }
</style>
