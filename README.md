# Budżet Domowy

Aplikacja desktopowa do planowania budżetu domowego: Tauri 2 + Rust (rusqlite/SQLite) + SvelteKit (Svelte 5, TypeScript).

## Uruchomienie

```sh
npm install
npm run tauri dev      # tryb deweloperski
npm run tauri build    # paczka instalacyjna (src-tauri/target/release/bundle)
```

### Instalacja lokalna (Omarchy / Hyprland)

```sh
scripts/install-omarchy.sh            # zbuduj i zainstaluj/zaktualizuj
scripts/install-omarchy.sh --restart  # j.w. + uruchom ponownie działającą aplikację
scripts/install-omarchy.sh --uninstall
```

Instaluje plik do `~/.local/bin/budzet-domowy`, ikony i wpis `.desktop` (widoczny w launcherze). Dane nie są ruszane.

Baza danych: `~/.local/share/pl.budzet.domowy/budzet.db` (Linux). Kwoty przechowywane są w groszach.

## Struktura

- `src-tauri/src/db.rs` – schemat, migracje (`PRAGMA user_version`), domyślne kategorie i źródła
- `src-tauri/src/commands.rs` – komendy Tauri (CRUD, podsumowania, eksport CSV, kopia zapasowa)
- `src/lib/views/` – zakładki: Pulpit, Wydatki/Przychody, Planowane, Bony, Ustawienia
- `src/lib/components/` – wykresy, formularze, modale
- `src/app.css` – tokeny jasnego i ciemnego motywu

## Testy

```sh
cd src-tauri && cargo test
npm run check
```
