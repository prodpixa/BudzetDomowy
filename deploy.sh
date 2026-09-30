#!/usr/bin/env bash
# Wdrażanie i obsługa Budżetu Domowego na Raspberry Pi („malinka”).
#
#   ./deploy.sh                    zbuduj obraz (arm64) i wdróż/zaktualizuj na serwerze
#   ./deploy.sh init-env           utwórz lokalny plik .env (konta i hasła) – interaktywnie
#   ./deploy.sh hash-password      wygeneruj hash hasła do .env
#   ./deploy.sh migrate-local [--force]
#                                  przenieś lokalną bazę z wersji desktopowej na serwer
#   ./deploy.sh backup             zrób kopię zapasową teraz
#   ./deploy.sh backups            pokaż kopie na serwerze
#   ./deploy.sh pull-backups       ściągnij kopie z serwera do ./backups-malinka/
#   ./deploy.sh restore <plik>     przywróć bazę z kopii (nazwa z „backups” albo lokalny plik .db)
#   ./deploy.sh rollback           wróć do poprzedniej wersji obrazu
#   ./deploy.sh status | logs      stan kontenera / logi na żywo
#
# Zmienne (opcjonalnie): DEPLOY_HOST (domyślnie malinka), DEPLOY_DIR (domyślnie /opt/budzet).
# Polskie cudzysłowy „…” w komunikatach są zamierzone.
# shellcheck disable=SC1111
set -euo pipefail

HOST="${DEPLOY_HOST:-malinka}"
REMOTE_DIR="${DEPLOY_DIR:-/opt/budzet}"
IMAGE="budzet-domowy"
APP_UID=10001 # użytkownik w kontenerze (Dockerfile: USER 10001)
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!!\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31mBłąd:\033[0m %s\n' "$*" >&2; exit 1; }

remote() { ssh "$HOST" "$@"; }

# Wartość zmiennej z lokalnego .env (bez cudzysłowów).
env_value() {
  local v
  v="$(grep -E "^$1=" .env 2>/dev/null | tail -1 | cut -d= -f2-)"
  v="${v%\'}"; v="${v#\'}"; v="${v%\"}"; v="${v#\"}"
  printf '%s' "$v"
}

data_path() {
  local p; p="$(env_value BUDZET_DATA_PATH)"
  printf '%s' "${p:-/mnt/dietpi_userdata/budzet}"
}

require_env() {
  [[ -f .env ]] || die "Brak pliku .env. Utwórz go: ./deploy.sh init-env"
  grep -q "ZAMIEN" .env && die ".env zawiera przykładowe hashe – uruchom ./deploy.sh init-env"
  grep -qE "^BUDZET_USER1_PASSWORD_HASH='\\\$argon2" .env \
    || die "BUDZET_USER1_PASSWORD_HASH w .env musi być hashem argon2 w pojedynczych cudzysłowach '…'"
}

require_docker() {
  docker info >/dev/null 2>&1 && return
  die "Brak dostępu do Dockera na tym komputerze.
  Jednorazowo:  sudo usermod -aG docker \$USER   (potem wyloguj się i zaloguj ponownie)
  albo, jeśli Docker nie działa:  sudo systemctl enable --now docker"
}

compose() { remote "cd $REMOTE_DIR && docker compose $*"; }

container_running() {
  [[ "$(remote "docker inspect -f '{{.State.Running}}' budzet 2>/dev/null" || true)" == "true" ]]
}

hash_password() {
  # Hash liczy ten sam kod co serwer (argon2id). Hasło czytane ze stdin – nie trafia do historii powłoki.
  cargo run -q --release --manifest-path server/Cargo.toml -- hash-password
}

ask_password() {
  local p1 p2
  while true; do
    read -rsp "  Hasło (min. 8 znaków): " p1; echo >&2
    read -rsp "  Powtórz hasło: " p2; echo >&2
    [[ "$p1" == "$p2" ]] || { warn "Hasła się różnią, spróbuj jeszcze raz."; continue; }
    [[ ${#p1} -ge 8 ]] || { warn "Za krótkie hasło."; continue; }
    printf '%s' "$p1"
    return
  done
}

cmd_init_env() {
  if [[ -f .env ]]; then
    read -rp ".env już istnieje. Nadpisać? [t/N] " a
    [[ "$a" =~ ^[tTyY]$ ]] || exit 0
  fi
  log "Kompiluję narzędzie do haszowania haseł (pierwszy raz może chwilę potrwać)"
  cargo build -q --release --manifest-path server/Cargo.toml
  cp .env.example .env
  chmod 600 .env
  for i in 1 2; do
    echo
    read -rp "Konto $i – login: " name
    [[ -n "$name" ]] || die "Login nie może być pusty"
    hash="$(ask_password | hash_password)"
    # Pojedyncze cudzysłowy: hash zawiera znaki $, których Compose nie może interpretować.
    sed -i "s|^BUDZET_USER${i}_NAME=.*|BUDZET_USER${i}_NAME=${name}|" .env
    sed -i "s|^BUDZET_USER${i}_PASSWORD_HASH=.*|BUDZET_USER${i}_PASSWORD_HASH='${hash}'|" .env
  done
  echo
  log "Zapisano .env (uprawnienia 600 – tylko Ty możesz go czytać)"
}

cmd_hash_password() {
  echo "Wpisz hasło (nie będzie widoczne):" >&2
  local h; h="$(ask_password | hash_password)"
  echo
  echo "Wklej do .env (z pojedynczymi cudzysłowami):"
  echo "BUDZET_USERx_PASSWORD_HASH='$h'"
}

cmd_deploy() {
  require_env
  require_docker

  log "Testy (Rust + TypeScript)"
  (cd server && cargo test -q) >/dev/null || die "Testy serwera nie przechodzą – przerywam"
  npm run -s check >/dev/null || die "npm run check zgłasza błędy – przerywam"

  local rev; rev="$(git rev-parse --short HEAD 2>/dev/null || echo dev)"
  git diff --quiet 2>/dev/null || { rev="$rev-zmiany"; warn "Wdrażasz niezacommitowane zmiany"; }

  log "Buduję obraz linux/arm64 ($rev)"
  docker buildx build --platform linux/arm64 \
    --label "org.opencontainers.image.revision=$rev" \
    -t "$IMAGE:latest" -t "$IMAGE:$rev" --load .

  log "Przygotowuję katalogi na $HOST"
  local data; data="$(data_path)"
  remote "mkdir -p $REMOTE_DIR $data/backups && chown -R $APP_UID:$APP_UID $data && chmod 700 $data"

  log "Wysyłam obraz na $HOST (docker save | ssh | docker load)"
  # Poprzednia wersja zostaje pod tagiem :previous – do szybkiego wycofania (./deploy.sh rollback).
  remote "docker image inspect $IMAGE:latest >/dev/null 2>&1 && docker tag $IMAGE:latest $IMAGE:previous || true"
  docker save "$IMAGE:latest" | gzip -1 | remote "gunzip | docker load" | sed 's/^/    /'

  log "Wysyłam compose.yml i .env"
  scp -q compose.yml "$HOST:$REMOTE_DIR/compose.yml"
  scp -q .env "$HOST:$REMOTE_DIR/.env"
  remote "chmod 600 $REMOTE_DIR/.env"

  if container_running; then
    log "Kopia zapasowa przed aktualizacją"
    remote "docker exec budzet budzet-server backup przed-wdrozeniem" | sed 's/^/    /'
  fi

  log "Uruchamiam"
  compose up -d --remove-orphans
  wait_healthy
  remote "docker image prune -f >/dev/null"
  print_urls
}

wait_healthy() {
  local port; port="$(env_value BUDZET_HOST_PORT)"; port="${port:-8420}"
  for _ in $(seq 1 30); do
    if remote "curl -fsS -m 2 http://127.0.0.1:$port/api/health >/dev/null 2>&1"; then
      log "Aplikacja odpowiada ✔"
      return
    fi
    sleep 1
  done
  compose logs --tail 30 || true
  die "Aplikacja nie odpowiada po 30 s – logi powyżej"
}

print_urls() {
  local port; port="$(env_value BUDZET_HOST_PORT)"; port="${port:-8420}"
  local lan ts
  # Adres w sieci domowej: źródło trasy do internetu (działa dla kabla i Wi-Fi).
  lan="$(remote "ip -4 route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2" || true)"
  ts="$(remote "tailscale ip -4 2>/dev/null" || true)"
  echo
  echo "  Tailscale:      http://$HOST:$port"
  if [[ -n "$ts" ]]; then echo "                  http://$ts:$port"; fi
  if [[ -n "$lan" ]]; then echo "  Sieć domowa:    http://$lan:$port"; fi
}

cmd_migrate_local() {
  local src="${XDG_DATA_HOME:-$HOME/.local/share}/pl.budzet.domowy/budzet.db"
  [[ -f "$src" ]] || die "Nie znaleziono lokalnej bazy: $src"
  local data; data="$(data_path)"
  if [[ "${1:-}" != "--force" ]] && remote "test -s $data/budzet.db"; then
    die "Na serwerze jest już baza ($data/budzet.db). Aby ją zastąpić: ./deploy.sh migrate-local --force"
  fi

  # Spójna kopia przez API backupu SQLite (bezpieczne nawet przy otwartej aplikacji).
  local tmp; tmp="$(mktemp --suffix=.db)"
  python3 - "$src" "$tmp" <<'PY'
import sqlite3, sys
src = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
dst = sqlite3.connect(sys.argv[2])
src.backup(dst)
for t in ["expenses", "incomes", "planned_items", "voucher_entries", "categories"]:
    print(f"    {t}: {dst.execute(f'SELECT COUNT(*) FROM {t}').fetchone()[0]}")
dst.close()
PY

  log "Wysyłam bazę na $HOST"
  remote "mkdir -p $data/backups"
  scp -q "$tmp" "$HOST:$data/budzet.db.migracja"
  rm -f "$tmp"

  local was_running=false
  if container_running; then was_running=true; compose stop; fi
  remote "set -e; cd $data
    if [ -s budzet.db ]; then cp budzet.db backups/przed-migracja-\$(date +%Y-%m-%d_%H%M%S).db; fi
    rm -f budzet.db-wal budzet.db-shm
    mv budzet.db.migracja budzet.db
    chown -R $APP_UID:$APP_UID $data"
  if $was_running; then compose start; wait_healthy; fi
  log "Gotowe. Serwer przy starcie sam zaktualizuje schemat bazy (migracje)."
}

cmd_backup() {
  container_running || die "Kontener nie działa"
  remote "docker exec budzet budzet-server backup reczna"
}

cmd_backups() {
  remote "ls -lh $(data_path)/backups/"
}

cmd_pull_backups() {
  mkdir -p backups-malinka
  log "Pobieram kopie do ./backups-malinka/"
  scp -q "$HOST:$(data_path)/backups/*.db" backups-malinka/
  ls -lh backups-malinka/
}

cmd_restore() {
  local arg="${1:-}"
  local data; data="$(data_path)"
  if [[ -z "$arg" ]]; then
    echo "Użycie: ./deploy.sh restore <plik>. Dostępne kopie na serwerze:"
    cmd_backups
    exit 1
  fi
  local name
  if [[ -f "$arg" ]]; then
    name="wgrana-$(date +%Y-%m-%d_%H%M%S).db"
    log "Wysyłam $arg na serwer"
    scp -q "$arg" "$HOST:$data/backups/$name"
  else
    name="$(basename "$arg")"
    remote "test -f $data/backups/$name" || die "Nie ma kopii $name na serwerze"
  fi

  read -rp "Zastąpić obecne dane kopią „${name}”? [t/N] " a
  [[ "$a" =~ ^[tTyY]$ ]] || exit 0

  compose stop
  remote "set -e; cd $data
    cp budzet.db backups/przed-przywroceniem-\$(date +%Y-%m-%d_%H%M%S).db
    rm -f budzet.db-wal budzet.db-shm
    cp backups/$name budzet.db
    chown -R $APP_UID:$APP_UID $data"
  compose start
  wait_healthy
  log "Przywrócono „${name}”. Poprzedni stan zapisano w backups/przed-przywroceniem-*.db"
}

cmd_rollback() {
  remote "docker image inspect $IMAGE:previous >/dev/null 2>&1" || die "Brak poprzedniej wersji obrazu"
  log "Przywracam poprzednią wersję obrazu"
  remote "docker tag $IMAGE:previous $IMAGE:latest"
  compose up -d --force-recreate
  wait_healthy
}

case "${1:-deploy}" in
  deploy) cmd_deploy ;;
  init-env) cmd_init_env ;;
  hash-password) cmd_hash_password ;;
  migrate-local) shift; cmd_migrate_local "$@" ;;
  backup) cmd_backup ;;
  backups) cmd_backups ;;
  pull-backups) cmd_pull_backups ;;
  restore) shift; cmd_restore "$@" ;;
  rollback) cmd_rollback ;;
  status) compose ps ;;
  logs) compose logs -f --tail 100 ;;
  -h | --help | help) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//' ;;
  *) die "Nieznane polecenie: $1 (./deploy.sh help)" ;;
esac
