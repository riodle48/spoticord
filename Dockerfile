# syntax=docker/dockerfile:1.7
# Targeting amd64 build host, cross-compile to amd64 + arm64

FROM --platform=linux/amd64 rust:1.80.1-slim AS builder
WORKDIR /app

# + git (needed for git deps) + set cargo to use git CLI
RUN apt-get update && apt-get install -yqq \
    git \
    cmake \
    gcc-aarch64-linux-gnu \
    binutils-aarch64-linux-gnu \
    libpq-dev \
    curl \
    bzip2

# Avoid libgit2 auth issues when fetching git dependencies
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

ENV PGVER=16.4
RUN curl -o postgresql.tar.bz2 https://ftp.postgresql.org/pub/source/v${PGVER}/postgresql-${PGVER}.tar.bz2 && \
    tar xjf postgresql.tar.bz2 && \
    cd postgresql-${PGVER} && \
    ./configure --host=aarch64-linux-gnu --enable-shared --disable-static --without-readline --without-zlib --without-icu && \
    cd src/interfaces/libpq && \
    make

COPY . .
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Railway requires cache IDs to be prefixed with s/<service-id>-
RUN --mount=type=cache,id=s/485dbb39-51a0-4498-96c9-4020aa9a1126-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=s/485dbb39-51a0-4498-96c9-4020aa9a1126-cargo-target,target=/app/target \
    cargo build --release --target=x86_64-unknown-linux-gnu && \
    RUSTFLAGS="-L /app/postgresql-${PGVER}/src/interfaces/libpq -C linker=aarch64-linux-gnu-gcc" \
    cargo build --release --target=aarch64-unknown-linux-gnu && \
    cp /app/target/x86_64-unknown-linux-gnu/release/spoticord /app/x86_64 && \
    cp /app/target/aarch64-unknown-linux-gnu/release/spoticord /app/aarch64

FROM debian:bookworm-slim

ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM}

# Runtime dependencies for Discord voice + Postgres + audio
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libopus0 \
    ffmpeg \
    libsodium23 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/x86_64 /tmp/x86_64
COPY --from=builder /app/aarch64 /tmp/aarch64

RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
      cp /tmp/x86_64 /usr/local/bin/spoticord; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
      cp /tmp/aarch64 /usr/local/bin/spoticord; \
    else \
      echo "Unsupported TARGETPLATFORM: ${TARGETPLATFORM}" && exit 1; \
    fi && \
    chmod +x /usr/local/bin/spoticord && \
    rm -rvf /tmp/x86_64 /tmp/aarch64

ENTRYPOINT ["/usr/local/bin/spoticord"]
