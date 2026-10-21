# ============================================================
#  Tableau de bord étudiant — image Docker (100 % Rust)
#  Axum (API + fichiers statiques) + frontend Leptos compilé en WebAssembly.
#  Aucun Python ni Node.js, ni à la compilation ni à l'exécution.
# ============================================================

# ---- 1) Frontend : Rust → WebAssembly (Trunk) ------------------------------
FROM rust:1-bookworm AS frontend
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
# Les crates du workspace non nécessaires ici sont réduites à des coquilles pour résoudre le workspace.
RUN mkdir -p src-tauri/src && echo "" > src-tauri/src/lib.rs
WORKDIR /src/crates/frontend
RUN trunk build --release

# ---- 2) Serveur : Rust natif ----------------------------------------------
FROM rust:1-bookworm AS serveur
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
RUN mkdir -p src-tauri/src && echo "" > src-tauri/src/lib.rs
RUN cargo build --release --locked -p tdb-server

# ---- 3) Image finale -------------------------------------------------------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /data tdb \
    && mkdir -p /data /app && chown tdb /data
COPY --from=serveur /src/target/release/tdb-server /app/tdb-server
COPY --from=frontend /src/crates/frontend/dist /app/frontend-dist
ENV FRONTEND_DIR=/app/frontend-dist \
    DATABASE_PATH=/data/tableau-de-bord.db \
    PORT=8000 \
    RUST_LOG=info
USER tdb
VOLUME ["/data"]
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS "http://127.0.0.1:${PORT}/api/health" || exit 1
CMD ["/app/tdb-server"]
