# Build Stage
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /usr/src/mysend

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY core/nova-crypto/Cargo.toml core/nova-crypto/
COPY core/nova-protocol/Cargo.toml core/nova-protocol/
COPY core/nova-storage/Cargo.toml core/nova-storage/
COPY core/nova-transport/Cargo.toml core/nova-transport/
COPY core/nova-engine/Cargo.toml core/nova-engine/
COPY server/Cargo.toml server/

# Copy all source code
COPY core core
COPY server server

# Build the release server binary
RUN cargo build --release -p nova-server

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/mysend/target/release/nova-server /usr/local/bin/nova-server

# Render passes $PORT dynamically; default is 8080
ENV PORT=8080
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:${PORT}/health || exit 1

CMD ["nova-server"]
