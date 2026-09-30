# Budżet Domowy

Aplikacja webowa do planowania budżetu domowego: przychody, wydatki z kategoriami, planowane
wydatki/przychody (ToDo), bony żywnościowe. Działa na Raspberry Pi w sieci domowej i przez Tailscale,
z logowaniem dla dwóch osób.

**Stack:** serwer w Rust (axum + SQLite przez rusqlite), frontend SvelteKit (Svelte 5, TypeScript)
budowany do statycznych plików, całość w jednym kontenerze Docker (~40 MB, ~10 MB RAM).

## Jak to działa

```
przeglądarka ──HTTP──▶ malinka:8420 ──▶ kontener „budzet”
                                         ├─ budzet-server (Rust)   API /api/*, logowanie, backupy
                                         ├─ /app/static            zbudowany frontend
                                         └─ /data  ◀── wolumen ── /mnt/dietpi_userdata/budzet/
                                               ├─ budzet.db        baza SQLite
                                               └─ backups/         kopie zapasowe
```

- **Logowanie:** dwa konta z pliku `.env` (hasła jako hash argon2id). Sesja w ciasteczku
  `HttpOnly; SameSite=Strict`, ważna 30 dni od ostatniego użycia. Po 5 błędnych hasłach – minuta blokady.
- **Dostęp:** port 8420 w sieci domowej i przez Tailscale. Router niczego nie przekierowuje,
  więc z internetu aplikacja jest niewidoczna.
- **Kopie zapasowe:** serwer sam robi kopię codziennie po 3:00 (`backups/budzet-RRRR-MM-DD.db`,
  14 ostatnich) oraz przed każdym wdrożeniem i przywróceniem (`backups/reczna-*.db`, 10 ostatnich).
  Kopie leżą na tym samym dysku – co jakiś czas ściągnij je na laptopa: `./deploy.sh pull-backups`.

## Pierwsze wdrożenie

Wymagania na laptopie: Docker z dostępem bez sudo, Rust (`cargo`), Node.js, `ssh malinka` bez hasła.

```sh
# 0. (jednorazowo) dostęp do Dockera bez sudo – potem wyloguj się i zaloguj ponownie
sudo usermod -aG docker $USER

# 1. konta i hasła → plik .env (hasła wpisujesz sam, zapisywane są tylko hashe)
./deploy.sh init-env

# 2. (opcjonalnie) przenieś dane z dawnej wersji desktopowej
./deploy.sh migrate-local

# 3. zbuduj obraz arm64, wyślij na malinkę i uruchom
./deploy.sh
```

Aplikacja: **http://malinka:8420** (Tailscale) lub **http://&lt;adres-malinki-w-LAN&gt;:8420** w sieci domowej
(`./deploy.sh` wypisuje oba adresy na końcu).
Na telefonie: otwórz adres i wybierz „Dodaj do ekranu głównego” – działa jak aplikacja.

## Aktualizacja

Po zmianach w kodzie:

```sh
./deploy.sh
```

Skrypt uruchamia testy, buduje nowy obraz, robi kopię bazy, podmienia kontener i sprawdza, czy
aplikacja odpowiada. Dane w bazie zostają; zmiany schematu bazy wykonują się same przy starcie
(migracje w `server/src/db.rs`).

Gdyby nowa wersja działała źle: `./deploy.sh rollback` (wraca do poprzedniego obrazu).

## Przywracanie kopii zapasowej

```sh
./deploy.sh backups                          # lista kopii na serwerze
./deploy.sh restore budzet-2026-09-30.db     # przywróć kopię z serwera
./deploy.sh restore ~/Pobrane/kopia.db       # albo plik z laptopa
```

Przed przywróceniem obecna baza jest zapisywana jako `backups/przed-przywroceniem-*.db`, więc
pomyłkę da się cofnąć. Kopię można też pobrać i przywrócić w aplikacji: **Ustawienia → Dane**.

<details>
<summary>Ręcznie, bez skryptu (na malince)</summary>

```sh
cd /opt/budzet
docker compose stop
cd /mnt/dietpi_userdata/budzet
cp budzet.db backups/przed-przywroceniem.db
rm -f budzet.db-wal budzet.db-shm
cp backups/budzet-2026-09-30.db budzet.db
chown 10001:10001 budzet.db
cd /opt/budzet && docker compose start
```
</details>

## Inne polecenia

| Polecenie | Co robi |
|---|---|
| `./deploy.sh status` / `logs` | stan kontenera / logi na żywo |
| `./deploy.sh backup` | kopia zapasowa teraz |
| `./deploy.sh pull-backups` | ściąga kopie do `./backups-malinka/` |
| `./deploy.sh hash-password` | hash nowego hasła (zmiana hasła: wklej do `.env` i `./deploy.sh`) |

## Rozwój lokalny

```sh
npm install
npm run build && npm run dev:server   # serwer na :8420 (dane w .dev-data/)
npm run dev                           # frontend z hot-reload na :5173, /api → :8420
```

`dev:server` potrzebuje konta w zmiennych środowiskowych, np.
`BUDZET_USER1_NAME=test BUDZET_USER1_PASSWORD_HASH='…' npm run dev:server`.

Testy: `cd server && cargo test` oraz `npm run check`.

## Struktura

- `server/src/main.rs` – start serwera, konfiguracja z `.env`, polecenia CLI (backup, hash-password)
- `server/src/api.rs` – trasy HTTP, sesje, pobieranie/wgrywanie plików
- `server/src/commands.rs` – logika aplikacji i zapytania SQL
- `server/src/db.rs` – schemat i migracje (`PRAGMA user_version`)
- `server/src/auth.rs` – hasła, sesje, ochrona przed zgadywaniem
- `server/src/backup.rs` – kopie zapasowe i ich rotacja
- `src/` – frontend (widoki w `src/lib/views/`, komponenty w `src/lib/components/`)
- `Dockerfile`, `compose.yml`, `.env.example`, `deploy.sh` – wdrożenie
