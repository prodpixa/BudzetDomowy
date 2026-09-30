<script lang="ts">
  import { LogIn, TriangleAlert } from "@lucide/svelte";
  import { api } from "$lib/api";
  import { app } from "$lib/state.svelte";

  let username = $state("");
  let password = $state("");
  let error = $state("");
  let busy = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    if (!username.trim() || !password || busy) return;
    busy = true;
    error = "";
    try {
      const r = await api.login(username.trim(), password);
      password = "";
      await app.loadDictionaries();
      app.user = r.username;
    } catch (err) {
      error = (err as Error).message;
    } finally {
      busy = false;
    }
  }
</script>

<div class="wrap">
  <form class="card" onsubmit={submit}>
    <img src="/favicon.png" alt="" width="48" height="48" />
    <h1>Budżet Domowy</h1>
    <p class="muted">Zaloguj się, aby kontynuować</p>

    <label class="field">
      <span>Login</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="input"
        bind:value={username}
        autocomplete="username"
        autocapitalize="none"
        spellcheck="false"
        autofocus
      />
    </label>
    <label class="field">
      <span>Hasło</span>
      <input class="input" type="password" bind:value={password} autocomplete="current-password" />
    </label>

    {#if error}
      <div class="error" role="alert"><TriangleAlert size={16} /> {error}</div>
    {/if}

    <button class="btn primary" type="submit" disabled={busy || !username.trim() || !password}>
      <LogIn size={17} />
      {busy ? "Logowanie…" : "Zaloguj"}
    </button>
  </form>
</div>

<style>
  .wrap {
    min-height: 100vh;
    min-height: 100dvh;
    display: grid;
    place-items: center;
    padding: 20px;
  }
  form {
    width: 100%;
    max-width: 360px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 32px 28px;
  }
  img {
    border-radius: 12px;
    align-self: center;
  }
  h1,
  p {
    text-align: center;
    margin: 0;
  }
  p {
    margin-top: -8px;
  }
  .btn {
    height: 42px;
    margin-top: 6px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--expense-soft);
    color: var(--expense);
    font-weight: 500;
  }
  .input {
    height: 42px;
    font-size: 16px; /* 16px: iOS nie przybliża strony przy wpisywaniu */
  }
</style>
