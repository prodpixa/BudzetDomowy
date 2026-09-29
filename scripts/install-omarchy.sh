#!/usr/bin/env bash
# Buduje wersję release i instaluje/aktualizuje Budżet Domowy lokalnie (Omarchy / Hyprland).
#
#   scripts/install-omarchy.sh             – zbuduj i zainstaluj
#   scripts/install-omarchy.sh --restart   – dodatkowo uruchom ponownie działającą aplikację
#   scripts/install-omarchy.sh --uninstall – usuń aplikację (dane w bazie zostają)
#
# Dane (baza SQLite) są w ~/.local/share/pl.budzet.domowy/ i nie są ruszane przez ten skrypt.
set -euo pipefail

APP=budzet-domowy
NAME="Budżet Domowy"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor"
DESKTOP="$APPS_DIR/$APP.desktop"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }

refresh_caches() {
  command -v update-desktop-database >/dev/null && update-desktop-database -q "$APPS_DIR" || true
  command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$ICON_DIR" 2>/dev/null || true
}

if [[ "${1:-}" == "--uninstall" ]]; then
  log "Usuwam $NAME"
  rm -f "$BIN_DIR/$APP" "$DESKTOP"
  for s in 32 64 128 256 512; do rm -f "$ICON_DIR/${s}x${s}/apps/$APP.png"; done
  refresh_caches
  log "Gotowe (dane w ~/.local/share/pl.budzet.domowy zostały zachowane)"
  exit 0
fi

cd "$ROOT"

if [[ ! -d node_modules ]]; then
  log "Instaluję zależności npm"
  npm ci
fi

log "Buduję wersję release (może potrwać kilka minut)"
npm run tauri build -- --no-bundle

log "Instaluję plik wykonywalny do $BIN_DIR"
install -Dm755 "src-tauri/target/release/$APP" "$BIN_DIR/$APP"

log "Instaluję ikony"
install -Dm644 src-tauri/icons/32x32.png "$ICON_DIR/32x32/apps/$APP.png"
install -Dm644 src-tauri/icons/64x64.png "$ICON_DIR/64x64/apps/$APP.png"
install -Dm644 src-tauri/icons/128x128.png "$ICON_DIR/128x128/apps/$APP.png"
install -Dm644 src-tauri/icons/128x128@2x.png "$ICON_DIR/256x256/apps/$APP.png"
install -Dm644 src-tauri/icons/icon.png "$ICON_DIR/512x512/apps/$APP.png"

log "Tworzę wpis w menu aplikacji"
mkdir -p "$APPS_DIR"
cat >"$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=$NAME
GenericName=Planowanie budżetu
Comment=Przychody, wydatki, plan miesiąca i bony żywnościowe
Exec=$BIN_DIR/$APP
Icon=$APP
Terminal=false
Categories=Office;Finance;
Keywords=budżet;wydatki;przychody;finanse;bony;
StartupWMClass=$APP
EOF
refresh_caches

if pgrep -x "$APP" >/dev/null; then
  if [[ "${1:-}" == "--restart" ]]; then
    log "Uruchamiam ponownie działającą aplikację"
    pkill -x "$APP" || true
    sleep 1
    setsid -f "$BIN_DIR/$APP" >/dev/null 2>&1
  else
    log "Aplikacja jest uruchomiona – nowa wersja zadziała po jej ponownym otwarciu"
  fi
fi

log "Zainstalowano $NAME ($(grep -m1 '"version"' package.json | tr -dc '0-9.'))"
