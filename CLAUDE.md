# Budżet Domowy – wskazówki dla Claude

Aplikacja desktopowa: Tauri 2 + Rust (rusqlite) + SvelteKit (Svelte 5 runes, TS). UI i komunikaty po polsku.

## Po każdej większej zmianie

Zaktualizuj aplikację zainstalowaną lokalnie (Omarchy):

```sh
scripts/install-omarchy.sh --restart
```

Wcześniej uruchom `cd src-tauri && cargo test` oraz `npm run check` – instaluj tylko, gdy przechodzą.
„Większa zmiana” = nowa funkcja, zmiana UI, poprawka błędu widoczna dla użytkownika, migracja bazy.
Drobne zmiany (komentarze, README, refaktor bez wpływu na działanie) nie wymagają reinstalacji.

## Zasady

- Kwoty zawsze w groszach (`i64` / `number`), formatowanie tylko w `src/lib/format.ts`.
- Zmiany schematu bazy wyłącznie jako nowy wpis w `MIGRATIONS` w `src-tauri/src/db.rs` – nigdy nie edytuj istniejących migracji (użytkownik ma realne dane).
- Bony żywnościowe są niezależne od głównego budżetu – nie mieszaj ich w podsumowaniach.
- Kolory kategorii to sloty palety `--s1..--s8` (`src/app.css`), osobno dla jasnego i ciemnego motywu.
- Testując aplikację (`npm run tauri dev`), używaj `XDG_DATA_HOME=/tmp/...`, żeby nie ruszać prawdziwej bazy użytkownika.
- Instalatorów/bundli nie commitujemy (tylko kod źródłowy).
