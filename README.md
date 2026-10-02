# Budżet Domowy

Aplikacja webowa do planowania budżetu domowego: przychody, wydatki z kategoriami, planowane
wydatki/przychody (ToDo), bony żywnościowe. Idealna do hostowania na własnym serwerze (np. Raspberry Pi, VPS) w sieci domowej lub przez VPN (np. Tailscale). Aplikacja obsługuje logowanie dla dwóch użytkowników.

**Stack:** serwer w Rust (axum + SQLite przez rusqlite), frontend SvelteKit (Svelte 5, TypeScript)
budowany do statycznych plików, całość w jednym kontenerze Docker (~40 MB, ~10 MB RAM).

## Jak to działa

```text
przeglądarka ──HTTP──▶ twój_serwer:8420 ──▶ kontener „budzet”
                                         ├─ budzet-server (Rust)   API /api/*, logowanie, backupy
                                         ├─ /app/static            zbudowany frontend
                                         └─ /data  ◀── wolumen ── /ścieżka/na/serwerze/
                                               ├─ budzet.db        baza SQLite
                                               └─ backups/         kopie zapasowe
```

- **Logowanie:** Konta konfiguruje się w pliku `.env` (hasła przetrzymywane jako bezpieczny hash argon2id). Sesja przechowywana jest w ciasteczku `HttpOnly; SameSite=Strict`, ważna 30 dni od ostatniego użycia. Po 5 błędnych hasłach następuje blokada czasowa.
- **Dostęp:** Aplikacja działa domyślnie na porcie `8420`. Najlepiej nie wystawiać jej bezpośrednio do internetu, lecz używać w sieci lokalnej (LAN) lub poprzez VPN (np. Tailscale / WireGuard).
- **Kopie zapasowe:** Serwer sam robi kopię codziennie w nocy (zachowuje 14 ostatnich) oraz przed każdym wdrożeniem z użyciem skryptu (zachowuje 10 ostatnich). Skrypt pozwala łatwo zgrać je na własny komputer.

## Instalacja i Pierwsze Wdrożenie

Projekt zawiera potężny skrypt `deploy.sh`, który zautomatyzuje proces testowania, budowania obrazu i wdrożenia na Twój zdalny serwer bez konieczności ręcznego kopiowania plików.

**Wymagania lokalne (na Twoim komputerze):**
- Docker (z dostępem bez `sudo`)
- Rust (`cargo`)
- Node.js i `npm`
- Skonfigurowany dostęp SSH do Twojego serwera na klucz (bez hasła)

### Krok 1. Inicjalizacja środowiska i haseł
Sklonuj to repozytorium i uruchom skrypt inicjujący. Skrypt wygeneruje bezpiecznie hashe do Twoich haseł i utworzy plik `.env`.

```sh
./deploy.sh init-env
```

### Krok 2. Konfiguracja serwera docelowego
Otwórz utworzony plik `.env` i zweryfikuj zmienne (m.in. port i ścieżkę do danych na serwerze).
Aby skrypt wdrożeniowy wiedział, gdzie wysłać aplikację, przed uruchomieniem deploy'a ustaw zmienne środowiskowe, np.:

```sh
export DEPLOY_HOST="user@adres_twojego_serwera" # domyślnie skrypt szuka hosta nazwanego "moj-serwer"
export DEPLOY_DIR="/opt/budzet" # miejsce instalacji na serwerze (opcjonalnie)
```
*(Wskazówka: Najwygodniej dodać w systemie w `~/.ssh/config` alias `moj-serwer` wskazujący na adres IP Twojego serwera, dzięki czemu skrypt zadziała domyślnie).*

### Krok 3. Wdrożenie (Deploy)
Teraz wystarczy uruchomić:

```sh
./deploy.sh deploy
```
> **Ważne:** Skrypt domyślnie kompiluje obraz pod architekturę ARM64 (np. dla Raspberry Pi). Jeśli Twój serwer ma standardowy procesor (np. Intel/AMD), wejdź do pliku `deploy.sh` i usuń/zakomentuj linijkę `--platform linux/arm64`.

Skrypt przeprowadzi testy, zbuduje obraz, wyśle go bezpiecznie w archiwum na serwer docelowy, zaktualizuje kontenery i uruchomi aplikację. 
Na koniec wypisze dostępne adresy w sieci lokalnej (LAN), pod jakimi działa Twój budżet!

## Aktualizacja

Po każdej zmianie w kodzie (git pull), wystarczy po prostu uruchomić:
```sh
./deploy.sh
```
Dane w bazie zostają na swoim miejscu, a ewentualne migracje struktury bazy danych wykonają się same przy starcie programu.
Jeśli nowa wersja sprawia problemy, wpisz `./deploy.sh rollback`, by błyskawicznie cofnąć się do poprzedniego działającego obrazu Dockera.

## Zarządzanie kopiami zapasowymi

Zarządzanie bazą jest zautomatyzowane w skrypcie:

```sh
./deploy.sh backups                          # lista dostępnych kopii na serwerze
./deploy.sh restore budzet-2026-09-30.db     # przywróć wybraną kopię z serwera
./deploy.sh restore ~/Pobrane/kopia.db       # wgraj i przywróć plik bazy z własnego komputera
./deploy.sh pull-backups                     # ściągnij wszystkie kopie ze zdalnego serwera (do ./lokalne-kopie/)
```
Przed każdym przywróceniem z kopii, obecny stan bazy zabezpieczany jest jako plik `przed-przywroceniem`, zapobiegając utracie danych przy ewentualnej pomyłce. Kopię można zresztą wygodnie pobrać również w samej aplikacji z zakładki Ustawień.

## Inne przydatne polecenia

| Polecenie | Co robi |
|---|---|
| `./deploy.sh status` / `logs` | Zwraca stan działającego kontenera / wyświetla logi serwera na żywo |
| `./deploy.sh backup` | Wymusza natychmiastowe utworzenie kopii bazy |
| `./deploy.sh hash-password` | Generuje hash nowego hasła (wynik należy zaktualizować w `.env` i wdrożyć) |
| `./deploy.sh migrate-local` | Bezpiecznie wgrywa istniejącą, starszą lokalną bazę na nowouruchomiony serwer |

## Rozwój lokalny (Dla programistów)

Aby pracować nad kodem na własnym komputerze i widzieć zmiany na żywo:

```sh
npm install
npm run build && npm run dev:server   # Uruchamia serwer na :8420 (dane trzymane lokalnie w .dev-data/)
npm run dev                           # Uruchamia frontend z hot-reload na :5173, /api przekierowuje na serwer Rust
```

> **Uwaga:** Serwer do działania na komputerze wymaga zmiennych środowiskowych, w tym kont domowników (możesz np. jednorazowo wstrzyknąć parametry: `BUDZET_USER1_NAME=test BUDZET_USER1_PASSWORD_HASH='…' npm run dev:server`).

Testy można uruchomić lokalnie wykonując `cd server && cargo test` oraz dla frontendu `npm run check`.

## 💡 Współtwórcy
- **prodpixa** - Architektura, kod (Rust/Svelte)
- **[bednarczyklucja](https://github.com/bednarczyklucja)** - Koncepcja, pomysł na projekt i logika aplikacji
