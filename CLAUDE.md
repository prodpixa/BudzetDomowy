# Budżet Domowy – wskazówki dla Claude

Aplikacja webowa: serwer Rust (axum + rusqlite/SQLite) w `server/`, frontend SvelteKit (Svelte 5 runes, TS,
adapter-static) w `src/`. Działa w Dockerze na Raspberry Pi „malinka” (arm64, 2 GB RAM). UI i komunikaty po polsku.

## Po każdej większej zmianie

Wdróż na serwer:

```sh
./deploy.sh
```

Skrypt sam uruchamia `cargo test` i `npm run check` i przerywa, jeśli nie przechodzą.
„Większa zmiana” = nowa funkcja, zmiana UI, poprawka błędu widoczna dla użytkownika, migracja bazy.
Drobne zmiany (komentarze, README, refaktor bez wpływu na działanie) nie wymagają wdrożenia.
Potem commit i push (repo prywatne, tylko kod – bez obrazów, baz, `.env`).

## Zasady

- Kwoty zawsze w groszach (`i64` / `number`), formatowanie tylko w `src/lib/format.ts`.
- Zmiany schematu bazy wyłącznie jako nowy wpis w `MIGRATIONS` w `server/src/db.rs` – nigdy nie edytuj istniejących migracji (na serwerze są realne dane).
- Nowa komenda API: funkcja w `server/src/commands.rs` (bierze `&Connection`) + wpis w `dispatch!` w `server/src/api.rs` + metoda w `src/lib/api.ts`.
- Bony żywnościowe są niezależne od głównego budżetu – nie mieszaj ich w podsumowaniach.
- Kolory kategorii to sloty palety `--s1..--s8` (`src/app.css`), osobno dla jasnego i ciemnego motywu.
- Układ musi działać na telefonie (≤760px: dolna nawigacja). Sprawdzaj oba rozmiary.
- Sekrety tylko w `.env` (w `.gitignore`). Hashe haseł w `.env` zawsze w pojedynczych cudzysłowach (znaki `$`).
- Pi ma mało zasobów – żadnych ciężkich zależności ani dodatkowych kontenerów bez potrzeby.
- Testy lokalne na osobnym katalogu danych (`BUDZET_DATA_DIR=/tmp/...`), nigdy na produkcyjnej bazie.
- Na serwerze nie ruszaj Jellyfin (port 8097) ani innych usług.
