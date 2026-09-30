<script lang="ts">
  import {
    Check,
    Database,
    Download,
    LogOut,
    Monitor,
    Moon,
    Pencil,
    Plus,
    Sun,
    Trash2,
    Upload,
    UserRound,
    Wallet,
  } from "@lucide/svelte";
  import { api, type Category, type Source } from "$lib/api";
  import { app, type Theme } from "$lib/state.svelte";
  import { money } from "$lib/format";
  import { CATEGORY_ICONS, SLOT_NAMES, categoryIcon, slotColor } from "$lib/icons";
  import PageHead from "$lib/components/PageHead.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import MoneyInput from "$lib/components/MoneyInput.svelte";

  const themes: { id: Theme; label: string; icon: typeof Sun }[] = [
    { id: "light", label: "Jasny", icon: Sun },
    { id: "dark", label: "Ciemny", icon: Moon },
    { id: "system", label: "Zgodny z systemem", icon: Monitor },
  ];

  // ---- saldo początkowe ----
  let opening = $state<number | null>(null);
  let openingSaved = $state(0);
  let openingNegative = $state(false);
  api.getOpeningBalance().then((v) => {
    openingSaved = v;
    openingNegative = v < 0;
    opening = Math.abs(v) || null;
  });
  const openingValue = $derived((opening ?? 0) * (openingNegative ? -1 : 1));

  async function saveOpening() {
    const v = openingValue;
    const ok = await app.mutate(() => api.setOpeningBalance(v), "Zapisano saldo początkowe");
    if (ok !== undefined) openingSaved = v;
  }

  // ---- kategorie ----
  let cat = $state<Category | null>(null);

  function newCategory() {
    const used = new Set(app.categories.map((c) => c.color));
    const free = [1, 2, 3, 4, 5, 6, 7, 8].find((s) => !used.has(s)) ?? 8;
    cat = { id: null, name: "", color: free, icon: "sparkles" };
  }

  async function saveCategory(e: Event) {
    e.preventDefault();
    if (!cat || !cat.name.trim()) return;
    const item = $state.snapshot(cat);
    const ok = await app.mutate(() => api.saveCategory(item), item.id ? "Zapisano kategorię" : "Dodano kategorię");
    if (ok !== undefined) {
      cat = null;
      await app.loadDictionaries();
    }
  }

  async function removeCategory(c: Category) {
    const ok = await app.confirm(
      "Usunąć kategorię?",
      `Kategoria „${c.name}” zostanie usunięta. Jej wydatki zostaną i trafią do „Bez kategorii”.`,
    );
    if (!ok) return;
    await app.mutate(() => api.deleteCategory(c.id!), "Usunięto kategorię");
    await app.loadDictionaries();
  }

  // ---- źródła przychodów ----
  let newSource = $state("");
  let editSource = $state<Source | null>(null);

  async function addSource(e: Event) {
    e.preventDefault();
    if (!newSource.trim()) return;
    const name = newSource.trim();
    const ok = await app.mutate(() => api.saveSource({ id: null, name }), "Dodano źródło");
    if (ok !== undefined) {
      newSource = "";
      await app.loadDictionaries();
    }
  }

  async function saveSource(e: Event) {
    e.preventDefault();
    if (!editSource || !editSource.name.trim()) return;
    const item = $state.snapshot(editSource);
    const ok = await app.mutate(() => api.saveSource(item), "Zapisano");
    if (ok !== undefined) {
      editSource = null;
      await app.loadDictionaries();
    }
  }

  async function removeSource(s: Source) {
    const ok = await app.confirm(
      "Usunąć źródło?",
      `Źródło „${s.name}” zostanie usunięte. Przychody zostaną i trafią do „Bez źródła”.`,
    );
    if (!ok) return;
    await app.mutate(() => api.deleteSource(s.id!), "Usunięto źródło");
    await app.loadDictionaries();
  }

  // ---- dane ----
  let fileInput = $state<HTMLInputElement>();

  async function restore() {
    const file = fileInput?.files?.[0];
    if (fileInput) fileInput.value = ""; // pozwala wybrać ten sam plik ponownie
    if (!file) return;
    const ok = await app.confirm(
      "Przywrócić kopię?",
      `Wszystkie obecne dane zostaną zastąpione danymi z pliku „${file.name}”. Serwer zrobi wcześniej kopię obecnego stanu.`,
      "Przywróć",
    );
    if (!ok) return;
    const r = await app.mutate(() => api.restoreDb(file), "Przywrócono dane z kopii");
    if (r !== undefined) {
      await app.loadDictionaries();
      const v = await api.getOpeningBalance();
      openingSaved = v;
      openingNegative = v < 0;
      opening = Math.abs(v) || null;
    }
  }

  async function logout() {
    try {
      await api.logout();
    } finally {
      app.user = null;
    }
  }
</script>

<div class="page">
  <PageHead title="Ustawienia" monthPicker={false} />

  <div class="grid">
    <section class="card">
      <div class="card-head"><h2>Wygląd</h2></div>
      <div class="themes">
        {#each themes as t (t.id)}
          <button class="theme" class:active={app.theme === t.id} onclick={() => app.setTheme(t.id)}>
            <t.icon size={20} />
            {t.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="card">
      <div class="card-head"><h2>Saldo początkowe</h2></div>
      <p class="hint">
        Kwota, którą masz na start (przed pierwszym wpisem). Dolicza się do salda przenoszonego na kolejne miesiące.
      </p>
      <div class="opening">
        <div class="sign">
          <button class:active={!openingNegative} onclick={() => (openingNegative = false)}>+</button>
          <button class:active={openingNegative} onclick={() => (openingNegative = true)}>−</button>
        </div>
        <div class="grow"><MoneyInput bind:value={opening} /></div>
        <button class="btn primary" disabled={openingValue === openingSaved} onclick={saveOpening}>Zapisz</button>
      </div>
      <span class="muted small">Obecnie: {money(openingSaved)}</span>
    </section>

    <section class="card">
      <div class="card-head">
        <h2>Kategorie wydatków</h2>
        <button class="btn sm" onclick={newCategory}><Plus size={15} /> Dodaj</button>
      </div>
      <ul>
        {#each app.categories as c (c.id)}
          {@const Icon = categoryIcon(c.icon)}
          <li>
            <span class="bubble" style:--c={slotColor(c.color)}><Icon size={16} /></span>
            <span class="name">{c.name}</span>
            <button class="icon-btn" onclick={() => (cat = { ...c })} aria-label="Edytuj"><Pencil size={15} /></button>
            <button class="icon-btn danger" onclick={() => removeCategory(c)} aria-label="Usuń"><Trash2 size={15} /></button>
          </li>
        {/each}
      </ul>
    </section>

    <section class="card">
      <div class="card-head"><h2>Źródła przychodów</h2></div>
      <form class="inline" onsubmit={addSource}>
        <input class="input" bind:value={newSource} placeholder="Nowe źródło, np. Zlecenie" />
        <button class="btn primary" type="submit" disabled={!newSource.trim()} aria-label="Dodaj"><Plus size={17} /></button>
      </form>
      <ul>
        {#each app.sources as s (s.id)}
          <li>
            {#if editSource?.id === s.id}
              <form class="inline grow" onsubmit={saveSource}>
                <!-- svelte-ignore a11y_autofocus -->
                <input class="input" bind:value={editSource.name} autofocus />
                <button class="btn sm primary" type="submit" aria-label="Zapisz"><Check size={15} /></button>
                <button class="btn sm" type="button" onclick={() => (editSource = null)}>Anuluj</button>
              </form>
            {:else}
              <span class="bubble" style:--c="var(--income)"><Wallet size={16} /></span>
              <span class="name">{s.name}</span>
              <button class="icon-btn" onclick={() => (editSource = { ...s })} aria-label="Edytuj"><Pencil size={15} /></button>
              <button class="icon-btn danger" onclick={() => removeSource(s)} aria-label="Usuń"><Trash2 size={15} /></button>
            {/if}
          </li>
        {/each}
      </ul>
    </section>

    <section class="card">
      <div class="card-head"><h2><Database size={17} /> Dane</h2></div>
      <p class="hint">
        Dane są na serwerze w bazie SQLite. Serwer codziennie sam robi kopię zapasową (14 ostatnich dni).
        Tutaj możesz pobrać kopię na to urządzenie albo przywrócić dane z pliku .db.
      </p>
      <div class="data-actions">
        <a class="btn" href={api.exportUrl} download><Download size={16} /> Eksport do CSV (ZIP)</a>
        <a class="btn" href={api.backupUrl} download><Database size={16} /> Pobierz kopię bazy</a>
        <button class="btn" onclick={() => fileInput?.click()}><Upload size={16} /> Przywróć z kopii…</button>
        <input bind:this={fileInput} type="file" accept=".db,application/vnd.sqlite3" hidden onchange={restore} />
      </div>
    </section>

    <section class="card">
      <div class="card-head"><h2><UserRound size={17} /> Konto</h2></div>
      <p class="hint">Zalogowano jako <strong>{app.user}</strong>. Dane budżetu są wspólne dla obu kont.</p>
      <button class="btn" onclick={logout}><LogOut size={16} /> Wyloguj</button>
    </section>
  </div>
</div>

{#if cat}
  <Modal title={cat.id ? "Edytuj kategorię" : "Nowa kategoria"} onclose={() => (cat = null)} width={520}>
    <form id="cat-form" onsubmit={saveCategory}>
      <label class="field">
        <span>Nazwa</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input" bind:value={cat.name} autofocus placeholder="np. Dzieci" />
      </label>
      <div class="field">
        <span>Kolor</span>
        <div class="swatches">
          {#each SLOT_NAMES as label, i (label)}
            <button
              type="button"
              class="swatch"
              class:active={cat.color === i + 1}
              style:background={slotColor(i + 1)}
              title={label}
              aria-label={label}
              onclick={() => (cat!.color = i + 1)}
            >
              {#if cat.color === i + 1}<Check size={14} strokeWidth={3} />{/if}
            </button>
          {/each}
        </div>
      </div>
      <div class="field">
        <span>Ikona</span>
        <div class="icons">
          {#each Object.entries(CATEGORY_ICONS) as [key, Icon] (key)}
            <button
              type="button"
              class="icon-choice"
              class:active={cat.icon === key}
              style:--c={slotColor(cat.color)}
              onclick={() => (cat!.icon = key)}
              aria-label={key}
            >
              <Icon size={18} />
            </button>
          {/each}
        </div>
      </div>
    </form>
    {#snippet footer()}
      <button class="btn" onclick={() => (cat = null)}>Anuluj</button>
      <button class="btn primary" type="submit" form="cat-form" disabled={!cat?.name.trim()}>Zapisz</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }
  .hint {
    margin: -6px 0 14px;
    color: var(--text-2);
  }
  .small {
    font-size: 12px;
    display: block;
    margin-top: 8px;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .theme {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-2);
    font-weight: 550;
    cursor: pointer;
  }
  .theme:hover {
    background: var(--surface-hover);
  }
  .theme.active {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
  }
  .opening,
  .inline {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .inline > .input {
    flex: 1;
  }
  .inline {
    margin-bottom: 8px;
  }
  .grow {
    flex: 1;
  }
  .sign {
    display: flex;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .sign button {
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 6px;
    background: transparent;
    font-size: 16px;
    font-weight: 700;
    color: var(--muted);
    cursor: pointer;
  }
  .sign button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
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
    padding: 7px 8px;
    border-radius: var(--radius-sm);
  }
  li:hover {
    background: var(--surface-2);
  }
  li .name {
    flex: 1;
    font-weight: 550;
  }
  .bubble {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 9px;
    background: color-mix(in srgb, var(--c) 16%, transparent);
    color: var(--c);
  }
  a.btn {
    text-decoration: none;
    color: inherit;
  }
  .data-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  form#cat-form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .swatches {
    display: flex;
    gap: 8px;
  }
  .swatch {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 9px;
    border: 2px solid transparent;
    color: #fff;
    cursor: pointer;
  }
  .swatch.active {
    box-shadow:
      0 0 0 2px var(--surface),
      0 0 0 4px var(--text);
  }
  .icons {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
    gap: 6px;
  }
  .icon-choice {
    display: grid;
    place-items: center;
    height: 40px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    cursor: pointer;
  }
  .icon-choice:hover {
    background: var(--surface-hover);
  }
  .icon-choice.active {
    border-color: var(--c);
    background: color-mix(in srgb, var(--c) 16%, transparent);
    color: var(--c);
  }
  @media (max-width: 1000px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
