# syntax=docker/dockerfile:1
#
# Obraz Budżetu Domowego dla Raspberry Pi (linux/arm64), budowany na zwykłym PC (x86_64):
#   docker buildx build --platform linux/arm64 -t budzet-domowy .
#
# Etapy „web” i „server” działają natywnie na komputerze budującym ($BUILDPLATFORM),
# a Rust kompiluje skrośnie (cross-compile) na docelową architekturę ($TARGETARCH).
# Ostatni etap nie ma żadnego RUN – tylko kopiuje pliki, więc nic nie jest emulowane.

# ---------- 1. Frontend (SvelteKit → statyczne HTML/JS/CSS) ----------
FROM --platform=$BUILDPLATFORM node:24-trixie-slim AS web
WORKDIR /src
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci --no-audit --no-fund
COPY svelte.config.js vite.config.js tsconfig.json ./
COPY static ./static
COPY src ./src
RUN npm run build

# ---------- 2. Serwer (Rust) ----------
FROM --platform=$BUILDPLATFORM rust:1-slim-trixie AS server
ARG TARGETARCH
# tzdata: strefy czasowe (kopiujemy je do obrazu końcowego – potrzebne do dat backupów).
# gcc-aarch64-linux-gnu: linker i kompilator C (SQLite) dla ARM.
RUN apt-get update \
 && apt-get install -y --no-install-recommends tzdata \
    $([ "$TARGETARCH" = arm64 ] && echo gcc-aarch64-linux-gnu libc6-dev-arm64-cross) \
 && rm -rf /var/lib/apt/lists/*
RUN case "$TARGETARCH" in \
      arm64) echo aarch64-unknown-linux-gnu ;; \
      amd64) echo x86_64-unknown-linux-gnu ;; \
      *) echo "Nieobsługiwana architektura: $TARGETARCH" >&2; exit 1 ;; \
    esac > /rust-target \
 && rustup target add "$(cat /rust-target)"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
WORKDIR /src
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
# Cache rejestru i katalogu target między buildami – kolejne wdrożenia kompilują tylko zmiany.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,id=budzet-target-$TARGETARCH \
    cargo build --release --locked --target "$(cat /rust-target)" \
 && mkdir /out && cp "target/$(cat /rust-target)/release/budzet-server" /out/

# ---------- 3. Obraz końcowy (~40 MB) ----------
FROM debian:trixie-slim
COPY --from=server /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=server /out/budzet-server /usr/local/bin/budzet-server
COPY --from=web /src/build /app/static
ENV BUDZET_DATA_DIR=/data \
    BUDZET_STATIC_DIR=/app/static \
    BUDZET_PORT=8420 \
    TZ=Europe/Warsaw
# Zwykły użytkownik zamiast roota: nawet przy błędzie w aplikacji nie ma uprawnień do systemu.
USER 10001:10001
EXPOSE 8420
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["budzet-server", "healthcheck"]
ENTRYPOINT ["budzet-server"]
CMD ["serve"]
