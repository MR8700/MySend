# Builds only the `nova-server` workspace member (the discovery/relay WebSocket server) — see
# server/src/service.rs's module doc comment for why this exists as a WebSocket service rather
# than the raw-UDP `nova-server` this repo used to ship: Render (like most PaaS hosts) has no raw
# UDP ingress, only HTTP(S)/TCP.
#
# The build context is the WHOLE repo (see the root-level docker build command in this file's
# accompanying render.yaml) because this is a Cargo workspace: `cargo build -p nova-server` still
# needs every workspace member's Cargo.toml present to resolve the workspace, even though only
# nova-server's own dependency subgraph (nova-crypto, nova-protocol, axum, tokio, ...) actually
# gets compiled — none of the other members' build scripts or system dependencies (e.g. the
# Tauri UI crate's webview/GTK requirements) are ever invoked for this build.

FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Layer-cache the dependency graph separately from this crate's own source: copy every manifest
# first (so a source-only change doesn't invalidate the dependency-compile layer), fetch, then
# copy the real source and build for real.
COPY Cargo.toml Cargo.lock ./
COPY core/nova-crypto/Cargo.toml core/nova-crypto/Cargo.toml
COPY core/nova-protocol/Cargo.toml core/nova-protocol/Cargo.toml
COPY core/nova-storage/Cargo.toml core/nova-storage/Cargo.toml
COPY core/nova-transport/Cargo.toml core/nova-transport/Cargo.toml
COPY core/nova-engine/Cargo.toml core/nova-engine/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY nova-bootstrap/Cargo.toml nova-bootstrap/Cargo.toml
COPY ui/src-tauri/Cargo.toml ui/src-tauri/Cargo.toml
RUN mkdir -p core/nova-crypto/src core/nova-protocol/src core/nova-storage/src \
        core/nova-transport/src core/nova-engine/src server/src nova-bootstrap/src ui/src-tauri/src \
    && echo 'fn main() {}' > server/src/main.rs \
    && for d in core/nova-crypto core/nova-protocol core/nova-storage core/nova-transport core/nova-engine nova-bootstrap ui/src-tauri; do \
        touch "$d/src/lib.rs" 2>/dev/null || true; \
       done \
    && echo 'fn main() {}' > nova-bootstrap/src/main.rs \
    && echo 'fn main() {}' > ui/src-tauri/src/main.rs \
    && cargo build --release -p nova-server 2>/dev/null || true

COPY . .
# `COPY` preserves each file's original mtime from the build context rather than stamping it with
# the current time, so real source files here can end up with an *older* mtime than the stub
# `lib.rs`/`main.rs` files the previous layer's dummy build just compiled moments ago. Cargo's
# staleness check is mtime-based: without this, it can wrongly conclude the real nova-protocol
# (etc.) source hasn't changed since that dummy build and reuse its cached, effectively-empty
# rlib — silently linking nova-server against stub crates instead of the real ones, surfacing as
# "unresolved import" errors for types (`PeerEndpoint`, `ServerRequest`, ...) that genuinely do
# exist in the real source. Touching every real local-crate `.rs` file to "now" after the copy
# guarantees Cargo sees them as newer than anything the dummy layer produced.
RUN find core server nova-bootstrap ui/src-tauri -name '*.rs' -exec touch {} + \
    && cargo build --release -p nova-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/nova-server /usr/local/bin/nova-server
# Render sets $PORT at deploy time and routes the public hostname's :443 traffic to it — see
# server/src/main.rs, which reads $PORT (falling back to NOVA_SERVER_BIND, then 8080).
ENV RUST_LOG=info
CMD ["/usr/local/bin/nova-server"]
